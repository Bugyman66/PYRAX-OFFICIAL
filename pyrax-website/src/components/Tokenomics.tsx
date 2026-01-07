'use client';

import { useTranslations } from 'next-intl';
import { useLocale } from 'next-intl';
import { motion } from 'framer-motion';
import { ArrowRightIcon } from '@heroicons/react/24/outline';
import Link from 'next/link';

export default function Tokenomics() {
  const t = useTranslations('tokenomics');
  const locale = useLocale();

  const distribution = [
    { name: t('distribution.presale'), percent: t('distribution.presalePercent'), desc: t('distribution.presaleDesc'), color: '#FF6B35' },
    { name: t('distribution.bdagCommunity'), percent: t('distribution.bdagPercent'), desc: t('distribution.bdagDesc'), color: '#3B82F6' },
    { name: t('distribution.mining'), percent: t('distribution.miningPercent'), desc: t('distribution.miningDesc'), color: '#10B981' },
    { name: t('distribution.zkProver'), percent: t('distribution.zkPercent'), desc: t('distribution.zkDesc'), color: '#8B5CF6' },
    { name: t('distribution.team'), percent: t('distribution.teamPercent'), desc: t('distribution.teamDesc'), color: '#EC4899' },
    { name: t('distribution.advisors'), percent: t('distribution.advisorsPercent'), desc: t('distribution.advisorsDesc'), color: '#F59E0B' },
    { name: t('distribution.ecosystem'), percent: t('distribution.ecosystemPercent'), desc: t('distribution.ecosystemDesc'), color: '#06B6D4' },
    { name: t('distribution.marketing'), percent: t('distribution.marketingPercent'), desc: t('distribution.marketingDesc'), color: '#EF4444' },
    { name: t('distribution.liquidity'), percent: t('distribution.liquidityPercent'), desc: t('distribution.liquidityDesc'), color: '#84CC16' },
    { name: t('distribution.treasury'), percent: t('distribution.treasuryPercent'), desc: t('distribution.treasuryDesc'), color: '#A855F7' },
    { name: t('distribution.reserve'), percent: t('distribution.reservePercent'), desc: t('distribution.reserveDesc'), color: '#6B7280' },
  ];

  const feeDistribution = [
    { name: t('feeDistribution.streamA'), percent: t('feeDistribution.streamAPercent'), color: '#FF6B35' },
    { name: t('feeDistribution.streamB'), percent: t('feeDistribution.streamBPercent'), color: '#3B82F6' },
    { name: t('feeDistribution.streamC'), percent: t('feeDistribution.streamCPercent'), color: '#8B5CF6' },
    { name: t('feeDistribution.treasury'), percent: t('feeDistribution.treasuryPercent'), color: '#10B981' },
  ];

  return (
    <section id="tokenomics" className="relative py-24 overflow-hidden">
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

        {/* Token Overview */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-16"
        >
          {[
            { label: t('overview.name'), value: t('overview.nameValue') },
            { label: t('overview.symbol'), value: t('overview.symbolValue') },
            { label: t('overview.supply'), value: t('overview.supplyValue') },
            { label: t('overview.decimals'), value: t('overview.decimalsValue') },
          ].map((item) => (
            <div key={item.label} className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6 text-center">
              <p className="text-sm text-stone-400 mb-2">{item.label}</p>
              <p className="text-2xl font-bold bg-gradient-to-r from-pyrax-400 to-pyrax-600 bg-clip-text text-transparent">
                {item.value}
              </p>
            </div>
          ))}
        </motion.div>

        {/* Distribution Chart */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="mb-16"
        >
          <h3 className="text-2xl font-bold text-white mb-8 text-center">{t('distribution.title')}</h3>
          
          {/* Visual Bar */}
          <div className="h-8 rounded-full overflow-hidden flex mb-8">
            {distribution.map((item) => (
              <div
                key={item.name}
                className="h-full transition-all hover:opacity-80"
                style={{
                  backgroundColor: item.color,
                  width: item.percent,
                }}
                title={`${item.name}: ${item.percent}`}
              />
            ))}
          </div>

          {/* Legend */}
          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {distribution.map((item, index) => (
              <motion.div
                key={item.name}
                initial={{ opacity: 0, x: -10 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.05 }}
                className="flex items-start gap-3 p-3 rounded-xl bg-stone-900/30 border border-stone-800/50"
              >
                <div
                  className="w-4 h-4 rounded-full flex-shrink-0 mt-0.5"
                  style={{ backgroundColor: item.color }}
                />
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-white font-medium truncate">{item.name}</span>
                    <span className="text-stone-400 font-mono text-sm">{item.percent}</span>
                  </div>
                  <p className="text-xs text-stone-500 truncate">{item.desc}</p>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>

        {/* Fee Distribution & Staking */}
        <div className="grid lg:grid-cols-2 gap-8">
          {/* Fee Distribution */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
          >
            <h3 className="text-xl font-bold text-white mb-6">{t('feeDistribution.title')}</h3>
            
            <div className="space-y-4">
              {feeDistribution.map((item) => (
                <div key={item.name} className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-stone-300">{item.name}</span>
                    <span className="text-white font-mono">{item.percent}</span>
                  </div>
                  <div className="h-2 rounded-full bg-stone-800 overflow-hidden">
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

          {/* Staking Parameters */}
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
          >
            <h3 className="text-xl font-bold text-white mb-6">{t('staking.title')}</h3>
            
            <div className="space-y-4">
              {[
                { label: t('staking.validatorStake'), value: t('staking.validatorStakeValue') },
                { label: t('staking.minDelegation'), value: t('staking.minDelegationValue') },
                { label: t('staking.providerStake'), value: t('staking.providerStakeValue') },
                { label: t('staking.unbonding'), value: t('staking.unbondingValue') },
                { label: t('staking.doubleSign'), value: t('staking.doubleSignValue') },
                { label: t('staking.downtime'), value: t('staking.downtimeValue') },
              ].map((item) => (
                <div key={item.label} className="flex justify-between items-center py-2 border-b border-stone-800 last:border-0">
                  <span className="text-stone-400">{item.label}</span>
                  <span className="text-white font-mono">{item.value}</span>
                </div>
              ))}
            </div>
          </motion.div>
        </div>

        {/* View Full Tokenomics Button */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mt-12"
        >
          <Link
            href={`/${locale}/tokenomics`}
            className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-pyrax-500 to-pyrax-600 px-8 py-4 text-lg font-semibold text-white hover:from-pyrax-600 hover:to-pyrax-700 transition-all shadow-lg shadow-pyrax-500/25"
          >
            {t('viewFullTokenomics')}
            <ArrowRightIcon className="h-5 w-5" />
          </Link>
        </motion.div>
      </div>
    </section>
  );
}
