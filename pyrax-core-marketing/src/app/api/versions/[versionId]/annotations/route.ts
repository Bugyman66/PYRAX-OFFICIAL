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

    const annotations = await prisma.annotation.findMany({
      where: { proofVersionId: versionId },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        comments: {
          include: {
            createdBy: { select: { id: true, name: true, email: true } },
          },
          orderBy: { createdAt: 'asc' },
        },
      },
      orderBy: { createdAt: 'asc' },
    });

    return NextResponse.json({ annotations });
  } catch (error) {
    console.error('Get annotations error:', error);
    return NextResponse.json({ error: 'Failed to fetch annotations' }, { status: 500 });
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
    const { geometryJson, pageOrTimecode, color } = body;

    if (!geometryJson || typeof geometryJson !== 'object') {
      return NextResponse.json({ error: 'geometryJson is required' }, { status: 400 });
    }

    const annotation = await prisma.annotation.create({
      data: {
        proofVersionId: versionId,
        geometryJson,
        pageOrTimecode: pageOrTimecode || null,
        color: color || '#FF5500',
        createdById: session.id,
      },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        comments: true,
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Annotation',
        entityId: annotation.id,
        action: 'created',
        actorId: session.id,
        payloadJson: { versionId, type: geometryJson.type },
      },
    });

    return NextResponse.json({ annotation }, { status: 201 });
  } catch (error) {
    console.error('Create annotation error:', error);
    return NextResponse.json({ error: 'Failed to create annotation' }, { status: 500 });
  }
}
