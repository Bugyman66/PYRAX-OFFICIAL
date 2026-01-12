import { prisma } from './prisma';
import { ProofStatus, WorkflowStatus, UserRole } from '@prisma/client';
import { WorkflowStep } from './workflow';

export interface DashboardStats {
  totalProofs: number;
  inReview: number;
  approved: number;
  waitingOnMe: number;
  overdue: number;
  needsChanges: number;
}

export interface ProofListItem {
  id: string;
  title: string;
  status: ProofStatus;
  folder: { id: string; name: string };
  createdBy: { id: string; name: string | null; email: string };
  latestVersion?: {
    id: string;
    versionNumber: number;
    fileName: string;
    mimeType: string;
  };
  workflow?: {
    id: string;
    status: WorkflowStatus;
    currentStepIndex: number;
    currentStepName: string | null;
    templateName: string;
  };
  createdAt: Date;
  updatedAt: Date;
}

export interface ActivityItem {
  id: string;
  type: 'proof_created' | 'version_uploaded' | 'comment_added' | 'decision_made' | 'workflow_completed';
  title: string;
  description: string;
  actor: { id: string; name: string | null; email: string } | null;
  proofId?: string;
  proofTitle?: string;
  createdAt: Date;
}

export class DashboardService {
  async getStats(userId: string, departmentId: string | null, role: UserRole): Promise<DashboardStats> {
    const baseWhere = this.getBaseWhereClause(departmentId, role);

    const [totalProofs, inReview, approved, waitingOnMe, overdueCount, needsChangesCount] = await Promise.all([
      prisma.proof.count({ where: baseWhere }),
      prisma.proof.count({ where: { ...baseWhere, status: ProofStatus.InReview } }),
      prisma.proof.count({ where: { ...baseWhere, status: ProofStatus.Approved } }),
      this.getWaitingOnMeCount(userId, departmentId, role),
      this.getOverdueCount(departmentId, role),
      this.getNeedsChangesCount(userId, departmentId, role),
    ]);

    return {
      totalProofs,
      inReview,
      approved,
      waitingOnMe,
      overdue: overdueCount,
      needsChanges: needsChangesCount,
    };
  }

