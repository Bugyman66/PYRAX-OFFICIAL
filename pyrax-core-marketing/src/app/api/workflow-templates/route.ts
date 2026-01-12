import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, Prisma } from '@prisma/client';
import { WorkflowStep, WorkflowFinalRule } from '@/lib/workflow';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const departmentId = searchParams.get('departmentId');

    const where: Prisma.WorkflowTemplateWhereInput = {};

    if (session.role === UserRole.OrgAdmin) {
      if (departmentId) {
        where.departmentId = departmentId;
      }
    } else if (session.role === UserRole.DepartmentHead || session.role === UserRole.Employee) {
      if (!session.departmentId) {
        return NextResponse.json({ error: 'User has no department' }, { status: 403 });
      }
      where.departmentId = session.departmentId;
    } else {
      return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
    }

    const templates = await prisma.workflowTemplate.findMany({
      where,
      include: {
        department: {
          select: { id: true, name: true },
        },
        _count: {
          select: { instances: true },
        },
      },
      orderBy: [
        { isDefault: 'desc' },
        { name: 'asc' },
      ],
    });

    const formattedTemplates = templates.map(t => ({
      id: t.id,
      name: t.name,
      description: t.description,
      department: t.department,
      steps: t.stepsJson as unknown as WorkflowStep[],
      finalRule: t.finalRuleJson as unknown as WorkflowFinalRule,
      isDefault: t.isDefault,
      instanceCount: t._count.instances,
      createdAt: t.createdAt,
      updatedAt: t.updatedAt,
    }));

    return NextResponse.json(formattedTemplates);
  } catch (error) {
    console.error('Get workflow templates error:', error);
    return NextResponse.json({ error: 'Failed to fetch templates' }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
    }

    const body = await request.json();
    const { name, description, departmentId, steps, finalRule, isDefault } = body;

    if (!name || typeof name !== 'string' || name.trim().length === 0) {
      return NextResponse.json({ error: 'Name is required' }, { status: 400 });
    }

    if (!steps || !Array.isArray(steps) || steps.length === 0) {
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
        return NextResponse.json({ error: `Step ${i + 1} requires valid approvalRule (any, all, single)` }, { status: 400 });
      }
      step.allowedUsers = step.allowedUsers || [];
    }

    let targetDepartmentId: string;
    if (session.role === UserRole.OrgAdmin) {
      if (!departmentId) {
        return NextResponse.json({ error: 'Department ID is required' }, { status: 400 });
      }
      targetDepartmentId = departmentId;
    } else {
      if (!session.departmentId) {
        return NextResponse.json({ error: 'User has no department' }, { status: 403 });
      }
      targetDepartmentId = session.departmentId;
    }

    const department = await prisma.department.findUnique({
      where: { id: targetDepartmentId },
    });

    if (!department) {
      return NextResponse.json({ error: 'Department not found' }, { status: 404 });
    }

    const normalizedFinalRule: WorkflowFinalRule = {
      type: finalRule?.type || 'single',
      description: finalRule?.description || 'Single approver required',
    };

    if (isDefault) {
      await prisma.workflowTemplate.updateMany({
        where: { departmentId: targetDepartmentId, isDefault: true },
        data: { isDefault: false },
      });
    }

    const template = await prisma.workflowTemplate.create({
      data: {
        name: name.trim(),
        description: description?.trim() || null,
        departmentId: targetDepartmentId,
        stepsJson: steps as unknown as Prisma.InputJsonValue,
        finalRuleJson: normalizedFinalRule as unknown as Prisma.InputJsonValue,
        isDefault: isDefault || false,
      },
      include: {
        department: {
          select: { id: true, name: true },
        },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'WorkflowTemplate',
        entityId: template.id,
        action: 'created',
        actorId: session.id,
        payloadJson: {
          name: template.name,
          departmentId: targetDepartmentId,
          departmentName: department.name,
          stepCount: steps.length,
        } as unknown as Prisma.InputJsonValue,
      },
    });

    return NextResponse.json({
      id: template.id,
      name: template.name,
      description: template.description,
      department: template.department,
      steps: template.stepsJson as unknown as WorkflowStep[],
      finalRule: template.finalRuleJson as unknown as WorkflowFinalRule,
      isDefault: template.isDefault,
      createdAt: template.createdAt,
      updatedAt: template.updatedAt,
    }, { status: 201 });
  } catch (error) {
    console.error('Create workflow template error:', error);
    return NextResponse.json({ error: 'Failed to create template' }, { status: 500 });
  }
}
