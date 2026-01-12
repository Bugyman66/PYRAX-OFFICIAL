import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole, ProofStatus } from '@prisma/client';

export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const folderId = searchParams.get('folderId');
    const status = searchParams.get('status') as ProofStatus | null;
    const limit = parseInt(searchParams.get('limit') || '50', 10);
    const offset = parseInt(searchParams.get('offset') || '0', 10);

    const where: Record<string, unknown> = {};

    if (folderId) {
      where.folderId = folderId;
    }

    if (status && Object.values(ProofStatus).includes(status)) {
      where.status = status;
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId) {
        where.folder = { departmentId: session.departmentId };
      } else {
        return NextResponse.json({ proofs: [], total: 0 });
      }
    }

    const [proofs, total] = await Promise.all([
      prisma.proof.findMany({
        where,
        include: {
          folder: { select: { id: true, name: true, departmentId: true } },
          createdBy: { select: { id: true, name: true, email: true } },
          versions: {
            orderBy: { versionNumber: 'desc' },
            take: 1,
            select: {
              id: true,
              versionNumber: true,
              fileName: true,
              mimeType: true,
              fileSize: true,
              createdAt: true,
            },
          },
          _count: { select: { versions: true } },
        },
        orderBy: { updatedAt: 'desc' },
        take: Math.min(limit, 100),
        skip: offset,
      }),
      prisma.proof.count({ where }),
    ]);

    return NextResponse.json({
      proofs: proofs.map((p) => ({
        id: p.id,
        title: p.title,
        description: p.description,
        status: p.status,
        folder: p.folder,
        createdBy: p.createdBy,
        latestVersion: p.versions[0] || null,
        versionCount: p._count.versions,
        isExternalSubmission: p.isExternalSubmission,
        createdAt: p.createdAt,
        updatedAt: p.updatedAt,
      })),
      total,
      limit,
      offset,
    });
  } catch (error) {
    console.error('Get proofs error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch proofs' },
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
    const { title, description, folderId } = body;

    if (!title?.trim()) {
      return NextResponse.json(
        { error: 'Proof title is required' },
        { status: 400 }
      );
    }

    if (!folderId) {
      return NextResponse.json(
        { error: 'Folder ID is required' },
        { status: 400 }
      );
    }

    const folder = await prisma.folder.findUnique({
      where: { id: folderId },
      select: { id: true, departmentId: true },
    });

    if (!folder) {
      return NextResponse.json(
        { error: 'Folder not found' },
        { status: 404 }
      );
    }

    if (session.role !== UserRole.OrgAdmin && session.departmentId !== folder.departmentId) {
      return NextResponse.json(
        { error: 'You do not have access to this folder' },
        { status: 403 }
      );
    }

    const proof = await prisma.proof.create({
      data: {
        title: title.trim(),
        description: description?.trim() || null,
        folderId,
        createdById: session.id,
        status: ProofStatus.Draft,
      },
      include: {
        folder: { select: { id: true, name: true } },
        createdBy: { select: { id: true, name: true, email: true } },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'Proof',
        entityId: proof.id,
        action: 'created',
        actorId: session.id,
        payloadJson: { title: proof.title, folderId: proof.folderId },
      },
    });

    return NextResponse.json({ proof }, { status: 201 });
  } catch (error) {
    console.error('Create proof error:', error);
    return NextResponse.json(
      { error: 'Failed to create proof' },
      { status: 500 }
    );
  }
}
