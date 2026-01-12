import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { driveService, DriveServiceError } from '@/lib/drive';

export const dynamic = 'force-dynamic';

export async function POST(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }

    if (!session.isInternal) {
      return NextResponse.json(
        { error: 'Forbidden - Internal users only' },
        { status: 403 }
      );
    }

    const body = await request.json();
    const { name, parentId } = body;

    if (!name || typeof name !== 'string') {
      return NextResponse.json(
        { error: 'Folder name is required' },
        { status: 400 }
      );
    }

    if (name.length > 255) {
      return NextResponse.json(
        { error: 'Folder name must be 255 characters or less' },
        { status: 400 }
      );
    }

    const folder = await driveService.createFolder(name, parentId);

    return NextResponse.json({
      success: true,
      folder,
    });
  } catch (error) {
    console.error('Drive folder creation error:', error);

    if (error instanceof DriveServiceError) {
      return NextResponse.json(
        { error: error.message, code: error.code },
        { status: error.statusCode }
      );
    }

    return NextResponse.json(
      { error: 'Failed to create folder' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }

    const { searchParams } = new URL(request.url);
    const folderId = searchParams.get('folderId') || undefined;
    const pageToken = searchParams.get('pageToken') || undefined;

    const result = await driveService.listFiles(folderId, pageToken);

    return NextResponse.json({
      success: true,
      ...result,
    });
  } catch (error) {
    console.error('Drive list files error:', error);

    if (error instanceof DriveServiceError) {
      return NextResponse.json(
        { error: error.message, code: error.code },
        { status: error.statusCode }
      );
    }

    return NextResponse.json(
      { error: 'Failed to list files' },
      { status: 500 }
    );
  }
}
