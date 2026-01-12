import { prisma } from './prisma';
import { sendEmail, getAppUrl } from './email';

export type NotificationType = 
  | 'workflow_started'
  | 'step_assigned'
  | 'decision_made'
  | 'workflow_completed'
  | 'needs_changes'
  | 'proof_rejected'
  | 'comment_added'
  | 'mention'
  | 'submission_received'
  | 'submission_status_changed';

interface NotificationPayload {
  type: NotificationType;
  recipientId?: string;
  recipientEmail?: string;
  data: Record<string, unknown>;
}

interface UserNotificationPrefs {
  onAssignment: boolean;
  onMention: boolean;
  onStepChange: boolean;
  onDecision: boolean;
  onApproval: boolean;
  onSubmission: boolean;
}

const DEFAULT_PREFS: UserNotificationPrefs = {
  onAssignment: true,
  onMention: true,
  onStepChange: true,
  onDecision: true,
  onApproval: true,
  onSubmission: true,
};

export class NotificationService {
  private appUrl: string;

  constructor() {
    this.appUrl = getAppUrl();
  }

  async send(payload: NotificationPayload): Promise<boolean> {
    try {
      let recipientEmail = payload.recipientEmail;
      let prefs = DEFAULT_PREFS;

      if (payload.recipientId) {
        const user = await prisma.user.findUnique({
          where: { id: payload.recipientId },
        });

        if (!user) {
          console.warn(`Notification recipient not found: ${payload.recipientId}`);
          return false;
        }

        recipientEmail = user.email;

        const userPrefs = await prisma.notificationPreference.findUnique({
          where: { userId: payload.recipientId },
        });

        if (userPrefs) {
          prefs = userPrefs as unknown as UserNotificationPrefs;
        }
      }

      if (!recipientEmail) {
        console.warn('No recipient email for notification');
        return false;
      }

      if (!this.shouldSendNotification(payload.type, prefs)) {
        return false;
      }

      const { subject, html } = this.buildEmailContent(payload);

      await sendEmail({
        to: recipientEmail,
        subject,
        htmlContent: html,
      });

      return true;
    } catch (error) {
      console.error('Failed to send notification:', error);
      return false;
    }
  }

  async notifyWorkflowStarted(
    proofId: string,
    proofTitle: string,
    templateName: string,
    creatorId: string
  ): Promise<void> {
    await this.send({
      type: 'workflow_started',
      recipientId: creatorId,
      data: { proofId, proofTitle, templateName },
    });
  }

  async notifyStepAssigned(
    proofId: string,
    proofTitle: string,
    stepName: string,
    assigneeIds: string[]
  ): Promise<void> {
    for (const userId of assigneeIds) {
      await this.send({
        type: 'step_assigned',
        recipientId: userId,
        data: { proofId, proofTitle, stepName },
      });
    }
  }

  async notifyDecisionMade(
    proofId: string,
    proofTitle: string,
    decisionType: string,
    decidedByName: string,
    creatorId: string,
    note?: string
  ): Promise<void> {
    await this.send({
      type: 'decision_made',
      recipientId: creatorId,
      data: { proofId, proofTitle, decisionType, decidedByName, note },
    });
  }

  async notifyWorkflowCompleted(
    proofId: string,
    proofTitle: string,
    creatorId: string
  ): Promise<void> {
    await this.send({
      type: 'workflow_completed',
      recipientId: creatorId,
      data: { proofId, proofTitle },
    });
  }

  async notifyNeedsChanges(
    proofId: string,
    proofTitle: string,
    creatorId: string,
    note?: string
  ): Promise<void> {
    await this.send({
      type: 'needs_changes',
      recipientId: creatorId,
      data: { proofId, proofTitle, note },
    });
  }

  async notifyProofRejected(
    proofId: string,
    proofTitle: string,
    creatorId: string,
    note?: string
  ): Promise<void> {
    await this.send({
      type: 'proof_rejected',
      recipientId: creatorId,
      data: { proofId, proofTitle, note },
    });
  }

