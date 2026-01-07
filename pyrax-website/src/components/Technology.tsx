'use client';

import { useTranslations } from 'next-intl';
import { useLocale } from 'next-intl';
import { motion } from 'framer-motion';
import { ArrowRightIcon } from '@heroicons/react/24/outline';
import Link from 'next/link';

export default function Technology() {
  const t = useTranslations('technology');
  const locale = useLocale();

  const streams = [
    {
      name: t('consensus.streamA.name'),
      algorithm: t('consensus.streamA.algorithm'),
      type: t('consensus.streamA.type'),
      blockTime: t('consensus.streamA.blockTime'),
      reward: t('consensus.streamA.reward'),
      feeShare: t('consensus.streamA.feeShare'),
      color: 'from-pyrax-500 to-orange-500',
      bgColor: 'bg-pyrax-500/10',
      borderColor: 'border-pyrax-500/30',
    },
    {
      name: t('consensus.streamB.name'),
      algorithm: t('consensus.streamB.algorithm'),
      type: t('consensus.streamB.type'),
      blockTime: t('consensus.streamB.blockTime'),
      reward: t('consensus.streamB.reward'),
      feeShare: t('consensus.streamB.feeShare'),
      color: 'from-blue-500 to-cyan-500',
      bgColor: 'bg-blue-500/10',
      borderColor: 'border-blue-500/30',
    },
    {
      name: t('consensus.streamC.name'),
      algorithm: t('consensus.streamC.algorithm'),
      type: t('consensus.streamC.type'),
      blockTime: t('consensus.streamC.blockTime'),
      reward: t('consensus.streamC.reward'),
      feeShare: t('consensus.streamC.feeShare'),
      color: 'from-purple-500 to-pink-500',
      bgColor: 'bg-purple-500/10',
      borderColor: 'border-purple-500/30',
    },
  ];

  return (
    <section id="technology" className="relative py-24 overflow-hidden">
      <div className="absolute inset-0 bg-stone-950" />
      
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

        {/* Architecture Diagram */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="mb-16"
        >
          <div className="rounded-2xl bg-stone-900/50 border border-stone-800 p-8 overflow-hidden">
            <h3 className="text-2xl font-bold text-white mb-8 text-center">{t('architecture.title')}</h3>
            
            <div className="space-y-4">
              {/* Layer 3 */}
              <div className="rounded-xl bg-gradient-to-r from-purple-500/20 to-pink-500/20 border border-purple-500/30 p-4">
                <div className="flex items-center justify-between flex-wrap gap-4">
                  <div>
                    <h4 className="text-lg font-semibold text-white">{t('architecture.layer3')}</h4>
                    <p className="text-sm text-stone-400">{t('architecture.layer3Desc')}</p>
                  </div>
                  <span className="text-purple-400 font-mono text-sm">500K+ TPS</span>
                </div>
              </div>

              {/* Layer 2 */}
              <div className="rounded-xl bg-gradient-to-r from-blue-500/20 to-cyan-500/20 border border-blue-500/30 p-4">
                <div className="flex items-center justify-between flex-wrap gap-4">
                  <div>
                    <h4 className="text-lg font-semibold text-white">{t('architecture.layer2')}</h4>
                    <p className="text-sm text-stone-400">{t('architecture.layer2Desc')}</p>
                  </div>
                  <span className="text-blue-400 font-mono text-sm">1K-5K TPS</span>
                </div>
              </div>

              {/* Layer 1 */}
              <div className="rounded-xl bg-gradient-to-r from-pyrax-500/20 to-orange-500/20 border border-pyrax-500/30 p-4">
                <div className="flex items-center justify-between flex-wrap gap-4">
                  <div>
                    <h4 className="text-lg font-semibold text-white">{t('architecture.layer1')}</h4>
                    <p className="text-sm text-stone-400">{t('architecture.layer1Desc')}</p>
                  </div>
                  <span className="text-pyrax-400 font-mono text-sm">3 Streams</span>
                </div>
              </div>
            </div>
          </div>
        </motion.div>

        {/* TriStream Consensus */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="mb-16"
        >
          <h3 className="text-2xl font-bold text-white mb-4 text-center">{t('consensus.title')}</h3>
          <p className="text-stone-400 text-center max-w-2xl mx-auto mb-8">{t('consensus.description')}</p>
          
          <div className="grid md:grid-cols-3 gap-6">
            {streams.map((stream, index) => (
              <motion.div
                key={stream.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className={`rounded-2xl ${stream.bgColor} border ${stream.borderColor} p-6`}
              >
                <div className={`inline-block px-3 py-1 rounded-full bg-gradient-to-r ${stream.color} text-white text-sm font-semibold mb-4`}>
                  {stream.name}
                </div>
                
                <div className="space-y-3">
                  <div className="flex justify-between text-sm">
                    <span className="text-stone-400">Algorithm</span>
                    <span className="text-white font-mono">{stream.algorithm}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-stone-400">Type</span>
                    <span className="text-white">{stream.type}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-stone-400">Block Time</span>
                    <span className="text-white font-mono">{stream.blockTime}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-stone-400">Reward</span>
                    <span className="text-white font-mono">{stream.reward}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-stone-400">Fee Share</span>
                    <span className="text-white font-mono">{stream.feeShare}</span>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>

        {/* Cryptography */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <div className="rounded-2xl bg-stone-900/50 border border-stone-800 p-8">
            <h3 className="text-2xl font-bold text-white mb-6">{t('cryptography.title')}</h3>
            
            <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
              {[
                { label: t('cryptography.addressDerivation'), value: t('cryptography.addressValue') },
                { label: t('cryptography.blockHash'), value: t('cryptography.blockHashValue') },
                { label: t('cryptography.txHash'), value: t('cryptography.txHashValue') },
                { label: t('cryptography.signatures'), value: t('cryptography.signaturesValue') },
                { label: t('cryptography.zkProofs'), value: t('cryptography.zkProofsValue') },
              ].map((item) => (
                <div key={item.label} className="rounded-xl bg-stone-800/50 p-4">
                  <p className="text-sm text-stone-400 mb-1">{item.label}</p>
                  <p className="text-white font-mono text-sm">{item.value}</p>
                </div>
              ))}
            </div>
          </div>
        </motion.div>

        {/* View Full Technology Button */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mt-12"
        >
          <Link
            href={`/${locale}/technology`}
            className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-8 py-4 text-lg font-semibold text-white hover:from-pyrax-600 hover:to-pyrax-700 transition-all shadow-lg shadow-pyrax-500/25"
          >
            {t('viewFullTechnology')}
            <ArrowRightIcon className="h-5 w-5" />
          </Link>
        </motion.div>
      </div>
    </section>
  );
}
