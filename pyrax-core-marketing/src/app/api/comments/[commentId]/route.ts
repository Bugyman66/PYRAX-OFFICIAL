import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole } from '@prisma/client';

export const dynamic = 'force-dynamic';

interface RouteParams {
  params: Promise<{ commentId: string }>;
}

export async function PATCH(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { commentId } = await params;

    const existing = await prisma.comment.findUnique({
      where: { id: commentId },
      select: { createdById: true },
    });

    if (!existing) {
      return NextResponse.json({ error: 'Comment not found' }, { status: 404 });
    }

    const body = await request.json();
    const { body: commentBody, resolved } = body;

    // Only creator can edit body
    if (commentBody !== undefined && existing.createdById !== session.id) {
      return NextResponse.json({ error: 'Not authorized to edit this comment' }, { status: 403 });
    }

    const updateData: Record<string, unknown> = {};
    if (commentBody !== undefined) updateData.body = commentBody.trim();
    if (resolved !== undefined) updateData.resolved = resolved;

    const comment = await prisma.comment.update({
      where: { id: commentId },
      data: updateData,
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
      },
    });

    return NextResponse.json({ comment });
  } catch (error) {
    console.error('Update comment error:', error);
    return NextResponse.json({ error: 'Failed to update comment' }, { status: 500 });
  }
}

export async function DELETE(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { commentId } = await params;

    const existing = await prisma.comment.findUnique({
      where: { id: commentId },
      select: { createdById: true },
    });

    if (!existing) {
      return NextResponse.json({ error: 'Comment not found' }, { status: 404 });
    }

    // Only creator or admin can delete
    if (existing.createdById !== session.id && session.role !== UserRole.OrgAdmin) {
      return NextResponse.json({ error: 'Not authorized to delete this comment' }, { status: 403 });
    }

    await prisma.comment.delete({
      where: { id: commentId },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Comment',
        entityId: commentId,
        action: 'deleted',
        actorId: session.id,
      },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Delete comment error:', error);
    return NextResponse.json({ error: 'Failed to delete comment' }, { status: 500 });
  }
}
