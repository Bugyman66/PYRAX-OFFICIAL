export const locales = [
  'en',    // English
  'zh',    // Chinese (Simplified)
  'zh-TW', // Chinese (Traditional)
  'ja',    // Japanese
  'ko',    // Korean
  'ru',    // Russian
  'es',    // Spanish
  'pt',    // Portuguese
  'de',    // German
  'fr',    // French
  'it',    // Italian
  'tr',    // Turkish
  'vi',    // Vietnamese
  'th',    // Thai
  'id',    // Indonesian
  'ar',    // Arabic
  'hi',    // Hindi
  'nl',    // Dutch
  'pl',    // Polish
  'uk',    // Ukrainian
] as const;

export type Locale = (typeof locales)[number];

export const defaultLocale: Locale = 'en';

export const localeNames: Record<Locale, string> = {
  'en': 'English',
  'zh': '简体中文',
  'zh-TW': '繁體中文',
  'ja': '日本語',
  'ko': '한국어',
  'ru': 'Русский',
  'es': 'Español',
  'pt': 'Português',
  'de': 'Deutsch',
  'fr': 'Français',
  'it': 'Italiano',
  'tr': 'Türkçe',
  'vi': 'Tiếng Việt',
  'th': 'ไทย',
  'id': 'Bahasa Indonesia',
  'ar': 'العربية',
  'hi': 'हिन्दी',
  'nl': 'Nederlands',
  'pl': 'Polski',
  'uk': 'Українська',
};

export const localeCountryCodes: Record<Locale, string> = {
  'en': 'us',
  'zh': 'cn',
  'zh-TW': 'tw',
  'ja': 'jp',
  'ko': 'kr',
  'ru': 'ru',
  'es': 'es',
  'pt': 'br',
  'de': 'de',
  'fr': 'fr',
  'it': 'it',
  'tr': 'tr',
  'vi': 'vn',
  'th': 'th',
  'id': 'id',
  'ar': 'sa',
  'hi': 'in',
  'nl': 'nl',
  'pl': 'pl',
  'uk': 'ua',
};
