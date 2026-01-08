'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import {
  RocketLaunchIcon,
  BookOpenIcon,
  ChatBubbleLeftRightIcon,
} from '@heroicons/react/24/outline';

export default function CTA() {
  const t = useTranslations('cta');

  return (
    <section className="relative py-24 overflow-hidden">
      {/* Background */}
      <div className="absolute inset-0 bg-gradient-to-b from-stone-950 via-stone-900 to-stone-950" />
      
      {/* Gradient orbs */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-1/2 left-1/4 -translate-y-1/2 w-[500px] h-[500px] rounded-full bg-pyrax-500/20 blur-[128px]" />
        <div className="absolute top-1/2 right-1/4 -translate-y-1/2 w-[400px] h-[400px] rounded-full bg-blue-500/20 blur-[128px]" />
      </div>
      
      <div className="relative z-10 max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <h2 className="text-4xl sm:text-5xl font-bold text-white mb-4">
            {t('title')}
          </h2>
          <p className="text-xl text-stone-400 mb-10">
            {t('subtitle')}
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <a
              href="https://explorer.testnet.pyrax.org"
              className="group inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-8 py-4 text-lg font-semibold text-white shadow-lg shadow-pyrax-500/25 hover:shadow-pyrax-500/40 transition-all hover:scale-105"
            >
              <RocketLaunchIcon className="h-5 w-5" />
              {t('buttons.explorer')}
            </a>
            
            <a
              href="https://docs.testnet.pyrax.org"
              className="inline-flex items-center gap-2 rounded-xl border border-stone-700 bg-stone-900/50 px-8 py-4 text-lg font-semibold text-white hover:bg-stone-800 transition-all"
            >
              <BookOpenIcon className="h-5 w-5" />
              {t('buttons.docs')}
            </a>
            
            <a
              href="https://discord.gg/z9kjrE9q"
              className="inline-flex items-center gap-2 rounded-xl border border-stone-700 bg-stone-900/50 px-8 py-4 text-lg font-semibold text-white hover:bg-stone-800 transition-all"
            >
              <ChatBubbleLeftRightIcon className="h-5 w-5" />
              {t('buttons.discord')}
            </a>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