  async notifyCommentAdded(
    proofId: string,
    proofTitle: string,
    commenterName: string,
    commentBody: string,
    recipientIds: string[]
  ): Promise<void> {
    for (const userId of recipientIds) {
      await this.send({
        type: 'comment_added',
        recipientId: userId,
        data: { proofId, proofTitle, commenterName, commentBody },
      });
    }
  }

  async notifyMention(
    proofId: string,
    proofTitle: string,
    mentionerName: string,
    commentBody: string,
    mentionedUserId: string
  ): Promise<void> {
    await this.send({
      type: 'mention',
      recipientId: mentionedUserId,
      data: { proofId, proofTitle, mentionerName, commentBody },
    });
  }

  async notifySubmissionReceived(
    submissionId: string,
    title: string,
    submitterEmail: string,
    adminUserIds: string[]
  ): Promise<void> {
    await this.send({
      type: 'submission_received',
      recipientEmail: submitterEmail,
      data: { submissionId, title, isSubmitter: true },
    });

    for (const adminId of adminUserIds) {
      await this.send({
        type: 'submission_received',
        recipientId: adminId,
        data: { submissionId, title, submitterEmail, isSubmitter: false },
      });
    }
  }

  async notifySubmissionStatusChanged(
    submissionId: string,
    title: string,
    newStatus: string,
    submitterEmail: string
  ): Promise<void> {
    await this.send({
      type: 'submission_status_changed',
      recipientEmail: submitterEmail,
      data: { submissionId, title, newStatus },
    });
  }

  private shouldSendNotification(type: NotificationType, prefs: UserNotificationPrefs): boolean {
    switch (type) {
      case 'step_assigned':
        return prefs.onAssignment;
      case 'mention':
        return prefs.onMention;
      case 'workflow_started':
      case 'workflow_completed':
        return prefs.onStepChange;
      case 'decision_made':
      case 'needs_changes':
      case 'proof_rejected':
        return prefs.onDecision;
      case 'comment_added':
        return prefs.onMention;
      case 'submission_received':
      case 'submission_status_changed':
        return prefs.onSubmission;
      default:
        return true;
    }
  }

  private buildEmailContent(payload: NotificationPayload): { subject: string; html: string } {
    const { type, data } = payload;
    let subject = '';
    let content = '';
    let actionUrl = '';
    let actionText = '';

    switch (type) {
      case 'workflow_started':
        subject = `Workflow Started: ${data.proofTitle}`;
        content = `A review workflow "${data.templateName}" has been started for your proof "${data.proofTitle}".`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'View Proof';
        break;

      case 'step_assigned':
        subject = `Action Required: ${data.proofTitle}`;
        content = `You have been assigned to review "${data.proofTitle}" at the "${data.stepName}" step. Please review and make your decision.`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'Review Now';
        break;

      case 'decision_made':
        subject = `Decision Made: ${data.proofTitle}`;
        content = `${data.decidedByName} has ${(data.decisionType as string).toLowerCase()} "${data.proofTitle}".${data.note ? ` Note: ${data.note}` : ''}`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'View Details';
        break;

      case 'workflow_completed':
        subject = `Approved: ${data.proofTitle}`;
        content = `Great news! Your proof "${data.proofTitle}" has been approved and the workflow is complete.`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'View Proof';
        break;

      case 'needs_changes':
        subject = `Changes Requested: ${data.proofTitle}`;
        content = `Changes have been requested for your proof "${data.proofTitle}".${data.note ? ` Feedback: ${data.note}` : ''} Please upload a new version.`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'Upload New Version';
        break;

      case 'proof_rejected':
        subject = `Not Approved: ${data.proofTitle}`;
        content = `Your proof "${data.proofTitle}" was not approved.${data.note ? ` Reason: ${data.note}` : ''}`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'View Details';
        break;

      case 'comment_added':
        subject = `New Comment: ${data.proofTitle}`;
        content = `${data.commenterName} commented on "${data.proofTitle}": "${(data.commentBody as string).substring(0, 100)}${(data.commentBody as string).length > 100 ? '...' : ''}"`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'View Comment';
        break;

      case 'mention':
        subject = `You were mentioned: ${data.proofTitle}`;
        content = `${data.mentionerName} mentioned you in a comment on "${data.proofTitle}": "${(data.commentBody as string).substring(0, 100)}${(data.commentBody as string).length > 100 ? '...' : ''}"`;
        actionUrl = `${this.appUrl}/app/proofs/${data.proofId}`;
        actionText = 'View Comment';
        break;

      case 'submission_received':
        if (data.isSubmitter) {
          subject = `Submission Received: ${data.title}`;
          content = `Thank you for your submission "${data.title}". Our team will review it and get back to you soon. Your reference number is: ${data.submissionId}`;
          actionUrl = `${this.appUrl}/submit/status?ref=${data.submissionId}`;
          actionText = 'Track Status';
        } else {
          subject = `New External Submission: ${data.title}`;
          content = `A new submission "${data.title}" has been received from ${data.submitterEmail}. Please review at your earliest convenience.`;
          actionUrl = `${this.appUrl}/app/submissions`;
          actionText = 'Review Submission';
        }
        break;

      case 'submission_status_changed':
        subject = `Submission Update: ${data.title}`;
        content = `The status of your submission "${data.title}" has been updated to: ${data.newStatus}`;
        actionUrl = `${this.appUrl}/submit/status?ref=${data.submissionId}`;
        actionText = 'View Status';
        break;

      default:
        subject = 'PYRAX Notification';
        content = 'You have a new notification from PYRAX Proofing Hub.';
        actionUrl = this.appUrl;
        actionText = 'View';
    }

    const html = this.buildEmailTemplate(subject, content, actionUrl, actionText);
    return { subject, html };
  }

