import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { driveService, DriveServiceError } from '@/lib/drive';

export const dynamic = 'force-dynamic';

const MAX_FILE_SIZE = 500 * 1024 * 1024; // 500MB

const ALLOWED_MIME_TYPES = [
  'application/pdf',
  'image/jpeg',
  'image/png',
  'image/gif',
  'image/webp',
  'image/svg+xml',
  'image/tiff',
  'video/mp4',
  'video/quicktime',
  'video/webm',
  'video/x-msvideo',
  'application/postscript',
  'application/illustrator',
  'image/vnd.adobe.photoshop',
  'application/x-photoshop',
  'application/vnd.adobe.indesign',
  'application/x-indesign',
  'application/zip',
  'application/x-zip-compressed',
];

export async function POST(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }

    if (!session.isInternal && session.role !== 'PublicSubmitter') {
      return NextResponse.json(
        { error: 'Forbidden' },
        { status: 403 }
      );
    }

    const formData = await request.formData();
    const file = formData.get('file') as File | null;
    const folderId = formData.get('folderId') as string | null;
    const description = formData.get('description') as string | null;

    if (!file) {
      return NextResponse.json(
        { error: 'No file provided' },
        { status: 400 }
      );
    }

    if (file.size > MAX_FILE_SIZE) {
      return NextResponse.json(
        { error: `File size exceeds maximum allowed (${MAX_FILE_SIZE / 1024 / 1024}MB)` },
        { status: 400 }
      );
    }

    const mimeType = file.type || 'application/octet-stream';
    
    if (!ALLOWED_MIME_TYPES.includes(mimeType) && !mimeType.startsWith('image/') && !mimeType.startsWith('video/')) {
      return NextResponse.json(
        { error: `File type not allowed: ${mimeType}` },
        { status: 400 }
      );
    }

    const buffer = Buffer.from(await file.arrayBuffer());

    const result = await driveService.uploadFile({
      buffer,
      fileName: file.name,
      mimeType,
      folderId: folderId || undefined,
      description: description || undefined,
    });

    return NextResponse.json({
      success: true,
      file: {
        id: result.id,
        name: result.name,
        mimeType: result.mimeType,
        size: result.size,
        createdTime: result.createdTime,
      },
    });
  } catch (error) {
    console.error('Drive upload error:', error);

    if (error instanceof DriveServiceError) {
      return NextResponse.json(
        { error: error.message, code: error.code },
        { status: error.statusCode }
      );
    }

    return NextResponse.json(
      { error: 'Failed to upload file' },
      { status: 500 }
    );
  }
}
