import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { driveService, DriveServiceError } from '@/lib/drive';

export const dynamic = 'force-dynamic';

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string }> }
) {
  try {
    const { slug } = await params;

    if (!slug) {
      return NextResponse.json(
        { error: 'Slug is required' },
        { status: 400 }
      );
    }

    const asset = await prisma.publicAsset.findUnique({
      where: { slug },
      include: {
        proof: {
          include: {
            versions: {
              where: { kind: 'Rendition' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
            },
          },
        },
      },
    });

    if (!asset) {
      return NextResponse.json(
        { error: 'Asset not found' },
        { status: 404 }
      );
    }

    if (!asset.isPublished) {
      return NextResponse.json(
        { error: 'Asset is not published' },
        { status: 404 }
      );
    }

    if (!asset.downloadAllowed) {
      return NextResponse.json(
        { error: 'Download not allowed for this asset' },
        { status: 403 }
      );
    }

    let version = asset.proof.versions[0];
    
    if (!version) {
      const sourceVersion = await prisma.proofVersion.findFirst({
        where: { 
          proofId: asset.proofId,
          kind: 'Source',
        },
        orderBy: { versionNumber: 'desc' },
      });
      
      if (!sourceVersion) {
        return NextResponse.json(
          { error: 'No downloadable version found' },
          { status: 404 }
        );
      }
      version = sourceVersion;
    }

    const { stream, metadata } = await driveService.downloadFile(version.driveFileId);

    const headers = new Headers({
      'Content-Type': metadata.mimeType,
      'Content-Length': metadata.size.toString(),
      'Content-Disposition': `attachment; filename="${encodeURIComponent(version.fileName)}"`,
      'Cache-Control': 'public, max-age=3600',
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
    console.error('Public download error:', error);

    if (error instanceof DriveServiceError) {
      return NextResponse.json(
        { error: error.message },
        { status: error.statusCode }
      );
    }

    return NextResponse.json(
      { error: 'Failed to download file' },
      { status: 500 }
    );
  }
}