  private buildEmailTemplate(title: string, content: string, actionUrl: string, actionText: string): string {
    return `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="margin: 0; padding: 0; background-color: #1c1917; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background-color: #1c1917; padding: 40px 20px;">
    <tr>
      <td align="center">
        <table width="600" cellpadding="0" cellspacing="0" style="max-width: 600px;">
          <!-- Header -->
          <tr>
            <td style="text-align: center; padding-bottom: 32px;">
              <h1 style="margin: 0; font-size: 28px; font-weight: bold; background: linear-gradient(135deg, #FF5500 0%, #FF7700 50%, #FF9944 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">PYRAX</h1>
              <p style="margin: 4px 0 0 0; font-size: 12px; color: #78716c; text-transform: uppercase; letter-spacing: 2px;">Proofing Hub</p>
            </td>
          </tr>
          
          <!-- Content Card -->
          <tr>
            <td style="background-color: #292524; border-radius: 16px; padding: 32px; border: 1px solid #44403c;">
              <h2 style="margin: 0 0 16px 0; font-size: 20px; color: #fafaf9;">${title}</h2>
              <p style="margin: 0 0 24px 0; font-size: 16px; color: #a8a29e; line-height: 1.6;">${content}</p>
              
              <table cellpadding="0" cellspacing="0" style="margin-top: 24px;">
                <tr>
                  <td style="background: linear-gradient(135deg, #FF5500 0%, #FF7700 100%); border-radius: 8px;">
                    <a href="${actionUrl}" style="display: inline-block; padding: 12px 24px; color: #ffffff; text-decoration: none; font-weight: 600; font-size: 14px;">${actionText}</a>
                  </td>
                </tr>
              </table>
            </td>
          </tr>
          
          <!-- Footer -->
          <tr>
            <td style="text-align: center; padding-top: 32px;">
              <p style="margin: 0; font-size: 12px; color: #57534e;">
                © ${new Date().getFullYear()} PYRAX Blockchain. All rights reserved.
              </p>
              <p style="margin: 8px 0 0 0; font-size: 11px; color: #44403c;">
                You received this email because you have notifications enabled.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>
    `.trim();
  }
}

export const notificationService = new NotificationService();
