import { prisma } from './prisma';
import { DecisionType, WorkflowStatus, UserRole, ProofStatus, Prisma } from '@prisma/client';
import { notificationService } from './notifications';

export interface WorkflowStep {
  name: string;
  allowedRoles: UserRole[];
  allowedUsers: string[];
  approvalRule: 'any' | 'all' | 'single';
}

export interface WorkflowFinalRule {
  type: 'single' | 'majority' | 'unanimous';
  description?: string;
}

export interface WorkflowTemplateConfig {
  steps: WorkflowStep[];
  finalRule: WorkflowFinalRule;
}

export interface CanDecideResult {
  canDecide: boolean;
  reason?: string;
  currentStep?: WorkflowStep;
  stepIndex?: number;
}

export interface DecisionResult {
  success: boolean;
  error?: string;
  workflowCompleted?: boolean;
  stepAdvanced?: boolean;
  needsChanges?: boolean;
  rejected?: boolean;
}

class WorkflowEngineError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 400
  ) {
    super(message);
    this.name = 'WorkflowEngineError';
  }
}

export class WorkflowEngine {
  async createInstance(proofId: string, templateId: string, dueDates?: Record<number, Date>): Promise<string> {
    const proof = await prisma.proof.findUnique({
      where: { id: proofId },
      include: { workflowInstance: true },
    });

    if (!proof) {
      throw new WorkflowEngineError('Proof not found', 'PROOF_NOT_FOUND', 404);
    }

    if (proof.workflowInstance) {
      throw new WorkflowEngineError('Proof already has an active workflow', 'WORKFLOW_EXISTS', 400);
    }

    const template = await prisma.workflowTemplate.findUnique({
      where: { id: templateId },
    });

    if (!template) {
      throw new WorkflowEngineError('Workflow template not found', 'TEMPLATE_NOT_FOUND', 404);
    }

    const instance = await prisma.workflowInstance.create({
      data: {
        proofId,
        templateId,
        currentStepIndex: 0,
        status: WorkflowStatus.Active,
        dueDatesJson: dueDates ? (dueDates as unknown as Prisma.InputJsonValue) : Prisma.JsonNull,
      },
    });

    await prisma.proof.update({
      where: { id: proofId },
      data: { status: ProofStatus.InReview },
    });

    await this.logAuditEvent('WorkflowInstance', instance.id, 'created', null, {
      proofId,
      templateId,
      templateName: template.name,
    });

    return instance.id;
  }

