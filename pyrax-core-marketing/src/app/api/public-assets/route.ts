import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, Prisma } from '@prisma/client';

function generateSlug(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .substring(0, 50) + '-' + Date.now().toString(36);
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const isPublic = searchParams.get('public') === 'true';
    const mediaKitSection = searchParams.get('section');

    if (isPublic) {
      const where: Prisma.PublicAssetWhereInput = {
        isPublished: true,
      };

      if (mediaKitSection) {
        where.mediaKitSection = mediaKitSection;
      }

      const assets = await prisma.publicAsset.findMany({
        where,
        include: {
          proof: {
            include: {
              versions: {
                where: { kind: 'Source' },
                orderBy: { versionNumber: 'desc' },
                take: 1,
                select: {
                  id: true,
                  fileName: true,
                  mimeType: true,
                  driveFileId: true,
                },
              },
            },
          },
        },
        orderBy: [
          { mediaKitSection: 'asc' },
          { displayOrder: 'asc' },
          { publishedAt: 'desc' },
        ],
      });

      return NextResponse.json(
        assets.map(asset => ({
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
          } : null,
        }))
      );
    }

    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const where: Prisma.PublicAssetWhereInput = {};

    if (session.role !== UserRole.OrgAdmin && session.departmentId) {
      where.proof = {
        folder: {
          departmentId: session.departmentId,
        },
      };
    }

    const assets = await prisma.publicAsset.findMany({
      where,
      include: {
        proof: {
          include: {
            folder: { select: { id: true, name: true } },
            versions: {
              where: { kind: 'Source' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
              select: { id: true, fileName: true, mimeType: true },
            },
          },
        },
        publishedBy: {
          select: { id: true, name: true, email: true },
        },
      },
      orderBy: { createdAt: 'desc' },
    });

    return NextResponse.json(
      assets.map(asset => ({
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
      }))
    );
  } catch (error) {
    console.error('Get public assets error:', error);
    return NextResponse.json({ error: 'Failed to fetch assets' }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    if (session.role !== UserRole.OrgAdmin && session.role !== UserRole.DepartmentHead) {
      return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
    }

    const body = await request.json();
    const { proofId, title, description, slug, downloadAllowed, mediaKitSection, isPublished } = body;

    if (!proofId) {
      return NextResponse.json({ error: 'Proof ID is required' }, { status: 400 });
    }

    const proof = await prisma.proof.findUnique({
      where: { id: proofId },
      include: {
        folder: true,
        publicAsset: true,
      },
    });

    if (!proof) {
      return NextResponse.json({ error: 'Proof not found' }, { status: 404 });
    }

    if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId !== proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    }

    if (proof.publicAsset) {
      return NextResponse.json({ error: 'Proof already has a public asset' }, { status: 400 });
    }

    const finalSlug = slug?.trim() || generateSlug(title || proof.title);

    const existingSlug = await prisma.publicAsset.findUnique({
      where: { slug: finalSlug },
    });

    if (existingSlug) {
      return NextResponse.json({ error: 'Slug already exists' }, { status: 400 });
    }

    const asset = await prisma.publicAsset.create({
      data: {
        proofId,
        slug: finalSlug,
        title: title?.trim() || null,
        description: description?.trim() || null,
        downloadAllowed: downloadAllowed !== false,
        mediaKitSection: mediaKitSection?.trim() || null,
        displayOrder: 0,
        isPublished: isPublished === true,
        publishedById: session.id,
        publishedAt: isPublished ? new Date() : null,
      },
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
        entityId: asset.id,
        action: 'created',
        actorId: session.id,
        payloadJson: {
          proofId,
          slug: finalSlug,
          isPublished,
        } as unknown as Prisma.InputJsonValue,
      },
    });

    return NextResponse.json({
      id: asset.id,
      slug: asset.slug,
      title: asset.title || asset.proof.title,
      description: asset.description,
      downloadAllowed: asset.downloadAllowed,
      mediaKitSection: asset.mediaKitSection,
      isPublished: asset.isPublished,
      publishedAt: asset.publishedAt,
      publishedBy: asset.publishedBy,
      proof: {
        id: asset.proof.id,
        title: asset.proof.title,
        folder: asset.proof.folder,
      },
    }, { status: 201 });
  } catch (error) {
    console.error('Create public asset error:', error);
    return NextResponse.json({ error: 'Failed to create asset' }, { status: 500 });
  }
}
