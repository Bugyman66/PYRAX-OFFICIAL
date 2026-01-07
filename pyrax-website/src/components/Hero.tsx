'use client';

import { useTranslations, useLocale } from 'next-intl';
import { motion } from 'framer-motion';
import Link from 'next/link';
import {
  ArrowRightIcon,
  DocumentTextIcon,
  BoltIcon,
  ClockIcon,
  CubeIcon,
  CpuChipIcon,
} from '@heroicons/react/24/outline';

export default function Hero() {
  const t = useTranslations('hero');
  const locale = useLocale();

  const stats = [
    { value: t('stats.tps'), label: t('stats.tpsLabel'), icon: BoltIcon },
    { value: t('stats.blockTime'), label: t('stats.blockTimeLabel'), icon: ClockIcon },
    { value: t('stats.supply'), label: t('stats.supplyLabel'), icon: CubeIcon },
    { value: t('stats.streams'), label: t('stats.streamsLabel'), icon: CpuChipIcon },
  ];

  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden pt-16">
      {/* Video Background */}
      <div className="absolute inset-0">
        <video
          autoPlay
          loop
          muted
          playsInline
          className="absolute inset-0 w-full h-full object-cover"
        >
          <source src="https://pyrax-assets.nyc3.cdn.digitaloceanspaces.com/media/EmberBG.mp4" type="video/mp4" />
        </video>
        {/* Dark overlay for text legibility */}
        <div className="absolute inset-0 bg-black/70" />
        {/* Gradient overlay for additional depth */}
        <div className="absolute inset-0 bg-gradient-to-b from-stone-950/50 via-transparent to-stone-950" />
      </div>

      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20">
        <div className="text-center">
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="inline-flex items-center gap-2 rounded-full bg-pyrax-500/10 border border-pyrax-500/20 px-4 py-2 text-sm text-pyrax-400 mb-8"
          >
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-pyrax-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-pyrax-500"></span>
            </span>
            {t('badge')}
          </motion.div>

          {/* Title */}
          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-5xl sm:text-6xl lg:text-7xl font-bold tracking-tight"
          >
            <span className="text-white">{t('title')}</span>
            <br />
            <span className="bg-gradient-to-r from-pyrax-400 via-pyrax-500 to-orange-400 bg-clip-text text-transparent">
              {t('titleHighlight')}
            </span>
          </motion.h1>

          {/* Subtitle */}
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="mt-6 text-lg sm:text-xl text-stone-400 max-w-3xl mx-auto leading-relaxed"
          >
            {t('subtitle')}
          </motion.p>

          {/* CTAs */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4"
          >
            <Link
              href={`/${locale}/technology`}
              className="group inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-8 py-4 text-lg font-semibold text-white shadow-lg shadow-pyrax-500/25 hover:shadow-pyrax-500/40 transition-all hover:scale-105"
            >
              {t('cta.primary')}
              <ArrowRightIcon className="h-5 w-5 group-hover:translate-x-1 transition-transform" />
            </Link>
            <a
              href="/whitepaper.pdf"
              className="inline-flex items-center gap-2 rounded-xl border border-stone-700 bg-stone-900/50 px-8 py-4 text-lg font-semibold text-white hover:bg-stone-800 transition-all"
            >
              <DocumentTextIcon className="h-5 w-5" />
              {t('cta.secondary')}
            </a>
          </motion.div>

          {/* Stats */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.4 }}
            className="mt-20 grid grid-cols-2 lg:grid-cols-4 gap-4 max-w-4xl mx-auto"
          >
            {stats.map((stat, index) => (
              <motion.div
                key={stat.label}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.3, delay: 0.5 + index * 0.1 }}
                className="relative group"
              >
                <div className="absolute inset-0 bg-gradient-to-r from-pyrax-500/20 to-orange-500/20 rounded-2xl blur-xl opacity-0 group-hover:opacity-100 transition-opacity" />
                <div className="relative rounded-2xl bg-stone-900/80 border border-stone-800 p-6 hover:border-pyrax-500/50 transition-colors">
                  <stat.icon className="h-6 w-6 text-pyrax-500 mb-3" />
                  <p className="text-3xl font-bold bg-gradient-to-r from-white to-stone-300 bg-clip-text text-transparent">
                    {stat.value}
                  </p>
                  <p className="text-sm text-stone-500 mt-1">{stat.label}</p>
                </div>
              </motion.div>
            ))}
          </motion.div>
        </div>

        {/* Scroll indicator */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1 }}
          className="absolute bottom-8 left-1/2 -translate-x-1/2"
        >
          <motion.div
            animate={{ y: [0, 10, 0] }}
            transition={{ duration: 2, repeat: Infinity }}
            className="w-6 h-10 rounded-full border-2 border-stone-700 flex justify-center"
          >
            <motion.div
              animate={{ y: [0, 12, 0], opacity: [1, 0, 1] }}
              transition={{ duration: 2, repeat: Infinity }}
              className="w-1.5 h-3 bg-pyrax-500 rounded-full mt-2"
            />
          </motion.div>
        </motion.div>
      </div>
    </section>
  );
}