  async canUserDecide(userId: string, instanceId: string): Promise<CanDecideResult> {
    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        template: true,
        decisions: {
          orderBy: { createdAt: 'desc' },
        },
        proof: {
          include: {
            versions: {
              orderBy: { createdAt: 'desc' },
              take: 1,
            },
          },
        },
      },
    });

    if (!instance) {
      return { canDecide: false, reason: 'Workflow instance not found' };
    }

    if (instance.status !== WorkflowStatus.Active) {
      return { canDecide: false, reason: 'Workflow is not active' };
    }

    const user = await prisma.user.findUnique({
      where: { id: userId },
    });

    if (!user) {
      return { canDecide: false, reason: 'User not found' };
    }

    const steps = instance.template.stepsJson as unknown as WorkflowStep[];
    const currentStep = steps[instance.currentStepIndex];

    if (!currentStep) {
      return { canDecide: false, reason: 'Invalid step index' };
    }

    // OrgAdmin can always approve any step
    const isOrgAdmin = user.role === UserRole.OrgAdmin;
    // Cast allowedRoles to string array and user.role to string for proper comparison
    const allowedRolesStr = currentStep.allowedRoles.map(r => String(r));
    const hasRole = allowedRolesStr.includes(String(user.role));
    const isAllowedUser = currentStep.allowedUsers.includes(userId);

    console.log('[Workflow Debug] canUserDecide:', {
      userId,
      userRole: user.role,
      currentStepIndex: instance.currentStepIndex,
      currentStepName: currentStep.name,
      allowedRoles: allowedRolesStr,
      isOrgAdmin,
      hasRole,
      isAllowedUser,
    });

    if (!isOrgAdmin && !hasRole && !isAllowedUser) {
      return { 
        canDecide: false, 
        reason: `User not authorized for this step (role: ${user.role}, allowed: ${allowedRolesStr.join(', ')})`,
        currentStep,
        stepIndex: instance.currentStepIndex,
      };
    }

    // Get all decisions for this step by this user
    const userDecisionsOnStep = instance.decisions.filter(
      d => d.stepIndex === instance.currentStepIndex && d.decidedById === userId
    );

    if (userDecisionsOnStep.length > 0) {
      // Check if there was a "NeedsChanges" decision that triggered a re-review cycle
      const lastNeedsChanges = instance.decisions.find(
        d => d.decisionType === DecisionType.NeedsChanges
      );
      
      // Get the latest version upload time
      const latestVersion = instance.proof.versions[0];
      
      // If there was a NeedsChanges decision and a new version was uploaded after it,
      // allow the user to re-review
      if (lastNeedsChanges && latestVersion) {
        const needsChangesTime = new Date(lastNeedsChanges.createdAt).getTime();
        const latestVersionTime = new Date(latestVersion.createdAt).getTime();
        
        // Check if user's last decision on this step was before the new version
        const userLastDecision = userDecisionsOnStep[0]; // Already sorted by createdAt desc
        const userLastDecisionTime = new Date(userLastDecision.createdAt).getTime();
        
        if (latestVersionTime > needsChangesTime && latestVersionTime > userLastDecisionTime) {
          // New version was uploaded after NeedsChanges and after user's last decision
          // Allow re-review
          return { 
            canDecide: true, 
            currentStep,
            stepIndex: instance.currentStepIndex,
          };
        }
      }

      return { 
        canDecide: false, 
        reason: 'User has already made a decision on this step',
        currentStep,
        stepIndex: instance.currentStepIndex,
      };
    }

    return { 
      canDecide: true, 
      currentStep,
      stepIndex: instance.currentStepIndex,
    };
  }

  async makeDecision(
    userId: string,
    instanceId: string,
    decisionType: DecisionType,
    note?: string
  ): Promise<DecisionResult> {
    const canDecideResult = await this.canUserDecide(userId, instanceId);

    if (!canDecideResult.canDecide) {
      return { success: false, error: canDecideResult.reason };
    }

    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        template: true,
        decisions: true,
        proof: true,
      },
    });

    if (!instance) {
      return { success: false, error: 'Workflow instance not found' };
    }

    const decision = await prisma.decision.create({
      data: {
        workflowInstanceId: instanceId,
        stepIndex: instance.currentStepIndex,
        decisionType,
        note,
        decidedById: userId,
      },
    });

    await this.logAuditEvent('Decision', decision.id, 'created', userId, {
      workflowInstanceId: instanceId,
      proofId: instance.proofId,
      stepIndex: instance.currentStepIndex,
      stepName: canDecideResult.currentStep?.name,
      decisionType,
      note,
    });

    if (decisionType === DecisionType.Reject) {
      await this.rejectWorkflow(instanceId, userId, note);
      return { success: true, rejected: true };
    }

    if (decisionType === DecisionType.NeedsChanges) {
      await this.returnForChanges(instanceId, userId, note);
      return { success: true, needsChanges: true };
    }

    // Re-fetch instance to include the new decision we just created
    const updatedInstance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        template: true,
        decisions: true,
      },
    });

    if (!updatedInstance) {
      return { success: false, error: 'Workflow instance not found' };
    }

    const shouldAdvance = await this.checkApprovalRule(updatedInstance, canDecideResult.currentStep!);

    console.log('[Workflow Debug] makeDecision:', {
      instanceId,
      currentStepIndex: updatedInstance.currentStepIndex,
      totalDecisions: updatedInstance.decisions.length,
      shouldAdvance,
    });

    if (shouldAdvance) {
      const steps = updatedInstance.template.stepsJson as unknown as WorkflowStep[];
      const isLastStep = updatedInstance.currentStepIndex >= steps.length - 1;

      console.log('[Workflow Debug] Advancing:', {
        totalSteps: steps.length,
        currentStepIndex: updatedInstance.currentStepIndex,
        isLastStep,
      });

      if (isLastStep) {
        await this.completeWorkflow(instanceId, userId);
        return { success: true, workflowCompleted: true };
      } else {
        await this.advanceStep(instanceId, userId);
        return { success: true, stepAdvanced: true };
      }
    }

    return { success: true };
  }

  private async checkApprovalRule(
    instance: {
      id: string;
      currentStepIndex: number;
      decisions: { stepIndex: number; decisionType: DecisionType; decidedById: string }[];
    },
    currentStep: WorkflowStep
  ): Promise<boolean> {
    // Compare as strings to handle potential enum/string mismatch from database
    const stepDecisions = instance.decisions.filter(
      d => d.stepIndex === instance.currentStepIndex && String(d.decisionType) === String(DecisionType.Approve)
    );

    console.log('[Workflow Debug] checkApprovalRule:', {
      currentStepIndex: instance.currentStepIndex,
      approvalRule: currentStep.approvalRule,
      approvalCount: stepDecisions.length,
      allDecisions: instance.decisions.map(d => ({ stepIndex: d.stepIndex, type: d.decisionType, typeStr: String(d.decisionType) })),
    });

    switch (currentStep.approvalRule) {
      case 'any':
        return stepDecisions.length >= 1;

      case 'single':
        return stepDecisions.length >= 1;

      case 'all':
        const requiredUsers = currentStep.allowedUsers.length > 0
          ? currentStep.allowedUsers.length
          : await this.countUsersWithRoles(currentStep.allowedRoles);
        return stepDecisions.length >= requiredUsers;

      default:
        return false;
    }
  }

  private async countUsersWithRoles(roles: UserRole[]): Promise<number> {
    const count = await prisma.user.count({
      where: {
        role: { in: roles },
        isInternal: true,
      },
    });
    return Math.max(count, 1);
  }

  private async getStepAssignees(step: WorkflowStep): Promise<string[]> {
    const userIds: string[] = [...step.allowedUsers];
    
    if (step.allowedRoles.length > 0) {
      const users = await prisma.user.findMany({
        where: {
          role: { in: step.allowedRoles },
          isInternal: true,
        },
        select: { id: true },
      });
      userIds.push(...users.map(u => u.id));
    }
    
    return Array.from(new Set(userIds));
  }

  private async advanceStep(instanceId: string, actorId: string): Promise<void> {
    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: { template: true, proof: true },
    });

    if (!instance) return;

    const steps = instance.template.stepsJson as unknown as WorkflowStep[];
    const newStepIndex = instance.currentStepIndex + 1;
    const newStep = steps[newStepIndex];

    console.log('[Workflow Debug] advanceStep:', {
      instanceId,
      previousStepIndex: instance.currentStepIndex,
      newStepIndex,
      newStepName: newStep?.name,
    });

    await prisma.workflowInstance.update({
      where: { id: instanceId },
      data: { currentStepIndex: newStepIndex },
    });

    await this.logAuditEvent('WorkflowInstance', instanceId, 'step_advanced', actorId, {
      previousStep: instance.currentStepIndex,
      previousStepName: steps[instance.currentStepIndex]?.name,
      newStep: newStepIndex,
      newStepName: newStep?.name,
    });

    // Notify assignees of new step
    if (newStep) {
      const assigneeIds = await this.getStepAssignees(newStep);
      notificationService.notifyStepAssigned(
        instance.proofId,
        instance.proof.title,
        newStep.name,
        assigneeIds
      ).catch(err => console.error('Failed to send step notification:', err));
    }
  }

  private async completeWorkflow(instanceId: string, actorId: string): Promise<void> {
    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: { proof: true },
    });

    if (!instance) return;

    await prisma.$transaction([
      prisma.workflowInstance.update({
        where: { id: instanceId },
        data: { status: WorkflowStatus.Completed },
      }),
      prisma.proof.update({
        where: { id: instance.proofId },
        data: { status: ProofStatus.Approved },
      }),
    ]);

    await this.logAuditEvent('WorkflowInstance', instanceId, 'completed', actorId, {
      proofId: instance.proofId,
      proofTitle: instance.proof.title,
    });

    // Notify proof creator of approval
    notificationService.notifyWorkflowCompleted(
      instance.proofId,
      instance.proof.title,
      instance.proof.createdById
    ).catch(err => console.error('Failed to send completion notification:', err));
  }

  private async rejectWorkflow(instanceId: string, actorId: string, note?: string): Promise<void> {
    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: { proof: true },
    });

    if (!instance) return;

    await prisma.$transaction([
      prisma.workflowInstance.update({
        where: { id: instanceId },
        data: { status: WorkflowStatus.Cancelled },
      }),
      prisma.proof.update({
        where: { id: instance.proofId },
        data: { status: ProofStatus.Rejected },
      }),
    ]);

    await this.logAuditEvent('WorkflowInstance', instanceId, 'rejected', actorId, {
      proofId: instance.proofId,
      proofTitle: instance.proof.title,
    });

    // Notify proof creator of rejection
    notificationService.notifyProofRejected(
      instance.proofId,
      instance.proof.title,
      instance.proof.createdById,
      note
    ).catch(err => console.error('Failed to send rejection notification:', err));
  }

  private async returnForChanges(instanceId: string, actorId: string, note?: string): Promise<void> {
    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: { proof: true, template: true },
    });

    if (!instance) return;

    const steps = instance.template.stepsJson as unknown as WorkflowStep[];

    await prisma.workflowInstance.update({
      where: { id: instanceId },
      data: { currentStepIndex: 0 },
    });

    await this.logAuditEvent('WorkflowInstance', instanceId, 'returned_for_changes', actorId, {
      proofId: instance.proofId,
      proofTitle: instance.proof.title,
      previousStep: instance.currentStepIndex,
      previousStepName: steps[instance.currentStepIndex]?.name,
    });

    // Notify proof creator that changes are needed
    notificationService.notifyNeedsChanges(
      instance.proofId,
      instance.proof.title,
      instance.proof.createdById,
      note
    ).catch(err => console.error('Failed to send needs changes notification:', err));
  }

  async cancelWorkflow(instanceId: string, actorId: string): Promise<void> {
    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: { proof: true },
    });

    if (!instance) {
      throw new WorkflowEngineError('Workflow instance not found', 'INSTANCE_NOT_FOUND', 404);
    }

    if (instance.status !== WorkflowStatus.Active) {
      throw new WorkflowEngineError('Workflow is not active', 'WORKFLOW_NOT_ACTIVE', 400);
    }

    await prisma.$transaction([
      prisma.workflowInstance.update({
        where: { id: instanceId },
        data: { status: WorkflowStatus.Cancelled },
      }),
      prisma.proof.update({
        where: { id: instance.proofId },
        data: { status: ProofStatus.Draft },
      }),
    ]);

    await this.logAuditEvent('WorkflowInstance', instanceId, 'cancelled', actorId, {
      proofId: instance.proofId,
      proofTitle: instance.proof.title,
    });
  }

  async getWorkflowStatus(instanceId: string): Promise<{
    instance: {
      id: string;
      status: WorkflowStatus;
      currentStepIndex: number;
      createdAt: Date;
    };
    template: {
      id: string;
      name: string;
      steps: WorkflowStep[];
    };
    decisions: {
      id: string;
      stepIndex: number;
      decisionType: DecisionType;
      note: string | null;
      decidedBy: { id: string; name: string | null; email: string };
      createdAt: Date;
    }[];
    currentStep: WorkflowStep | null;
    progress: {
      completedSteps: number;
      totalSteps: number;
      percentComplete: number;
    };
  } | null> {
    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        template: true,
        decisions: {
          include: {
            decidedBy: {
              select: { id: true, name: true, email: true },
            },
          },
          orderBy: { createdAt: 'asc' },
        },
      },
    });

    if (!instance) return null;

    const steps = instance.template.stepsJson as unknown as WorkflowStep[];
    const currentStep = steps[instance.currentStepIndex] || null;

    return {
      instance: {
        id: instance.id,
        status: instance.status,
        currentStepIndex: instance.currentStepIndex,
        createdAt: instance.createdAt,
      },
      template: {
        id: instance.template.id,
        name: instance.template.name,
        steps,
      },
      decisions: instance.decisions.map(d => ({
        id: d.id,
        stepIndex: d.stepIndex,
        decisionType: d.decisionType,
        note: d.note,
        decidedBy: d.decidedBy,
        createdAt: d.createdAt,
      })),
      currentStep,
      progress: {
        completedSteps: instance.status === WorkflowStatus.Completed 
          ? steps.length 
          : instance.currentStepIndex,
        totalSteps: steps.length,
        percentComplete: instance.status === WorkflowStatus.Completed
          ? 100
          : Math.round((instance.currentStepIndex / steps.length) * 100),
      },
    };
  }

  async getAuditTrail(entityType: string, entityId: string): Promise<{
    id: string;
    action: string;
    actor: { id: string; name: string | null; email: string } | null;
    payload: Record<string, unknown> | null;
    createdAt: Date;
  }[]> {
    const events = await prisma.auditEvent.findMany({
      where: { entityType, entityId },
      include: {
        actor: {
          select: { id: true, name: true, email: true },
        },
      },
      orderBy: { createdAt: 'desc' },
    });

    return events.map(e => ({
      id: e.id,
      action: e.action,
      actor: e.actor,
      payload: e.payloadJson as Record<string, unknown> | null,
      createdAt: e.createdAt,
    }));
  }

  private async logAuditEvent(
    entityType: string,
    entityId: string,
    action: string,
    actorId: string | null,
    payload?: Record<string, unknown>
  ): Promise<void> {
    await prisma.auditEvent.create({
      data: {
        entityType,
        entityId,
        action,
        actorId,
        payloadJson: payload ? (payload as unknown as Prisma.InputJsonValue) : Prisma.JsonNull,
      },
    });
  }
}

export const workflowEngine = new WorkflowEngine();
