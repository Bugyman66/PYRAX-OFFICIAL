import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole, WorkflowStatus } from '@prisma/client';

export const dynamic = 'force-dynamic';

interface WorkflowStep {
  name: string;
  allowedRoles: string[];
  allowedUsers: string[];
  approvalRule: 'any' | 'all';
}

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const type = searchParams.get('type') || 'pending';

    if (type === 'pending') {
      // Get all active workflow instances
      const activeInstances = await prisma.workflowInstance.findMany({
        where: { status: WorkflowStatus.Active },
        include: {
          proof: {
            include: {
              folder: {
                include: { department: true }
              },
              createdBy: { select: { id: true, name: true, email: true } },
              versions: {
                orderBy: { versionNumber: 'desc' },
                take: 1,
                select: { id: true, versionNumber: true, mimeType: true }
              }
            }
          },
          template: true,
          decisions: {
            where: { decidedById: session.id },
            select: { stepIndex: true }
          }
        },
        orderBy: { updatedAt: 'desc' }
      });

      // Filter to only instances where user can decide on current step
      const pendingReviews = activeInstances.filter(instance => {
        const steps = instance.template.stepsJson as unknown as WorkflowStep[];
        const currentStep = steps[instance.currentStepIndex];
        
        if (!currentStep) return false;

        // Check if user already decided on this step
        const alreadyDecided = instance.decisions.some(
          (d: { stepIndex: number }) => d.stepIndex === instance.currentStepIndex
        );
        if (alreadyDecided) return false;

        // Check if user's role is allowed
        const roleAllowed = currentStep.allowedRoles.includes(session.role);
        
        // Check if user is specifically allowed
        const userAllowed = currentStep.allowedUsers.includes(session.id);

        // OrgAdmin can access all
        if (session.role === UserRole.OrgAdmin) return true;

        return roleAllowed || userAllowed;
      }).map(instance => {
        const steps = instance.template.stepsJson as unknown as WorkflowStep[];
        const currentStep = steps[instance.currentStepIndex];
        
        return {
          id: instance.id,
          proofId: instance.proof.id,
          proofTitle: instance.proof.title,
          proofDescription: instance.proof.description,
          proofStatus: instance.proof.status,
          folder: instance.proof.folder,
          createdBy: instance.proof.createdBy,
          latestVersion: instance.proof.versions[0] || null,
          currentStep: {
            index: instance.currentStepIndex,
            name: currentStep?.name || 'Unknown',
            totalSteps: steps.length
          },
          workflowName: instance.template.name,
          createdAt: instance.createdAt,
          updatedAt: instance.updatedAt
        };
      });

      return NextResponse.json({ reviews: pendingReviews });
    }

    if (type === 'completed') {
      // Get decisions made by this user with related data
      const userDecisions = await prisma.decision.findMany({
        where: { decidedById: session.id },
        include: {
          workflowInstance: {
            include: {
              proof: {
                include: {
                  folder: {
                    include: { department: true }
                  },
                  createdBy: { select: { id: true, name: true, email: true } },
                  versions: {
                    orderBy: { versionNumber: 'desc' },
                    take: 1,
                    select: { id: true, versionNumber: true, mimeType: true }
                  }
                }
              },
              template: true
            }
          }
        },
        orderBy: { createdAt: 'desc' },
        take: 50
      });

      const completedReviews = userDecisions.map(decision => {
        const steps = decision.workflowInstance.template.stepsJson as unknown as WorkflowStep[];
        const stepName = steps[decision.stepIndex]?.name || 'Unknown';

        return {
          id: decision.id,
          proofId: decision.workflowInstance.proof.id,
          proofTitle: decision.workflowInstance.proof.title,
          proofDescription: decision.workflowInstance.proof.description,
          proofStatus: decision.workflowInstance.proof.status,
          folder: decision.workflowInstance.proof.folder,
          createdBy: decision.workflowInstance.proof.createdBy,
          latestVersion: decision.workflowInstance.proof.versions[0] || null,
          decision: {
            type: decision.decisionType,
            note: decision.note,
            stepName,
            createdAt: decision.createdAt
          },
          workflowName: decision.workflowInstance.template.name,
          workflowStatus: decision.workflowInstance.status
        };
      });

      return NextResponse.json({ reviews: completedReviews });
    }

    return NextResponse.json({ error: 'Invalid type parameter' }, { status: 400 });
  } catch (error) {
    console.error('Get reviews error:', error);
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    return NextResponse.json(
      { error: `Failed to fetch reviews: ${errorMessage}` },
      { status: 500 }
    );
  }
}
