import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { getSession } from '@/lib/auth';

export async function GET() {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    let prefs = await prisma.notificationPreference.findUnique({
      where: { userId: session.id },
    });

    if (!prefs) {
      prefs = await prisma.notificationPreference.create({
        data: {
          userId: session.id,
          onAssignment: true,
          onMention: true,
          onStepChange: true,
          onDecision: true,
          onApproval: true,
          onSubmission: true,
        },
      });
    }

    return NextResponse.json({
      onAssignment: prefs.onAssignment,
      onMention: prefs.onMention,
      onStepChange: prefs.onStepChange,
      onDecision: prefs.onDecision,
      onApproval: prefs.onApproval,
      onSubmission: prefs.onSubmission,
    });
  } catch (error) {
    console.error('Get notification preferences error:', error);
    return NextResponse.json({ error: 'Failed to fetch preferences' }, { status: 500 });
  }
}

export async function PATCH(request: NextRequest) {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { onAssignment, onMention, onStepChange, onDecision, onApproval, onSubmission } = body;

    const prefs = await prisma.notificationPreference.upsert({
      where: { userId: session.id },
      update: {
        ...(onAssignment !== undefined && { onAssignment }),
        ...(onMention !== undefined && { onMention }),
        ...(onStepChange !== undefined && { onStepChange }),
        ...(onDecision !== undefined && { onDecision }),
        ...(onApproval !== undefined && { onApproval }),
        ...(onSubmission !== undefined && { onSubmission }),
      },
      create: {
        userId: session.id,
        onAssignment: onAssignment ?? true,
        onMention: onMention ?? true,
        onStepChange: onStepChange ?? true,
        onDecision: onDecision ?? true,
        onApproval: onApproval ?? true,
        onSubmission: onSubmission ?? true,
      },
    });

    return NextResponse.json({
      onAssignment: prefs.onAssignment,
      onMention: prefs.onMention,
      onStepChange: prefs.onStepChange,
      onDecision: prefs.onDecision,
      onApproval: prefs.onApproval,
      onSubmission: prefs.onSubmission,
    });
  } catch (error) {
    console.error('Update notification preferences error:', error);
    return NextResponse.json({ error: 'Failed to update preferences' }, { status: 500 });
  }
}
