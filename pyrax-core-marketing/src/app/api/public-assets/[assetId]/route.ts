import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, Prisma } from '@prisma/client';

interface RouteParams {
  params: Promise<{ assetId: string }>;
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const { assetId } = await params;

    const asset = await prisma.publicAsset.findFirst({
      where: {
        OR: [
          { id: assetId },
          { slug: assetId },
        ],
      },
      include: {
        proof: {
          include: {
            folder: { select: { id: true, name: true, departmentId: true } },
            versions: {
              where: { kind: 'Source' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
              select: {
                id: true,
                fileName: true,
                mimeType: true,
                driveFileId: true,
                fileSize: true,
              },
            },
          },
        },
        publishedBy: {
          select: { id: true, name: true, email: true },
        },
      },
    });

    if (!asset) {
      return NextResponse.json({ error: 'Asset not found' }, { status: 404 });
    }

    const { searchParams } = new URL(request.url);
    const isPublicRequest = searchParams.get('public') === 'true';

    if (isPublicRequest) {
      if (!asset.isPublished) {
        return NextResponse.json({ error: 'Asset not found' }, { status: 404 });
      }

      return NextResponse.json({
        id: asset.id,
        slug: asset.slug,
        title: asset.title || asset.proof.title,
        description: asset.description,
        downloadAllowed: asset.downloadAllowed,
        mediaKitSection: asset.mediaKitSection,
        publishedAt: asset.publishedAt,
        version: asset.proof.versions[0] ? {
          id: asset.proof.versions[0].id,
          fileName: asset.proof.versions[0].fileName,
          mimeType: asset.proof.versions[0].mimeType,
          fileSize: asset.proof.versions[0].fileSize,
        } : null,
      });
    }

    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== asset.proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    return NextResponse.json({
      id: asset.id,
      slug: asset.slug,
      title: asset.title || asset.proof.title,
      description: asset.description,
      downloadAllowed: asset.downloadAllowed,
      mediaKitSection: asset.mediaKitSection,
      displayOrder: asset.displayOrder,
      isPublished: asset.isPublished,
      publishedAt: asset.publishedAt,
      publishedBy: asset.publishedBy,
      proof: {
        id: asset.proof.id,
        title: asset.proof.title,
        folder: asset.proof.folder,
      },
      version: asset.proof.versions[0] || null,
      createdAt: asset.createdAt,
      updatedAt: asset.updatedAt,
    });
  } catch (error) {
    console.error('Get public asset error:', error);
    return NextResponse.json({ error: 'Failed to fetch asset' }, { status: 500 });
  }
}

export async function PATCH(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
    }

    const { assetId } = await params;

    const asset = await prisma.publicAsset.findUnique({
      where: { id: assetId },
      include: {
        proof: {
          include: {
            folder: true,
          },
        },
      },
    });

    if (!asset) {
      return NextResponse.json({ error: 'Asset not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== asset.proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    const body = await request.json();
    const { title, description, slug, downloadAllowed, mediaKitSection, displayOrder, isPublished } = body;

    const updateData: Prisma.PublicAssetUpdateInput = {};

    if (title !== undefined) {
      updateData.title = title?.trim() || null;
    }

    if (description !== undefined) {
      updateData.description = description?.trim() || null;
    }

    if (slug !== undefined && slug !== asset.slug) {
      const existingSlug = await prisma.publicAsset.findFirst({
        where: {
          slug: slug.trim(),
          id: { not: assetId },
        },
      });

      if (existingSlug) {
        return NextResponse.json({ error: 'Slug already exists' }, { status: 400 });
      }

      updateData.slug = slug.trim();
    }

    if (downloadAllowed !== undefined) {
      updateData.downloadAllowed = downloadAllowed;
    }

    if (mediaKitSection !== undefined) {
      updateData.mediaKitSection = mediaKitSection?.trim() || null;
    }

    if (displayOrder !== undefined) {
      updateData.displayOrder = displayOrder;
    }

    if (isPublished !== undefined) {
      updateData.isPublished = isPublished;
      if (isPublished && !asset.isPublished) {
        updateData.publishedAt = new Date();
      }
    }

    const updated = await prisma.publicAsset.update({
      where: { id: assetId },
      data: updateData,
      include: {
        proof: {
          include: {
            folder: { select: { id: true, name: true } },
          },
        },
        publishedBy: {
          select: { id: true, name: true, email: true },
        },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'PublicAsset',
        entityId: assetId,
        action: 'updated',
        actorId: session.id,
        payloadJson: {
          changes: Object.keys(body),
        } as unknown as Prisma.InputJsonValue,
      },
    });

    return NextResponse.json({
      id: updated.id,
      slug: updated.slug,
      title: updated.title || updated.proof.title,
      description: updated.description,
      downloadAllowed: updated.downloadAllowed,
      mediaKitSection: updated.mediaKitSection,
      displayOrder: updated.displayOrder,
      isPublished: updated.isPublished,
      publishedAt: updated.publishedAt,
      publishedBy: updated.publishedBy,
      proof: {
        id: updated.proof.id,
        title: updated.proof.title,
        folder: updated.proof.folder,
      },
    });
  } catch (error) {
    console.error('Update public asset error:', error);
    return NextResponse.json({ error: 'Failed to update asset' }, { status: 500 });
  }
}

export async function DELETE(request: NextRequest, { params }: RouteParams) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
    }

    const { assetId } = await params;

    const asset = await prisma.publicAsset.findUnique({
      where: { id: assetId },
      include: {
        proof: {
          include: {
            folder: true,
          },
        },
      },
    });

    if (!asset) {
      return NextResponse.json({ error: 'Asset not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== asset.proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    await prisma.publicAsset.delete({
      where: { id: assetId },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'PublicAsset',
        entityId: assetId,
        action: 'deleted',
        actorId: session.id,
        payloadJson: {
          slug: asset.slug,
          proofId: asset.proofId,
        } as unknown as Prisma.InputJsonValue,
      },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Delete public asset error:', error);
    return NextResponse.json({ error: 'Failed to delete asset' }, { status: 500 });
  }
}
