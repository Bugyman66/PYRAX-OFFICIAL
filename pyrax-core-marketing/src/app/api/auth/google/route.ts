import { NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { getAuthUrl } from '@/lib/drive';

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

    const state = Buffer.from(JSON.stringify({
      oderId: session.id,
      timestamp: Date.now(),
    })).toString('base64');

    const authUrl = getAuthUrl(state);

    return NextResponse.json({
      success: true,
      authUrl,
    });
  } catch (error) {
    console.error('Google auth URL error:', error);
    
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to generate auth URL' },
      { status: 500 }
    );
  }
}
