import { NextRequest, NextResponse } from 'next/server';
import { createMagicLinkToken, isInternalEmail } from '@/lib/auth';
import { sendMagicLinkEmail } from '@/lib/email';

export async function POST(request: NextRequest) {
  try {
    const { email, type } = await request.json();

    if (!email || typeof email !== 'string') {
      return NextResponse.json({ error: 'Email is required' }, { status: 400 });
    }

    const normalizedEmail = email.toLowerCase().trim();
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(normalizedEmail)) {
      return NextResponse.json({ error: 'Invalid email format' }, { status: 400 });
    }

    if (type === 'internal' && !isInternalEmail(normalizedEmail)) {
      return NextResponse.json(
        { error: 'Internal login requires a @pyrax.org email address' },
        { status: 403 }
      );
    }

    const token = await createMagicLinkToken(normalizedEmail);
    
    if (!token) {
      return NextResponse.json(
        { error: 'Unable to create login link. Please try again.' },
        { status: 500 }
      );
    }

    const emailSent = await sendMagicLinkEmail(normalizedEmail, token);
    
    if (!emailSent) {
      console.error('Failed to send magic link email to:', normalizedEmail);
    }

    return NextResponse.json({
      success: true,
      message: 'If an account exists, a login link has been sent to your email.',
    });
  } catch (error) {
    console.error('Magic link request error:', error);
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    return NextResponse.json(
      { error: `An error occurred: ${errorMessage}` },
      { status: 500 }
    );
  }
}
