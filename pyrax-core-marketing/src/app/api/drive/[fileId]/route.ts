import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { driveService, DriveServiceError } from '@/lib/drive';

export const dynamic = 'force-dynamic';

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ fileId: string }> }
) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }

    const { fileId } = await params;

    if (!fileId) {
      return NextResponse.json(
        { error: 'File ID is required' },
        { status: 400 }
      );
    }

    const metadata = await driveService.getFileMetadata(fileId);

    return NextResponse.json({
      success: true,
      file: metadata,
    });
  } catch (error) {
    console.error('Drive metadata error:', error);

    if (error instanceof DriveServiceError) {
      return NextResponse.json(
        { error: error.message, code: error.code },
        { status: error.statusCode }
      );
    }

    return NextResponse.json(
      { error: 'Failed to get file metadata' },
      { status: 500 }
    );
  }
}

export async function DELETE(
  request: NextRequest,
  { params }: { params: Promise<{ fileId: string }> }
) {
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

    const { fileId } = await params;

    if (!fileId) {
      return NextResponse.json(
        { error: 'File ID is required' },
        { status: 400 }
      );
    }

    await driveService.deleteFile(fileId);

    return NextResponse.json({
      success: true,
      message: 'File deleted successfully',
    });
  } catch (error) {
    console.error('Drive delete error:', error);

    if (error instanceof DriveServiceError) {
      return NextResponse.json(
        { error: error.message, code: error.code },
        { status: error.statusCode }
      );
    }

    return NextResponse.json(
      { error: 'Failed to delete file' },
      { status: 500 }
    );
  }
}
