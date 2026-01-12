import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole } from '@prisma/client';

export const dynamic = 'force-dynamic';

interface RouteParams {
  params: Promise<{ versionId: string }>;
}

async function canAccessVersion(session: NonNullable<Awaited<ReturnType<typeof getSession>>>, versionId: string) {
  const version = await prisma.proofVersion.findUnique({
    where: { id: versionId },
    include: {
      proof: {
        include: { folder: { select: { departmentId: true } } },
      },
    },
  });

  if (!version) return { allowed: false, version: null };

  if (session.role === UserRole.OrgAdmin) {
    return { allowed: true, version };
  }

  if (session.departmentId === version.proof.folder.departmentId) {
    return { allowed: true, version };
  }

  return { allowed: false, version: null };
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { versionId } = await params;
    const access = await canAccessVersion(session, versionId);

    if (!access.allowed) {
      return NextResponse.json({ error: 'Access denied' }, { status: 403 });
    }

    const { searchParams } = new URL(request.url);
    const annotationId = searchParams.get('annotationId');

    const where: Record<string, unknown> = {
      proofVersionId: versionId,
      parentId: null, // Only top-level comments
    };

    if (annotationId) {
      where.annotationId = annotationId;
    } else {
      where.annotationId = null; // General comments not linked to annotations
    }

    const comments = await prisma.comment.findMany({
      where,
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        annotation: { select: { id: true, color: true } },
        replies: {
          include: {
            createdBy: { select: { id: true, name: true, email: true } },
          },
          orderBy: { createdAt: 'asc' },
        },
      },
      orderBy: { createdAt: 'desc' },
    });

    return NextResponse.json({ comments });
  } catch (error) {
    console.error('Get comments error:', error);
    return NextResponse.json({ error: 'Failed to fetch comments' }, { status: 500 });
  }
}

export async function POST(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { versionId } = await params;
    const access = await canAccessVersion(session, versionId);

    if (!access.allowed) {
      return NextResponse.json({ error: 'Access denied' }, { status: 403 });
    }

    const body = await request.json();
    const { body: commentBody, annotationId, parentId, mentionsJson } = body;

    if (!commentBody?.trim()) {
      return NextResponse.json({ error: 'Comment body is required' }, { status: 400 });
    }

    // Validate annotation belongs to this version
    if (annotationId) {
      const annotation = await prisma.annotation.findUnique({
        where: { id: annotationId },
        select: { proofVersionId: true },
      });
      if (!annotation || annotation.proofVersionId !== versionId) {
        return NextResponse.json({ error: 'Invalid annotation' }, { status: 400 });
      }
    }

    // Validate parent comment belongs to this version
    if (parentId) {
      const parentComment = await prisma.comment.findUnique({
        where: { id: parentId },
        select: { proofVersionId: true },
      });
      if (!parentComment || parentComment.proofVersionId !== versionId) {
        return NextResponse.json({ error: 'Invalid parent comment' }, { status: 400 });
      }
    }

    const comment = await prisma.comment.create({
      data: {
        proofVersionId: versionId,
        body: commentBody.trim(),
        annotationId: annotationId || null,
        parentId: parentId || null,
        mentionsJson: mentionsJson || null,
        createdById: session.id,
      },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        annotation: { select: { id: true, color: true } },
        replies: true,
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Comment',
        entityId: comment.id,
        action: 'created',
        actorId: session.id,
        payloadJson: { 
          versionId, 
          hasAnnotation: !!annotationId,
          isReply: !!parentId,
        },
      },
    });

    return NextResponse.json({ comment }, { status: 201 });
  } catch (error) {
    console.error('Create comment error:', error);
    return NextResponse.json({ error: 'Failed to create comment' }, { status: 500 });
  }
}
