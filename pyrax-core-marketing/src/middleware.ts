import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const sessionToken = request.cookies.get('session')?.value;

  const publicRoutes = [
    '/auth/login',
    '/auth/register',
    '/auth/verify',
    '/media-kit',
    '/public-library',
    '/api/auth',
    '/api/public',
  ];

  const isPublicRoute = publicRoutes.some(route => pathname.startsWith(route));
  const isApiRoute = pathname.startsWith('/api/');
  const isStaticRoute = pathname.startsWith('/_next') || pathname.startsWith('/favicon') || pathname === '/tailwind.css';

  if (isStaticRoute) {
    return NextResponse.next();
  }

  if (pathname === '/') {
    if (sessionToken) {
      return NextResponse.redirect(new URL('/app', request.url));
    }
    return NextResponse.redirect(new URL('/auth/login', request.url));
  }

  if (isPublicRoute) {
    return NextResponse.next();
  }

  if (!sessionToken) {
    if (isApiRoute) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }
    const loginUrl = new URL('/auth/login', request.url);
    loginUrl.searchParams.set('redirect', pathname);
    return NextResponse.redirect(loginUrl);
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    '/((?!_next/static|_next/image|favicon.ico|tailwind.css).*)',
  ],
};
