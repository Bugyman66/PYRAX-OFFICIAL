'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import {
  CubeIcon,
  CircleStackIcon,
  BeakerIcon,
  SparklesIcon,
  CommandLineIcon,
  CpuChipIcon,
  CheckCircleIcon,
} from '@heroicons/react/24/outline';

export default function Features() {
  const t = useTranslations('features');

  const features = [
    {
      icon: CubeIcon,
      title: t('tristream.title'),
      description: t('tristream.description'),
      color: 'from-pyrax-500 to-orange-500',
      highlights: [
        t('tristream.streamA'),
        t('tristream.streamB'),
        t('tristream.streamC'),
      ],
    },
    {
      icon: CircleStackIcon,
      title: t('evm.title'),
      description: t('evm.description'),
      color: 'from-blue-500 to-cyan-500',
      highlights: [
        t('evm.feature1'),
        t('evm.feature2'),
        t('evm.feature3'),
      ],
    },
    {
      icon: BeakerIcon,
      title: t('zkrollup.title'),
      description: t('zkrollup.description'),
      color: 'from-purple-500 to-pink-500',
      highlights: [
        t('zkrollup.feature1'),
        t('zkrollup.feature2'),
        t('zkrollup.feature3'),
      ],
    },
    {
      icon: SparklesIcon,
      title: t('ai.title'),
      description: t('ai.description'),
      color: 'from-green-500 to-emerald-500',
      highlights: [
        t('ai.feature1'),
        t('ai.feature2'),
        t('ai.feature3'),
      ],
    },
    {
      icon: CommandLineIcon,
      title: t('wasm.title'),
      description: t('wasm.description'),
      color: 'from-rose-500 to-red-500',
      highlights: [
        t('wasm.feature1'),
        t('wasm.feature2'),
        t('wasm.feature3'),
      ],
    },
    {
      icon: CpuChipIcon,
      title: t('mining.title'),
      description: t('mining.description'),
      color: 'from-amber-500 to-yellow-500',
      highlights: [
        t('mining.feature1'),
        t('mining.feature2'),
        t('mining.feature3'),
      ],
    },
  ];

  return (
    <section id="features" className="relative py-24 overflow-hidden">
      {/* Background */}
      <div className="absolute inset-0 bg-gradient-to-b from-stone-950 via-stone-900 to-stone-950" />
      
      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center mb-16"
        >
          <h2 className="text-4xl sm:text-5xl font-bold text-white mb-4">
            {t('title')}
          </h2>
          <p className="text-lg text-stone-400 max-w-3xl mx-auto">
            {t('subtitle')}
          </p>
        </motion.div>

        {/* Features Grid */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="group relative"
            >
              {/* Glow effect */}
              <div className={`absolute inset-0 bg-gradient-to-r ${feature.color} rounded-2xl blur-xl opacity-0 group-hover:opacity-20 transition-opacity duration-500`} />
              
              <div className="relative h-full rounded-2xl bg-stone-900/80 border border-stone-800 p-6 hover:border-stone-700 transition-all duration-300">
                {/* Icon */}
                <div className={`inline-flex items-center justify-center w-12 h-12 rounded-xl bg-gradient-to-r ${feature.color} mb-4`}>
                  <feature.icon className="h-6 w-6 text-white" />
                </div>

                {/* Title */}
                <h3 className="text-xl font-semibold text-white mb-3">
                  {feature.title}
                </h3>

                {/* Description */}
                <p className="text-stone-400 text-sm mb-4 leading-relaxed">
                  {feature.description}
                </p>

                {/* Highlights */}
                <ul className="space-y-2">
                  {feature.highlights.map((highlight, i) => (
                    <li key={i} className="flex items-start gap-2 text-sm">
                      <CheckCircleIcon className={`h-5 w-5 flex-shrink-0 bg-gradient-to-r ${feature.color} bg-clip-text text-transparent`} />
                      <span className="text-stone-300">{highlight}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
