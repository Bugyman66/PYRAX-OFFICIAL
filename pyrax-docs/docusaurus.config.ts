import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'PYRAX Documentation',
  tagline: 'Complete guides for the PYRAX Network - AI-Powered GPU Mining & Decentralized Computing',
  favicon: 'img/favicon.png',

  future: {
    v4: true,
  },

  url: 'https://docs.pyrax.org',
  baseUrl: '/',

  organizationName: 'PYRAX-Chain',
  projectName: 'pyrax-docs',

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'zh', 'zh-TW', 'es', 'de', 'fr', 'ja', 'ko', 'pt', 'ru', 'it', 'nl', 'pl', 'tr', 'vi', 'th', 'id', 'ar', 'hi', 'uk'],
    localeConfigs: {
      en: { label: 'English', direction: 'ltr' },
      zh: { label: '简体中文', direction: 'ltr' },
      'zh-TW': { label: '繁體中文', direction: 'ltr' },
      es: { label: 'Español', direction: 'ltr' },
      de: { label: 'Deutsch', direction: 'ltr' },
      fr: { label: 'Français', direction: 'ltr' },
      ja: { label: '日本語', direction: 'ltr' },
      ko: { label: '한국어', direction: 'ltr' },
      pt: { label: 'Português', direction: 'ltr' },
      ru: { label: 'Русский', direction: 'ltr' },
      it: { label: 'Italiano', direction: 'ltr' },
      nl: { label: 'Nederlands', direction: 'ltr' },
      pl: { label: 'Polski', direction: 'ltr' },
      tr: { label: 'Türkçe', direction: 'ltr' },
      vi: { label: 'Tiếng Việt', direction: 'ltr' },
      th: { label: 'ไทย', direction: 'ltr' },
      id: { label: 'Bahasa Indonesia', direction: 'ltr' },
      ar: { label: 'العربية', direction: 'rtl' },
      hi: { label: 'हिन्दी', direction: 'ltr' },
      uk: { label: 'Українська', direction: 'ltr' },
    },
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/pyrax-social-card.png',
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: '',
      logo: {
        alt: 'PYRAX Logo',
        src: 'img/pyrax-logo.svg',
        href: '/',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'generalSidebar',
          position: 'left',
          label: 'General',
        },
        {
          type: 'docSidebar',
          sidebarId: 'developersSidebar',
          position: 'left',
          label: 'Developers',
        },
        {
          href: 'https://pyrax.org',
          label: '← Back to pyrax.org',
          position: 'right',
          className: 'navbar-back-link',
        },
        {
          type: 'localeDropdown',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            { label: 'Getting Started', to: '/docs/general/getting-started' },
            { label: 'What is PYRAX?', to: '/docs/general/what-is-pyrax' },
            { label: 'Developer Guides', to: '/docs/developers/overview' },
          ],
        },
        {
          title: 'Community',
          items: [
            { label: 'Discord', href: 'https://discord.gg/sS7kaacRwU' },
            { label: 'Telegram', href: 'https://t.me/+TmDvlOc8TxxmNzAx' },
            { label: 'Twitter', href: 'https://twitter.com/pyrax_org' },
            { label: 'Reddit', href: 'https://www.reddit.com/r/PyraxNetwork/' },
          ],
        },
        {
          title: 'Resources',
          items: [
            { label: 'Main Website', href: 'https://pyrax.org' },
            { label: 'GitHub', href: 'https://github.com/PYRAX-Chain' },
            { label: 'Whitepaper', href: 'https://pyrax.org/whitepaper' },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} PYRAX Network. All rights reserved.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['bash', 'json', 'typescript', 'solidity', 'rust'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
