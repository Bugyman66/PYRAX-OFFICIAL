'use client';

import { useState } from 'react';
import { useTranslations, useLocale } from 'next-intl';
import { useRouter, usePathname } from 'next/navigation';
import { Dialog, Popover, PopoverButton, PopoverPanel } from '@headlessui/react';
import {
  Bars3Icon,
  XMarkIcon,
  ChevronDownIcon,
  CpuChipIcon,
  CubeIcon,
  CurrencyDollarIcon,
  RocketLaunchIcon,
  BookOpenIcon,
  ArrowDownTrayIcon,
  DocumentTextIcon,
  MapIcon,
  ChartBarIcon,
  CodeBracketIcon,
  ChatBubbleLeftRightIcon,
  GlobeAltIcon,
  UserGroupIcon,
  NewspaperIcon,
  ServerStackIcon,
  CommandLineIcon,
  BuildingLibraryIcon,
  GiftIcon,
  MegaphoneIcon,
  FilmIcon,
  WrenchScrewdriverIcon,
} from '@heroicons/react/24/outline';
import { locales, localeNames, localeCountryCodes, type Locale } from '@/i18n/config';
import Link from 'next/link';
import Image from 'next/image';

export default function Navbar() {
  const t = useTranslations('nav');
  const locale = useLocale();
  const router = useRouter();
  const pathname = usePathname();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const switchLocale = (newLocale: Locale) => {
    const pathWithoutLocale = pathname.replace(`/${locale}`, '') || '/';
    router.push(`/${newLocale}${pathWithoutLocale}`);
  };

  type NavItem = {
    name: string;
    href: string;
    icon: React.ComponentType<{ className?: string }>;
    description: string;
    external?: boolean;
    newUntil?: Date; // Show "New" badge until this date
  };

  // Helper function to check if an item should show "New" badge
  const isNew = (item: NavItem): boolean => {
    if (!item.newUntil) return false;
    return new Date() < item.newUntil;
  };

  // Blockchain dropdown
  const networkItems: NavItem[] = [
    { name: t('technology'), href: `/${locale}/technology`, icon: CpuChipIcon, description: t('technologyDesc') },
    { name: t('tokenomics'), href: `/${locale}/tokenomics`, icon: CurrencyDollarIcon, description: t('tokenomicsDesc') },
    { name: t('mining'), href: `/${locale}/mining`, icon: CubeIcon, description: t('miningDesc') },
    { name: t('roadmap'), href: `/${locale}/roadmap`, icon: MapIcon, description: t('roadmapDesc') },
  ];

  // AI Systems dropdown
  const aiSystemsItems: NavItem[] = [
    { name: t('foundry'), href: `/${locale}/foundry`, icon: ServerStackIcon, description: t('foundryDesc') },
    { name: t('crucible'), href: `/${locale}/crucible`, icon: CommandLineIcon, description: t('crucibleDesc') },
  ];

  // Resources dropdown
  const resourceItems: NavItem[] = [
    { name: t('whitepaperNav'), href: `/${locale}/whitepaper`, icon: DocumentTextIcon, description: t('whitepaperDesc') },
    { name: t('technicalWhitepaperNav'), href: `/${locale}/technical-whitepaper`, icon: DocumentTextIcon, description: t('technicalWhitepaperDesc') },
    { name: t('downloads'), href: `/${locale}/downloads`, icon: ArrowDownTrayIcon, description: t('downloadsDesc') },
    { name: t('explorer'), href: 'https://explorer.pyrax.org', icon: ChartBarIcon, description: t('explorerDesc'), external: true },
    { name: t('docs'), href: 'https://docs.pyrax.org', icon: BookOpenIcon, description: t('docsDesc'), external: true },
    { name: 'GitHub', href: 'https://github.com/PYRAX-Chain', icon: CodeBracketIcon, description: t('githubDesc'), external: true },
  ];

  // Community dropdown with platform-specific icons
  type SocialNavItem = {
    name: string;
    href: string;
    iconSrc: string;
    description: string;
  };

  // DAO item for community
  const daoItem: NavItem = { name: t('dao'), href: `/${locale}/dao`, icon: BuildingLibraryIcon, description: t('daoDesc') };

  // BDAG Community Initiative item
  const bdagItem: NavItem = { name: t('bdagCommunity'), href: `/${locale}/tokenomics/bdag`, icon: GiftIcon, description: t('bdagCommunityDesc') };

  // Team page item
  const teamItem: NavItem = { name: t('team'), href: `/${locale}/team`, icon: UserGroupIcon, description: t('teamDesc') };

  // Releases dropdown items - newUntil is set to 5 days after the latest article was added
  const releasesItems: NavItem[] = [
    { name: t('communityReleases'), href: `/${locale}/releases/community`, icon: MegaphoneIcon, description: t('communityReleasesDesc'), newUntil: new Date('2026-01-12') }, // Legal launch article added Jan 7
    { name: t('mediaReleases'), href: `/${locale}/releases/media`, icon: FilmIcon, description: t('mediaReleasesDesc') }, // No new articles yet
    { name: t('devReleases'), href: `/${locale}/releases/dev`, icon: WrenchScrewdriverIcon, description: t('devReleasesDesc'), newUntil: new Date('2026-01-12') }, // Dev status article added Jan 7
  ];

  const communityItems: SocialNavItem[] = [
    { name: 'Discord', href: 'https://discord.gg/z9kjrE9q', iconSrc: '/icons/discord.svg', description: t('discordDesc') },
    { name: 'Telegram', href: 'https://t.me/+TmDvlOc8TxxmNzAx', iconSrc: '/icons/telegram.svg', description: t('telegramDesc') },
    { name: 'Reddit', href: 'https://www.reddit.com/r/PyraxNetwork/', iconSrc: '/icons/reddit.svg', description: t('redditDesc') },
    { name: 'Twitter', href: 'https://twitter.com/pyrax_org', iconSrc: '/icons/twitter.svg', description: t('twitterDesc') },
    { name: 'Facebook', href: 'https://www.facebook.com/share/1BH17cWju3/?mibextid=wwXIfr', iconSrc: '/icons/facebook.svg', description: t('facebookDesc') },
    { name: 'Instagram', href: 'https://www.instagram.com/pyrax_network?igsh=MWJ1emNjczZhbnJrbw==', iconSrc: '/icons/instagram.svg', description: t('instagramDesc') },
    { name: t('facebookGroup'), href: 'https://www.facebook.com/groups/873178285202587', iconSrc: '/icons/facebook-group.svg', description: t('facebookGroupDesc') },
  ];

  const renderNavItem = (item: NavItem) => {
    const content = (
      <div className="group relative flex gap-x-6 rounded-lg p-4 hover:bg-stone-800/50">
        <div className="mt-1 flex size-11 flex-none items-center justify-center rounded-lg bg-stone-800 group-hover:bg-pyrax-500/20">
          <item.icon
            aria-hidden="true"
            className="size-6 text-stone-400 group-hover:text-pyrax-400"
          />
        </div>
        <div>
          <span className="font-semibold text-white flex items-center gap-2">
            {item.name}
            {isNew(item) && (
              <span className="inline-flex items-center px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide bg-pyrax-500 text-white rounded animate-pulse">
                New
              </span>
            )}
            <span className="absolute inset-0" />
          </span>
          <p className="mt-1 text-stone-400">{item.description}</p>
        </div>
      </div>
    );

    if (item.external) {
      return (
        <a
          key={item.name}
          href={item.href}
          target="_blank"
          rel="noopener noreferrer"
        >
          {content}
        </a>
      );
    }

    return (
      <Link key={item.name} href={item.href}>
        {content}
      </Link>
    );
  };

  return (
    <header className="fixed top-0 left-0 right-0 z-50 bg-stone-950/90 backdrop-blur-xl border-b border-stone-800/50">
      <nav className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8" aria-label="Global">
        <div className="flex h-16 items-center justify-between">
          {/* Logo */}
          <Link href={`/${locale}`} className="flex items-center gap-3">
            <Image
              src="/pyrax-brand/pyrax-logo.svg"
              alt="PYRAX"
              width={40}
              height={40}
              className="h-10 w-auto"
            />
          </Link>

          {/* Desktop Navigation */}
          <div className="hidden lg:flex lg:items-center lg:gap-1">
            {/* Network Dropdown */}
            <Popover className="relative">
              <PopoverButton className="inline-flex items-center gap-x-1 px-4 py-2 text-sm font-semibold text-stone-300 hover:text-white transition-colors rounded-lg hover:bg-stone-800/50 outline-none">
                <span>{t('blockchain')}</span>
                <ChevronDownIcon aria-hidden="true" className="size-5" />
              </PopoverButton>

              <PopoverPanel
                transition
                className="absolute left-1/2 z-10 mt-5 flex w-screen max-w-max -translate-x-1/2 px-4 transition data-[closed]:translate-y-1 data-[closed]:opacity-0 data-[enter]:duration-200 data-[enter]:ease-out data-[leave]:duration-150 data-[leave]:ease-in"
              >
                <div className="w-screen max-w-md flex-auto overflow-hidden rounded-3xl bg-stone-900 text-sm/6 shadow-lg ring-1 ring-stone-800 lg:max-w-xl">
                  <div className="grid grid-cols-1 gap-x-6 gap-y-1 p-4 lg:grid-cols-2">
                    {networkItems.map((item) => renderNavItem(item))}
                  </div>
                </div>
              </PopoverPanel>
            </Popover>

            {/* AI Systems Dropdown */}
            <Popover className="relative">
              <PopoverButton className="inline-flex items-center gap-x-1 px-4 py-2 text-sm font-semibold text-stone-300 hover:text-white transition-colors rounded-lg hover:bg-stone-800/50 outline-none">
                <span>{t('aiSystems')}</span>
                <ChevronDownIcon aria-hidden="true" className="size-5" />
              </PopoverButton>

              <PopoverPanel
                transition
                className="absolute left-1/2 z-10 mt-5 flex w-screen max-w-max -translate-x-1/2 px-4 transition data-[closed]:translate-y-1 data-[closed]:opacity-0 data-[enter]:duration-200 data-[enter]:ease-out data-[leave]:duration-150 data-[leave]:ease-in"
              >
                <div className="w-screen max-w-sm flex-auto overflow-hidden rounded-3xl bg-stone-900 text-sm/6 shadow-lg ring-1 ring-stone-800">
                  <div className="p-4">
                    {aiSystemsItems.map((item) => renderNavItem(item))}
                  </div>
                  <div className="bg-stone-800/50 px-8 py-6">
                    <div className="flex items-center gap-x-3">
                      <h3 className="text-sm/6 font-semibold text-white">{t('aiPlatform')}</h3>
                      <p className="rounded-full bg-pyrax-500/10 px-2.5 py-1.5 text-xs font-semibold text-pyrax-400">
                        {t('aiCompute')}
                      </p>
                    </div>
                    <p className="mt-2 text-sm/6 text-stone-400">
                      {t('aiSystemsDesc')}
                    </p>
                  </div>
                </div>
              </PopoverPanel>
            </Popover>

            {/* Resources Dropdown */}
            <Popover className="relative">
              <PopoverButton className="inline-flex items-center gap-x-1 px-4 py-2 text-sm font-semibold text-stone-300 hover:text-white transition-colors rounded-lg hover:bg-stone-800/50 outline-none">
                <span>{t('resources')}</span>
                <ChevronDownIcon aria-hidden="true" className="size-5" />
              </PopoverButton>

              <PopoverPanel
                transition
                className="absolute left-1/2 z-10 mt-5 flex w-screen max-w-max -translate-x-1/2 px-4 transition data-[closed]:translate-y-1 data-[closed]:opacity-0 data-[enter]:duration-200 data-[enter]:ease-out data-[leave]:duration-150 data-[leave]:ease-in"
              >
                <div className="w-screen max-w-md flex-auto overflow-hidden rounded-3xl bg-stone-900 text-sm/6 shadow-lg ring-1 ring-stone-800 lg:max-w-xl">
                  <div className="grid grid-cols-1 gap-x-6 gap-y-1 p-4 lg:grid-cols-2">
                    {resourceItems.map((item) => renderNavItem(item))}
                  </div>
                  <div className="bg-stone-800/50 px-8 py-6">
                    <div className="flex items-center gap-x-3">
                      <h3 className="text-sm/6 font-semibold text-white">{t('developerTools')}</h3>
                      <p className="rounded-full bg-pyrax-500/10 px-2.5 py-1.5 text-xs font-semibold text-pyrax-400">
                        {t('openSource')}
                      </p>
                    </div>
                    <p className="mt-2 text-sm/6 text-stone-400">
                      {t('buildOnPyrax')}
                    </p>
                  </div>
                </div>
              </PopoverPanel>
            </Popover>

            {/* Community Dropdown */}
            <Popover className="relative">
              <PopoverButton className="inline-flex items-center gap-x-1 px-4 py-2 text-sm font-semibold text-stone-300 hover:text-white transition-colors rounded-lg hover:bg-stone-800/50 outline-none">
                <span>{t('community')}</span>
                <ChevronDownIcon aria-hidden="true" className="size-5" />
              </PopoverButton>

              <PopoverPanel
                transition
                className="absolute left-1/2 z-10 mt-5 flex w-screen max-w-max -translate-x-1/2 px-4 transition data-[closed]:translate-y-1 data-[closed]:opacity-0 data-[enter]:duration-200 data-[enter]:ease-out data-[leave]:duration-150 data-[leave]:ease-in"
              >
                <div className="w-screen max-w-md flex-auto overflow-hidden rounded-3xl bg-stone-900 text-sm/6 shadow-lg ring-1 ring-stone-800 lg:max-w-3xl">
                  <div className="p-4 border-b border-stone-800">
                    {renderNavItem(teamItem)}
                    {renderNavItem(daoItem)}
                    {renderNavItem(bdagItem)}
                  </div>
                  <div className="grid grid-cols-1 gap-x-6 gap-y-1 p-4 lg:grid-cols-2">
                    {communityItems.map((item) => (
                      <a
                        key={item.name}
                        href={item.href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="group relative flex gap-x-6 rounded-lg p-4 hover:bg-stone-800/50"
                      >
                        <div className="mt-1 flex size-11 flex-none items-center justify-center rounded-lg bg-stone-800 group-hover:bg-pyrax-500/20">
                          <Image
                            src={item.iconSrc}
                            alt={item.name}
                            width={24}
                            height={24}
                            className="size-6 opacity-70 group-hover:opacity-100"
                          />
                        </div>
                        <div>
                          <span className="font-semibold text-white">
                            {item.name}
                            <span className="absolute inset-0" />
                          </span>
                          <p className="mt-1 text-stone-400">{item.description}</p>
                        </div>
                      </a>
                    ))}
                  </div>
                  <div className="bg-stone-800/50 px-8 py-6">
                    <div className="flex items-center gap-x-3">
                      <h3 className="text-sm/6 font-semibold text-white">{t('joinMovement')}</h3>
                      <p className="rounded-full bg-pyrax-500/10 px-2.5 py-1.5 text-xs font-semibold text-pyrax-400">
                        {t('growingFast')}
                      </p>
                    </div>
                    <p className="mt-2 text-sm/6 text-stone-400">
                      {t('connectCommunity')}
                    </p>
                  </div>
                </div>
              </PopoverPanel>
            </Popover>

            {/* Releases Dropdown */}
            <Popover className="relative">
              <PopoverButton className="inline-flex items-center gap-x-1 px-4 py-2 text-sm font-semibold text-stone-300 hover:text-white transition-colors rounded-lg hover:bg-stone-800/50 outline-none">
                <span>{t('releases')}</span>
                <ChevronDownIcon aria-hidden="true" className="size-5" />
              </PopoverButton>

              <PopoverPanel
                transition
                className="absolute left-1/2 z-10 mt-5 flex w-screen max-w-max -translate-x-1/2 px-4 transition data-[closed]:translate-y-1 data-[closed]:opacity-0 data-[enter]:duration-200 data-[enter]:ease-out data-[leave]:duration-150 data-[leave]:ease-in"
              >
                <div className="w-screen max-w-sm flex-auto overflow-hidden rounded-3xl bg-stone-900 text-sm/6 shadow-lg ring-1 ring-stone-800">
                  <div className="p-4">
                    {releasesItems.map((item) => renderNavItem(item))}
                  </div>
                  <div className="bg-stone-800/50 px-8 py-6">
                    <div className="flex items-center gap-x-3">
                      <h3 className="text-sm/6 font-semibold text-white">{t('latestNews')}</h3>
                      <p className="rounded-full bg-pyrax-500/10 px-2.5 py-1.5 text-xs font-semibold text-pyrax-400">
                        {t('stayUpdated')}
                      </p>
                    </div>
                    <p className="mt-2 text-sm/6 text-stone-400">
                      {t('releasesDesc')}
                    </p>
                  </div>
                </div>
              </PopoverPanel>
            </Popover>
          </div>

          {/* Right side - Language & CTA */}
          <div className="flex items-center gap-2">
            {/* Language Selector */}
            <Popover className="relative">
              <PopoverButton className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-stone-300 hover:text-white transition-colors rounded-lg hover:bg-stone-800/50 border border-stone-700 outline-none">
                <span className={`fi fi-${localeCountryCodes[locale as Locale]} rounded-sm`} style={{ width: '20px', height: '15px' }} />
                <ChevronDownIcon className="h-4 w-4" />
              </PopoverButton>

              <PopoverPanel
                transition
                className="absolute right-0 z-10 mt-5 w-72 max-h-96 overflow-auto transition data-[closed]:translate-y-1 data-[closed]:opacity-0 data-[enter]:duration-200 data-[enter]:ease-out data-[leave]:duration-150 data-[leave]:ease-in"
              >
                <div className="overflow-hidden rounded-3xl bg-stone-900 shadow-lg ring-1 ring-stone-800 p-4">
                  <div className="grid grid-cols-2 gap-1">
                    {locales.map((loc) => (
                      <button
                        key={loc}
                        onClick={() => switchLocale(loc)}
                        className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors ${
                          locale === loc
                            ? 'bg-pyrax-500/20 text-pyrax-400'
                            : 'text-stone-300 hover:bg-stone-800 hover:text-white'
                        }`}
                      >
                        <span className={`fi fi-${localeCountryCodes[loc]} rounded-sm`} style={{ width: '20px', height: '15px' }} />
                        <span className="truncate">{localeNames[loc]}</span>
                      </button>
                    ))}
                  </div>
                </div>
              </PopoverPanel>
            </Popover>

            {/* CTA Button */}
            <a
              href="https://explorer.pyrax.org"
              target="_blank"
              rel="noopener noreferrer"
              className="hidden sm:inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-4 py-2 text-sm font-semibold text-white shadow-lg shadow-pyrax-500/25 hover:shadow-pyrax-500/40 transition-all hover:scale-105"
            >
              <RocketLaunchIcon className="h-4 w-4" />
              {t('launchExplorer')}
            </a>

            {/* Mobile menu button */}
            <button
              type="button"
              className="lg:hidden inline-flex items-center justify-center rounded-lg p-2 text-stone-400 hover:bg-stone-800 hover:text-white"
              onClick={() => setMobileMenuOpen(true)}
            >
              <Bars3Icon className="h-6 w-6" />
            </button>
          </div>
        </div>
      </nav>

      {/* Mobile menu */}
      <Dialog as="div" className="lg:hidden" open={mobileMenuOpen} onClose={setMobileMenuOpen}>
        <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" />
        <Dialog.Panel className="fixed inset-y-0 right-0 z-50 w-full overflow-y-auto bg-stone-950 px-6 py-6 sm:max-w-sm border-l border-stone-800">
          <div className="flex items-center justify-between">
            <Link href={`/${locale}`} className="flex items-center gap-3" onClick={() => setMobileMenuOpen(false)}>
              <Image src="/pyrax-brand/pyrax-logo.svg" alt="PYRAX" width={40} height={40} className="h-10 w-auto" />
            </Link>
            <button
              type="button"
              className="rounded-lg p-2 text-stone-400 hover:bg-stone-800 hover:text-white"
              onClick={() => setMobileMenuOpen(false)}
            >
              <XMarkIcon className="h-6 w-6" />
            </button>
          </div>

          <div className="mt-6 flow-root">
            <div className="-my-6 divide-y divide-stone-800">
              {/* Network */}
              <div className="py-6">
                <p className="px-3 py-2 text-xs font-semibold text-stone-500 uppercase">{t('blockchain')}</p>
                {networkItems.map((item) => (
                  item.external ? (
                    <a
                      key={item.name}
                      href={item.href}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <item.icon className="h-5 w-5 text-pyrax-400" />
                      {item.name}
                    </a>
                  ) : (
                    <Link
                      key={item.name}
                      href={item.href}
                      className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <item.icon className="h-5 w-5 text-pyrax-400" />
                      {item.name}
                    </Link>
                  )
                ))}
              </div>

              {/* AI Systems */}
              <div className="py-6">
                <p className="px-3 py-2 text-xs font-semibold text-stone-500 uppercase">{t('aiSystems')}</p>
                {aiSystemsItems.map((item) => (
                  <Link
                    key={item.name}
                    href={item.href}
                    className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <item.icon className="h-5 w-5 text-pyrax-400" />
                    {item.name}
                  </Link>
                ))}
              </div>

              {/* Resources */}
              <div className="py-6">
                <p className="px-3 py-2 text-xs font-semibold text-stone-500 uppercase">{t('resources')}</p>
                {resourceItems.map((item) => (
                  item.external ? (
                    <a
                      key={item.name}
                      href={item.href}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <item.icon className="h-5 w-5 text-pyrax-400" />
                      {item.name}
                    </a>
                  ) : (
                    <Link
                      key={item.name}
                      href={item.href}
                      className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <item.icon className="h-5 w-5 text-pyrax-400" />
                      {item.name}
                    </Link>
                  )
                ))}
              </div>

              {/* Community */}
              <div className="py-6">
                <p className="px-3 py-2 text-xs font-semibold text-stone-500 uppercase">{t('community')}</p>
                <Link
                  href={teamItem.href}
                  className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <teamItem.icon className="h-5 w-5 text-pyrax-400" />
                  {teamItem.name}
                </Link>
                <Link
                  href={daoItem.href}
                  className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <daoItem.icon className="h-5 w-5 text-pyrax-400" />
                  {daoItem.name}
                </Link>
                <Link
                  href={bdagItem.href}
                  className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  <bdagItem.icon className="h-5 w-5 text-pyrax-400" />
                  {bdagItem.name}
                </Link>
                {communityItems.map((item) => (
                  <a
                    key={item.name}
                    href={item.href}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <Image src={item.iconSrc} alt={item.name} width={20} height={20} className="opacity-80" />
                    {item.name}
                  </a>
                ))}
              </div>

              {/* Releases */}
              <div className="py-6">
                <p className="px-3 py-2 text-xs font-semibold text-stone-500 uppercase">{t('releases')}</p>
                {releasesItems.map((item) => (
                  <Link
                    key={item.name}
                    href={item.href}
                    className="flex items-center gap-3 rounded-lg px-3 py-2 text-base font-medium text-stone-300 hover:bg-stone-800 hover:text-white"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    <item.icon className="h-5 w-5 text-pyrax-400" />
                    <span className="flex items-center gap-2">
                      {item.name}
                      {isNew(item) && (
                        <span className="inline-flex items-center px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide bg-pyrax-500 text-white rounded animate-pulse">
                          New
                        </span>
                      )}
                    </span>
                  </Link>
                ))}
              </div>

              {/* Language Selector */}
              <div className="py-6">
                <p className="px-3 py-2 text-xs font-semibold text-stone-500 uppercase">{t('language')}</p>
                <div className="grid grid-cols-2 gap-1 px-3">
                  {locales.map((loc) => (
                    <button
                      key={loc}
                      onClick={() => {
                        switchLocale(loc);
                        setMobileMenuOpen(false);
                      }}
                      className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors ${
                        locale === loc
                          ? 'bg-pyrax-500/20 text-pyrax-400'
                          : 'text-stone-300 hover:bg-stone-800 hover:text-white'
                      }`}
                    >
                      <span className={`fi fi-${localeCountryCodes[loc]} rounded-sm`} style={{ width: '20px', height: '15px' }} />
                      <span className="truncate">{localeNames[loc]}</span>
                    </button>
                  ))}
                </div>
              </div>

              {/* CTA */}
              <div className="py-6">
                <a
                  href="https://explorer.pyrax.org"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-4 py-3 text-sm font-semibold text-white"
                >
                  <RocketLaunchIcon className="h-5 w-5" />
                  {t('launchApp')}
                </a>
              </div>
            </div>
          </div>
        </Dialog.Panel>
      </Dialog>
    </header>
  );
}
