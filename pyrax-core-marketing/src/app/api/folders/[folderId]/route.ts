import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole } from '@prisma/client';

export const dynamic = 'force-dynamic';

interface RouteParams {
  params: Promise<{ folderId: string }>;
}

async function canAccessFolder(session: NonNullable<Awaited<ReturnType<typeof getSession>>>, folderId: string) {
  const folder = await prisma.folder.findUnique({
    where: { id: folderId },
    select: { id: true, departmentId: true },
  });

  if (!folder) return { allowed: false, folder: null, reason: 'Folder not found' };

  if (session.role === UserRole.OrgAdmin) {
    return { allowed: true, folder };
  }

  if (session.departmentId === folder.departmentId) {
    return { allowed: true, folder };
  }

  return { allowed: false, folder: null, reason: 'Access denied' };
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { folderId } = await params;
    const access = await canAccessFolder(session, folderId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Folder not found' ? 404 : 403 }
      );
    }

    const folder = await prisma.folder.findUnique({
      where: { id: folderId },
      include: {
        department: { select: { id: true, name: true } },
        createdBy: { select: { id: true, name: true, email: true } },
        proofs: {
          include: {
            createdBy: { select: { id: true, name: true, email: true } },
            versions: {
              orderBy: { versionNumber: 'desc' },
              take: 1,
              select: { id: true, versionNumber: true, fileName: true, mimeType: true },
            },
            _count: { select: { versions: true } },
          },
          orderBy: { updatedAt: 'desc' },
        },
      },
    });

    return NextResponse.json({ folder });
  } catch (error) {
    console.error('Get folder error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch folder' },
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

    const { folderId } = await params;
    const access = await canAccessFolder(session, folderId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Folder not found' ? 404 : 403 }
      );
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json(
        { error: 'Only admins and department heads can update folders' },
        { status: 403 }
      );
    }

    const body = await request.json();
    const { name, description } = body;

    const updateData: Record<string, unknown> = {};
    if (name?.trim()) updateData.name = name.trim();
    if (description !== undefined) updateData.description = description?.trim() || null;

    if (Object.keys(updateData).length === 0) {
      return NextResponse.json(
        { error: 'No valid fields to update' },
        { status: 400 }
      );
    }

    const folder = await prisma.folder.update({
      where: { id: folderId },
      data: updateData,
      include: {
        department: { select: { id: true, name: true } },
        createdBy: { select: { id: true, name: true, email: true } },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Folder',
        entityId: folder.id,
        action: 'updated',
        actorId: session.id,
        payloadJson: updateData,
      },
    });

    return NextResponse.json({ folder });
  } catch (error) {
    console.error('Update folder error:', error);
    return NextResponse.json(
      { error: 'Failed to update folder' },
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

    const { folderId } = await params;
    const access = await canAccessFolder(session, folderId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Folder not found' ? 404 : 403 }
      );
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json(
        { error: 'Only admins and department heads can delete folders' },
        { status: 403 }
      );
    }

    const proofCount = await prisma.proof.count({
      where: { folderId },
    });

    if (proofCount > 0) {
      return NextResponse.json(
        { error: `Cannot delete folder with ${proofCount} proofs. Move or delete proofs first.` },
        { status: 400 }
      );
    }

    await prisma.folder.delete({
      where: { id: folderId },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Folder',
        entityId: folderId,
        action: 'deleted',
        actorId: session.id,
      },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Delete folder error:', error);
    return NextResponse.json(
      { error: 'Failed to delete folder' },
      { status: 500 }
    );
  }
}
