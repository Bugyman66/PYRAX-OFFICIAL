import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, Prisma } from '@prisma/client';
import { workflowEngine, WorkflowStep, WorkflowFinalRule } from '@/lib/workflow';

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

    const status = await workflowEngine.getWorkflowStatus(instanceId);

    if (!status) {
      return NextResponse.json({ error: 'Workflow instance not found' }, { status: 404 });
    }

    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        proof: {
          include: {
            folder: {
              select: { id: true, name: true, departmentId: true },
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

    const canDecide = await workflowEngine.canUserDecide(session.id, instanceId);

    const auditTrail = await workflowEngine.getAuditTrail('WorkflowInstance', instanceId);

    return NextResponse.json({
      ...status,
      proof: {
        id: instance.proof.id,
        title: instance.proof.title,
        status: instance.proof.status,
        folder: instance.proof.folder,
      },
      canDecide: canDecide.canDecide,
      canDecideReason: canDecide.reason,
      auditTrail,
    });
  } catch (error) {
    console.error('Get workflow instance error:', error);
    return NextResponse.json({ error: 'Failed to fetch instance' }, { status: 500 });
  }
}

export async function DELETE(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
    }

    const { instanceId } = await params;

    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        proof: {
          include: {
            folder: true,
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

    await workflowEngine.cancelWorkflow(instanceId, session.id);

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Cancel workflow instance error:', error);
    const message = error instanceof Error ? error.message : 'Failed to cancel instance';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
