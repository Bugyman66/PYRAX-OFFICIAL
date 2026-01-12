import { NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';

export const dynamic = 'force-dynamic';

export async function GET() {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const departments = await prisma.department.findMany({
      include: {
        _count: {
          select: {
            users: true,
            folders: true,
          },
        },
      },
      orderBy: { name: 'asc' },
    });

    return NextResponse.json(
      departments.map((d) => ({
        id: d.id,
        name: d.name,
        userCount: d._count.users,
        folderCount: d._count.folders,
        createdAt: d.createdAt,
      }))
    );
  } catch (error) {
    console.error('Get departments error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch departments' },
      { status: 500 }
    );
  }
}
