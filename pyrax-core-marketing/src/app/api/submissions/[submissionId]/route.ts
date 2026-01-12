import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';
import { UserRole } from '@prisma/client';

interface RouteParams {
  params: Promise<{ submissionId: string }>;
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const { submissionId } = await params;

    const submission = await prisma.externalSubmission.findUnique({
      where: { id: submissionId },
      include: {
        user: {
          select: { id: true, name: true, email: true },
        },
        proof: {
          include: {
            folder: {
              select: { id: true, name: true, departmentId: true },
              include: {
                department: { select: { id: true, name: true } },
              },
            },
            versions: {
              orderBy: { versionNumber: 'desc' },
              select: {
                id: true,
                versionNumber: true,
                fileName: true,
                mimeType: true,
                fileSize: true,
                kind: true,
                createdAt: true,
              },
            },
            workflowInstance: {
              include: {
                template: { select: { id: true, name: true } },
                decisions: {
                  orderBy: { createdAt: 'desc' },
                  take: 5,
                  include: {
                    decidedBy: { select: { id: true, name: true, email: true } },
                  },
                },
              },
            },
            createdBy: { select: { id: true, name: true, email: true } },
          },
        },
      },
    });

    if (!submission) {
      return NextResponse.json({ error: 'Submission not found' }, { status: 404 });
    }

    const session = await getSession();
    
    if (session) {
      if (session.role === UserRole.PublicSubmitter && session.id !== submission.userId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
      
      if (session.role !== UserRole.OrgAdmin && 
          session.role !== UserRole.PublicSubmitter &&
          session.departmentId !== submission.proof.folder.departmentId) {
        return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
      }
    } else {
      return NextResponse.json({
        id: submission.id,
        message: submission.message,
        category: submission.category,
        createdAt: submission.createdAt,
        proof: {
          id: submission.proof.id,
          title: submission.proof.title,
          status: submission.proof.status,
        },
      });
    }

    return NextResponse.json({
      id: submission.id,
      message: submission.message,
      category: submission.category,
      createdAt: submission.createdAt,
      submitter: submission.user,
      proof: {
        id: submission.proof.id,
        title: submission.proof.title,
        description: submission.proof.description,
        status: submission.proof.status,
        folder: submission.proof.folder,
        versions: submission.proof.versions,
        workflow: submission.proof.workflowInstance ? {
          id: submission.proof.workflowInstance.id,
          status: submission.proof.workflowInstance.status,
          currentStepIndex: submission.proof.workflowInstance.currentStepIndex,
          template: submission.proof.workflowInstance.template,
          recentDecisions: submission.proof.workflowInstance.decisions,
        } : null,
        createdBy: submission.proof.createdBy,
        createdAt: submission.proof.createdAt,
      },
    });
  } catch (error) {
    console.error('Get submission error:', error);
    return NextResponse.json({ error: 'Failed to fetch submission' }, { status: 500 });
  }
}
