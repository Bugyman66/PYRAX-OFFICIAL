import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, DecisionType } from '@prisma/client';
import { workflowEngine } from '@/lib/workflow';

interface RouteParams {
  params: Promise<{ instanceId: string }>;
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { instanceId } = await params;

    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        proof: {
          include: {
            folder: {
              select: { departmentId: true },
            },
          },
        },
      },
    });

    if (!instance) {
      return NextResponse.json({ error: 'Workflow instance not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== instance.proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    const decisions = await prisma.decision.findMany({
      where: { workflowInstanceId: instanceId },
      include: {
        decidedBy: {
          select: { id: true, name: true, email: true },
        },
      },
      orderBy: { createdAt: 'asc' },
    });

    return NextResponse.json(decisions);
  } catch (error) {
    console.error('Get decisions error:', error);
    return NextResponse.json({ error: 'Failed to fetch decisions' }, { status: 500 });
  }
}

export async function POST(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { instanceId } = await params;
    const body = await request.json();
    const { decisionType, note } = body;

    if (!decisionType || !Object.values(DecisionType).includes(decisionType)) {
      return NextResponse.json({ 
        error: 'Valid decision type is required (Approve, NeedsChanges, Reject)' 
      }, { status: 400 });
    }

    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        proof: {
          include: {
            folder: {
              select: { departmentId: true },
            },
          },
        },
      },
    });

    if (!instance) {
      return NextResponse.json({ error: 'Workflow instance not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== instance.proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    const result = await workflowEngine.makeDecision(
      session.id,
      instanceId,
      decisionType as DecisionType,
      note
    );

    if (!result.success) {
      return NextResponse.json({ error: result.error }, { status: 400 });
    }

    const updatedStatus = await workflowEngine.getWorkflowStatus(instanceId);

    return NextResponse.json({
      success: true,
      workflowCompleted: result.workflowCompleted || false,
      stepAdvanced: result.stepAdvanced || false,
      needsChanges: result.needsChanges || false,
      rejected: result.rejected || false,
      currentStatus: updatedStatus,
    }, { status: 201 });
  } catch (error) {
    console.error('Create decision error:', error);
    const message = error instanceof Error ? error.message : 'Failed to create decision';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
