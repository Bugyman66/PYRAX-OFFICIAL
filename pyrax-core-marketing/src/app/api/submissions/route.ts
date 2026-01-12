import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole, ProofStatus, Prisma } from '@prisma/client';

export async function GET(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const status = searchParams.get('status');

    const where: Prisma.ExternalSubmissionWhereInput = {};

    if (session.role === UserRole.PublicSubmitter) {
      where.userId = session.id;
    } else if (session.role !== UserRole.OrgAdmin) {
      if (session.departmentId) {
        where.proof = {
          folder: {
            departmentId: session.departmentId,
          },
        };
      }
    }

    if (status) {
      where.proof = {
        ...(where.proof as Prisma.ProofWhereInput || {}),
        status: status as ProofStatus,
      };
    }

    const submissions = await prisma.externalSubmission.findMany({
      where,
      include: {
        user: {
          select: { id: true, name: true, email: true },
        },
        proof: {
          include: {
            folder: {
              select: { id: true, name: true },
              include: {
                department: { select: { id: true, name: true } },
              },
            },
            versions: {
              where: { kind: 'Source' },
              orderBy: { versionNumber: 'desc' },
              take: 1,
              select: { id: true, fileName: true, mimeType: true },
            },
            workflowInstance: {
              select: { id: true, status: true, currentStepIndex: true },
            },
          },
        },
      },
      orderBy: { createdAt: 'desc' },
    });

    return NextResponse.json(
      submissions.map(sub => ({
        id: sub.id,
        message: sub.message,
        category: sub.category,
        createdAt: sub.createdAt,
        submitter: sub.user,
        proof: {
          id: sub.proof.id,
          title: sub.proof.title,
          status: sub.proof.status,
          folder: sub.proof.folder,
          latestVersion: sub.proof.versions[0] || null,
          workflow: sub.proof.workflowInstance,
        },
      }))
    );
  } catch (error) {
    console.error('Get submissions error:', error);
    return NextResponse.json({ error: 'Failed to fetch submissions' }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const session = await getSession();
    
    const body = await request.json();
    const { email, name, title, description, message, category, folderId } = body;

    if (!title || typeof title !== 'string' || title.trim().length === 0) {
      return NextResponse.json({ error: 'Title is required' }, { status: 400 });
    }

    let userId: string;
    let targetFolderId: string;

    if (session) {
      userId = session.id;
      
      if (session.role === UserRole.PublicSubmitter) {
        const submissionFolder = await prisma.folder.findFirst({
          where: { name: 'External Submissions' },
        });
        
        if (!submissionFolder) {
          return NextResponse.json({ error: 'Submission folder not configured' }, { status: 500 });
        }
        targetFolderId = submissionFolder.id;
      } else if (folderId) {
        targetFolderId = folderId;
      } else {
        return NextResponse.json({ error: 'Folder ID required for internal users' }, { status: 400 });
      }
    } else {
      if (!email || typeof email !== 'string') {
        return NextResponse.json({ error: 'Email is required' }, { status: 400 });
      }

      let user = await prisma.user.findUnique({
        where: { email: email.toLowerCase() },
      });

      if (!user) {
        user = await prisma.user.create({
          data: {
            email: email.toLowerCase(),
            name: name?.trim() || null,
            role: UserRole.PublicSubmitter,
            isInternal: false,
          },
        });
      }

      userId = user.id;

      const submissionFolder = await prisma.folder.findFirst({
        where: { name: 'External Submissions' },
      });
      
      if (!submissionFolder) {
        const defaultDepartment = await prisma.department.findFirst();
        if (!defaultDepartment) {
          return NextResponse.json({ error: 'No departments configured' }, { status: 500 });
        }

        const adminUser = await prisma.user.findFirst({
          where: { role: UserRole.OrgAdmin },
        });

        const newFolder = await prisma.folder.create({
          data: {
            name: 'External Submissions',
            description: 'Folder for external user submissions',
            departmentId: defaultDepartment.id,
            createdById: adminUser?.id || userId,
          },
        });
        targetFolderId = newFolder.id;
      } else {
        targetFolderId = submissionFolder.id;
      }
    }

    const proof = await prisma.proof.create({
      data: {
        title: title.trim(),
        description: description?.trim() || null,
        folderId: targetFolderId,
        createdById: userId,
        status: ProofStatus.Draft,
        isExternalSubmission: true,
      },
    });

    const submission = await prisma.externalSubmission.create({
      data: {
        userId,
        proofId: proof.id,
        message: message?.trim() || null,
        category: category?.trim() || null,
      },
      include: {
        user: {
          select: { id: true, name: true, email: true },
        },
        proof: {
          include: {
            folder: {
              select: { id: true, name: true },
            },
          },
        },
      },
    });

    await prisma.auditEvent.create({
      data: {
        entityType: 'ExternalSubmission',
        entityId: submission.id,
        action: 'created',
        actorId: userId,
        payloadJson: {
          proofId: proof.id,
          title: proof.title,
          category,
        } as unknown as Prisma.InputJsonValue,
      },
    });

    return NextResponse.json({
      id: submission.id,
      proofId: proof.id,
      message: submission.message,
      category: submission.category,
      createdAt: submission.createdAt,
      submitter: submission.user,
      proof: {
        id: proof.id,
        title: proof.title,
        status: proof.status,
        folder: submission.proof.folder,
      },
    }, { status: 201 });
  } catch (error) {
    console.error('Create submission error:', error);
    return NextResponse.json({ error: 'Failed to create submission' }, { status: 500 });
  }
}
