import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, Prisma } from '@prisma/client';
import { WorkflowStep, WorkflowFinalRule } from '@/lib/workflow';

interface RouteParams {
  params: Promise<{ templateId: string }>;
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { templateId } = await params;

    const template = await prisma.workflowTemplate.findUnique({
      where: { id: templateId },
      include: {
        department: {
          select: { id: true, name: true },
        },
        _count: {
          select: { instances: true },
        },
      },
    });

    if (!template) {
      return NextResponse.json({ error: 'Template not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== template.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    return NextResponse.json({
      id: template.id,
      name: template.name,
      description: template.description,
      department: template.department,
      steps: template.stepsJson as unknown as WorkflowStep[],
      finalRule: template.finalRuleJson as unknown as WorkflowFinalRule,
      isDefault: template.isDefault,
      instanceCount: template._count.instances,
      createdAt: template.createdAt,
      updatedAt: template.updatedAt,
    });
  } catch (error) {
    console.error('Get workflow template error:', error);
    return NextResponse.json({ error: 'Failed to fetch template' }, { status: 500 });
  }
}

export async function PATCH(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
    }

    const { templateId } = await params;

    const template = await prisma.workflowTemplate.findUnique({
      where: { id: templateId },
      include: {
        _count: { select: { instances: true } },
      },
    });

    if (!template) {
      return NextResponse.json({ error: 'Template not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== template.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    const body = await request.json();
    const { name, description, steps, finalRule, isDefault } = body;

    const updateData: Prisma.WorkflowTemplateUpdateInput = {};

    if (name !== undefined) {
      if (typeof name !== 'string' || name.trim().length === 0) {
        return NextResponse.json({ error: 'Name cannot be empty' }, { status: 400 });
      }
      updateData.name = name.trim();
    }

    if (description !== undefined) {
      updateData.description = description?.trim() || null;
    }

    if (steps !== undefined) {
      if (!Array.isArray(steps) || steps.length === 0) {
        return NextResponse.json({ error: 'At least one step is required' }, { status: 400 });
      }

      for (let i = 0; i < steps.length; i++) {
        const step = steps[i];
        if (!step.name || typeof step.name !== 'string') {
          return NextResponse.json({ error: `Step ${i + 1} requires a name` }, { status: 400 });
        }
        if (!step.allowedRoles || !Array.isArray(step.allowedRoles)) {
          return NextResponse.json({ error: `Step ${i + 1} requires allowedRoles array` }, { status: 400 });
        }
        if (!step.approvalRule || !['any', 'all', 'single'].includes(step.approvalRule)) {
          return NextResponse.json({ error: `Step ${i + 1} requires valid approvalRule` }, { status: 400 });
        }
        step.allowedUsers = step.allowedUsers || [];
      }

      updateData.stepsJson = steps as unknown as Prisma.InputJsonValue;
    }

    if (finalRule !== undefined) {
      const normalizedFinalRule: WorkflowFinalRule = {
        type: finalRule?.type || 'single',
        description: finalRule?.description || 'Single approver required',
      };
      updateData.finalRuleJson = normalizedFinalRule as unknown as Prisma.InputJsonValue;
    }

    if (isDefault !== undefined) {
      if (isDefault === true) {
        await prisma.workflowTemplate.updateMany({
          where: { 
            departmentId: template.departmentId, 
            isDefault: true,
            id: { not: templateId },
          },
          data: { isDefault: false },
        });
      }
      updateData.isDefault = isDefault;
    }

    const updated = await prisma.workflowTemplate.update({
      where: { id: templateId },
      data: updateData,
      include: {
        department: {
          select: { id: true, name: true },
        },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'WorkflowTemplate',
        entityId: templateId,
        action: 'updated',
        actorId: session.id,
        payloadJson: {
          changes: Object.keys(body),
        } as unknown as Prisma.InputJsonValue,
      },
    });

    return NextResponse.json({
      id: updated.id,
      name: updated.name,
      description: updated.description,
      department: updated.department,
      steps: updated.stepsJson as unknown as WorkflowStep[],
      finalRule: updated.finalRuleJson as unknown as WorkflowFinalRule,
      isDefault: updated.isDefault,
      createdAt: updated.createdAt,
      updatedAt: updated.updatedAt,
    });
  } catch (error) {
    console.error('Update workflow template error:', error);
    return NextResponse.json({ error: 'Failed to update template' }, { status: 500 });
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

    const { templateId } = await params;

    const template = await prisma.workflowTemplate.findUnique({
      where: { id: templateId },
      include: {
        _count: { select: { instances: true } },
      },
    });

    if (!template) {
      return NextResponse.json({ error: 'Template not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== template.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    if (template._count.instances > 0) {
      return NextResponse.json({ 
        error: 'Cannot delete template with active workflow instances' 
      }, { status: 400 });
    }

    await prisma.workflowTemplate.delete({
      where: { id: templateId },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'WorkflowTemplate',
        entityId: templateId,
        action: 'deleted',
        actorId: session.id,
        payloadJson: {
          name: template.name,
          departmentId: template.departmentId,
        } as unknown as Prisma.InputJsonValue,
      },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Delete workflow template error:', error);
    return NextResponse.json({ error: 'Failed to delete template' }, { status: 500 });
  }
}
