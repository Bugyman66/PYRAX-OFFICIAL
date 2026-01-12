import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole } from '@prisma/client';

export const dynamic = 'force-dynamic';

interface RouteParams {
  params: Promise<{ versionId: string; annotationId: string }>;
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { annotationId } = await params;

    const annotation = await prisma.annotation.findUnique({
      where: { id: annotationId },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        comments: {
          include: {
            createdBy: { select: { id: true, name: true, email: true } },
            replies: {
              include: {
                createdBy: { select: { id: true, name: true, email: true } },
              },
              orderBy: { createdAt: 'asc' },
            },
          },
          where: { parentId: null },
          orderBy: { createdAt: 'asc' },
        },
      },
    });

    if (!annotation) {
      return NextResponse.json({ error: 'Annotation not found' }, { status: 404 });
    }

    return NextResponse.json({ annotation });
  } catch (error) {
    console.error('Get annotation error:', error);
    return NextResponse.json({ error: 'Failed to fetch annotation' }, { status: 500 });
  }
}

export async function PATCH(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { annotationId } = await params;

    const existing = await prisma.annotation.findUnique({
      where: { id: annotationId },
      select: { createdById: true },
    });

    if (!existing) {
      return NextResponse.json({ error: 'Annotation not found' }, { status: 404 });
    }

    // Only creator or admin can update
    if (existing.createdById !== session.id && session.role !== UserRole.OrgAdmin) {
      return NextResponse.json({ error: 'Not authorized to update this annotation' }, { status: 403 });
    }

    const body = await request.json();
    const { geometryJson, color } = body;

    const updateData: Record<string, unknown> = {};
    if (geometryJson) updateData.geometryJson = geometryJson;
    if (color) updateData.color = color;

    const annotation = await prisma.annotation.update({
      where: { id: annotationId },
      data: updateData,
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
      },
    });

    return NextResponse.json({ annotation });
  } catch (error) {
    console.error('Update annotation error:', error);
    return NextResponse.json({ error: 'Failed to update annotation' }, { status: 500 });
  }
}

export async function DELETE(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { annotationId } = await params;

    const existing = await prisma.annotation.findUnique({
      where: { id: annotationId },
      select: { createdById: true },
    });

    if (!existing) {
      return NextResponse.json({ error: 'Annotation not found' }, { status: 404 });
    }

    // Only creator or admin can delete
    if (existing.createdById !== session.id && session.role !== UserRole.OrgAdmin) {
      return NextResponse.json({ error: 'Not authorized to delete this annotation' }, { status: 403 });
    }

    await prisma.annotation.delete({
      where: { id: annotationId },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Annotation',
        entityId: annotationId,
        action: 'deleted',
        actorId: session.id,
      },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Delete annotation error:', error);
    return NextResponse.json({ error: 'Failed to delete annotation' }, { status: 500 });
  }
}
