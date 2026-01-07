'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import {
  SparklesIcon,
  CpuChipIcon,
  DocumentMagnifyingGlassIcon,
  CloudArrowUpIcon,
  ServerStackIcon,
  ShieldCheckIcon,
  PhotoIcon,
  TableCellsIcon,
  ChatBubbleBottomCenterTextIcon,
  AcademicCapIcon,
  AdjustmentsHorizontalIcon,
  CubeTransparentIcon,
} from '@heroicons/react/24/outline';

export default function AIPlatform() {
  const t = useTranslations('aiPlatform');

  const jobTypes = [
    { icon: ChatBubbleBottomCenterTextIcon, name: t('jobTypes.inference'), desc: t('jobTypes.inferenceDesc') },
    { icon: AcademicCapIcon, name: t('jobTypes.training'), desc: t('jobTypes.trainingDesc') },
    { icon: AdjustmentsHorizontalIcon, name: t('jobTypes.fineTuning'), desc: t('jobTypes.fineTuningDesc') },
    { icon: CubeTransparentIcon, name: t('jobTypes.embedding'), desc: t('jobTypes.embeddingDesc') },
    { icon: PhotoIcon, name: t('jobTypes.imageGen'), desc: t('jobTypes.imageGenDesc') },
    { icon: TableCellsIcon, name: t('jobTypes.batch'), desc: t('jobTypes.batchDesc') },
  ];

  const tiers = [
    {
      name: t('providerTiers.hobbyist'),
      gpu: t('providerTiers.hobbyistGpu'),
      stake: t('providerTiers.hobbyistStake'),
      revenue: t('providerTiers.hobbyistRevenue'),
      color: 'from-green-500 to-emerald-500',
    },
    {
      name: t('providerTiers.professional'),
      gpu: t('providerTiers.professionalGpu'),
      stake: t('providerTiers.professionalStake'),
      revenue: t('providerTiers.professionalRevenue'),
      color: 'from-blue-500 to-cyan-500',
    },
    {
      name: t('providerTiers.enterprise'),
      gpu: t('providerTiers.enterpriseGpu'),
      stake: t('providerTiers.enterpriseStake'),
      revenue: t('providerTiers.enterpriseRevenue'),
      color: 'from-purple-500 to-pink-500',
    },
  ];

  return (
    <section id="ai-platform" className="relative py-24 overflow-hidden">
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

        {/* Foundry & Crucible */}
        <div className="grid lg:grid-cols-2 gap-8 mb-16">
          {/* Foundry */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="rounded-2xl bg-gradient-to-br from-green-500/10 to-emerald-500/10 border border-green-500/20 p-8"
          >
            <div className="flex items-center gap-4 mb-6">
              <div className="w-14 h-14 rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 flex items-center justify-center">
                <SparklesIcon className="h-7 w-7 text-white" />
              </div>
              <div>
                <h3 className="text-2xl font-bold text-white">{t('foundry.title')}</h3>
                <p className="text-green-400">{t('foundry.subtitle')}</p>
              </div>
            </div>
            
            <p className="text-stone-400 mb-6">{t('foundry.description')}</p>
            
            <div className="space-y-4">
              {[
                { title: t('foundry.features.registration'), desc: t('foundry.features.registrationDesc'), icon: CloudArrowUpIcon },
                { title: t('foundry.features.versioning'), desc: t('foundry.features.versioningDesc'), icon: DocumentMagnifyingGlassIcon },
                { title: t('foundry.features.discovery'), desc: t('foundry.features.discoveryDesc'), icon: DocumentMagnifyingGlassIcon },
              ].map((feature) => (
                <div key={feature.title} className="flex items-start gap-3">
                  <feature.icon className="h-5 w-5 text-green-400 mt-0.5 flex-shrink-0" />
                  <div>
                    <p className="text-white font-medium">{feature.title}</p>
                    <p className="text-sm text-stone-400">{feature.desc}</p>
                  </div>
                </div>
              ))}
            </div>
          </motion.div>

          {/* Crucible */}
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="rounded-2xl bg-gradient-to-br from-pyrax-500/10 to-orange-500/10 border border-pyrax-500/20 p-8"
          >
            <div className="flex items-center gap-4 mb-6">
              <div className="w-14 h-14 rounded-xl bg-gradient-to-r from-pyrax-500 to-orange-500 flex items-center justify-center">
                <CpuChipIcon className="h-7 w-7 text-white" />
              </div>
              <div>
                <h3 className="text-2xl font-bold text-white">{t('crucible.title')}</h3>
                <p className="text-pyrax-400">{t('crucible.subtitle')}</p>
              </div>
            </div>
            
            <p className="text-stone-400 mb-6">{t('crucible.description')}</p>
            
            <div className="space-y-4">
              {[
                { title: t('crucible.features.submission'), desc: t('crucible.features.submissionDesc'), icon: CloudArrowUpIcon },
                { title: t('crucible.features.execution'), desc: t('crucible.features.executionDesc'), icon: ServerStackIcon },
                { title: t('crucible.features.verification'), desc: t('crucible.features.verificationDesc'), icon: ShieldCheckIcon },
              ].map((feature) => (
                <div key={feature.title} className="flex items-start gap-3">
                  <feature.icon className="h-5 w-5 text-pyrax-400 mt-0.5 flex-shrink-0" />
                  <div>
                    <p className="text-white font-medium">{feature.title}</p>
                    <p className="text-sm text-stone-400">{feature.desc}</p>
                  </div>
                </div>
              ))}
            </div>
          </motion.div>
        </div>

        {/* Job Types */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="mb-16"
        >
          <h3 className="text-2xl font-bold text-white mb-8 text-center">{t('jobTypes.title')}</h3>
          
          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {jobTypes.map((job, index) => (
              <motion.div
                key={job.name}
                initial={{ opacity: 0, scale: 0.95 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.05 }}
                className="rounded-xl bg-stone-900/50 border border-stone-800 p-5 hover:border-pyrax-500/50 transition-colors"
              >
                <job.icon className="h-8 w-8 text-pyrax-500 mb-3" />
                <h4 className="text-white font-semibold mb-1">{job.name}</h4>
                <p className="text-sm text-stone-400">{job.desc}</p>
              </motion.div>
            ))}
          </div>
        </motion.div>

        {/* Provider Tiers */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <h3 className="text-2xl font-bold text-white mb-8 text-center">{t('providerTiers.title')}</h3>
          
          <div className="grid md:grid-cols-3 gap-6">
            {tiers.map((tier, index) => (
              <motion.div
                key={tier.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6 hover:border-stone-700 transition-colors"
              >
                <div className={`inline-block px-4 py-1.5 rounded-full bg-gradient-to-r ${tier.color} text-white font-semibold mb-4`}>
                  {tier.name}
                </div>
                
                <div className="space-y-3">
                  <div>
                    <p className="text-sm text-stone-400">GPU Requirement</p>
                    <p className="text-white font-mono">{tier.gpu}</p>
                  </div>
                  <div>
                    <p className="text-sm text-stone-400">Stake Required</p>
                    <p className="text-white font-mono">{tier.stake}</p>
                  </div>
                  <div>
                    <p className="text-sm text-stone-400">Expected Revenue</p>
                    <p className="text-white font-mono text-lg">{tier.revenue}</p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  );
}
