'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import {
  WrenchScrewdriverIcon,
} from '@heroicons/react/24/outline';

export default function DevReleasesPage() {
  const t = useTranslations('releasesPage');

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

      {/* Coming Soon */}
      <section className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="bg-stone-900/50 border border-stone-800 rounded-2xl p-12 text-center"
          >
            <WrenchScrewdriverIcon className="w-16 h-16 text-stone-600 mx-auto mb-6" />
            <h2 className="text-2xl font-bold text-white mb-4">{t('comingSoon')}</h2>
            <p className="text-stone-400 max-w-md mx-auto">
              {t('dev.comingSoonDesc')}
            </p>
          </motion.div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
