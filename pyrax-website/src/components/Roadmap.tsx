'use client';

import { useTranslations } from 'next-intl';
import { useLocale } from 'next-intl';
import { motion } from 'framer-motion';
import { CheckCircleIcon, ArrowPathIcon, ClockIcon, ArrowRightIcon } from '@heroicons/react/24/outline';
import Link from 'next/link';

export default function Roadmap() {
  const t = useTranslations('roadmap');
  const locale = useLocale();

  const quarters = [
    {
      title: 'Q4 2025',
      subtitle: t('q4_2025.subtitle'),
      status: 'completed',
      items: [
        t('q4_2025.item1'),
        t('q4_2025.item2'),
        t('q4_2025.item3'),
        t('q4_2025.item4'),
        t('q4_2025.item5'),
        t('q4_2025.item6'),
      ],
    },
    {
      title: 'Q1 - Q2 2026',
      subtitle: t('q1_q2_2026.subtitle'),
      status: 'in-progress',
      items: [
        t('q1_q2_2026.item1'),
        t('q1_q2_2026.item2'),
        t('q1_q2_2026.item3'),
        t('q1_q2_2026.item4'),
        t('q1_q2_2026.item5'),
        t('q1_q2_2026.item6'),
      ],
    },
    {
      title: 'Q3 2026',
      subtitle: t('q3_2026.subtitle'),
      status: 'upcoming',
      items: [
        t('q3_2026.item1'),
        t('q3_2026.item2'),
        t('q3_2026.item3'),
        t('q3_2026.item4'),
        t('q3_2026.item5'),
        t('q3_2026.item6'),
      ],
    },
    {
      title: 'Q4 2026',
      subtitle: t('q4_2026.subtitle'),
      status: 'upcoming',
      items: [
        t('q4_2026.item1'),
        t('q4_2026.item2'),
        t('q4_2026.item3'),
        t('q4_2026.item4'),
        t('q4_2026.item5'),
        t('q4_2026.item6'),
      ],
    },
    {
      title: '2027+',
      subtitle: t('future.subtitle'),
      status: 'future',
      items: [
        t('future.item1'),
        t('future.item2'),
        t('future.item3'),
        t('future.item4'),
      ],
    },
  ];

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircleIcon className="h-6 w-6 text-green-500" />;
      case 'in-progress':
        return <ArrowPathIcon className="h-6 w-6 text-pyrax-500 animate-spin" />;
      default:
        return <ClockIcon className="h-6 w-6 text-stone-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'border-green-500/30 bg-green-500/5';
      case 'in-progress':
        return 'border-pyrax-500/30 bg-pyrax-500/5';
      default:
        return 'border-stone-700 bg-stone-900/50';
    }
  };

  return (
    <section id="roadmap" className="relative py-24 overflow-hidden">
      <div className="absolute inset-0 bg-gradient-to-b from-stone-950 via-stone-900 to-stone-950" />
      
      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <h2 className="text-4xl sm:text-5xl font-bold text-white mb-4">
            {t('title')}
          </h2>
          <p className="text-lg text-stone-400 max-w-3xl mx-auto">
            {t('subtitle')}
          </p>
        </motion.div>

        {/* Timeline */}
        <div className="relative">
          {/* Vertical line */}
          <div className="absolute left-1/2 top-0 bottom-0 w-px bg-gradient-to-b from-green-500 via-pyrax-500 to-stone-700 hidden lg:block" />

          <div className="space-y-12">
            {quarters.map((quarter, index) => (
              <motion.div
                key={quarter.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className={`relative flex flex-col lg:flex-row items-center gap-8 ${
                  index % 2 === 0 ? 'lg:flex-row' : 'lg:flex-row-reverse'
                }`}
              >
                {/* Card */}
                <div className={`flex-1 w-full lg:w-auto ${index % 2 === 0 ? 'lg:text-right' : 'lg:text-left'}`}>
                  <div className={`rounded-2xl border p-6 ${getStatusColor(quarter.status)}`}>
                    <div className={`flex items-center gap-3 mb-4 ${index % 2 === 0 ? 'lg:justify-end' : ''}`}>
                      {getStatusIcon(quarter.status)}
                      <div>
                        <h3 className="text-xl font-bold text-white">{quarter.title}</h3>
                        <p className="text-sm text-stone-400">{quarter.subtitle}</p>
                      </div>
                    </div>
                    
                    <ul className={`space-y-2 ${index % 2 === 0 ? 'lg:text-right' : ''}`}>
                      {quarter.items.map((item, i) => (
                        <li
                          key={i}
                          className={`text-sm text-stone-300 flex items-center gap-2 ${
                            index % 2 === 0 ? 'lg:flex-row-reverse' : ''
                          }`}
                        >
                          <span className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${
                            quarter.status === 'completed' ? 'bg-green-500' :
                            quarter.status === 'in-progress' ? 'bg-pyrax-500' : 'bg-stone-600'
                          }`} />
                          {item}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>

                {/* Center dot */}
                <div className="hidden lg:flex items-center justify-center w-12 h-12 rounded-full bg-stone-900 border-4 border-stone-800 z-10">
                  <div className={`w-4 h-4 rounded-full ${
                    quarter.status === 'completed' ? 'bg-green-500' :
                    quarter.status === 'in-progress' ? 'bg-pyrax-500 animate-pulse' : 'bg-stone-600'
                  }`} />
                </div>

                {/* Spacer */}
                <div className="flex-1 hidden lg:block" />
              </motion.div>
            ))}
          </div>

          {/* View Full Roadmap Button */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mt-16"
          >
            <Link
              href={`/${locale}/roadmap`}
              className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-8 py-4 text-lg font-semibold text-white hover:from-pyrax-600 hover:to-pyrax-700 transition-all shadow-lg shadow-pyrax-500/25"
            >
              {t('viewFullRoadmap')}
              <ArrowRightIcon className="h-5 w-5" />
            </Link>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
