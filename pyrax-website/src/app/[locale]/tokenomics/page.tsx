'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';

export default function TokenomicsPage() {
  const t = useTranslations('tokenomicsPage');

  const distribution = [
    { name: t('distribution.presale'), percent: '7%', amount: '7B', desc: t('distribution.presaleDesc'), color: '#FF6B35' },
    { name: t('distribution.bdagCommunity'), percent: '5%', amount: '5B', desc: t('distribution.bdagDesc'), color: '#3B82F6' },
    { name: t('distribution.mining'), percent: '40%', amount: '40B', desc: t('distribution.miningDesc'), color: '#10B981' },
    { name: t('distribution.zkProver'), percent: '10%', amount: '10B', desc: t('distribution.zkDesc'), color: '#8B5CF6' },
    { name: t('distribution.team'), percent: '10%', amount: '10B', desc: t('distribution.teamDesc'), color: '#EC4899' },
    { name: t('distribution.advisors'), percent: '3%', amount: '3B', desc: t('distribution.advisorsDesc'), color: '#F59E0B' },
    { name: t('distribution.ecosystem'), percent: '10%', amount: '10B', desc: t('distribution.ecosystemDesc'), color: '#06B6D4' },
    { name: t('distribution.marketing'), percent: '5%', amount: '5B', desc: t('distribution.marketingDesc'), color: '#EF4444' },
    { name: t('distribution.liquidity'), percent: '5%', amount: '5B', desc: t('distribution.liquidityDesc'), color: '#84CC16' },
    { name: t('distribution.treasury'), percent: '3%', amount: '3B', desc: t('distribution.treasuryDesc'), color: '#A855F7' },
    { name: t('distribution.reserve'), percent: '2%', amount: '2B', desc: t('distribution.reserveDesc'), color: '#6B7280' },
  ];

  const feeDistribution = [
    { name: t('fees.streamA'), percent: '40%', color: '#FF6B35' },
    { name: t('fees.streamB'), percent: '35%', color: '#3B82F6' },
    { name: t('fees.streamC'), percent: '15%', color: '#8B5CF6' },
    { name: t('fees.treasury'), percent: '10%', color: '#10B981' },
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

      {/* Token Overview */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-16">
            {[
              { label: t('overview.name'), value: 'PYRAX' },
              { label: t('overview.symbol'), value: 'PYRAX' },
              { label: t('overview.supply'), value: '100B' },
              { label: t('overview.decimals'), value: '18' },
            ].map((item, index) => (
              <motion.div
                key={item.label}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6 text-center"
              >
                <p className="text-sm text-stone-400 mb-2">{item.label}</p>
                <p className="text-3xl font-bold bg-gradient-to-r from-pyrax-400 to-pyrax-600 bg-clip-text text-transparent">
                  {item.value}
                </p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Distribution */}
      <section className="py-20 bg-stone-900/30">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('distribution.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('distribution.subtitle')}</p>
          </motion.div>

          {/* Visual Bar */}
          <div className="h-12 rounded-2xl overflow-hidden flex mb-12">
            {distribution.map((item) => (
              <div
                key={item.name}
                className="h-full transition-all hover:opacity-80 relative group"
                style={{
                  backgroundColor: item.color,
                  width: item.percent,
                }}
              >
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1 bg-stone-800 rounded-lg text-xs text-white whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity">
                  {item.name}: {item.percent}
                </div>
              </div>
            ))}
          </div>

          {/* Cards */}
          <div className="grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {distribution.map((item, index) => (
              <motion.div
                key={item.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.05 }}
                className="rounded-xl bg-stone-900/50 border border-stone-800 p-4"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div
                    className="w-4 h-4 rounded-full flex-shrink-0"
                    style={{ backgroundColor: item.color }}
                  />
                  <span className="text-white font-semibold">{item.name}</span>
                </div>
                <div className="flex items-baseline gap-2 mb-2">
                  <span className="text-2xl font-bold text-white">{item.percent}</span>
                  <span className="text-stone-500">({item.amount} PYRAX)</span>
                </div>
                <p className="text-sm text-stone-400">{item.desc}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Fee Distribution */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12">
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
            >
              <h2 className="text-4xl font-bold text-white mb-4">{t('fees.title')}</h2>
              <p className="text-lg text-stone-400 mb-8">{t('fees.description')}</p>

              <div className="space-y-4">
                {feeDistribution.map((item) => (
                  <div key={item.name} className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-stone-300">{item.name}</span>
                      <span className="text-white font-mono">{item.percent}</span>
                    </div>
                    <div className="h-3 rounded-full bg-stone-800 overflow-hidden">
                      <div
                        className="h-full rounded-full transition-all"
                        style={{
                          backgroundColor: item.color,
                          width: item.percent,
                        }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </motion.div>

            <motion.div
              initial={{ opacity: 0, x: 20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
            >
              <h3 className="text-2xl font-bold text-white mb-6">{t('staking.title')}</h3>
              <div className="space-y-4">
                {[
                  { label: t('staking.validatorStake'), value: '100,000 PYRAX' },
                  { label: t('staking.minDelegation'), value: '100 PYRAX' },
                  { label: t('staking.providerStake'), value: '10,000 PYRAX' },
                  { label: t('staking.unbonding'), value: '21 days' },
                  { label: t('staking.doubleSign'), value: '5% slash' },
                  { label: t('staking.downtime'), value: '0.1% slash' },
                ].map((item) => (
                  <div key={item.label} className="flex justify-between items-center py-3 border-b border-stone-800 last:border-0">
                    <span className="text-stone-400">{item.label}</span>
                    <span className="text-white font-mono">{item.value}</span>
                  </div>
                ))}
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Emission Schedule */}
      <section className="py-20 bg-stone-900/30">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl font-bold text-white mb-4">{t('emission.title')}</h2>
            <p className="text-lg text-stone-400 max-w-3xl mx-auto">{t('emission.description')}</p>
          </motion.div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              { year: t('emission.year1'), rate: '25%', total: '10B' },
              { year: t('emission.year2'), rate: '20%', total: '18B' },
              { year: t('emission.year3'), rate: '15%', total: '24B' },
              { year: t('emission.year4'), rate: '10%', total: '28B' },
            ].map((item, index) => (
              <motion.div
                key={item.year}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6 text-center"
              >
                <h3 className="text-lg font-semibold text-white mb-2">{item.year}</h3>
                <p className="text-3xl font-bold text-pyrax-400 mb-1">{item.rate}</p>
                <p className="text-sm text-stone-500">{t('emission.miningRate')}</p>
                <div className="mt-4 pt-4 border-t border-stone-800">
                  <p className="text-sm text-stone-400">{t('emission.cumulative')}</p>
                  <p className="text-xl font-bold text-white">{item.total}</p>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
