import createMiddleware from 'next-intl/middleware';
import { locales, defaultLocale } from './i18n/config';

export default createMiddleware({
  locales,
  defaultLocale,
  localePrefix: 'always',
  localeDetection: true
});

export const config = {
  matcher: [
    // Match all pathnames except static files and API routes
    '/',
    '/(en|zh|zh-TW|ja|ko|ru|es|pt|de|fr|it|tr|vi|th|id|ar|hi|nl|pl|uk)/:path*',
    '/((?!api|_next|_vercel|pyrax-brand|.*\\..*).*)'
  ]
};
