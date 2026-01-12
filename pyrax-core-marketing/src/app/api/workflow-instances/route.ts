import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, WorkflowStatus, Prisma } from '@prisma/client';
import { workflowEngine, WorkflowStep, WorkflowFinalRule } from '@/lib/workflow';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const proofId = searchParams.get('proofId');
    const status = searchParams.get('status') as WorkflowStatus | null;

    const where: Prisma.WorkflowInstanceWhereInput = {};

    if (proofId) {
      where.proofId = proofId;
    }

    if (status) {
      where.status = status;
    }

    if (session.role !== UserRole.OrgAdmin) {
      where.proof = {
        folder: {
          departmentId: session.departmentId || undefined,
        },
      };
    }

    const instances = await prisma.workflowInstance.findMany({
      where,
      include: {
        template: {
          select: { id: true, name: true, stepsJson: true },
        },
        proof: {
          select: { 
            id: true, 
            title: true, 
            status: true,
            folder: {
              select: { id: true, name: true, departmentId: true },
            },
          },
        },
        decisions: {
          include: {
            decidedBy: {
              select: { id: true, name: true, email: true },
            },
          },
          orderBy: { createdAt: 'desc' },
          take: 5,
        },
        _count: {
          select: { decisions: true },
        },
      },
      orderBy: { updatedAt: 'desc' },
    });

    const formattedInstances = instances.map(instance => {
      const steps = instance.template.stepsJson as unknown as WorkflowStep[];
      return {
        id: instance.id,
        status: instance.status,
        currentStepIndex: instance.currentStepIndex,
        currentStepName: steps[instance.currentStepIndex]?.name || null,
        totalSteps: steps.length,
        template: {
          id: instance.template.id,
          name: instance.template.name,
        },
        proof: instance.proof,
        recentDecisions: instance.decisions,
        totalDecisions: instance._count.decisions,
        createdAt: instance.createdAt,
        updatedAt: instance.updatedAt,
      };
    });

    return NextResponse.json(formattedInstances);
  } catch (error) {
    console.error('Get workflow instances error:', error);
    return NextResponse.json({ error: 'Failed to fetch instances' }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { proofId, templateId, dueDates } = body;

    if (!proofId || typeof proofId !== 'string') {
      return NextResponse.json({ error: 'Proof ID is required' }, { status: 400 });
    }

    if (!templateId || typeof templateId !== 'string') {
      return NextResponse.json({ error: 'Template ID is required' }, { status: 400 });
    }

    const proof = await prisma.proof.findUnique({
      where: { id: proofId },
      include: {
        folder: true,
        workflowInstance: true,
      },
    });

    if (!proof) {
      return NextResponse.json({ error: 'Proof not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    if (proof.workflowInstance) {
      return NextResponse.json({ 
        error: 'Proof already has an active workflow' 
      }, { status: 400 });
    }

    const template = await prisma.workflowTemplate.findUnique({
      where: { id: templateId },
    });

    if (!template) {
      return NextResponse.json({ error: 'Template not found' }, { status: 404 });
    }

    const instanceId = await workflowEngine.createInstance(proofId, templateId, dueDates);

    const instance = await prisma.workflowInstance.findUnique({
      where: { id: instanceId },
      include: {
        template: {
          select: { id: true, name: true, stepsJson: true, finalRuleJson: true },
        },
        proof: {
          select: { id: true, title: true, status: true },
        },
      },
    });

    if (!instance) {
      return NextResponse.json({ error: 'Failed to create instance' }, { status: 500 });
    }

    const steps = instance.template.stepsJson as unknown as WorkflowStep[];

    return NextResponse.json({
      id: instance.id,
      status: instance.status,
      currentStepIndex: instance.currentStepIndex,
      currentStepName: steps[instance.currentStepIndex]?.name || null,
      totalSteps: steps.length,
      template: {
        id: instance.template.id,
        name: instance.template.name,
        steps,
        finalRule: instance.template.finalRuleJson as unknown as WorkflowFinalRule,
      },
      proof: instance.proof,
      createdAt: instance.createdAt,
    }, { status: 201 });
  } catch (error) {
    console.error('Create workflow instance error:', error);
    const message = error instanceof Error ? error.message : 'Failed to create instance';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
