import { getRequestConfig } from 'next-intl/server';
import { locales, type Locale } from './config';

export default getRequestConfig(async ({ requestLocale }) => {
  let locale = await requestLocale;
  
  if (!locale || !locales.includes(locale as Locale)) {
    locale = 'en';
  }

  // Always load English as fallback
  const englishMessages = (await import(`../messages/en.json`)).default;
  
  // Load locale messages and merge with English fallback
  let localeMessages = englishMessages;
  if (locale !== 'en') {
    try {
      const imported = (await import(`../messages/${locale}.json`)).default;
      // Deep merge: locale messages override English fallback
      localeMessages = deepMerge(englishMessages, imported);
    } catch {
      // If locale file fails, use English
      localeMessages = englishMessages;
    }
  }

  return {
    locale,
    messages: localeMessages,
    onError: (error) => {
      // Silently handle missing translations in production
      if (process.env.NODE_ENV === 'development') {
        console.warn(error.message);
      }
    },
    getMessageFallback: ({ key, namespace }) => {
      // Return the key as fallback instead of throwing
      return namespace ? `${namespace}.${key}` : key;
    }
  };
});

// Deep merge helper function
function deepMerge(target: Record<string, unknown>, source: Record<string, unknown>): Record<string, unknown> {
  const result = { ...target };
  for (const key of Object.keys(source)) {
    if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
      result[key] = deepMerge(
        (target[key] as Record<string, unknown>) || {},
        source[key] as Record<string, unknown>
      );
    } else {
      result[key] = source[key];
    }
  }
  return result;
}
