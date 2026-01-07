'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Image from 'next/image';

export default function TechnologyPage() {
  const t = useTranslations('technologyPage');

  const streams = [
    {
      name: t('streams.streamA.name'),
      algorithm: 'BLAKE3',
      type: t('streams.streamA.type'),
      description: t('streams.streamA.description'),
      blockTime: '6s',
      reward: '50%',
      feeShare: '40%',
      color: 'from-pyrax-500 to-orange-500',
      features: [
        t('streams.streamA.feature1'),
        t('streams.streamA.feature2'),
        t('streams.streamA.feature3'),
        t('streams.streamA.feature4'),
      ],
    },
    {
      name: t('streams.streamB.name'),
      algorithm: 'KAWPOW',
      type: t('streams.streamB.type'),
      description: t('streams.streamB.description'),
      blockTime: '6s',
      reward: '30%',
      feeShare: '35%',
      color: 'from-blue-500 to-cyan-500',
      features: [
        t('streams.streamB.feature1'),
        t('streams.streamB.feature2'),
        t('streams.streamB.feature3'),
        t('streams.streamB.feature4'),
      ],
    },
    {
      name: t('streams.streamC.name'),
      algorithm: 'ZK-STARK',
      type: t('streams.streamC.type'),
      description: t('streams.streamC.description'),
      blockTime: '6s',
      reward: '20%',
      feeShare: '25%',
      color: 'from-purple-500 to-pink-500',
      features: [
        t('streams.streamC.feature1'),
        t('streams.streamC.feature2'),
        t('streams.streamC.feature3'),
        t('streams.streamC.feature4'),
      ],
    },
  ];

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/10 via-transparent to-transparent" />
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

      {/* TriStream DAG */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('tristream.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('tristream.description')}</p>
          </motion.div>

          <div className="grid lg:grid-cols-3 gap-8">
            {streams.map((stream, index) => (
              <motion.div
                key={stream.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 overflow-hidden"
              >
                <div className={`h-2 bg-gradient-to-r ${stream.color}`} />
                <div className="p-6">
                  <div className={`inline-block px-4 py-1.5 rounded-full bg-gradient-to-r ${stream.color} text-white font-semibold mb-4`}>
                    {stream.name}
                  </div>
                  <p className="text-stone-400 mb-6">{stream.description}</p>
                  
                  <div className="grid grid-cols-2 gap-4 mb-6">
                    <div className="bg-stone-800/50 rounded-lg p-3">
                      <p className="text-xs text-stone-500">{t('labels.algorithm')}</p>
                      <p className="text-white font-mono">{stream.algorithm}</p>
                    </div>
                    <div className="bg-stone-800/50 rounded-lg p-3">
                      <p className="text-xs text-stone-500">{t('labels.type')}</p>
                      <p className="text-white">{stream.type}</p>
                    </div>
                    <div className="bg-stone-800/50 rounded-lg p-3">
                      <p className="text-xs text-stone-500">{t('labels.blockTime')}</p>
                      <p className="text-white font-mono">{stream.blockTime}</p>
                    </div>
                    <div className="bg-stone-800/50 rounded-lg p-3">
                      <p className="text-xs text-stone-500">{t('labels.reward')}</p>
                      <p className="text-white font-mono">{stream.reward}</p>
                    </div>
                  </div>

                  <h4 className="text-white font-semibold mb-3">{t('labels.features')}</h4>
                  <ul className="space-y-2">
                    {stream.features.map((feature, i) => (
                      <li key={i} className="flex items-start gap-2 text-sm text-stone-300">
                        <span className={`w-1.5 h-1.5 rounded-full mt-1.5 bg-gradient-to-r ${stream.color}`} />
                        {feature}
                      </li>
                    ))}
                  </ul>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Architecture */}
      <section className="py-20 bg-stone-900/30">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('architecture.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('architecture.description')}</p>
          </motion.div>

          <div className="space-y-6">
            {[
              { layer: t('architecture.layer3'), desc: t('architecture.layer3Desc'), tps: '500K+ TPS', color: 'purple' },
              { layer: t('architecture.layer2'), desc: t('architecture.layer2Desc'), tps: '1K-5K TPS', color: 'blue' },
              { layer: t('architecture.layer1'), desc: t('architecture.layer1Desc'), tps: '3 Streams', color: 'pyrax' },
            ].map((item, index) => (
              <motion.div
                key={item.layer}
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className={`rounded-2xl border p-6 ${
                  item.color === 'purple' ? 'bg-purple-500/10 border-purple-500/30' :
                  item.color === 'blue' ? 'bg-blue-500/10 border-blue-500/30' :
                  'bg-pyrax-500/10 border-pyrax-500/30'
                }`}
              >
                <div className="flex items-center justify-between flex-wrap gap-4">
                  <div>
                    <h3 className="text-xl font-bold text-white">{item.layer}</h3>
                    <p className="text-stone-400">{item.desc}</p>
                  </div>
                  <span className={`font-mono text-lg ${
                    item.color === 'purple' ? 'text-purple-400' :
                    item.color === 'blue' ? 'text-blue-400' :
                    'text-pyrax-400'
                  }`}>{item.tps}</span>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* EVM Sidechain */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
            >
              <h2 className="text-4xl font-bold text-white mb-4">{t('evm.title')}</h2>
              <p className="text-lg text-stone-400 mb-6">{t('evm.description')}</p>
              
              <ul className="space-y-4">
                {[
                  t('evm.feature1'),
                  t('evm.feature2'),
                  t('evm.feature3'),
                  t('evm.feature4'),
                  t('evm.feature5'),
                ].map((feature, i) => (
                  <li key={i} className="flex items-start gap-3">
                    <span className="w-2 h-2 rounded-full bg-blue-500 mt-2" />
                    <span className="text-stone-300">{feature}</span>
                  </li>
                ))}
              </ul>
            </motion.div>

            <motion.div
              initial={{ opacity: 0, x: 20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
            >
              <h3 className="text-xl font-bold text-white mb-4">{t('evm.specs')}</h3>
              <div className="space-y-3">
                {[
                  { label: t('evm.specBlockTime'), value: '2 seconds' },
                  { label: t('evm.specTps'), value: '1,000-5,000 TPS' },
                  { label: t('evm.specSolidity'), value: '0.8.x' },
                  { label: t('evm.specChainId'), value: 'TBA' },
                  { label: t('evm.specGasUnit'), value: 'Cinder' },
                ].map((spec) => (
                  <div key={spec.label} className="flex justify-between py-2 border-b border-stone-800 last:border-0">
                    <span className="text-stone-400">{spec.label}</span>
                    <span className="text-white font-mono">{spec.value}</span>
                  </div>
                ))}
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* ZK Rollups */}
      <section className="py-20 bg-stone-900/30">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('zkrollups.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('zkrollups.description')}</p>
          </motion.div>

          <div className="grid md:grid-cols-3 gap-6">
            {[
              { title: t('zkrollups.feature1Title'), desc: t('zkrollups.feature1Desc'), icon: '⚡' },
              { title: t('zkrollups.feature2Title'), desc: t('zkrollups.feature2Desc'), icon: '🔐' },
              { title: t('zkrollups.feature3Title'), desc: t('zkrollups.feature3Desc'), icon: '🌐' },
            ].map((item, index) => (
              <motion.div
                key={item.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="rounded-2xl bg-purple-500/10 border border-purple-500/30 p-6 text-center"
              >
                <span className="text-4xl mb-4 block">{item.icon}</span>
                <h3 className="text-xl font-bold text-white mb-2">{item.title}</h3>
                <p className="text-stone-400">{item.desc}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Cryptography */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('crypto.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('crypto.description')}</p>
          </motion.div>

          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {[
              { label: t('crypto.addressLabel'), value: 'Keccak-256 → Last 20 bytes' },
              { label: t('crypto.blockHashLabel'), value: 'BLAKE3' },
              { label: t('crypto.txHashLabel'), value: 'Keccak-256' },
              { label: t('crypto.signaturesLabel'), value: 'ECDSA secp256k1' },
              { label: t('crypto.zkProofsLabel'), value: 'ZK-STARK' },
              { label: t('crypto.merkleLabel'), value: 'BLAKE3 Merkle Tree' },
            ].map((item) => (
              <motion.div
                key={item.label}
                initial={{ opacity: 0, scale: 0.95 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                className="rounded-xl bg-stone-900/50 border border-stone-800 p-4"
              >
                <p className="text-sm text-stone-400 mb-1">{item.label}</p>
                <p className="text-white font-mono text-sm">{item.value}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
