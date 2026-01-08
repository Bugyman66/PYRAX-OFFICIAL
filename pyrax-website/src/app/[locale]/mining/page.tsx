'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Image from 'next/image';

export default function MiningPage() {
  const t = useTranslations('miningPage');

  const streams = [
    {
      id: 'stream-a',
      name: t('streamA.name'),
      algorithm: 'BLAKE3',
      type: t('streamA.type'),
      hardware: t('streamA.hardware'),
      color: 'from-pyrax-500 to-orange-500',
      bgColor: 'bg-pyrax-500/10',
      borderColor: 'border-pyrax-500/30',
      reward: '50%',
      requirements: [
        { label: t('streamA.req1Label'), value: t('streamA.req1Value') },
        { label: t('streamA.req2Label'), value: t('streamA.req2Value') },
        { label: t('streamA.req3Label'), value: t('streamA.req3Value') },
        { label: t('streamA.req4Label'), value: t('streamA.req4Value') },
      ],
      steps: [
        t('streamA.step1'),
        t('streamA.step2'),
        t('streamA.step3'),
        t('streamA.step4'),
      ],
    },
    {
      id: 'stream-b',
      name: t('streamB.name'),
      algorithm: 'KAWPOW',
      type: t('streamB.type'),
      hardware: t('streamB.hardware'),
      color: 'from-blue-500 to-cyan-500',
      bgColor: 'bg-blue-500/10',
      borderColor: 'border-blue-500/30',
      reward: '30%',
      requirements: [
        { label: t('streamB.req1Label'), value: t('streamB.req1Value') },
        { label: t('streamB.req2Label'), value: t('streamB.req2Value') },
        { label: t('streamB.req3Label'), value: t('streamB.req3Value') },
        { label: t('streamB.req4Label'), value: t('streamB.req4Value') },
      ],
      steps: [
        t('streamB.step1'),
        t('streamB.step2'),
        t('streamB.step3'),
        t('streamB.step4'),
      ],
    },
    {
      id: 'stream-c',
      name: t('streamC.name'),
      algorithm: 'ZK-STARK',
      type: t('streamC.type'),
      hardware: t('streamC.hardware'),
      color: 'from-purple-500 to-pink-500',
      bgColor: 'bg-purple-500/10',
      borderColor: 'border-purple-500/30',
      reward: '20%',
      requirements: [
        { label: t('streamC.req1Label'), value: t('streamC.req1Value') },
        { label: t('streamC.req2Label'), value: t('streamC.req2Value') },
        { label: t('streamC.req3Label'), value: t('streamC.req3Value') },
        { label: t('streamC.req4Label'), value: t('streamC.req4Value') },
      ],
      steps: [
        t('streamC.step1'),
        t('streamC.step2'),
        t('streamC.step3'),
        t('streamC.step4'),
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

      {/* Overview */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('overview.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('overview.description')}</p>
          </motion.div>

          <div className="grid md:grid-cols-3 gap-6 mb-16">
            {streams.map((stream, index) => (
              <motion.a
                key={stream.id}
                href={`#${stream.id}`}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className={`rounded-2xl ${stream.bgColor} border ${stream.borderColor} p-6 hover:scale-105 transition-transform`}
              >
                <div className={`inline-block px-4 py-1.5 rounded-full bg-gradient-to-r ${stream.color} text-white font-semibold mb-4`}>
                  {stream.name}
                </div>
                <p className="text-white font-mono text-lg mb-2">{stream.algorithm}</p>
                <p className="text-stone-400 mb-4">{stream.type}</p>
                <div className="flex justify-between items-center pt-4 border-t border-stone-700/50">
                  <span className="text-stone-400">{t('labels.reward')}</span>
                  <span className="text-white font-bold">{stream.reward}</span>
                </div>
              </motion.a>
            ))}
          </div>
        </div>
      </section>

      {/* Stream Details */}
      {streams.map((stream, streamIndex) => (
        <section
          key={stream.id}
          id={stream.id}
          className={`py-20 ${streamIndex % 2 === 0 ? 'bg-stone-900/30' : ''}`}
        >
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              className="mb-12"
            >
              <div className={`inline-block px-4 py-1.5 rounded-full bg-gradient-to-r ${stream.color} text-white font-semibold mb-4`}>
                {stream.name}
              </div>
              <h2 className="text-4xl font-bold text-white mb-4">{stream.algorithm} {t('labels.mining')}</h2>
              <p className="text-lg text-stone-400">{stream.hardware}</p>
            </motion.div>

            <div className="grid lg:grid-cols-2 gap-12">
              {/* Requirements */}
              <motion.div
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
              >
                <h3 className="text-2xl font-bold text-white mb-6">{t('labels.requirements')}</h3>
                <div className="space-y-4">
                  {stream.requirements.map((req) => (
                    <div key={req.label} className="flex justify-between items-center py-3 border-b border-stone-800 last:border-0">
                      <span className="text-stone-400">{req.label}</span>
                      <span className="text-white font-mono">{req.value}</span>
                    </div>
                  ))}
                </div>
              </motion.div>

              {/* How to Start */}
              <motion.div
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
              >
                <h3 className="text-2xl font-bold text-white mb-6">{t('labels.howToStart')}</h3>
                <div className="space-y-4">
                  {stream.steps.map((step, i) => (
                    <div key={i} className="flex items-start gap-4">
                      <div className={`w-8 h-8 rounded-full bg-gradient-to-r ${stream.color} flex items-center justify-center flex-shrink-0 text-white font-bold`}>
                        {i + 1}
                      </div>
                      <p className="text-stone-300 pt-1">{step}</p>
                    </div>
                  ))}
                </div>
              </motion.div>
            </div>
          </div>
        </section>
      ))}

      {/* Staking Section */}
      <section className="py-20 bg-gradient-to-b from-stone-900/30 to-stone-950">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('staking.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('staking.description')}</p>
          </motion.div>

          <div className="grid lg:grid-cols-2 gap-8">
            {/* Validator Staking */}
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="rounded-2xl bg-purple-500/10 border border-purple-500/30 p-6"
            >
              <h3 className="text-2xl font-bold text-white mb-4">{t('staking.validatorTitle')}</h3>
              <p className="text-stone-400 mb-6">{t('staking.validatorDesc')}</p>
              
              <div className="space-y-3">
                {[
                  { label: t('staking.minStake'), value: '100,000 PYRAX' },
                  { label: t('staking.apy'), value: '8-15% APY' },
                  { label: t('staking.unbonding'), value: '21 days' },
                  { label: t('staking.slashing'), value: '0.1% - 5%' },
                ].map((item) => (
                  <div key={item.label} className="flex justify-between py-2 border-b border-purple-500/20 last:border-0">
                    <span className="text-stone-400">{item.label}</span>
                    <span className="text-white font-mono">{item.value}</span>
                  </div>
                ))}
              </div>
            </motion.div>

            {/* Delegation */}
            <motion.div
              initial={{ opacity: 0, x: 20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="rounded-2xl bg-blue-500/10 border border-blue-500/30 p-6"
            >
              <h3 className="text-2xl font-bold text-white mb-4">{t('staking.delegationTitle')}</h3>
              <p className="text-stone-400 mb-6">{t('staking.delegationDesc')}</p>
              
              <div className="space-y-3">
                {[
                  { label: t('staking.minDelegation'), value: '100 PYRAX' },
                  { label: t('staking.delegatorApy'), value: '5-10% APY' },
                  { label: t('staking.commission'), value: '5-20%' },
                  { label: t('staking.rewards'), value: t('staking.rewardsValue') },
                ].map((item) => (
                  <div key={item.label} className="flex justify-between py-2 border-b border-blue-500/20 last:border-0">
                    <span className="text-stone-400">{item.label}</span>
                    <span className="text-white font-mono">{item.value}</span>
                  </div>
                ))}
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Rewards Calculator CTA */}
      <section className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('cta.title')}</h2>
            <p className="text-lg text-stone-400 mb-8">{t('cta.description')}</p>
            
            <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
              <a
                href="https://github.com/pyrax-official/pyrax-miner"
                className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-8 py-4 text-lg font-semibold text-white shadow-lg shadow-pyrax-500/25 hover:shadow-pyrax-500/40 transition-all hover:scale-105"
              >
                {t('cta.downloadMiner')}
              </a>
              <a
                href="https://docs.testnet.pyrax.org/mining"
                className="inline-flex items-center gap-2 rounded-xl border border-stone-700 bg-stone-900/50 px-8 py-4 text-lg font-semibold text-white hover:bg-stone-800 transition-all"
              >
                {t('cta.readDocs')}
              </a>
            </div>
          </motion.div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
