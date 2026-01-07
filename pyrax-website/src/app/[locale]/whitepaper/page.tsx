'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Link from 'next/link';
import { 
  CpuChipIcon, 
  CubeIcon, 
  ShieldCheckIcon,
  BoltIcon,
  CurrencyDollarIcon,
  UserGroupIcon,
  RocketLaunchIcon,
  GlobeAltIcon,
  ServerStackIcon,
  ChartBarIcon,
  LockClosedIcon,
  ArrowRightIcon,
} from '@heroicons/react/24/outline';

export default function WhitepaperPage() {
  const t = useTranslations('whitepaper');

  const fadeIn = {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
    transition: { duration: 0.5 }
  };

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/10 via-transparent to-transparent" />
        <div className="absolute top-20 left-1/4 w-96 h-96 bg-pyrax-500/20 rounded-full blur-3xl" />
        <div className="absolute top-40 right-1/4 w-64 h-64 bg-blue-500/10 rounded-full blur-3xl" />
        
        <div className="relative z-10 max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div {...fadeIn}>
            <span className="inline-block px-4 py-2 rounded-full bg-pyrax-500/10 text-pyrax-400 text-sm font-medium mb-6">
              {t('badge')}
            </span>
            <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6">
              {t('title')}
            </h1>
            <p className="text-xl text-stone-400 mb-8 leading-relaxed">
              {t('subtitle')}
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <Link 
                href="#what-is-pyrax"
                className="px-6 py-3 bg-gradient-to-r from-pyrax-500 to-pyrax-600 text-white font-semibold rounded-xl hover:shadow-lg hover:shadow-pyrax-500/25 transition-all"
              >
                {t('startReading')}
              </Link>
              <Link 
                href="/technical-whitepaper"
                className="px-6 py-3 bg-stone-800 text-white font-semibold rounded-xl hover:bg-stone-700 transition-all border border-stone-700"
              >
                {t('technicalVersion')}
              </Link>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Table of Contents */}
      <section className="py-12 border-y border-stone-800">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-lg font-semibold text-white mb-6">{t('toc.title')}</h2>
          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {[
              { href: '#what-is-pyrax', label: t('toc.whatIsPyrax') },
              { href: '#problem', label: t('toc.problem') },
              { href: '#solution', label: t('toc.solution') },
              { href: '#how-it-works', label: t('toc.howItWorks') },
              { href: '#mining', label: t('toc.mining') },
              { href: '#ai-platform', label: t('toc.aiPlatform') },
              { href: '#tokenomics', label: t('toc.tokenomics') },
              { href: '#roadmap', label: t('toc.roadmap') },
              { href: '#conclusion', label: t('toc.conclusion') },
            ].map((item) => (
              <a 
                key={item.href}
                href={item.href}
                className="flex items-center gap-2 text-stone-400 hover:text-pyrax-400 transition-colors"
              >
                <ArrowRightIcon className="w-4 h-4" />
                {item.label}
              </a>
            ))}
          </div>
        </div>
      </section>

      {/* Section 1: What is PYRAX */}
      <section id="what-is-pyrax" className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-pyrax-500/20 rounded-lg">
                <CpuChipIcon className="w-6 h-6 text-pyrax-400" />
              </div>
              <span className="text-pyrax-400 font-medium">{t('sections.whatIs.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.whatIs.title')}</h2>
            <div className="prose prose-lg prose-invert max-w-none">
              <p className="text-stone-300 text-lg leading-relaxed mb-6">
                {t('sections.whatIs.p1')}
              </p>
              <p className="text-stone-300 text-lg leading-relaxed mb-6">
                {t('sections.whatIs.p2')}
              </p>
              <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 my-8">
                <h4 className="text-white font-semibold mb-4">{t('sections.whatIs.keyPoints')}</h4>
                <ul className="space-y-3">
                  <li className="flex items-start gap-3 text-stone-300">
                    <span className="text-pyrax-400 mt-1">✓</span>
                    {t('sections.whatIs.point1')}
                  </li>
                  <li className="flex items-start gap-3 text-stone-300">
                    <span className="text-pyrax-400 mt-1">✓</span>
                    {t('sections.whatIs.point2')}
                  </li>
                  <li className="flex items-start gap-3 text-stone-300">
                    <span className="text-pyrax-400 mt-1">✓</span>
                    {t('sections.whatIs.point3')}
                  </li>
                  <li className="flex items-start gap-3 text-stone-300">
                    <span className="text-pyrax-400 mt-1">✓</span>
                    {t('sections.whatIs.point4')}
                  </li>
                </ul>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 2: The Problem */}
      <section id="problem" className="py-20 bg-stone-900/30">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-red-500/20 rounded-lg">
                <ShieldCheckIcon className="w-6 h-6 text-red-400" />
              </div>
              <span className="text-red-400 font-medium">{t('sections.problem.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.problem.title')}</h2>
            <div className="space-y-6">
              <p className="text-stone-300 text-lg leading-relaxed">
                {t('sections.problem.intro')}
              </p>
              <div className="grid md:grid-cols-2 gap-6">
                <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                  <h4 className="text-white font-semibold mb-3">{t('sections.problem.issue1.title')}</h4>
                  <p className="text-stone-400">{t('sections.problem.issue1.desc')}</p>
                </div>
                <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                  <h4 className="text-white font-semibold mb-3">{t('sections.problem.issue2.title')}</h4>
                  <p className="text-stone-400">{t('sections.problem.issue2.desc')}</p>
                </div>
                <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                  <h4 className="text-white font-semibold mb-3">{t('sections.problem.issue3.title')}</h4>
                  <p className="text-stone-400">{t('sections.problem.issue3.desc')}</p>
                </div>
                <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                  <h4 className="text-white font-semibold mb-3">{t('sections.problem.issue4.title')}</h4>
                  <p className="text-stone-400">{t('sections.problem.issue4.desc')}</p>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 3: Our Solution */}
      <section id="solution" className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-green-500/20 rounded-lg">
                <BoltIcon className="w-6 h-6 text-green-400" />
              </div>
              <span className="text-green-400 font-medium">{t('sections.solution.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.solution.title')}</h2>
            <p className="text-stone-300 text-lg leading-relaxed mb-8">
              {t('sections.solution.intro')}
            </p>
            <div className="space-y-4">
              <div className="bg-gradient-to-r from-pyrax-500/10 to-transparent border border-pyrax-500/20 rounded-xl p-6">
                <h4 className="text-white font-semibold mb-2">{t('sections.solution.benefit1.title')}</h4>
                <p className="text-stone-400">{t('sections.solution.benefit1.desc')}</p>
              </div>
              <div className="bg-gradient-to-r from-blue-500/10 to-transparent border border-blue-500/20 rounded-xl p-6">
                <h4 className="text-white font-semibold mb-2">{t('sections.solution.benefit2.title')}</h4>
                <p className="text-stone-400">{t('sections.solution.benefit2.desc')}</p>
              </div>
              <div className="bg-gradient-to-r from-purple-500/10 to-transparent border border-purple-500/20 rounded-xl p-6">
                <h4 className="text-white font-semibold mb-2">{t('sections.solution.benefit3.title')}</h4>
                <p className="text-stone-400">{t('sections.solution.benefit3.desc')}</p>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 4: How It Works */}
      <section id="how-it-works" className="py-20 bg-stone-900/30">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-blue-500/20 rounded-lg">
                <CubeIcon className="w-6 h-6 text-blue-400" />
              </div>
              <span className="text-blue-400 font-medium">{t('sections.howItWorks.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.howItWorks.title')}</h2>
            <p className="text-stone-300 text-lg leading-relaxed mb-8">
              {t('sections.howItWorks.intro')}
            </p>

            {/* TriStream Explanation */}
            <div className="mb-12">
              <h3 className="text-2xl font-bold text-white mb-6">{t('sections.howItWorks.tristream.title')}</h3>
              <p className="text-stone-300 mb-6">{t('sections.howItWorks.tristream.desc')}</p>
              <div className="grid md:grid-cols-3 gap-6">
                <div className="bg-gradient-to-b from-pyrax-500/20 to-stone-900 border border-pyrax-500/30 rounded-xl p-6">
                  <div className="text-3xl font-bold text-pyrax-400 mb-2">A</div>
                  <h4 className="text-white font-semibold mb-2">{t('sections.howItWorks.tristream.streamA.name')}</h4>
                  <p className="text-stone-400 text-sm mb-3">{t('sections.howItWorks.tristream.streamA.desc')}</p>
                  <div className="text-xs text-stone-500">
                    <div>BLAKE3 • 10s blocks</div>
                  </div>
                </div>
                <div className="bg-gradient-to-b from-blue-500/20 to-stone-900 border border-blue-500/30 rounded-xl p-6">
                  <div className="text-3xl font-bold text-blue-400 mb-2">B</div>
                  <h4 className="text-white font-semibold mb-2">{t('sections.howItWorks.tristream.streamB.name')}</h4>
                  <p className="text-stone-400 text-sm mb-3">{t('sections.howItWorks.tristream.streamB.desc')}</p>
                  <div className="text-xs text-stone-500">
                    <div>KAWPOW • 60s blocks</div>
                  </div>
                </div>
                <div className="bg-gradient-to-b from-purple-500/20 to-stone-900 border border-purple-500/30 rounded-xl p-6">
                  <div className="text-3xl font-bold text-purple-400 mb-2">C</div>
                  <h4 className="text-white font-semibold mb-2">{t('sections.howItWorks.tristream.streamC.name')}</h4>
                  <p className="text-stone-400 text-sm mb-3">{t('sections.howItWorks.tristream.streamC.desc')}</p>
                  <div className="text-xs text-stone-500">
                    <div>ZK-STARK • Finality</div>
                  </div>
                </div>
              </div>
            </div>

            {/* Layers */}
            <div>
              <h3 className="text-2xl font-bold text-white mb-6">{t('sections.howItWorks.layers.title')}</h3>
              <div className="space-y-4">
                <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                  <div className="flex items-center gap-3 mb-3">
                    <span className="px-3 py-1 bg-purple-500/20 text-purple-400 text-sm font-medium rounded-full">Layer 3</span>
                    <h4 className="text-white font-semibold">{t('sections.howItWorks.layers.l3.title')}</h4>
                  </div>
                  <p className="text-stone-400">{t('sections.howItWorks.layers.l3.desc')}</p>
                </div>
                <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                  <div className="flex items-center gap-3 mb-3">
                    <span className="px-3 py-1 bg-blue-500/20 text-blue-400 text-sm font-medium rounded-full">Layer 2</span>
                    <h4 className="text-white font-semibold">{t('sections.howItWorks.layers.l2.title')}</h4>
                  </div>
                  <p className="text-stone-400">{t('sections.howItWorks.layers.l2.desc')}</p>
                </div>
                <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                  <div className="flex items-center gap-3 mb-3">
                    <span className="px-3 py-1 bg-pyrax-500/20 text-pyrax-400 text-sm font-medium rounded-full">Layer 1</span>
                    <h4 className="text-white font-semibold">{t('sections.howItWorks.layers.l1.title')}</h4>
                  </div>
                  <p className="text-stone-400">{t('sections.howItWorks.layers.l1.desc')}</p>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 5: Mining */}
      <section id="mining" className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-yellow-500/20 rounded-lg">
                <CubeIcon className="w-6 h-6 text-yellow-400" />
              </div>
              <span className="text-yellow-400 font-medium">{t('sections.mining.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.mining.title')}</h2>
            <p className="text-stone-300 text-lg leading-relaxed mb-8">
              {t('sections.mining.intro')}
            </p>
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 mb-8">
              <h4 className="text-white font-semibold mb-4">{t('sections.mining.ways.title')}</h4>
              <div className="space-y-4">
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 bg-pyrax-500/20 rounded-full flex items-center justify-center text-pyrax-400 font-bold">1</div>
                  <div>
                    <h5 className="text-white font-medium">{t('sections.mining.ways.way1.title')}</h5>
                    <p className="text-stone-400 text-sm">{t('sections.mining.ways.way1.desc')}</p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 bg-blue-500/20 rounded-full flex items-center justify-center text-blue-400 font-bold">2</div>
                  <div>
                    <h5 className="text-white font-medium">{t('sections.mining.ways.way2.title')}</h5>
                    <p className="text-stone-400 text-sm">{t('sections.mining.ways.way2.desc')}</p>
                  </div>
                </div>
                <div className="flex items-start gap-4">
                  <div className="w-8 h-8 bg-purple-500/20 rounded-full flex items-center justify-center text-purple-400 font-bold">3</div>
                  <div>
                    <h5 className="text-white font-medium">{t('sections.mining.ways.way3.title')}</h5>
                    <p className="text-stone-400 text-sm">{t('sections.mining.ways.way3.desc')}</p>
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 6: AI Platform */}
      <section id="ai-platform" className="py-20 bg-stone-900/30">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-cyan-500/20 rounded-lg">
                <ServerStackIcon className="w-6 h-6 text-cyan-400" />
              </div>
              <span className="text-cyan-400 font-medium">{t('sections.ai.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.ai.title')}</h2>
            <p className="text-stone-300 text-lg leading-relaxed mb-8">
              {t('sections.ai.intro')}
            </p>
            <div className="grid md:grid-cols-2 gap-6 mb-8">
              <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                <h4 className="text-white font-semibold mb-3">{t('sections.ai.foundry.title')}</h4>
                <p className="text-stone-400 mb-4">{t('sections.ai.foundry.desc')}</p>
                <ul className="space-y-2 text-sm text-stone-400">
                  <li className="flex items-center gap-2"><span className="text-green-400">•</span> {t('sections.ai.foundry.feature1')}</li>
                  <li className="flex items-center gap-2"><span className="text-green-400">•</span> {t('sections.ai.foundry.feature2')}</li>
                  <li className="flex items-center gap-2"><span className="text-green-400">•</span> {t('sections.ai.foundry.feature3')}</li>
                </ul>
              </div>
              <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
                <h4 className="text-white font-semibold mb-3">{t('sections.ai.crucible.title')}</h4>
                <p className="text-stone-400 mb-4">{t('sections.ai.crucible.desc')}</p>
                <ul className="space-y-2 text-sm text-stone-400">
                  <li className="flex items-center gap-2"><span className="text-green-400">•</span> {t('sections.ai.crucible.feature1')}</li>
                  <li className="flex items-center gap-2"><span className="text-green-400">•</span> {t('sections.ai.crucible.feature2')}</li>
                  <li className="flex items-center gap-2"><span className="text-green-400">•</span> {t('sections.ai.crucible.feature3')}</li>
                </ul>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 7: Tokenomics */}
      <section id="tokenomics" className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-green-500/20 rounded-lg">
                <CurrencyDollarIcon className="w-6 h-6 text-green-400" />
              </div>
              <span className="text-green-400 font-medium">{t('sections.tokenomics.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.tokenomics.title')}</h2>
            <p className="text-stone-300 text-lg leading-relaxed mb-8">
              {t('sections.tokenomics.intro')}
            </p>
            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
              <div className="bg-stone-900 border border-stone-800 rounded-xl p-4 text-center">
                <div className="text-2xl font-bold text-white">$PYRAX</div>
                <div className="text-stone-500 text-sm">{t('sections.tokenomics.symbol')}</div>
              </div>
              <div className="bg-stone-900 border border-stone-800 rounded-xl p-4 text-center">
                <div className="text-2xl font-bold text-white">100B</div>
                <div className="text-stone-500 text-sm">{t('sections.tokenomics.supply')}</div>
              </div>
              <div className="bg-stone-900 border border-stone-800 rounded-xl p-4 text-center">
                <div className="text-2xl font-bold text-white">8</div>
                <div className="text-stone-500 text-sm">{t('sections.tokenomics.decimals')}</div>
              </div>
              <div className="bg-stone-900 border border-stone-800 rounded-xl p-4 text-center">
                <div className="text-2xl font-bold text-white">~4 yr</div>
                <div className="text-stone-500 text-sm">{t('sections.tokenomics.halving')}</div>
              </div>
            </div>
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
              <h4 className="text-white font-semibold mb-4">{t('sections.tokenomics.distribution.title')}</h4>
              <div className="grid sm:grid-cols-2 gap-4">
                <div className="flex justify-between items-center py-2 border-b border-stone-800">
                  <span className="text-stone-400">{t('sections.tokenomics.distribution.mining')}</span>
                  <span className="text-white font-medium">35%</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-stone-800">
                  <span className="text-stone-400">{t('sections.tokenomics.distribution.presale')}</span>
                  <span className="text-white font-medium">15%</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-stone-800">
                  <span className="text-stone-400">{t('sections.tokenomics.distribution.ecosystem')}</span>
                  <span className="text-white font-medium">10%</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-stone-800">
                  <span className="text-stone-400">{t('sections.tokenomics.distribution.liquidity')}</span>
                  <span className="text-white font-medium">10%</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-stone-800">
                  <span className="text-stone-400">{t('sections.tokenomics.distribution.bdag')}</span>
                  <span className="text-white font-medium">10%</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-stone-800">
                  <span className="text-stone-400">{t('sections.tokenomics.distribution.other')}</span>
                  <span className="text-white font-medium">20%</span>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 8: Roadmap */}
      <section id="roadmap" className="py-20 bg-stone-900/30">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-indigo-500/20 rounded-lg">
                <RocketLaunchIcon className="w-6 h-6 text-indigo-400" />
              </div>
              <span className="text-indigo-400 font-medium">{t('sections.roadmap.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.roadmap.title')}</h2>
            <div className="space-y-6">
              <div className="flex gap-4">
                <div className="w-24 shrink-0 text-right">
                  <span className="text-pyrax-400 font-semibold">Q4 2025</span>
                </div>
                <div className="flex-1 bg-stone-900 border border-stone-800 rounded-xl p-4">
                  <h4 className="text-white font-medium mb-2">{t('sections.roadmap.q4_2025.title')}</h4>
                  <p className="text-stone-400 text-sm">{t('sections.roadmap.q4_2025.desc')}</p>
                </div>
              </div>
              <div className="flex gap-4">
                <div className="w-24 shrink-0 text-right">
                  <span className="text-blue-400 font-semibold">Q1-Q2 2026</span>
                </div>
                <div className="flex-1 bg-stone-900 border border-stone-800 rounded-xl p-4">
                  <h4 className="text-white font-medium mb-2">{t('sections.roadmap.q1q2_2026.title')}</h4>
                  <p className="text-stone-400 text-sm">{t('sections.roadmap.q1q2_2026.desc')}</p>
                </div>
              </div>
              <div className="flex gap-4">
                <div className="w-24 shrink-0 text-right">
                  <span className="text-green-400 font-semibold">Q3 2026</span>
                </div>
                <div className="flex-1 bg-gradient-to-r from-green-500/10 to-stone-900 border border-green-500/30 rounded-xl p-4">
                  <h4 className="text-white font-medium mb-2">{t('sections.roadmap.q3_2026.title')}</h4>
                  <p className="text-stone-400 text-sm">{t('sections.roadmap.q3_2026.desc')}</p>
                </div>
              </div>
              <div className="flex gap-4">
                <div className="w-24 shrink-0 text-right">
                  <span className="text-purple-400 font-semibold">Q4 2026+</span>
                </div>
                <div className="flex-1 bg-stone-900 border border-stone-800 rounded-xl p-4">
                  <h4 className="text-white font-medium mb-2">{t('sections.roadmap.q4_2026.title')}</h4>
                  <p className="text-stone-400 text-sm">{t('sections.roadmap.q4_2026.desc')}</p>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Section 9: Conclusion */}
      <section id="conclusion" className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center"
          >
            <div className="flex items-center justify-center gap-3 mb-6">
              <div className="p-2 bg-pyrax-500/20 rounded-lg">
                <GlobeAltIcon className="w-6 h-6 text-pyrax-400" />
              </div>
              <span className="text-pyrax-400 font-medium">{t('sections.conclusion.label')}</span>
            </div>
            <h2 className="text-4xl font-bold text-white mb-6">{t('sections.conclusion.title')}</h2>
            <p className="text-stone-300 text-lg leading-relaxed mb-8 max-w-2xl mx-auto">
              {t('sections.conclusion.text')}
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <Link 
                href="/technical-whitepaper"
                className="px-6 py-3 bg-stone-800 text-white font-semibold rounded-xl hover:bg-stone-700 transition-all border border-stone-700"
              >
                {t('sections.conclusion.readTechnical')}
              </Link>
              <a 
                href="https://discord.gg/sS7kaacRwU"
                target="_blank"
                rel="noopener noreferrer"
                className="px-6 py-3 bg-gradient-to-r from-pyrax-500 to-pyrax-600 text-white font-semibold rounded-xl hover:shadow-lg hover:shadow-pyrax-500/25 transition-all"
              >
                {t('sections.conclusion.joinCommunity')}
              </a>
            </div>
          </motion.div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
