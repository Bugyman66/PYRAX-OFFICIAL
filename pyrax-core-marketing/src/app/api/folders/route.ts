import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole } from '@prisma/client';
import { driveService } from '@/lib/drive';

export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const departmentId = searchParams.get('departmentId');

    const where: Record<string, unknown> = {};

    if (session.role === UserRole.OrgAdmin) {
      if (departmentId) {
        where.departmentId = departmentId;
      }
    } else if (session.role === UserRole.DepartmentHead || session.role === UserRole.Employee) {
      if (session.departmentId) {
        where.departmentId = session.departmentId;
      } else {
        return NextResponse.json({ folders: [] });
      }
    }

    const folders = await prisma.folder.findMany({
      where,
      include: {
        department: { select: { id: true, name: true } },
        createdBy: { select: { id: true, name: true, email: true } },
        _count: { select: { proofs: true } },
      },
      orderBy: { createdAt: 'desc' },
    });

    return NextResponse.json({
      folders: folders.map((f) => ({
        id: f.id,
        name: f.name,
        description: f.description,
        department: f.department,
        createdBy: f.createdBy,
        proofCount: f._count.proofs,
        createdAt: f.createdAt,
        updatedAt: f.updatedAt,
      })),
    });
  } catch (error) {
    console.error('Get folders error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch folders' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { name, description, departmentId } = body;

    if (!name?.trim()) {
      return NextResponse.json(
        { error: 'Folder name is required' },
        { status: 400 }
      );
    }

    let targetDepartmentId = departmentId;

    if (session.role === UserRole.OrgAdmin) {
      if (!departmentId) {
        return NextResponse.json(
          { error: 'Department ID is required for OrgAdmin' },
          { status: 400 }
        );
      }
    } else {
      if (!session.departmentId) {
        return NextResponse.json(
          { error: 'You are not assigned to a department' },
          { status: 403 }
        );
      }
      targetDepartmentId = session.departmentId;
    }

    const department = await prisma.department.findUnique({
      where: { id: targetDepartmentId },
    });

    if (!department) {
      return NextResponse.json(
        { error: 'Department not found' },
        { status: 404 }
      );
    }

    // Create folder in Google Drive under the root folder
    let driveFolderId: string | null = null;
    try {
      const driveFolder = await driveService.createFolder(name.trim());
      driveFolderId = driveFolder.id;
    } catch (driveError) {
      console.error('Failed to create Drive folder:', driveError);
      // Continue without Drive folder - it can be created later
    }

    const folder = await prisma.folder.create({
      data: {
        name: name.trim(),
        description: description?.trim() || null,
        departmentId: targetDepartmentId,
        createdById: session.id,
        driveFolderId,
      },
      include: {
        department: { select: { id: true, name: true } },
        createdBy: { select: { id: true, name: true, email: true } },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Folder',
        entityId: folder.id,
        action: 'created',
        actorId: session.id,
        payloadJson: { name: folder.name, departmentId: folder.departmentId, driveFolderId },
      },
    });

    return NextResponse.json({ folder }, { status: 201 });
  } catch (error) {
    console.error('Create folder error:', error);
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    return NextResponse.json(
      { error: `Failed to create folder: ${errorMessage}` },
      { status: 500 }
    );
  }
}
