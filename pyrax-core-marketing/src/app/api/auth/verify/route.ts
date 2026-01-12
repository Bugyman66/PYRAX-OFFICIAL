import { NextRequest, NextResponse } from 'next/server';
import { verifyMagicLinkToken, createSession } from '@/lib/auth';
import { cookies } from 'next/headers';

export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  try {
    const token = request.nextUrl.searchParams.get('token');

    if (!token) {
      return NextResponse.redirect(new URL('/auth/login?error=invalid', request.url));
    }

    const user = await verifyMagicLinkToken(token);

    if (!user) {
      return NextResponse.redirect(new URL('/auth/login?error=expired', request.url));
    }

    const sessionToken = await createSession(user.id);

    const cookieStore = await cookies();
    cookieStore.set('session', sessionToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 30 * 24 * 60 * 60,
      path: '/',
    });

    const redirectUrl = user.isInternal ? '/app' : '/me/submissions';
    return NextResponse.redirect(new URL(redirectUrl, request.url));
  } catch (error) {
    console.error('Token verification error:', error);
    return NextResponse.redirect(new URL('/auth/login?error=server', request.url));
  }
}
