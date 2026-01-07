'use client';

import { useTranslations, useLocale } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Link from 'next/link';
import {
  CheckCircleIcon,
  FireIcon,
  ClockIcon,
  CalendarIcon,
  CurrencyDollarIcon,
  UserGroupIcon,
  ShieldCheckIcon,
  DocumentCheckIcon,
  IdentificationIcon,
  ExclamationTriangleIcon,
  ArrowRightIcon,
  SparklesIcon,
} from '@heroicons/react/24/outline';

export default function BDAGCommunityPage() {
  const t = useTranslations('bdagPage');
  const locale = useLocale();

  const stats = [
    { value: '10B', label: t('stats.poolSize'), sublabel: t('stats.poolSizeSub') },
    { value: '31.25%', label: t('stats.allocationRate'), sublabel: t('stats.allocationRateSub') },
    { value: '12 mo', label: t('stats.cliff'), sublabel: t('stats.cliffSub') },
    { value: '24 mo', label: t('stats.vesting'), sublabel: t('stats.vestingSub') },
  ];

  const allocationFormula = [
    { label: t('formula.base'), value: '25%', desc: t('formula.baseDesc') },
    { label: t('formula.bonus'), value: '+25%', desc: t('formula.bonusDesc') },
    { label: t('formula.total'), value: '31.25%', desc: t('formula.totalDesc') },
  ];

  const examples = [
    { purchase: '$100', base: '$25', bonus: '$6.25', total: '$31.25', tokens: '5,040' },
    { purchase: '$500', base: '$125', bonus: '$31.25', total: '$156.25', tokens: '25,201' },
    { purchase: '$1,000', base: '$250', bonus: '$62.50', total: '$312.50', tokens: '50,403' },
    { purchase: '$5,000', base: '$1,250', bonus: '$312.50', total: '$1,562.50', tokens: '252,016' },
    { purchase: '$10,000', base: '$2,500', bonus: '$625', total: '$3,125', tokens: '504,032' },
  ];

  const eligibility = [
    { icon: CurrencyDollarIcon, text: t('eligibility.req1') },
    { icon: ClockIcon, text: t('eligibility.req4') },
  ];

  const timeline = [
    { label: '0', desc: t('timeline.claimOpens') },
    { label: '-24h', desc: t('timeline.deadline') },
    { label: 'TGE', desc: t('timeline.tge') },
    { label: '12', desc: t('timeline.cliffEnds') },
    { label: '24', desc: t('timeline.fullyVested') },
  ];

  const burnBenefits = [
    t('burn.benefit1'),
    t('burn.benefit2'),
    t('burn.benefit3'),
    t('burn.benefit4'),
  ];

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-blue-500/10 via-transparent to-transparent" />
        <div className="absolute inset-0 bg-[linear-gradient(rgba(59,130,246,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(59,130,246,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        
        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <div className="inline-flex items-center gap-2 rounded-full bg-blue-500/10 border border-blue-500/20 px-4 py-2 text-sm text-blue-400 mb-6">
              <UserGroupIcon className="h-4 w-4" />
              {t('hero.badge')}
            </div>
            <h1 className="text-4xl sm:text-5xl lg:text-6xl font-bold text-white mb-6">
              {t('hero.title')}
            </h1>
            <p className="text-xl text-stone-400 max-w-3xl mx-auto mb-8">
              {t('hero.subtitle')}
            </p>
          </motion.div>

          {/* Stats Grid */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2, duration: 0.5 }}
            className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 max-w-4xl mx-auto mt-12"
          >
            {stats.map((stat, index) => (
              <div
                key={stat.label}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
              >
                <p className="text-3xl font-bold text-white mb-1">{stat.value}</p>
                <p className="text-sm font-medium text-blue-400">{stat.label}</p>
                <p className="text-xs text-stone-500 mt-1">{stat.sublabel}</p>
              </div>
            ))}
          </motion.div>
        </div>
      </section>

      {/* Allocation Formula */}
      <section className="py-20 border-t border-stone-800/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">
              {t('formula.title')}
            </h2>
          </motion.div>

          <div className="grid md:grid-cols-3 gap-6 max-w-4xl mx-auto mb-12">
            {allocationFormula.map((item, index) => (
              <motion.div
                key={item.label}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6 text-center"
              >
                <p className="text-sm text-stone-400 mb-2">{item.label}</p>
                <p className="text-4xl font-bold bg-gradient-to-r from-blue-400 to-blue-600 bg-clip-text text-transparent mb-2">
                  {item.value}
                </p>
                <p className="text-sm text-stone-500">{item.desc}</p>
              </motion.div>
            ))}
          </div>

          {/* Example calculation */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="max-w-3xl mx-auto rounded-2xl bg-gradient-to-r from-blue-500/10 to-purple-500/10 border border-blue-500/20 p-6"
          >
            <p className="text-stone-300 text-center">
              <span className="text-white font-semibold">{t('formula.example')}</span>{' '}
              {t('formula.exampleText')}
            </p>
          </motion.div>
        </div>
      </section>

      {/* Allocation Examples Table */}
      <section className="py-20 border-t border-stone-800/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">
              {t('examples.title')}
            </h2>
            <p className="text-stone-400">{t('examples.subtitle')}</p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="overflow-x-auto"
          >
            <table className="w-full max-w-4xl mx-auto">
              <thead>
                <tr className="border-b border-stone-800">
                  <th className="px-4 py-4 text-left text-sm font-semibold text-stone-400">{t('examples.colPurchase')}</th>
                  <th className="px-4 py-4 text-left text-sm font-semibold text-stone-400">{t('examples.colBase')}</th>
                  <th className="px-4 py-4 text-left text-sm font-semibold text-stone-400">{t('examples.colBonus')}</th>
                  <th className="px-4 py-4 text-left text-sm font-semibold text-stone-400">{t('examples.colTotal')}</th>
                  <th className="px-4 py-4 text-left text-sm font-semibold text-blue-400">{t('examples.colTokens')}</th>
                </tr>
              </thead>
              <tbody>
                {examples.map((row, index) => (
                  <tr key={index} className="border-b border-stone-800/50 hover:bg-stone-900/50 transition-colors">
                    <td className="px-4 py-4 text-white font-medium">{row.purchase}</td>
                    <td className="px-4 py-4 text-stone-300">{row.base}</td>
                    <td className="px-4 py-4 text-stone-300">{row.bonus}</td>
                    <td className="px-4 py-4 text-stone-300">{row.total}</td>
                    <td className="px-4 py-4 text-blue-400 font-semibold">{row.tokens}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </motion.div>
        </div>
      </section>

      {/* Eligibility Requirements */}
      <section className="py-20 border-t border-stone-800/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">
              {t('eligibility.title')}
            </h2>
          </motion.div>

          <div className="grid sm:grid-cols-2 gap-4 max-w-3xl mx-auto">
            {eligibility.map((item, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="flex items-start gap-4 rounded-xl bg-stone-900/50 border border-stone-800 p-5"
              >
                <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-green-500/10 flex items-center justify-center">
                  <CheckCircleIcon className="h-5 w-5 text-green-400" />
                </div>
                <p className="text-stone-300 pt-2">{item.text}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Vesting Schedule */}
      <section className="py-20 border-t border-stone-800/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">
              {t('vesting.title')}
            </h2>
          </motion.div>

          <div className="grid md:grid-cols-2 gap-6 max-w-3xl mx-auto mb-12">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
            >
              <div className="flex items-center gap-3 mb-4">
                <div className="w-12 h-12 rounded-xl bg-orange-500/10 flex items-center justify-center">
                  <ClockIcon className="h-6 w-6 text-orange-400" />
                </div>
                <div>
                  <p className="text-sm text-stone-400">{t('vesting.cliffPeriod')}</p>
                  <p className="text-2xl font-bold text-white">12 mo</p>
                </div>
              </div>
              <p className="text-stone-500">{t('vesting.cliffDesc')}</p>
            </motion.div>

            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.1 }}
              className="rounded-2xl bg-stone-900/50 border border-stone-800 p-6"
            >
              <div className="flex items-center gap-3 mb-4">
                <div className="w-12 h-12 rounded-xl bg-blue-500/10 flex items-center justify-center">
                  <CalendarIcon className="h-6 w-6 text-blue-400" />
                </div>
                <div>
                  <p className="text-sm text-stone-400">{t('vesting.vestingPeriod')}</p>
                  <p className="text-2xl font-bold text-white">12 mo</p>
                </div>
              </div>
              <p className="text-stone-500">{t('vesting.vestingDesc')}</p>
            </motion.div>
          </div>

          {/* Timeline */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-8"
          >
            <h3 className="text-xl font-semibold text-white mb-8">{t('timeline.title')}</h3>
          </motion.div>

          <div className="max-w-4xl mx-auto">
            <div className="flex justify-between items-center relative">
              <div className="absolute top-5 left-0 right-0 h-1 bg-stone-700" />
              {timeline.map((item, index) => (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: index * 0.1 }}
                  className="relative flex flex-col items-center"
                >
                  <div className="w-10 h-10 rounded-full flex items-center justify-center z-10 bg-stone-700">
                    <span className="text-xs font-bold text-stone-400">{item.label}</span>
                  </div>
                  <p className="mt-3 text-xs sm:text-sm text-stone-500 text-center max-w-[80px]">{item.desc}</p>
                </motion.div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Unclaimed Tokens Burned */}
      <section className="py-20 border-t border-stone-800/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="max-w-4xl mx-auto rounded-3xl bg-gradient-to-br from-red-500/10 to-orange-500/10 border border-red-500/20 p-8 md:p-12"
          >
            <div className="flex items-start gap-4 mb-6">
              <div className="flex-shrink-0 w-14 h-14 rounded-2xl bg-red-500/20 flex items-center justify-center">
                <FireIcon className="h-7 w-7 text-red-400" />
              </div>
              <div>
                <h3 className="text-2xl font-bold text-white mb-2">{t('burn.title')}</h3>
                <p className="text-stone-400">{t('burn.subtitle')}</p>
              </div>
            </div>

            <div className="grid sm:grid-cols-2 gap-4">
              {burnBenefits.map((benefit, index) => (
                <div key={index} className="flex items-center gap-3">
                  <CheckCircleIcon className="h-5 w-5 text-red-400 flex-shrink-0" />
                  <span className="text-stone-300">{benefit}</span>
                </div>
              ))}
            </div>
          </motion.div>
        </div>
      </section>

      {/* Claim Early */}
      <section className="py-20 border-t border-stone-800/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="max-w-4xl mx-auto rounded-3xl bg-gradient-to-br from-blue-500/10 to-purple-500/10 border border-blue-500/20 p-8 md:p-12"
          >
            <div className="flex items-start gap-4 mb-6">
              <div className="flex-shrink-0 w-14 h-14 rounded-2xl bg-blue-500/20 flex items-center justify-center">
                <SparklesIcon className="h-7 w-7 text-blue-400" />
              </div>
              <div>
                <h3 className="text-2xl font-bold text-white mb-2">{t('claimEarly.title')}</h3>
                <p className="text-stone-400">{t('claimEarly.subtitle')}</p>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 border-t border-stone-800/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center"
          >
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-8">
              {t('cta.title')}
            </h2>
            
            <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-8">
              <button
                disabled
                className="inline-flex items-center gap-2 rounded-xl bg-stone-700 px-8 py-4 text-lg font-semibold text-stone-400 cursor-not-allowed opacity-70"
              >
                {t('cta.submit')}
                <ArrowRightIcon className="h-5 w-5" />
              </button>
              <Link
                href={`/${locale}/tokenomics`}
                className="inline-flex items-center gap-2 rounded-xl border border-stone-700 bg-stone-900/50 px-8 py-4 text-lg font-semibold text-white hover:bg-stone-800 transition-all"
              >
                {t('cta.viewVesting')}
              </Link>
            </div>

            {/* Social Media Disclaimer */}
            <div className="mt-8 p-6 rounded-2xl bg-stone-900/50 border border-stone-800 max-w-2xl mx-auto">
              <p className="text-stone-400 mb-4">
                {t('cta.portalDisclaimer')}
              </p>
              <div className="flex items-center justify-center gap-4">
                <a
                  href="https://t.me/+TmDvlOc8TxxmNzAx"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-12 h-12 rounded-full bg-stone-800 hover:bg-blue-500/20 flex items-center justify-center transition-colors"
                >
                  <img src="/icons/telegram.svg" alt="Telegram" className="w-6 h-6" />
                </a>
                <a
                  href="https://discord.gg/z9kjrE9q"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-12 h-12 rounded-full bg-stone-800 hover:bg-blue-500/20 flex items-center justify-center transition-colors"
                >
                  <img src="/icons/discord.svg" alt="Discord" className="w-6 h-6" />
                </a>
                <a
                  href="https://www.facebook.com/share/1BH17cWju3/?mibextid=wwXIfr"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-12 h-12 rounded-full bg-stone-800 hover:bg-blue-500/20 flex items-center justify-center transition-colors"
                >
                  <img src="/icons/facebook.svg" alt="Facebook" className="w-6 h-6" />
                </a>
                <a
                  href="https://twitter.com/pyrax_org"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-12 h-12 rounded-full bg-stone-800 hover:bg-blue-500/20 flex items-center justify-center transition-colors"
                >
                  <img src="/icons/twitter.svg" alt="Twitter" className="w-6 h-6" />
                </a>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
