import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { prisma } from '@/lib/prisma';
import { UserRole, VersionKind } from '@prisma/client';
import { driveService } from '@/lib/drive';

export const dynamic = 'force-dynamic';

const MAX_FILE_SIZE = 500 * 1024 * 1024; // 500MB

const ALLOWED_MIME_TYPES = [
  // PDF
  'application/pdf',
  
  // Raster Images
  'image/jpeg',
  'image/png',
  'image/gif',
  'image/webp',
  'image/tiff',
  'image/bmp',
  'image/x-icon',
  'image/ico',
  'image/heic',
  'image/heif',
  'image/avif',
  'image/jxl',
  
  // Vector Images
  'image/svg+xml',
  'application/postscript',
  'application/eps',
  'application/x-eps',
  'image/x-eps',
  
  // Adobe Creative Suite
  'application/illustrator',
  'application/vnd.adobe.illustrator',
  'application/x-photoshop',
  'image/vnd.adobe.photoshop',
  'image/x-photoshop',
  'application/photoshop',
  'application/psd',
  'image/psd',
  'application/vnd.adobe.indesign',
  'application/x-indesign',
  'application/vnd.adobe.xd',
  'application/vnd.adobe.aftereffects.project',
  'application/vnd.adobe.premiere',
  
  // Other Design Tools
  'application/x-sketch',
  'application/sketch',
  'application/figma',
  'application/vnd.figma',
  'application/x-xcf',  // GIMP
  'image/x-xcf',
  'application/x-krita',
  'application/x-clip-studio-paint',
  'application/vnd.corel-draw',
  'application/coreldraw',
  'application/x-coreldraw',
  'application/vnd.afdesign',  // Affinity Designer
  'application/vnd.afphoto',   // Affinity Photo
  
  // Video Formats
  'video/mp4',
  'video/quicktime',
  'video/webm',
  'video/x-msvideo',
  'video/avi',
  'video/x-matroska',
  'video/mkv',
  'video/x-flv',
  'video/x-ms-wmv',
  'video/mpeg',
  'video/3gpp',
  'video/3gpp2',
  'video/ogg',
  'video/x-m4v',
  'video/mp2t',
  'video/x-mng',
  'video/vnd.avi',
  
  // Animation
  'image/apng',
  'image/gif',
  
  // Raw Camera Formats
  'image/x-raw',
  'image/x-canon-cr2',
  'image/x-canon-cr3',
  'image/x-nikon-nef',
  'image/x-sony-arw',
  'image/x-adobe-dng',
  'image/x-fuji-raf',
  'image/x-panasonic-rw2',
  'image/x-olympus-orf',
];

interface RouteParams {
  params: Promise<{ proofId: string }>;
}

async function canAccessProof(session: NonNullable<Awaited<ReturnType<typeof getSession>>>, proofId: string) {
  const proof = await prisma.proof.findUnique({
    where: { id: proofId },
    include: {
      folder: { select: { departmentId: true } },
    },
  });

  if (!proof) return { allowed: false, proof: null, reason: 'Proof not found' };

  if (session.role === UserRole.OrgAdmin) {
    return { allowed: true, proof };
  }

  if (session.departmentId === proof.folder.departmentId) {
    return { allowed: true, proof };
  }

  return { allowed: false, proof: null, reason: 'Access denied' };
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { proofId } = await params;
    const access = await canAccessProof(session, proofId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Proof not found' ? 404 : 403 }
      );
    }

    const versions = await prisma.proofVersion.findMany({
      where: { proofId },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
        _count: { select: { annotations: true, comments: true } },
      },
      orderBy: { versionNumber: 'desc' },
    });

    return NextResponse.json({ versions });
  } catch (error) {
    console.error('Get versions error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch versions' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    
    if (!session || !session.isInternal) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { proofId } = await params;
    const access = await canAccessProof(session, proofId);

    if (!access.allowed) {
      return NextResponse.json(
        { error: access.reason },
        { status: access.reason === 'Proof not found' ? 404 : 403 }
      );
    }

    const formData = await request.formData();
    const file = formData.get('file') as File | null;
    const kindParam = formData.get('kind') as string | null;

    if (!file) {
      return NextResponse.json(
        { error: 'No file provided' },
        { status: 400 }
      );
    }

    if (file.size > MAX_FILE_SIZE) {
      return NextResponse.json(
        { error: `File size exceeds maximum of ${MAX_FILE_SIZE / 1024 / 1024}MB` },
        { status: 400 }
      );
    }

    const mimeType = file.type || 'application/octet-stream';
    const isAllowed = ALLOWED_MIME_TYPES.some(
      (allowed) => mimeType === allowed || mimeType.startsWith(allowed.split('/')[0] + '/')
    );

    if (!isAllowed && !mimeType.startsWith('image/') && !mimeType.startsWith('video/')) {
      return NextResponse.json(
        { error: 'File type not allowed' },
        { status: 400 }
      );
    }

    const kind: VersionKind = kindParam === 'Rendition' ? VersionKind.Rendition : VersionKind.Source;

    const latestVersion = await prisma.proofVersion.findFirst({
      where: { proofId, kind },
      orderBy: { versionNumber: 'desc' },
      select: { versionNumber: true },
    });

    const nextVersionNumber = (latestVersion?.versionNumber || 0) + 1;

    const buffer = Buffer.from(await file.arrayBuffer());

    const proof = await prisma.proof.findUnique({
      where: { id: proofId },
      include: { folder: true },
    });

    let folderId: string | undefined;
    try {
      const proofFolder = await driveService.createFolder(
        `${proof!.title} (${proofId.slice(0, 8)})`,
        undefined
      );
      folderId = proofFolder.id;
    } catch {
      folderId = undefined;
    }

    const driveFile = await driveService.uploadFile({
      buffer,
      fileName: `v${nextVersionNumber}_${file.name}`,
      mimeType,
      folderId,
      description: `${proof!.title} - Version ${nextVersionNumber}`,
    });

    const version = await prisma.proofVersion.create({
      data: {
        proofId,
        versionNumber: nextVersionNumber,
        driveFileId: driveFile.id,
        fileName: file.name,
        mimeType,
        fileSize: file.size,
        kind,
        createdById: session.id,
        metadata: {
          originalName: file.name,
          uploadedAt: new Date().toISOString(),
          driveWebViewLink: driveFile.webViewLink,
        },
      },
      include: {
        createdBy: { select: { id: true, name: true, email: true } },
      },
    });

    await prisma.proof.update({
      where: { id: proofId },
      data: { updatedAt: new Date() },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'ProofVersion',
        entityId: version.id,
        action: 'created',
        actorId: session.id,
        payloadJson: {
          proofId,
          versionNumber: nextVersionNumber,
          fileName: file.name,
          fileSize: file.size,
          kind,
        },
      },
    });

    return NextResponse.json({ version }, { status: 201 });
  } catch (error) {
    console.error('Create version error:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to create version' },
      { status: 500 }
    );
  }
}
