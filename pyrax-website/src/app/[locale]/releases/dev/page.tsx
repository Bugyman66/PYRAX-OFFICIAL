'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Link from 'next/link';
import { useLocale } from 'next-intl';
import {
  WrenchScrewdriverIcon,
  CalendarIcon,
  ArrowRightIcon,
} from '@heroicons/react/24/outline';

interface Release {
  slug: string;
  title: string;
  date: string;
  excerpt: string;
}

export default function DevReleasesPage() {
  const t = useTranslations('releasesPage');
  const locale = useLocale();

  const releases: Release[] = [
    {
      slug: 'development-status-january-2026',
      title: 'Development Status Update',
      date: 'January 7, 2026',
      excerpt: 'A comprehensive look at where PYRAX stands today — ~85% core infrastructure complete. Learn about the TriStream consensus, PYRAX Desktop app, AI platform, and our roadmap to mainnet.',
    },
  ];

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />

      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-green-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-green-500/5 blur-[128px]" />

        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-green-500/10 border border-green-500/20 mb-6">
              <WrenchScrewdriverIcon className="w-5 h-5 text-green-500" />
              <span className="text-green-400 font-medium">{t('dev.badge')}</span>
            </div>
            <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6">
              {t('dev.title')}
            </h1>
            <p className="text-xl text-stone-400 max-w-3xl mx-auto">
              {t('dev.subtitle')}
            </p>
          </motion.div>
        </div>
      </section>

      {/* Releases List */}
      <section className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="space-y-6">
            {releases.map((release, index) => (
              <motion.div
                key={release.slug}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
              >
                <Link
                  href={`/${locale}/releases/dev/${release.slug}`}
                  className="block group"
                >
                  <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 hover:border-green-500/50 transition-all duration-300">
                    <div className="flex items-center gap-2 text-sm text-stone-500 mb-3">
                      <CalendarIcon className="w-4 h-4" />
                      <span>{release.date}</span>
                    </div>
                    <h2 className="text-2xl font-bold text-white mb-3 group-hover:text-green-400 transition-colors">
                      {release.title}
                    </h2>
                    <p className="text-stone-400 mb-4">
                      {release.excerpt}
                    </p>
                    <div className="flex items-center gap-2 text-green-500 font-medium">
                      <span>{t('readMore')}</span>
                      <ArrowRightIcon className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                    </div>
                  </div>
                </Link>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
