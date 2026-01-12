import { NextResponse } from 'next/server';
import { destroySession } from '@/lib/auth';
import { cookies } from 'next/headers';

export const dynamic = 'force-dynamic';

export async function POST() {
  try {
    await destroySession();

    const cookieStore = await cookies();
    cookieStore.delete('session');

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Logout error:', error);
    return NextResponse.json({ error: 'Logout failed' }, { status: 500 });
  }
}

export async function GET() {
  try {
    await destroySession();

    const cookieStore = await cookies();
    cookieStore.delete('session');

    return NextResponse.redirect(new URL('/auth/login', process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'));
  } catch (error) {
    console.error('Logout error:', error);
    return NextResponse.redirect(new URL('/auth/login', process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'));
  }
}
