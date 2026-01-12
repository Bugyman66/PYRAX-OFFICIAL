import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole, ProofStatus } from '@prisma/client';
import { driveService } from '@/lib/drive';

export const dynamic = 'force-dynamic';

interface RouteParams {
  params: Promise<{ proofId: string }>;
}

async function canAccessProof(session: NonNullable<Awaited<ReturnType<typeof getSession>>>, proofId: string) {
  const proof = await prisma.proof.findUnique({
    where: { id: proofId },
    include: {
      folder: { select: { departmentId: true } },
    },
  });

  if (!proof) return { allowed: false, proof: null, reason: 'Proof not found' };

  if (session.role === UserRole.OrgAdmin) {
    return { allowed: true, proof };
  }

  if (session.departmentId === proof.folder.departmentId) {
    return { allowed: true, proof };
  }

  return { allowed: false, proof: null, reason: 'Access denied' };
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { proofId } = await params;
    const access = await canAccessProof(session, proofId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Proof not found' ? 404 : 403 }
      );
    }

    const proof = await prisma.proof.findUnique({
      where: { id: proofId },
      include: {
        folder: {
          select: { id: true, name: true, departmentId: true },
        },
        createdBy: { select: { id: true, name: true, email: true } },
        versions: {
          orderBy: { versionNumber: 'desc' },
          include: {
            createdBy: { select: { id: true, name: true, email: true } },
            _count: { select: { annotations: true, comments: true } },
          },
        },
        workflowInstance: {
          include: {
            template: { select: { id: true, name: true } },
            decisions: {
              include: {
                decidedBy: { select: { id: true, name: true, email: true } },
              },
              orderBy: { createdAt: 'desc' },
            },
          },
        },
        publicAsset: true,
      },
    });

    return NextResponse.json({ proof });
  } catch (error) {
    console.error('Get proof error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch proof' },
      { status: 500 }
    );
  }
}

export async function PATCH(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { proofId } = await params;
    const access = await canAccessProof(session, proofId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Proof not found' ? 404 : 403 }
      );
    }

    const body = await request.json();
    const { title, description, status, folderId } = body;

    const updateData: Record<string, unknown> = {};
    
    if (title?.trim()) updateData.title = title.trim();
    if (description !== undefined) updateData.description = description?.trim() || null;
    
    if (status && Object.values(ProofStatus).includes(status)) {
      if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
        return NextResponse.json(
          { error: 'Only admins and department heads can change proof status' },
          { status: 403 }
        );
      }
      updateData.status = status;
    }

    if (folderId) {
      const targetFolder = await prisma.folder.findUnique({
        where: { id: folderId },
        select: { id: true, departmentId: true },
      });

      if (!targetFolder) {
        return NextResponse.json({ error: 'Target folder not found' }, { status: 404 });
      }

      if (session.role !== UserRole.OrgAdmin && session.departmentId !== targetFolder.departmentId) {
        return NextResponse.json(
          { error: 'You do not have access to the target folder' },
          { status: 403 }
        );
      }

      updateData.folderId = folderId;
    }

    if (Object.keys(updateData).length === 0) {
      return NextResponse.json(
        { error: 'No valid fields to update' },
        { status: 400 }
      );
    }

    const proof = await prisma.proof.update({
      where: { id: proofId },
      data: updateData,
      include: {
        folder: { select: { id: true, name: true } },
        createdBy: { select: { id: true, name: true, email: true } },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Proof',
        entityId: proof.id,
        action: 'updated',
        actorId: session.id,
        payloadJson: updateData,
      },
    });

    return NextResponse.json({ proof });
  } catch (error) {
    console.error('Update proof error:', error);
    return NextResponse.json(
      { error: 'Failed to update proof' },
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

    const { proofId } = await params;
    const access = await canAccessProof(session, proofId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Proof not found' ? 404 : 403 }
      );
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json(
        { error: 'Only admins and department heads can delete proofs' },
        { status: 403 }
      );
    }

    const versions = await prisma.proofVersion.findMany({
      where: { proofId },
      select: { driveFileId: true },
    });

    for (const version of versions) {
      try {
        await driveService.deleteFile(version.driveFileId);
      } catch (error) {
        console.warn(`Failed to delete Drive file ${version.driveFileId}:`, error);
      }
    }

    await prisma.proof.delete({
      where: { id: proofId },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Proof',
        entityId: proofId,
        action: 'deleted',
        actorId: session.id,
      },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Delete proof error:', error);
    return NextResponse.json(
      { error: 'Failed to delete proof' },
      { status: 500 }
    );
  }
}
