'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import { BuildingLibraryIcon, ClockIcon } from '@heroicons/react/24/outline';

export default function DAOPage() {
  const t = useTranslations('daoPage');

  return (
    <div className="min-h-screen bg-stone-950">
      <Navbar />
      
      <main className="relative pt-24 pb-16">
        {/* Background gradient */}
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/5 via-transparent to-transparent" />
        
        <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          {/* Hero Section */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            className="text-center py-20 lg:py-32"
          >
            {/* Icon */}
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              transition={{ delay: 0.2, type: 'spring', stiffness: 200 }}
              className="mx-auto w-24 h-24 rounded-3xl bg-gradient-to-br from-pyrax-500/20 to-pyrax-600/20 border border-pyrax-500/30 flex items-center justify-center mb-8"
            >
              <BuildingLibraryIcon className="w-12 h-12 text-pyrax-400" />
            </motion.div>

            {/* Title */}
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold text-white mb-4">
              {t('title')}
            </h1>
            
            {/* Subtitle */}
            <p className="text-xl text-stone-400 mb-12">
              {t('subtitle')}
            </p>

            {/* Coming Soon Card */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4, duration: 0.6 }}
              className="max-w-2xl mx-auto"
            >
              <div className="bg-stone-900/50 border border-stone-800 rounded-3xl p-8 md:p-12 backdrop-blur-sm">
                <div className="flex items-center justify-center gap-3 mb-6">
                  <ClockIcon className="w-8 h-8 text-pyrax-400" />
                  <h2 className="text-2xl md:text-3xl font-bold text-white">
                    {t('comingSoon')}
                  </h2>
                </div>
                
                <p className="text-stone-400 text-lg leading-relaxed">
                  {t('description')}
                </p>

                {/* Decorative elements */}
                <div className="mt-8 flex justify-center gap-2">
                  {[...Array(3)].map((_, i) => (
                    <motion.div
                      key={i}
                      className="w-2 h-2 rounded-full bg-pyrax-500/50"
                      animate={{
                        scale: [1, 1.5, 1],
                        opacity: [0.5, 1, 0.5],
                      }}
                      transition={{
                        duration: 1.5,
                        repeat: Infinity,
                        delay: i * 0.3,
                      }}
                    />
                  ))}
                </div>
              </div>
            </motion.div>
          </motion.div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
