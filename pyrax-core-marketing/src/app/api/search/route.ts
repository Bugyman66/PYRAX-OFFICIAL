import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { searchService } from '@/lib/search';
import { ProofStatus } from '@prisma/client';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const query = searchParams.get('q') || undefined;
    const departmentId = searchParams.get('departmentId') || undefined;
    const status = searchParams.get('status') as ProofStatus | undefined;
    const createdById = searchParams.get('createdById') || undefined;
    const dateFrom = searchParams.get('dateFrom');
    const dateTo = searchParams.get('dateTo');
    const page = parseInt(searchParams.get('page') || '1', 10);
    const pageSize = parseInt(searchParams.get('pageSize') || '20', 10);
    const suggestions = searchParams.get('suggestions') === 'true';

    if (suggestions && query) {
      const items = await searchService.getSuggestions(
        query,
        session.departmentId,
        session.role
      );
      return NextResponse.json(items);
    }

    const results = await searchService.search(
      {
        query,
        departmentId,
        status,
        createdById,
        dateFrom: dateFrom ? new Date(dateFrom) : undefined,
        dateTo: dateTo ? new Date(dateTo) : undefined,
      },
      session.departmentId,
      session.role,
      page,
      Math.min(pageSize, 50)
    );

    return NextResponse.json(results);
  } catch (error) {
    console.error('Search error:', error);
    return NextResponse.json({ error: 'Search failed' }, { status: 500 });
  }
}
