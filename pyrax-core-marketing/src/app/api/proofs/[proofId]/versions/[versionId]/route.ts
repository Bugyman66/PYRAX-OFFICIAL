import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole } from '@prisma/client';
import { driveService } from '@/lib/drive';

export const dynamic = 'force-dynamic';

interface RouteParams {
  params: Promise<{ proofId: string; versionId: string }>;
}

async function canAccessVersion(
  session: NonNullable<Awaited<ReturnType<typeof getSession>>>,
  proofId: string,
  versionId: string
) {
  const version = await prisma.proofVersion.findUnique({
    where: { id: versionId },
    include: {
      proof: {
        include: {
          folder: { select: { departmentId: true } },
        },
      },
    },
  });

  if (!version) return { allowed: false, version: null, reason: 'Version not found' };
  if (version.proofId !== proofId) return { allowed: false, version: null, reason: 'Version not found' };

  if (session.role === UserRole.OrgAdmin) {
    return { allowed: true, version };
  }

  if (session.departmentId === version.proof.folder.departmentId) {
    return { allowed: true, version };
  }

  return { allowed: false, version: null, reason: 'Access denied' };
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { proofId, versionId } = await params;
    const access = await canAccessVersion(session, proofId, versionId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Version not found' ? 404 : 403 }
      );
    }

    const version = await prisma.proofVersion.findUnique({
      where: { id: versionId },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        annotations: {
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
        },
        comments: {
          where: { annotationId: null },
          include: {
            createdBy: { select: { id: true, name: true, email: true } },
            replies: {
              include: {
                createdBy: { select: { id: true, name: true, email: true } },
              },
              orderBy: { createdAt: 'asc' },
            },
          },
          orderBy: { createdAt: 'asc' },
        },
      },
    });

    return NextResponse.json({ version });
  } catch (error) {
    console.error('Get version error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch version' },
      { status: 500 }
    );
  }
}

export async function DELETE(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { proofId, versionId } = await params;
    const access = await canAccessVersion(session, proofId, versionId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Version not found' ? 404 : 403 }
      );
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json(
        { error: 'Only admins and department heads can delete versions' },
        { status: 403 }
      );
    }

    const versionCount = await prisma.proofVersion.count({
      where: { proofId },
    });

    if (versionCount <= 1) {
      return NextResponse.json(
        { error: 'Cannot delete the only version. Delete the proof instead.' },
        { status: 400 }
      );
    }

    try {
      await driveService.deleteFile(access.version!.driveFileId);
    } catch (error) {
      console.warn(`Failed to delete Drive file ${access.version!.driveFileId}:`, error);
    }

    await prisma.proofVersion.delete({
      where: { id: versionId },
    });

    await prisma.proof.update({
      where: { id: proofId },
      data: { updatedAt: new Date() },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'ProofVersion',
        entityId: versionId,
        action: 'deleted',
        actorId: session.id,
        payloadJson: { proofId },
      },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Delete version error:', error);
    return NextResponse.json(
      { error: 'Failed to delete version' },
      { status: 500 }
    );
  }
}
