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

    const { stream, metadata } = await driveService.downloadFile(fileId);

    const rangeHeader = request.headers.get('range');
    
    if (rangeHeader && metadata.mimeType.startsWith('video/')) {
      const parts = rangeHeader.replace(/bytes=/, '').split('-');
      const start = parseInt(parts[0], 10);
      const end = parts[1] ? parseInt(parts[1], 10) : metadata.size - 1;
      const chunkSize = end - start + 1;

      const headers = new Headers({
        'Content-Type': metadata.mimeType,
        'Content-Range': `bytes ${start}-${end}/${metadata.size}`,
        'Accept-Ranges': 'bytes',
        'Content-Length': chunkSize.toString(),
        'Content-Disposition': `inline; filename="${encodeURIComponent(metadata.name)}"`,
        'Cache-Control': 'private, max-age=3600',
      });

      const chunks: Uint8Array[] = [];
      for await (const chunk of stream) {
        chunks.push(chunk);
      }
      const fullBuffer = Buffer.concat(chunks);
      const slicedBuffer = fullBuffer.slice(start, end + 1);

      return new NextResponse(slicedBuffer, {
        status: 206,
        headers,
      });
    }

    const headers = new Headers({
      'Content-Type': metadata.mimeType,
      'Content-Length': metadata.size.toString(),
      'Content-Disposition': `inline; filename="${encodeURIComponent(metadata.name)}"`,
      'Cache-Control': 'private, max-age=3600',
      'Accept-Ranges': 'bytes',
    });

    const chunks: Uint8Array[] = [];
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    const buffer = Buffer.concat(chunks);

    return new NextResponse(buffer, {
      status: 200,
      headers,
    });
  } catch (error) {
    console.error('Drive stream error:', error);

    if (error instanceof DriveServiceError) {
      return NextResponse.json(
        { error: error.message, code: error.code },
        { status: error.statusCode }
      );
    }

    return NextResponse.json(
      { error: 'Failed to stream file' },
      { status: 500 }
    );
  }
}
