import { NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { driveService } from '@/lib/drive';

export const dynamic = 'force-dynamic';

export async function GET() {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }

    if (session.role !== 'OrgAdmin') {
      return NextResponse.json(
        { error: 'Forbidden - OrgAdmin only' },
        { status: 403 }
      );
    }

    const status = await driveService.getConnectionStatus();

    return NextResponse.json({
      success: true,
      ...status,
    });
  } catch (error) {
    console.error('Drive status error:', error);

    return NextResponse.json(
      { 
        success: false,
        connected: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      },
      { status: 500 }
    );
  }
}
