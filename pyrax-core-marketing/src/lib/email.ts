const BREVO_API_KEY = process.env.BREVO_API_KEY;
const SENDER_EMAIL = process.env.BREVO_SENDER_EMAIL || 'noreply@pyrax.org';
const SENDER_NAME = process.env.BREVO_SENDER_NAME || 'PYRAX Proofing Hub';

export function getAppUrl(): string {
  const env = process.env.NODE_ENV;
  
  if (env === 'development') {
    return 'http://localhost:3000';
  }
  
  // For testnet/mainnet (production), use the configured URL or default
  return process.env.NEXT_PUBLIC_APP_URL || 'https://marketing.pyrax.org';
}

interface EmailParams {
  to: string;
  subject: string;
  htmlContent: string;
  textContent?: string;
}

export async function sendEmail(params: EmailParams): Promise<boolean> {
  if (!BREVO_API_KEY) {
    console.error('BREVO_API_KEY not configured');
    return false;
  }

  try {
    const response = await fetch('https://api.brevo.com/v3/smtp/email', {
      method: 'POST',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'api-key': BREVO_API_KEY,
      },
      body: JSON.stringify({
        sender: { name: SENDER_NAME, email: SENDER_EMAIL },
        to: [{ email: params.to }],
        subject: params.subject,
        htmlContent: params.htmlContent,
        textContent: params.textContent,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      console.error('Brevo email error:', error);
      return false;
    }

    return true;
  } catch (error) {
    console.error('Email send error:', error);
    return false;
  }
}

export async function sendMagicLinkEmail(email: string, token: string): Promise<boolean> {
  const appUrl = getAppUrl();
  const magicLink = `${appUrl}/auth/verify?token=${token}`;
  
  console.log(`[Email] Sending magic link to ${email}`);
  console.log(`[Email] Magic link URL: ${magicLink}`);
  
  const htmlContent = `
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
    </head>
    <body style="margin: 0; padding: 0; background-color: #0c0a09; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
      <table width="100%" cellpadding="0" cellspacing="0" style="background-color: #0c0a09; padding: 40px 20px;">
        <tr>
          <td align="center">
            <table width="600" cellpadding="0" cellspacing="0" style="background-color: #1c1917; border-radius: 16px; overflow: hidden; box-shadow: 0 4px 24px rgba(0, 0, 0, 0.4);">
              <!-- Header with PYRAX branding -->
              <tr>
                <td style="padding: 48px 40px; text-align: center; background: linear-gradient(135deg, #1c1917 0%, #292524 100%); border-bottom: 1px solid #44403c;">
                  <div style="display: inline-block; margin-bottom: 8px;">
                    <span style="font-size: 36px; font-weight: 800; background: linear-gradient(135deg, #f97316 0%, #ea580c 50%, #c2410c 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">PYRAX</span>
                  </div>
                  <p style="color: #a8a29e; margin: 0; font-size: 14px; letter-spacing: 2px; text-transform: uppercase;">Proofing Hub</p>
                </td>
              </tr>
              <!-- Main content -->
              <tr>
                <td style="padding: 48px 40px;">
                  <h2 style="color: #fafaf9; margin: 0 0 16px 0; font-size: 26px; font-weight: 600;">Sign In to Your Account</h2>
                  <p style="color: #a8a29e; margin: 0 0 32px 0; font-size: 16px; line-height: 1.7;">
                    Click the button below to securely sign in to PYRAX Proofing Hub. This link will expire in <strong style="color: #fafaf9;">15 minutes</strong>.
                  </p>
                  <table cellpadding="0" cellspacing="0" style="margin: 0 0 32px 0;">
                    <tr>
                      <td style="border-radius: 10px; background: linear-gradient(135deg, #f97316 0%, #ea580c 50%, #c2410c 100%); box-shadow: 0 4px 14px rgba(249, 115, 22, 0.4);">
                        <a href="${magicLink}" style="display: inline-block; color: #ffffff; text-decoration: none; padding: 16px 36px; font-size: 16px; font-weight: 600; letter-spacing: 0.5px;">
                          Sign In Now →
                        </a>
                      </td>
                    </tr>
                  </table>
                  <p style="color: #78716c; margin: 0; font-size: 14px; line-height: 1.6;">
                    If you didn't request this email, you can safely ignore it.
                  </p>
                </td>
              </tr>
              <!-- Link fallback -->
              <tr>
                <td style="padding: 24px 40px; background-color: #292524; border-top: 1px solid #44403c;">
                  <p style="color: #78716c; margin: 0 0 8px 0; font-size: 12px;">
                    If the button doesn't work, copy and paste this link:
                  </p>
                  <a href="${magicLink}" style="color: #f97316; font-size: 12px; word-break: break-all; text-decoration: none;">${magicLink}</a>
                </td>
              </tr>
              <!-- Footer -->
              <tr>
                <td style="padding: 24px 40px; background-color: #0c0a09; text-align: center; border-top: 1px solid #1c1917;">
                  <p style="color: #57534e; margin: 0; font-size: 11px;">
                    © 2026 PYRAX Blockchain. All rights reserved.
                  </p>
                  <p style="color: #44403c; margin: 8px 0 0 0; font-size: 10px;">
                    This is an automated message from PYRAX Proofing Hub.
                  </p>
                </td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    </body>
    </html>
  `;

  const textContent = `
PYRAX Proofing Hub - Sign In

Click the link below to sign in to your account:
${magicLink}

This link will expire in 15 minutes.

If you didn't request this email, you can safely ignore it.

---
© 2026 PYRAX Blockchain. All rights reserved.
  `;

  return sendEmail({
    to: email,
    subject: '🔐 Sign in to PYRAX Proofing Hub',
    htmlContent,
    textContent,
  });
}

export async function sendNotificationEmail(
  to: string,
  type: 'assignment' | 'mention' | 'step_change' | 'decision' | 'approval' | 'submission',
  data: Record<string, string>
): Promise<boolean> {
  const templates: Record<string, { subject: string; title: string; message: string }> = {
    assignment: {
      subject: `New Assignment: ${data.proofTitle}`,
      title: 'You have a new assignment',
      message: `You have been assigned to review "${data.proofTitle}".`,
    },
    mention: {
      subject: `You were mentioned in ${data.proofTitle}`,
      title: 'You were mentioned',
      message: `${data.mentionedBy} mentioned you in a comment on "${data.proofTitle}".`,
    },
    step_change: {
      subject: `Workflow Update: ${data.proofTitle}`,
      title: 'Workflow step changed',
      message: `"${data.proofTitle}" has moved to step: ${data.stepName}.`,
    },
    decision: {
      subject: `Decision Made: ${data.proofTitle}`,
      title: 'A decision was made',
      message: `${data.decidedBy} marked "${data.proofTitle}" as ${data.decision}.`,
    },
    approval: {
      subject: `Approved: ${data.proofTitle}`,
      title: 'Proof Approved',
      message: `"${data.proofTitle}" has been fully approved and is ready for publication.`,
    },
    submission: {
      subject: 'Submission Received',
      title: 'Your submission was received',
      message: `Thank you for your submission "${data.title}". Our team will review it shortly.`,
    },
  };

  const template = templates[type];
  if (!template) return false;

  const appUrl = getAppUrl();
  const linkUrl = data.link ? (data.link.startsWith('http') ? data.link : `${appUrl}${data.link}`) : null;
  
  const htmlContent = `
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
    </head>
    <body style="margin: 0; padding: 0; background-color: #0c0a09; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
      <table width="100%" cellpadding="0" cellspacing="0" style="background-color: #0c0a09; padding: 40px 20px;">
        <tr>
          <td align="center">
            <table width="600" cellpadding="0" cellspacing="0" style="background-color: #1c1917; border-radius: 16px; overflow: hidden; box-shadow: 0 4px 24px rgba(0, 0, 0, 0.4);">
              <!-- Header with PYRAX branding -->
              <tr>
                <td style="padding: 40px; text-align: center; background: linear-gradient(135deg, #1c1917 0%, #292524 100%); border-bottom: 1px solid #44403c;">
                  <div style="display: inline-block; margin-bottom: 8px;">
                    <span style="font-size: 32px; font-weight: 800; background: linear-gradient(135deg, #f97316 0%, #ea580c 50%, #c2410c 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">PYRAX</span>
                  </div>
                  <p style="color: #a8a29e; margin: 0; font-size: 13px; letter-spacing: 2px; text-transform: uppercase;">Proofing Hub</p>
                </td>
              </tr>
              <!-- Main content -->
              <tr>
                <td style="padding: 40px;">
                  <h2 style="color: #fafaf9; margin: 0 0 16px 0; font-size: 24px; font-weight: 600;">${template.title}</h2>
                  <p style="color: #a8a29e; margin: 0 0 32px 0; font-size: 16px; line-height: 1.7;">
                    ${template.message}
                  </p>
                  ${linkUrl ? `
                  <table cellpadding="0" cellspacing="0">
                    <tr>
                      <td style="border-radius: 10px; background: linear-gradient(135deg, #f97316 0%, #ea580c 50%, #c2410c 100%); box-shadow: 0 4px 14px rgba(249, 115, 22, 0.4);">
                        <a href="${linkUrl}" style="display: inline-block; color: #ffffff; text-decoration: none; padding: 14px 32px; font-size: 15px; font-weight: 600;">
                          View Details →
                        </a>
                      </td>
                    </tr>
                  </table>
                  ` : ''}
                </td>
              </tr>
              <!-- Footer -->
              <tr>
                <td style="padding: 24px 40px; background-color: #0c0a09; text-align: center; border-top: 1px solid #1c1917;">
                  <p style="color: #57534e; margin: 0; font-size: 11px;">
                    © 2026 PYRAX Blockchain. All rights reserved.
                  </p>
                  <p style="color: #44403c; margin: 8px 0 0 0; font-size: 10px;">
                    This is an automated notification from PYRAX Proofing Hub.
                  </p>
                </td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    </body>
    </html>
  `;

  return sendEmail({
    to,
    subject: template.subject,
    htmlContent,
  });
}