  async getWaitingOnMe(userId: string, departmentId: string | null, role: UserRole, limit = 10): Promise<ProofListItem[]> {
    const user = await prisma.user.findUnique({ where: { id: userId } });
    if (!user) return [];

    const instances = await prisma.workflowInstance.findMany({
      where: {
        status: WorkflowStatus.Active,
        proof: this.getBaseWhereClause(departmentId, role),
      },
      include: {
        template: true,
        proof: {
          include: {
            folder: { select: { id: true, name: true } },
            createdBy: { select: { id: true, name: true, email: true } },
            versions: {
              where: { kind: 'Source' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
              select: { id: true, versionNumber: true, fileName: true, mimeType: true },
            },
          },
        },
        decisions: true,
      },
      orderBy: { updatedAt: 'desc' },
    });

    const waitingOnMe: ProofListItem[] = [];

    for (const instance of instances) {
      const steps = instance.template.stepsJson as unknown as WorkflowStep[];
      const currentStep = steps[instance.currentStepIndex];
      
      if (!currentStep) continue;

      const hasRole = currentStep.allowedRoles.includes(user.role);
      const isAllowedUser = currentStep.allowedUsers.includes(userId);
      
      if (!hasRole && !isAllowedUser) continue;

      const existingDecision = instance.decisions.find(
        d => d.stepIndex === instance.currentStepIndex && d.decidedById === userId
      );
      
      if (existingDecision) continue;

      waitingOnMe.push({
        id: instance.proof.id,
        title: instance.proof.title,
        status: instance.proof.status,
        folder: instance.proof.folder,
        createdBy: instance.proof.createdBy,
        latestVersion: instance.proof.versions[0] || undefined,
        workflow: {
          id: instance.id,
          status: instance.status,
          currentStepIndex: instance.currentStepIndex,
          currentStepName: currentStep.name,
          templateName: instance.template.name,
        },
        createdAt: instance.proof.createdAt,
        updatedAt: instance.proof.updatedAt,
      });

      if (waitingOnMe.length >= limit) break;
    }

    return waitingOnMe;
  }

  async getOverdue(departmentId: string | null, role: UserRole, limit = 10): Promise<ProofListItem[]> {
    const sevenDaysAgo = new Date();
    sevenDaysAgo.setDate(sevenDaysAgo.getDate() - 7);

    const instances = await prisma.workflowInstance.findMany({
      where: {
        status: WorkflowStatus.Active,
        updatedAt: { lt: sevenDaysAgo },
        proof: this.getBaseWhereClause(departmentId, role),
      },
      include: {
        template: true,
        proof: {
          include: {
            folder: { select: { id: true, name: true } },
            createdBy: { select: { id: true, name: true, email: true } },
            versions: {
              where: { kind: 'Source' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
              select: { id: true, versionNumber: true, fileName: true, mimeType: true },
            },
          },
        },
      },
      orderBy: { updatedAt: 'asc' },
      take: limit,
    });

    return instances.map(instance => {
      const steps = instance.template.stepsJson as unknown as WorkflowStep[];
      const currentStep = steps[instance.currentStepIndex];

      return {
        id: instance.proof.id,
        title: instance.proof.title,
        status: instance.proof.status,
        folder: instance.proof.folder,
        createdBy: instance.proof.createdBy,
        latestVersion: instance.proof.versions[0] || undefined,
        workflow: {
          id: instance.id,
          status: instance.status,
          currentStepIndex: instance.currentStepIndex,
          currentStepName: currentStep?.name || null,
          templateName: instance.template.name,
        },
        createdAt: instance.proof.createdAt,
        updatedAt: instance.proof.updatedAt,
      };
    });
  }

  async getNeedsChanges(userId: string, departmentId: string | null, role: UserRole, limit = 10): Promise<ProofListItem[]> {
    const instances = await prisma.workflowInstance.findMany({
      where: {
        status: WorkflowStatus.Active,
        currentStepIndex: 0,
        decisions: {
          some: {
            decisionType: 'NeedsChanges',
          },
        },
        proof: {
          ...this.getBaseWhereClause(departmentId, role),
          createdById: userId,
        },
      },
      include: {
        template: true,
        proof: {
          include: {
            folder: { select: { id: true, name: true } },
            createdBy: { select: { id: true, name: true, email: true } },
            versions: {
              where: { kind: 'Source' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
              select: { id: true, versionNumber: true, fileName: true, mimeType: true },
            },
          },
        },
        decisions: {
          where: { decisionType: 'NeedsChanges' },
          orderBy: { createdAt: 'desc' },
          take: 1,
          include: {
            decidedBy: { select: { id: true, name: true, email: true } },
          },
        },
      },
      orderBy: { updatedAt: 'desc' },
      take: limit,
    });

    return instances.map(instance => {
      const steps = instance.template.stepsJson as unknown as WorkflowStep[];
      const currentStep = steps[instance.currentStepIndex];

      return {
        id: instance.proof.id,
        title: instance.proof.title,
        status: instance.proof.status,
        folder: instance.proof.folder,
        createdBy: instance.proof.createdBy,
        latestVersion: instance.proof.versions[0] || undefined,
        workflow: {
          id: instance.id,
          status: instance.status,
          currentStepIndex: instance.currentStepIndex,
          currentStepName: currentStep?.name || null,
          templateName: instance.template.name,
        },
        createdAt: instance.proof.createdAt,
        updatedAt: instance.proof.updatedAt,
      };
    });
  }

  async getRecentlyApproved(departmentId: string | null, role: UserRole, limit = 10): Promise<ProofListItem[]> {
    const proofs = await prisma.proof.findMany({
      where: {
        ...this.getBaseWhereClause(departmentId, role),
        status: ProofStatus.Approved,
      },
      include: {
        folder: { select: { id: true, name: true } },
        createdBy: { select: { id: true, name: true, email: true } },
        versions: {
          where: { kind: 'Source' },
          orderBy: { versionNumber: 'desc' },
          take: 1,
          select: { id: true, versionNumber: true, fileName: true, mimeType: true },
        },
        workflowInstance: {
          include: {
            template: { select: { name: true } },
          },
        },
      },
      orderBy: { updatedAt: 'desc' },
      take: limit,
    });

    return proofs.map(proof => ({
      id: proof.id,
      title: proof.title,
      status: proof.status,
      folder: proof.folder,
      createdBy: proof.createdBy,
      latestVersion: proof.versions[0] || undefined,
      workflow: proof.workflowInstance ? {
        id: proof.workflowInstance.id,
        status: proof.workflowInstance.status,
        currentStepIndex: proof.workflowInstance.currentStepIndex,
        currentStepName: null,
        templateName: proof.workflowInstance.template.name,
      } : undefined,
      createdAt: proof.createdAt,
      updatedAt: proof.updatedAt,
    }));
  }

  async getRecentActivity(departmentId: string | null, role: UserRole, limit = 20): Promise<ActivityItem[]> {
    const baseWhere = this.getBaseWhereClause(departmentId, role);

    const [recentProofs, recentVersions, recentComments, recentDecisions] = await Promise.all([
      prisma.proof.findMany({
        where: baseWhere,
        orderBy: { createdAt: 'desc' },
        take: 5,
        include: {
          createdBy: { select: { id: true, name: true, email: true } },
        },
      }),
      prisma.proofVersion.findMany({
        where: {
          proof: baseWhere,
        },
        orderBy: { createdAt: 'desc' },
        take: 5,
        include: {
          createdBy: { select: { id: true, name: true, email: true } },
          proof: { select: { id: true, title: true } },
        },
      }),
      prisma.comment.findMany({
        where: {
          proofVersion: {
            proof: baseWhere,
          },
        },
        orderBy: { createdAt: 'desc' },
        take: 5,
        include: {
          createdBy: { select: { id: true, name: true, email: true } },
          proofVersion: {
            include: {
              proof: { select: { id: true, title: true } },
            },
          },
        },
      }),
      prisma.decision.findMany({
        where: {
          workflowInstance: {
            proof: baseWhere,
          },
        },
        orderBy: { createdAt: 'desc' },
        take: 5,
        include: {
          decidedBy: { select: { id: true, name: true, email: true } },
          workflowInstance: {
            include: {
              proof: { select: { id: true, title: true } },
            },
          },
        },
      }),
    ]);

    const activities: ActivityItem[] = [];

    for (const proof of recentProofs) {
      activities.push({
        id: `proof-${proof.id}`,
        type: 'proof_created',
        title: 'New Proof Created',
        description: proof.title,
        actor: proof.createdBy,
        proofId: proof.id,
        proofTitle: proof.title,
        createdAt: proof.createdAt,
      });
    }

    for (const version of recentVersions) {
      activities.push({
        id: `version-${version.id}`,
        type: 'version_uploaded',
        title: 'New Version Uploaded',
        description: `Version ${version.versionNumber} of "${version.proof.title}"`,
        actor: version.createdBy,
        proofId: version.proof.id,
        proofTitle: version.proof.title,
        createdAt: version.createdAt,
      });
    }

    for (const comment of recentComments) {
      activities.push({
        id: `comment-${comment.id}`,
        type: 'comment_added',
        title: 'Comment Added',
        description: `On "${comment.proofVersion.proof.title}"`,
        actor: comment.createdBy,
        proofId: comment.proofVersion.proof.id,
        proofTitle: comment.proofVersion.proof.title,
        createdAt: comment.createdAt,
      });
    }

    for (const decision of recentDecisions) {
      const decisionLabel = decision.decisionType === 'Approve' ? 'Approved' :
                           decision.decisionType === 'NeedsChanges' ? 'Requested Changes' :
                           'Rejected';
      activities.push({
        id: `decision-${decision.id}`,
        type: 'decision_made',
        title: decisionLabel,
        description: `"${decision.workflowInstance.proof.title}"`,
        actor: decision.decidedBy,
        proofId: decision.workflowInstance.proof.id,
        proofTitle: decision.workflowInstance.proof.title,
        createdAt: decision.createdAt,
      });
    }

    activities.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());

    return activities.slice(0, limit);
  }

  private async getWaitingOnMeCount(userId: string, departmentId: string | null, role: UserRole): Promise<number> {
    const items = await this.getWaitingOnMe(userId, departmentId, role, 100);
    return items.length;
  }

  private async getOverdueCount(departmentId: string | null, role: UserRole): Promise<number> {
    const sevenDaysAgo = new Date();
    sevenDaysAgo.setDate(sevenDaysAgo.getDate() - 7);

    return prisma.workflowInstance.count({
      where: {
        status: WorkflowStatus.Active,
        updatedAt: { lt: sevenDaysAgo },
        proof: this.getBaseWhereClause(departmentId, role),
      },
    });
  }

  private async getNeedsChangesCount(userId: string, departmentId: string | null, role: UserRole): Promise<number> {
    return prisma.workflowInstance.count({
      where: {
        status: WorkflowStatus.Active,
        currentStepIndex: 0,
        decisions: {
          some: {
            decisionType: 'NeedsChanges',
          },
        },
        proof: {
          ...this.getBaseWhereClause(departmentId, role),
          createdById: userId,
        },
      },
    });
  }

  private getBaseWhereClause(departmentId: string | null, role: UserRole) {
    if (role === UserRole.OrgAdmin) {
      return {};
    }
    
    return {
      folder: {
        departmentId: departmentId || undefined,
      },
    };
  }
}

export const dashboardService = new DashboardService();
