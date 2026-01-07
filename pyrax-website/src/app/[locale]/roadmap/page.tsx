'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import {
  CheckCircleIcon,
  ClockIcon,
  RocketLaunchIcon,
  CubeIcon,
  CpuChipIcon,
  GlobeAltIcon,
  ShieldCheckIcon,
  SparklesIcon,
} from '@heroicons/react/24/outline';

export default function RoadmapPage() {
  const t = useTranslations('roadmapPage');

  const phases = [
    {
      id: 'q4-2025',
      title: 'Q4 2025',
      subtitle: t('q1.subtitle'),
      status: 'completed',
      icon: CubeIcon,
      color: 'from-green-500 to-emerald-500',
      items: [
        { text: t('q1.item1'), done: true },
        { text: t('q1.item2'), done: true },
        { text: t('q1.item3'), done: true },
        { text: t('q1.item4'), done: true },
        { text: t('q1.item5'), done: true },
        { text: t('q1.item6'), done: true },
      ],
    },
    {
      id: 'q1-q2-2026',
      title: 'Q1 - Q2 2026',
      subtitle: t('q2.subtitle'),
      status: 'in-progress',
      icon: CpuChipIcon,
      color: 'from-pyrax-500 to-orange-500',
      items: [
        { text: t('q2.item1'), done: true },
        { text: t('q2.item2'), done: true },
        { text: t('q2.item3'), done: true },
        { text: t('q2.item4'), done: true },
        { text: t('q2.item5'), done: true },
        { text: t('q2.item6'), done: true },
        { text: t('q2.item7'), done: true },
        { text: t('q2.item8'), done: false },
      ],
    },
    {
      id: 'q3-2026',
      title: 'Q3 2026',
      subtitle: t('q3.subtitle'),
      status: 'upcoming',
      icon: RocketLaunchIcon,
      color: 'from-blue-500 to-cyan-500',
      items: [
        { text: t('q3.item1'), done: false },
        { text: t('q3.item2'), done: false },
        { text: t('q3.item3'), done: false },
        { text: t('q3.item4'), done: false },
        { text: t('q3.item5'), done: false },
        { text: t('q3.item6'), done: false },
      ],
    },
    {
      id: 'q4-2026',
      title: 'Q4 2026',
      subtitle: t('q4.subtitle'),
      status: 'upcoming',
      icon: GlobeAltIcon,
      color: 'from-purple-500 to-pink-500',
      items: [
        { text: t('q4.item1'), done: false },
        { text: t('q4.item2'), done: false },
        { text: t('q4.item3'), done: false },
        { text: t('q4.item4'), done: false },
        { text: t('q4.item5'), done: false },
        { text: t('q4.item6'), done: false },
      ],
    },
    {
      id: '2027',
      title: '2027+',
      subtitle: t('future.subtitle'),
      status: 'future',
      icon: SparklesIcon,
      color: 'from-stone-500 to-stone-600',
      items: [
        { text: t('future.item1'), done: false },
        { text: t('future.item2'), done: false },
        { text: t('future.item3'), done: false },
        { text: t('future.item4'), done: false },
      ],
    },
  ];

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'completed':
        return <span className="px-3 py-1 text-xs font-semibold rounded-full bg-green-500/20 text-green-400 border border-green-500/30">{t('status.completed')}</span>;
      case 'in-progress':
        return <span className="px-3 py-1 text-xs font-semibold rounded-full bg-pyrax-500/20 text-pyrax-400 border border-pyrax-500/30">{t('status.inProgress')}</span>;
      case 'upcoming':
        return <span className="px-3 py-1 text-xs font-semibold rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30">{t('status.upcoming')}</span>;
      default:
        return <span className="px-3 py-1 text-xs font-semibold rounded-full bg-stone-500/20 text-stone-400 border border-stone-500/30">{t('status.future')}</span>;
    }
  };

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-pyrax-500/5 blur-[128px]" />
        
        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6">
              {t('hero.title')}
            </h1>
            <p className="text-xl text-stone-400 max-w-3xl mx-auto">
              {t('hero.subtitle')}
            </p>
          </motion.div>
        </div>
      </section>

      {/* Timeline */}
      <section className="py-20">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="relative">
            {/* Vertical line */}
            <div className="absolute left-8 top-0 bottom-0 w-0.5 bg-gradient-to-b from-green-500 via-pyrax-500 to-stone-700" />
            
            <div className="space-y-12">
              {phases.map((phase, index) => (
                <motion.div
                  key={phase.id}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: index * 0.1 }}
                  className="relative pl-20"
                >
                  {/* Icon */}
                  <div className={`absolute left-0 w-16 h-16 rounded-2xl bg-gradient-to-br ${phase.color} flex items-center justify-center shadow-lg`}>
                    <phase.icon className="h-8 w-8 text-white" />
                  </div>
                  
                  {/* Content */}
                  <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
                    <div className="flex flex-wrap items-center gap-4 mb-4">
                      <h3 className="text-2xl font-bold text-white">{phase.title}</h3>
                      <span className="text-stone-400">—</span>
                      <span className="text-lg text-stone-300">{phase.subtitle}</span>
                      {getStatusBadge(phase.status)}
                    </div>
                    
                    <div className="grid sm:grid-cols-2 gap-3">
                      {phase.items.map((item, i) => (
                        <div
                          key={i}
                          className={`flex items-start gap-3 p-3 rounded-lg ${
                            item.done ? 'bg-green-500/10' : 'bg-stone-800/50'
                          }`}
                        >
                          {item.done ? (
                            <CheckCircleIcon className="h-5 w-5 text-green-500 flex-shrink-0 mt-0.5" />
                          ) : (
                            <ClockIcon className="h-5 w-5 text-stone-500 flex-shrink-0 mt-0.5" />
                          )}
                          <span className={item.done ? 'text-stone-300' : 'text-stone-500'}>
                            {item.text}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Progress Stats */}
      <section className="py-20 bg-stone-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">{t('progress.title')}</h2>
            <p className="text-stone-400">{t('progress.subtitle')}</p>
          </div>
          
          <div className="grid md:grid-cols-4 gap-6">
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6 text-center">
              <div className="text-4xl font-bold text-green-500 mb-2">85%</div>
              <div className="text-stone-400">{t('progress.coreNode')}</div>
            </div>
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6 text-center">
              <div className="text-4xl font-bold text-pyrax-500 mb-2">70%</div>
              <div className="text-stone-400">{t('progress.evmSidechain')}</div>
            </div>
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6 text-center">
              <div className="text-4xl font-bold text-blue-500 mb-2">60%</div>
              <div className="text-stone-400">{t('progress.aiPlatform')}</div>
            </div>
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6 text-center">
              <div className="text-4xl font-bold text-purple-500 mb-2">90%</div>
              <div className="text-stone-400">{t('progress.desktop')}</div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
