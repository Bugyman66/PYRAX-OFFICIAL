import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole } from '@prisma/client';

export const dynamic = 'force-dynamic';

interface RouteParams {
  params: Promise<{ versionId: string }>;
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { versionId } = await params;

    const version = await prisma.proofVersion.findUnique({
      where: { id: versionId },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        proof: {
          include: {
            folder: { select: { id: true, name: true, departmentId: true } },
            versions: {
              select: { id: true, versionNumber: true, kind: true },
              orderBy: { versionNumber: 'desc' },
            },
          },
        },
      },
    });

    if (!version) {
      return NextResponse.json({ error: 'Version not found' }, { status: 404 });
    }

    // Check access
    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== version.proof.folder.departmentId) {
        return NextResponse.json({ error: 'Access denied' }, { status: 403 });
      }
    }

    return NextResponse.json({ version });
  } catch (error) {
    console.error('Get version error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch version' },
      { status: 500 }
    );
  }
}
