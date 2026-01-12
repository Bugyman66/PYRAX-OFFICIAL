import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { dashboardService } from '@/lib/dashboard';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const section = searchParams.get('section');

    if (section === 'stats') {
      const stats = await dashboardService.getStats(
        session.id,
        session.departmentId,
        session.role
      );
      return NextResponse.json(stats);
    }

    if (section === 'waiting') {
      const items = await dashboardService.getWaitingOnMe(
        session.id,
        session.departmentId,
        session.role
      );
      return NextResponse.json(items);
    }

    if (section === 'overdue') {
      const items = await dashboardService.getOverdue(
        session.departmentId,
        session.role
      );
      return NextResponse.json(items);
    }

    if (section === 'needs-changes') {
      const items = await dashboardService.getNeedsChanges(
        session.id,
        session.departmentId,
        session.role
      );
      return NextResponse.json(items);
    }

    if (section === 'approved') {
      const items = await dashboardService.getRecentlyApproved(
        session.departmentId,
        session.role
      );
      return NextResponse.json(items);
    }

    if (section === 'activity') {
      const items = await dashboardService.getRecentActivity(
        session.departmentId,
        session.role
      );
      return NextResponse.json(items);
    }

    const [stats, waitingOnMe, overdue, needsChanges, recentlyApproved, activity] = await Promise.all([
      dashboardService.getStats(session.id, session.departmentId, session.role),
      dashboardService.getWaitingOnMe(session.id, session.departmentId, session.role, 5),
      dashboardService.getOverdue(session.departmentId, session.role, 5),
      dashboardService.getNeedsChanges(session.id, session.departmentId, session.role, 5),
      dashboardService.getRecentlyApproved(session.departmentId, session.role, 5),
      dashboardService.getRecentActivity(session.departmentId, session.role, 10),
    ]);

    return NextResponse.json({
      stats,
      waitingOnMe,
      overdue,
      needsChanges,
      recentlyApproved,
      activity,
    });
  } catch (error) {
    console.error('Dashboard error:', error);
    return NextResponse.json({ error: 'Failed to fetch dashboard data' }, { status: 500 });
  }
}
