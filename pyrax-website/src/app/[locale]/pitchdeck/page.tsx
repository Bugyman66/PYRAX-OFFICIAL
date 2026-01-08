'use client';

import { useState, useEffect } from 'react';
import { useTranslations, useLocale } from 'next-intl';
import { motion, AnimatePresence } from 'framer-motion';
import Link from 'next/link';
import Image from 'next/image';
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  HomeIcon,
  ChevronDownIcon,
  CpuChipIcon,
  ServerStackIcon,
  BoltIcon,
  CurrencyDollarIcon,
  ShieldCheckIcon,
  GlobeAltIcon,
  RocketLaunchIcon,
  ChartBarIcon,
  UserGroupIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  ArrowTrendingUpIcon,
  ClockIcon,
  CommandLineIcon,
  CubeTransparentIcon,
  SparklesIcon,
} from '@heroicons/react/24/outline';

const slideIds = ['title', 'problem', 'solution', 'market', 'technology', 'tristream', 'aiPlatform', 'providers', 'tokenomics', 'traction', 'roadmap', 'competitive', 'cta'];

const cardVariants = {
  hidden: { opacity: 0, y: 30, scale: 0.95 },
  visible: (i: number) => ({
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { delay: i * 0.1, duration: 0.4, ease: [0.25, 0.46, 0.45, 0.94] as const }
  }),
  hover: {
    scale: 1.02,
    y: -5,
    transition: { duration: 0.2 }
  }
};

const staggerContainer = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.08, delayChildren: 0.1 }
  }
};

const fadeInUp = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.4 } }
};

function AnimatedCounter({ value, suffix = '', prefix = '', duration = 2 }: { value: number; suffix?: string; prefix?: string; duration?: number }) {
  const [count, setCount] = useState(0);
  useEffect(() => {
    let start = 0;
    const end = value;
    const increment = end / (duration * 60);
    const timer = setInterval(() => {
      start += increment;
      if (start >= end) {
        setCount(end);
        clearInterval(timer);
      } else {
        setCount(Math.floor(start));
      }
    }, 1000 / 60);
    return () => clearInterval(timer);
  }, [value, duration]);
  return <span>{prefix}{count.toLocaleString()}{suffix}</span>;
}

function LiveDataBar({ label, value, maxValue, color, delay = 0, suffix = '', prefix = '' }: { label: string; value: number; maxValue: number; color: string; delay?: number; suffix?: string; prefix?: string }) {
  return (
    <div className="mb-3">
      <div className="flex justify-between text-sm mb-1">
        <span className="text-stone-400">{label}</span>
        <span className={`font-bold ${color}`}>{prefix}{value.toLocaleString()}{suffix}</span>
      </div>
      <div className="h-3 bg-stone-800 rounded-full overflow-hidden">
        <motion.div
          className={`h-full ${color.replace('text-', 'bg-')}`}
          initial={{ width: 0 }}
          animate={{ width: `${(value / maxValue) * 100}%` }}
          transition={{ delay, duration: 1.2, ease: 'easeOut' }}
        />
      </div>
    </div>
  );
}

export default function PitchDeckPage() {
  const t = useTranslations('pitchDeck');
  const locale = useLocale();
  const [current, setCurrent] = useState(0);

  const next = () => current < slideIds.length - 1 && setCurrent(current + 1);
  const prev = () => current > 0 && setCurrent(current - 1);

  return (
    <div className="min-h-screen bg-gradient-to-b from-stone-950 via-stone-900 to-stone-950">
      <div className="fixed top-0 left-0 right-0 z-50 bg-stone-950/80 backdrop-blur-xl border-b border-stone-800">
        <div className="max-w-7xl mx-auto px-4 py-3 flex items-center justify-between">
          <Link href={`/${locale}`} className="flex items-center gap-2 text-stone-400 hover:text-white transition-colors">
            <HomeIcon className="h-5 w-5" />
            <span className="text-sm">{t('backToHome')}</span>
          </Link>
          <div className="flex items-center gap-2">
            <Image src="/pyrax-brand/pyrax-logo.svg" alt="PYRAX" width={80} height={28} className="h-7 w-auto" />
            <span className="text-stone-500">|</span>
            <span className="text-stone-400 text-sm">{t('title')}</span>
          </div>
          <span className="text-stone-500">{current + 1}/{slideIds.length}</span>
        </div>
      </div>

      <div className="fixed top-[53px] left-0 right-0 z-40 h-1 bg-stone-800">
        <motion.div 
          className="h-full bg-gradient-to-r from-pyrax-600 to-pyrax-400" 
          animate={{ width: `${((current + 1) / slideIds.length) * 100}%` }}
          transition={{ duration: 0.3, ease: 'easeOut' }}
        />
      </div>

      <div className="pt-24 pb-28 px-4 min-h-screen flex items-center justify-center">
        <AnimatePresence mode="wait">
          <motion.div
            key={current}
            initial={{ opacity: 0, x: 50, scale: 0.98 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: -50, scale: 0.98 }}
            transition={{ duration: 0.4, ease: 'easeOut' }}
            className="max-w-5xl w-full"
          >
            <Slide id={slideIds[current]} t={t} locale={locale} />
          </motion.div>
        </AnimatePresence>
      </div>

      <div className="fixed bottom-0 left-0 right-0 z-50 bg-stone-950/80 backdrop-blur-xl border-t border-stone-800">
        <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
          <button onClick={prev} disabled={current === 0} className={`flex items-center gap-2 px-6 py-3 rounded-xl font-medium transition-all duration-200 ${current === 0 ? 'bg-stone-800/50 text-stone-600 cursor-not-allowed' : 'bg-stone-800 text-white hover:bg-stone-700 hover:scale-105'}`}>
            <ChevronLeftIcon className="h-5 w-5" />{t('previous')}
          </button>
          <div className="flex gap-1.5">
            {slideIds.map((_, i) => (
              <button 
                key={i} 
                onClick={() => setCurrent(i)} 
                className={`h-2 rounded-full transition-all duration-300 ${current === i ? 'bg-pyrax-500 w-6' : 'bg-stone-600 w-2 hover:bg-stone-500'}`} 
              />
            ))}
          </div>
          <button onClick={next} disabled={current === slideIds.length - 1} className={`flex items-center gap-2 px-6 py-3 rounded-xl font-medium transition-all duration-200 ${current === slideIds.length - 1 ? 'bg-stone-800/50 text-stone-600 cursor-not-allowed' : 'bg-gradient-to-r from-pyrax-500 to-pyrax-600 text-white hover:shadow-lg hover:shadow-pyrax-500/30 hover:scale-105'}`}>
            {t('next')}<ChevronRightIcon className="h-5 w-5" />
          </button>
        </div>
      </div>
    </div>
  );
}

function Slide({ id, t, locale }: { id: string; t: any; locale: string }) {
  const [expandedCards, setExpandedCards] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState(0);
  const [hoveredFeature, setHoveredFeature] = useState<string | null>(null);
  
  const toggleCard = (cardId: string) => {
    setExpandedCards(prev => 
      prev.includes(cardId) ? prev.filter(id => id !== cardId) : [...prev, cardId]
    );
  };

  // SLIDE: TITLE - Impressive hero with live stats
  if (id === 'title') return (
    <div className="text-center">
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="inline-flex items-center gap-3 px-5 py-2.5 bg-gradient-to-r from-pyrax-500/20 to-amber-500/20 border border-pyrax-500/40 rounded-full mb-8"
      >
        <span className="relative flex h-3 w-3">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-pyrax-400 opacity-75"></span>
          <span className="relative inline-flex rounded-full h-3 w-3 bg-pyrax-500"></span>
        </span>
        <span className="text-pyrax-300 font-semibold">INVESTOR PRESENTATION 2026</span>
      </motion.div>
      
      <motion.div
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.6, delay: 0.1 }}
        className="mb-4"
      >
        <Image 
          src="/pyrax-brand/pyrax-logo.svg" 
          alt="PYRAX" 
          width={450} 
          height={160} 
          className="h-36 md:h-44 w-auto mx-auto drop-shadow-2xl"
          priority
        />
      </motion.div>
      
      <motion.p 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="text-2xl md:text-4xl text-white font-bold mb-3"
      >
        The Future of Decentralized AI Compute
      </motion.p>
      
      <motion.p 
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.4 }}
        className="text-lg text-stone-400 mb-10 max-w-2xl mx-auto"
      >
        Layer 1 blockchain with native AI marketplace • TriStream consensus • 500K+ TPS
      </motion.p>
      
      {/* Key Metrics Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-5xl mx-auto mb-8">
        {[
          { icon: CubeTransparentIcon, value: 100, suffix: 'B', label: 'Total Supply', color: 'text-pyrax-400' },
          { icon: BoltIcon, value: 500, suffix: 'K+', label: 'TPS (Layer 2)', color: 'text-amber-400' },
          { icon: ShieldCheckIcon, value: 3, suffix: '', label: 'Mining Streams', color: 'text-green-400' },
          { icon: RocketLaunchIcon, value: 2026, suffix: '', label: 'Mainnet Launch', color: 'text-blue-400' },
        ].map((stat, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5 + i * 0.1 }}
            whileHover={{ scale: 1.05, y: -5 }}
            className="bg-gradient-to-br from-stone-800/80 to-stone-900/80 border border-stone-700/50 rounded-2xl p-5 backdrop-blur-sm group cursor-pointer"
          >
            <stat.icon className={`h-8 w-8 ${stat.color} mx-auto mb-3 group-hover:scale-110 transition-transform`} />
            <div className={`text-3xl font-black ${stat.color}`}>
              <AnimatedCounter value={stat.value} suffix={stat.suffix} duration={1.5} />
            </div>
            <div className="text-stone-500 text-sm mt-1">{stat.label}</div>
          </motion.div>
        ))}
      </div>

      {/* Value Proposition */}
      <motion.div 
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1 }}
        className="flex flex-wrap justify-center gap-3"
      >
        {['GPU Mining + AI Compute', 'EVM Compatible', 'ZK-Rollups', 'Decentralized'].map((tag, i) => (
          <span key={i} className="px-4 py-2 bg-stone-800/50 border border-stone-700 rounded-full text-stone-300 text-sm">
            {tag}
          </span>
        ))}
      </motion.div>
    </div>
  );

  // SLIDE: PROBLEM - Market pain points with real data
  if (id === 'problem') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-red-400 font-semibold text-sm uppercase tracking-wider">The $50B+ Problem</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Why the Market Needs PYRAX</h2>
      </motion.div>

      <div className="grid lg:grid-cols-2 gap-6">
        {/* Problem Cards */}
        <motion.div variants={staggerContainer} initial="hidden" animate="visible" className="space-y-4">
          {[
            { 
              icon: CurrencyDollarIcon, 
              title: 'AI Compute is Unaffordable', 
              stat: '$100M+', 
              statLabel: 'estimated cost to train GPT-4',
              desc: 'Training large AI models costs tens of millions. Cloud GPU rates of $2-8/hour price out small developers and researchers entirely.',
              color: 'red'
            },
            { 
              icon: ServerStackIcon, 
              title: 'Centralized Monopoly', 
              stat: '65%', 
              statLabel: 'controlled by 3 companies',
              desc: 'AWS, Azure, and GCP dominate. Single points of failure, vendor lock-in, and geographic restrictions.',
              color: 'orange'
            },
            { 
              icon: CpuChipIcon, 
              title: 'Massive GPU Waste', 
              stat: '500M+', 
              statLabel: 'idle gaming GPUs',
              desc: 'Gaming GPUs average <20% utilization. Over $100B in hardware sits idle that could power AI.',
              color: 'amber'
            },
            { 
              icon: ClockIcon, 
              title: 'Blockchains Too Slow', 
              stat: '15 TPS', 
              statLabel: 'Ethereum average',
              desc: 'Current blockchains cannot scale for AI workloads. Need 100,000+ TPS for real applications.',
              color: 'yellow'
            },
          ].map((problem, i) => (
            <motion.div
              key={i}
              variants={fadeInUp}
              whileHover={{ x: 10, scale: 1.01 }}
              className={`bg-gradient-to-r from-${problem.color}-900/30 to-stone-900/50 border border-${problem.color}-900/40 rounded-xl p-5 cursor-pointer group`}
            >
              <div className="flex items-start gap-4">
                <div className={`w-12 h-12 bg-${problem.color}-500/20 rounded-xl flex items-center justify-center flex-shrink-0`}>
                  <problem.icon className={`h-6 w-6 text-${problem.color}-400`} />
                </div>
                <div className="flex-1">
                  <div className="flex items-center justify-between mb-1">
                    <h3 className="text-lg font-bold text-white">{problem.title}</h3>
                    <div className="text-right">
                      <span className={`text-${problem.color}-400 font-black text-xl`}>{problem.stat}</span>
                      <span className="text-stone-500 text-xs block">{problem.statLabel}</span>
                    </div>
                  </div>
                  <p className="text-stone-400 text-sm">{problem.desc}</p>
                </div>
              </div>
            </motion.div>
          ))}
        </motion.div>

        {/* Market Impact Visualization */}
        <motion.div
          initial={{ opacity: 0, x: 30 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.3 }}
          className="bg-gradient-to-br from-stone-800/60 to-stone-900/60 border border-stone-700 rounded-2xl p-6"
        >
          <h3 className="text-xl font-bold text-white mb-6 flex items-center gap-2">
            <ChartBarIcon className="h-6 w-6 text-red-400" />
            Market Pain Points
          </h3>
          
          <div className="space-y-5">
            <LiveDataBar label="Annual AI Compute Spend (2024)" value={50} maxValue={100} prefix="$" suffix="B+ globally" color="text-red-400" delay={0.5} />
            <LiveDataBar label="Average GPU Utilization in Data Centers" value={20} maxValue={100} suffix="% idle capacity" color="text-amber-400" delay={0.7} />
            <LiveDataBar label="Cloud AI Market Share (AWS, Azure, GCP)" value={65} maxValue={100} suffix="% concentrated" color="text-orange-400" delay={0.9} />
            <LiveDataBar label="Indie Developers with AI Access" value={15} maxValue={100} suffix="% can afford it" color="text-yellow-400" delay={1.1} />
          </div>

          <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 1.5 }}
            className="mt-6 p-4 bg-red-500/10 border border-red-500/30 rounded-xl"
          >
            <p className="text-red-300 text-sm font-medium text-center">
              💡 PYRAX solves ALL of these problems with a single platform
            </p>
          </motion.div>
        </motion.div>
      </div>
    </div>
  );

  // SLIDE: SOLUTION - Three pillars with interactive architecture
  if (id === 'solution') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-pyrax-400 font-semibold text-sm uppercase tracking-wider">The PYRAX Solution</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Three Innovations, One Platform</h2>
      </motion.div>

      {/* Interactive Solution Pillars */}
      <div className="grid lg:grid-cols-3 gap-6 mb-8">
        {[
          {
            icon: ShieldCheckIcon,
            title: 'TriStream Consensus',
            subtitle: 'Maximum Security',
            color: 'pyrax',
            features: ['3 parallel mining streams', 'ASIC + GPU + Validators', 'GHOSTDAG ordering', 'No orphan blocks'],
            metric: '3x',
            metricLabel: 'security vs single-stream'
          },
          {
            icon: CpuChipIcon,
            title: 'Dual-Purpose Mining',
            subtitle: '2x Revenue Potential',
            color: 'amber',
            features: ['Mine blocks for rewards', 'Execute AI jobs', 'Seamless switching', 'KAWPOW ASIC-resistant'],
            metric: '2x',
            metricLabel: 'GPU owner revenue'
          },
          {
            icon: SparklesIcon,
            title: 'Native AI Marketplace',
            subtitle: 'On-Chain Settlement',
            color: 'blue',
            features: ['Foundry model registry', 'Crucible job execution', 'Automatic escrow', 'Multi-node verification'],
            metric: '0%',
            metricLabel: 'middleman fees'
          },
        ].map((pillar, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.15 }}
            whileHover={{ y: -10, scale: 1.02 }}
            onHoverStart={() => setHoveredFeature(pillar.title)}
            onHoverEnd={() => setHoveredFeature(null)}
            className={`bg-gradient-to-b from-${pillar.color}-900/30 to-stone-900/60 border-2 ${hoveredFeature === pillar.title ? `border-${pillar.color}-400` : 'border-stone-700'} rounded-2xl p-6 cursor-pointer transition-all duration-300`}
          >
            <div className={`w-16 h-16 bg-${pillar.color}-500/20 rounded-2xl flex items-center justify-center mb-4 mx-auto`}>
              <pillar.icon className={`h-8 w-8 text-${pillar.color}-400`} />
            </div>
            <h3 className="text-xl font-bold text-white text-center mb-1">{pillar.title}</h3>
            <p className={`text-${pillar.color}-400 text-sm text-center mb-4`}>{pillar.subtitle}</p>
            
            <div className="space-y-2 mb-4">
              {pillar.features.map((feature, j) => (
                <motion.div 
                  key={j}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: 0.5 + j * 0.1 }}
                  className="flex items-center gap-2"
                >
                  <CheckCircleIcon className={`h-4 w-4 text-${pillar.color}-400 flex-shrink-0`} />
                  <span className="text-stone-300 text-sm">{feature}</span>
                </motion.div>
              ))}
            </div>

            <div className={`bg-${pillar.color}-500/10 border border-${pillar.color}-500/30 rounded-xl p-3 text-center`}>
              <span className={`text-3xl font-black text-${pillar.color}-400`}>{pillar.metric}</span>
              <span className="text-stone-400 text-xs block">{pillar.metricLabel}</span>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Architecture Flow */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="bg-stone-800/40 border border-stone-700 rounded-2xl p-6"
      >
        <h3 className="text-lg font-bold text-white text-center mb-4">How It All Connects</h3>
        <div className="flex items-center justify-center gap-2 flex-wrap">
          {['GPU Miner', '→', 'PYRAX Network', '→', 'AI Jobs', '→', '$PYRAX Rewards'].map((step, i) => (
            <motion.span
              key={i}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.8 + i * 0.1 }}
              className={step === '→' ? 'text-pyrax-500 text-2xl' : 'px-4 py-2 bg-stone-700/50 rounded-lg text-white font-medium'}
            >
              {step}
            </motion.span>
          ))}
        </div>
      </motion.div>
    </div>
  );

  // SLIDE: MARKET - Total addressable market with growth projections
  if (id === 'market') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-green-400 font-semibold text-sm uppercase tracking-wider">Market Opportunity</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">$344.5B by 2030</h2>
      </motion.div>

      <div className="grid lg:grid-cols-3 gap-6 mb-6">
        {/* TAM Card */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="bg-gradient-to-br from-green-900/30 to-stone-900/60 border border-green-500/40 rounded-2xl p-6 col-span-1"
        >
          <div className="flex items-center gap-2 mb-4">
            <GlobeAltIcon className="h-6 w-6 text-green-400" />
            <h3 className="text-lg font-bold text-white">AI Cloud Computing</h3>
          </div>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-stone-400">2024</span>
              <span className="text-white font-bold">$62.3B</span>
            </div>
            <div className="h-3 bg-stone-800 rounded-full overflow-hidden">
              <motion.div className="h-full bg-green-500" initial={{ width: 0 }} animate={{ width: '18%' }} transition={{ delay: 0.5, duration: 1 }} />
            </div>
            <div className="flex justify-between items-center">
              <span className="text-stone-400">2030</span>
              <span className="text-green-400 font-black text-2xl">$344.5B</span>
            </div>
            <div className="h-3 bg-stone-800 rounded-full overflow-hidden">
              <motion.div className="h-full bg-gradient-to-r from-green-500 to-green-400" initial={{ width: 0 }} animate={{ width: '100%' }} transition={{ delay: 0.7, duration: 1.2 }} />
            </div>
          </div>
          <div className="mt-4 text-center">
            <span className="text-3xl font-black text-green-400">32.8%</span>
            <span className="text-stone-400 text-sm block">CAGR</span>
          </div>
        </motion.div>

        {/* Market Segments */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="bg-gradient-to-br from-stone-800/60 to-stone-900/60 border border-stone-700 rounded-2xl p-6 col-span-1"
        >
          <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <ChartBarIcon className="h-6 w-6 text-pyrax-400" />
            Market Segments
          </h3>
          <div className="space-y-3">
            {[
              { label: 'AI Training', value: 45, color: 'bg-pyrax-500' },
              { label: 'Inference', value: 30, color: 'bg-amber-500' },
              { label: 'Fine-Tuning', value: 15, color: 'bg-blue-500' },
              { label: 'Embeddings', value: 10, color: 'bg-purple-500' },
            ].map((seg, i) => (
              <div key={i}>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-stone-300">{seg.label}</span>
                  <span className="text-white font-bold">{seg.value}%</span>
                </div>
                <div className="h-2 bg-stone-800 rounded-full overflow-hidden">
                  <motion.div className={`h-full ${seg.color}`} initial={{ width: 0 }} animate={{ width: `${seg.value}%` }} transition={{ delay: 0.5 + i * 0.15, duration: 0.8 }} />
                </div>
              </div>
            ))}
          </div>
        </motion.div>

        {/* Competitor Comparison */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="bg-gradient-to-br from-stone-800/60 to-stone-900/60 border border-stone-700 rounded-2xl p-6 col-span-1"
        >
          <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <ArrowTrendingUpIcon className="h-6 w-6 text-amber-400" />
            Similar Projects
          </h3>
          <div className="space-y-3">
            {[
              { name: 'Render Network', mcap: '$1.5B', growth: '+340%' },
              { name: 'Akash Network', mcap: '$800M', growth: '+280%' },
              { name: 'Golem', mcap: '$400M', growth: '+150%' },
            ].map((comp, i) => (
              <motion.div 
                key={i}
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.6 + i * 0.1 }}
                className="flex items-center justify-between p-3 bg-stone-800/50 rounded-xl"
              >
                <span className="text-white font-medium">{comp.name}</span>
                <div className="text-right">
                  <span className="text-stone-300">{comp.mcap}</span>
                  <span className="text-green-400 text-xs block">{comp.growth} YTD</span>
                </div>
              </motion.div>
            ))}
          </div>
          <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 1 }}
            className="mt-4 p-3 bg-pyrax-500/10 border border-pyrax-500/30 rounded-xl text-center"
          >
            <span className="text-pyrax-300 text-sm font-medium">PYRAX: Only L1 with native AI</span>
          </motion.div>
        </motion.div>
      </div>

      {/* Key Differentiators */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.8 }}
        className="grid grid-cols-2 md:grid-cols-4 gap-3"
      >
        {[
          { label: 'Native L1 Blockchain', icon: CubeTransparentIcon },
          { label: 'Full EVM Compatibility', icon: CommandLineIcon },
          { label: '500K+ TPS via ZK-Rollups', icon: BoltIcon },
          { label: 'Dual-Purpose GPU Mining', icon: CpuChipIcon },
        ].map((diff, i) => (
          <div key={i} className="flex items-center gap-2 p-3 bg-stone-800/40 border border-stone-700 rounded-xl">
            <diff.icon className="h-5 w-5 text-pyrax-400" />
            <span className="text-stone-300 text-sm">{diff.label}</span>
          </div>
        ))}
      </motion.div>
    </div>
  );

  // SLIDE: TECHNOLOGY - Interactive layer architecture
  if (id === 'technology') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-blue-400 font-semibold text-sm uppercase tracking-wider">Technical Architecture</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Three-Layer Scalability</h2>
      </motion.div>

      {/* Interactive Layer Stack */}
      <div className="relative max-w-4xl mx-auto">
        {[
          { 
            layer: 'Layer 3', 
            name: 'ZK-Rollups', 
            tps: '500,000+', 
            desc: 'Application-specific rollups for AI, DeFi, Gaming',
            features: ['STARK/SNARK proofs', 'Decentralized sequencer', '7-day challenge period'],
            color: 'purple',
            icon: BoltIcon
          },
          { 
            layer: 'Layer 2', 
            name: 'EVM Sidechain', 
            tps: '5,000', 
            desc: 'Full Solidity compatibility with Rust/WASM support',
            features: ['2-second blocks', 'Solidity 0.8.x', 'Cross-VM interop'],
            color: 'blue',
            icon: CommandLineIcon
          },
          { 
            layer: 'Layer 1', 
            name: 'TriStream DAG', 
            tps: 'Base Layer', 
            desc: 'GHOSTDAG consensus with three parallel mining streams',
            features: ['BLAKE3 + KAWPOW + ZK-STARK', 'No orphan blocks', 'Instant finality'],
            color: 'pyrax',
            icon: CubeTransparentIcon
          },
        ].map((layer, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.2 }}
            whileHover={{ scale: 1.02, zIndex: 10 }}
            onClick={() => toggleCard(`layer-${i}`)}
            className={`relative bg-gradient-to-r from-${layer.color}-900/40 to-stone-900/60 border-2 border-${layer.color}-500/50 rounded-2xl p-6 mb-4 cursor-pointer transition-all duration-300 hover:border-${layer.color}-400`}
          >
            <div className="flex items-start justify-between">
              <div className="flex items-start gap-4">
                <div className={`w-14 h-14 bg-${layer.color}-500/20 rounded-xl flex items-center justify-center`}>
                  <layer.icon className={`h-7 w-7 text-${layer.color}-400`} />
                </div>
                <div>
                  <div className="flex items-center gap-3 mb-1">
                    <span className={`text-${layer.color}-400 text-sm font-bold`}>{layer.layer}</span>
                    <h3 className="text-xl font-bold text-white">{layer.name}</h3>
                  </div>
                  <p className="text-stone-400 text-sm mb-3">{layer.desc}</p>
                  
                  <AnimatePresence>
                    {expandedCards.includes(`layer-${i}`) && (
                      <motion.div
                        initial={{ opacity: 0, height: 0 }}
                        animate={{ opacity: 1, height: 'auto' }}
                        exit={{ opacity: 0, height: 0 }}
                        className="flex flex-wrap gap-2 mt-3"
                      >
                        {layer.features.map((f, j) => (
                          <span key={j} className={`px-3 py-1 bg-${layer.color}-500/20 text-${layer.color}-300 text-xs rounded-full`}>
                            {f}
                          </span>
                        ))}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              </div>
              <div className="text-right">
                <div className={`text-2xl font-black text-${layer.color}-400`}>{layer.tps}</div>
                <span className="text-stone-500 text-xs">TPS</span>
              </div>
            </div>
            <ChevronDownIcon className={`absolute bottom-3 right-3 h-5 w-5 text-stone-500 transition-transform ${expandedCards.includes(`layer-${i}`) ? 'rotate-180' : ''}`} />
          </motion.div>
        ))}
      </div>

      {/* Tech Specs */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.8 }}
        className="grid grid-cols-2 md:grid-cols-4 gap-3 mt-6"
      >
        {[
          { label: 'Block Time (L2)', value: '2s' },
          { label: 'Finality', value: 'Instant' },
          { label: 'Smart Contracts', value: 'EVM + WASM' },
          { label: 'Signatures', value: 'ECDSA' },
        ].map((spec, i) => (
          <div key={i} className="bg-stone-800/50 border border-stone-700 rounded-xl p-3 text-center">
            <div className="text-pyrax-400 font-bold text-lg">{spec.value}</div>
            <div className="text-stone-500 text-xs">{spec.label}</div>
          </div>
        ))}
      </motion.div>
    </div>
  );

  // SLIDE: TRISTREAM - Deep dive into consensus
  if (id === 'tristream') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-pyrax-400 font-semibold text-sm uppercase tracking-wider">Core Innovation</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">TriStream Consensus</h2>
        <p className="text-stone-400 mt-2">Three parallel mining streams for maximum security & decentralization</p>
      </motion.div>

      {/* Stream Cards */}
      <div className="grid lg:grid-cols-3 gap-6 mb-6">
        {[
          {
            stream: 'Stream A',
            algorithm: 'BLAKE3',
            type: 'ASIC Mining',
            blockTime: '10 seconds',
            reward: '50%',
            color: 'amber',
            icon: ServerStackIcon,
            desc: 'High-throughput base layer security from specialized hardware'
          },
          {
            stream: 'Stream B',
            algorithm: 'KAWPOW',
            type: 'GPU Mining',
            blockTime: '60 seconds',
            reward: '30%',
            color: 'green',
            icon: CpuChipIcon,
            desc: 'ASIC-resistant algorithm enabling fair GPU participation + AI jobs'
          },
          {
            stream: 'Stream C',
            algorithm: 'ZK-STARK',
            type: 'Proof of Stake',
            blockTime: 'Checkpoints',
            reward: '20%',
            color: 'blue',
            icon: ShieldCheckIcon,
            desc: 'Validator finality proofs making transactions irreversible'
          },
        ].map((stream, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.15 }}
            whileHover={{ y: -10 }}
            className={`bg-gradient-to-b from-${stream.color}-900/30 to-stone-900/60 border border-${stream.color}-500/40 rounded-2xl p-6 hover:border-${stream.color}-400 transition-all`}
          >
            <div className={`w-14 h-14 bg-${stream.color}-500/20 rounded-xl flex items-center justify-center mb-4`}>
              <stream.icon className={`h-7 w-7 text-${stream.color}-400`} />
            </div>
            <div className="flex items-center gap-2 mb-2">
              <span className={`px-2 py-1 bg-${stream.color}-500/20 text-${stream.color}-400 text-xs font-bold rounded`}>{stream.stream}</span>
              <span className="text-white font-bold">{stream.algorithm}</span>
            </div>
            <p className="text-stone-400 text-sm mb-4">{stream.desc}</p>
            
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-stone-500">Type</span>
                <span className="text-white">{stream.type}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-stone-500">Block Time</span>
                <span className="text-white">{stream.blockTime}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-stone-500">Reward Share</span>
                <span className={`text-${stream.color}-400 font-bold`}>{stream.reward}</span>
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Why TriStream */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
        className="bg-stone-800/40 border border-stone-700 rounded-2xl p-6"
      >
        <h3 className="text-lg font-bold text-white mb-4 text-center">Why TriStream Matters</h3>
        <div className="grid md:grid-cols-3 gap-4 text-center">
          {[
            { metric: '3x', label: 'More decentralized than single-stream' },
            { metric: '0', label: 'Orphan blocks (GHOSTDAG ordering)' },
            { metric: '100%', label: 'Hardware diversity coverage' },
          ].map((item, i) => (
            <div key={i} className="p-4 bg-stone-900/50 rounded-xl">
              <div className="text-3xl font-black text-pyrax-400">{item.metric}</div>
              <div className="text-stone-400 text-sm">{item.label}</div>
            </div>
          ))}
        </div>
      </motion.div>
    </div>
  );

  // SLIDE: AI PLATFORM - Foundry & Crucible deep dive
  if (id === 'aiPlatform') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-amber-400 font-semibold text-sm uppercase tracking-wider">AI Infrastructure</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Decentralized AI Marketplace</h2>
      </motion.div>

      <div className="grid lg:grid-cols-2 gap-6 mb-6">
        {/* Foundry */}
        <motion.div
          initial={{ opacity: 0, x: -30 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.2 }}
          className="bg-gradient-to-br from-blue-900/30 to-stone-900/60 border border-blue-500/40 rounded-2xl p-6"
        >
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-blue-500/20 rounded-xl flex items-center justify-center">
              <ServerStackIcon className="h-6 w-6 text-blue-400" />
            </div>
            <div>
              <h3 className="text-xl font-bold text-white">Foundry</h3>
              <span className="text-blue-400 text-sm">Model Registry</span>
            </div>
          </div>
          <p className="text-stone-400 text-sm mb-4">On-chain registry for AI models with versioning, benchmarks, and discovery</p>
          
          <div className="space-y-2 mb-4">
            {['Model registration with metadata', 'Semantic versioning (major/minor/patch)', 'Community ratings & benchmarks', 'IPFS/Filecoin storage'].map((f, i) => (
              <motion.div 
                key={i}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.4 + i * 0.1 }}
                className="flex items-center gap-2"
              >
                <CheckCircleIcon className="h-4 w-4 text-blue-400" />
                <span className="text-stone-300 text-sm">{f}</span>
              </motion.div>
            ))}
          </div>

          <div className="bg-blue-500/10 border border-blue-500/30 rounded-xl p-3">
            <div className="text-blue-400 text-sm font-medium">Revenue: 2% platform fee on model usage</div>
          </div>
        </motion.div>

        {/* Crucible */}
        <motion.div
          initial={{ opacity: 0, x: 30 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.3 }}
          className="bg-gradient-to-br from-amber-900/30 to-stone-900/60 border border-amber-500/40 rounded-2xl p-6"
        >
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-amber-500/20 rounded-xl flex items-center justify-center">
              <BoltIcon className="h-6 w-6 text-amber-400" />
            </div>
            <div>
              <h3 className="text-xl font-bold text-white">Crucible</h3>
              <span className="text-amber-400 text-sm">Job Execution Engine</span>
            </div>
          </div>
          <p className="text-stone-400 text-sm mb-4">Submit AI jobs to global GPU network with automatic matching and settlement</p>
          
          <div className="space-y-2 mb-4">
            {['Inference, Training, Fine-Tuning', 'Multi-node verification (3-of-5)', 'ZK proof verification option', 'Automatic escrow release'].map((f, i) => (
              <motion.div 
                key={i}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.5 + i * 0.1 }}
                className="flex items-center gap-2"
              >
                <CheckCircleIcon className="h-4 w-4 text-amber-400" />
                <span className="text-stone-300 text-sm">{f}</span>
              </motion.div>
            ))}
          </div>

          <div className="bg-amber-500/10 border border-amber-500/30 rounded-xl p-3">
            <div className="text-amber-400 text-sm font-medium">Matchmaking: Hardware + reputation + price</div>
          </div>
        </motion.div>
      </div>

      {/* Job Flow */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
        className="bg-stone-800/40 border border-stone-700 rounded-2xl p-6"
      >
        <h3 className="text-lg font-bold text-white mb-4 text-center">AI Job Lifecycle</h3>
        <div className="flex items-center justify-between max-w-3xl mx-auto">
          {[
            { step: '1', label: 'Submit', desc: 'User sends job + escrow' },
            { step: '2', label: 'Match', desc: 'Find optimal GPU' },
            { step: '3', label: 'Execute', desc: 'Provider runs job' },
            { step: '4', label: 'Verify', desc: 'Multi-node check' },
            { step: '5', label: 'Settle', desc: 'Auto payment' },
          ].map((s, i) => (
            <motion.div 
              key={i}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.8 + i * 0.1 }}
              className="text-center flex-1"
            >
              <div className="w-10 h-10 bg-pyrax-500/20 rounded-full flex items-center justify-center mx-auto mb-2">
                <span className="text-pyrax-400 font-bold">{s.step}</span>
              </div>
              <div className="text-white font-medium text-sm">{s.label}</div>
              <div className="text-stone-500 text-xs">{s.desc}</div>
            </motion.div>
          ))}
        </div>
      </motion.div>
    </div>
  );

  // SLIDE: PROVIDERS - GPU provider economics
  if (id === 'providers') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-green-400 font-semibold text-sm uppercase tracking-wider">Provider Economics</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Earn With Your Hardware</h2>
      </motion.div>

      {/* Provider Tiers */}
      <div className="grid lg:grid-cols-3 gap-6 mb-6">
        {[
          {
            tier: 'Hobbyist',
            hardware: '1x RTX 3080+',
            stake: '1,000 PYRAX',
            revenue: '200-500',
            color: 'stone',
            popular: false
          },
          {
            tier: 'Professional',
            hardware: '4x RTX 4090',
            stake: '5,000 PYRAX',
            revenue: '1,500-3,000',
            color: 'pyrax',
            popular: true
          },
          {
            tier: 'Enterprise',
            hardware: '8x A100/H100',
            stake: '25,000 PYRAX',
            revenue: '5,000-15,000',
            color: 'amber',
            popular: false
          },
        ].map((tier, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.15 }}
            whileHover={{ y: -10 }}
            className={`relative bg-gradient-to-b from-${tier.color}-900/30 to-stone-900/60 border-2 ${tier.popular ? 'border-pyrax-400' : 'border-stone-700'} rounded-2xl p-6`}
          >
            {tier.popular && (
              <div className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 bg-pyrax-500 text-white text-xs font-bold rounded-full">
                MOST POPULAR
              </div>
            )}
            <h3 className="text-2xl font-bold text-white text-center mb-4">{tier.tier}</h3>
            
            <div className="space-y-4 mb-6">
              <div className="text-center">
                <div className="text-stone-500 text-sm">Hardware</div>
                <div className="text-white font-medium">{tier.hardware}</div>
              </div>
              <div className="text-center">
                <div className="text-stone-500 text-sm">Minimum Stake</div>
                <div className="text-white font-medium">{tier.stake}</div>
              </div>
              <div className="text-center p-4 bg-green-500/10 border border-green-500/30 rounded-xl">
                <div className="text-stone-400 text-sm">Est. Monthly Revenue</div>
                <div className="text-3xl font-black text-green-400">{tier.revenue}</div>
                <div className="text-green-400 text-sm">PYRAX/month</div>
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Dual Revenue */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
        className="grid md:grid-cols-2 gap-4"
      >
        <div className="bg-stone-800/40 border border-stone-700 rounded-xl p-4">
          <div className="flex items-center gap-3 mb-2">
            <CpuChipIcon className="h-6 w-6 text-green-400" />
            <span className="text-white font-bold">Revenue Stream 1</span>
          </div>
          <p className="text-stone-400 text-sm">Block Mining Rewards (KAWPOW Stream B)</p>
        </div>
        <div className="bg-stone-800/40 border border-stone-700 rounded-xl p-4">
          <div className="flex items-center gap-3 mb-2">
            <SparklesIcon className="h-6 w-6 text-amber-400" />
            <span className="text-white font-bold">Revenue Stream 2</span>
          </div>
          <p className="text-stone-400 text-sm">AI Job Execution Payments (Crucible)</p>
        </div>
      </motion.div>
    </div>
  );

  // SLIDE: TOKENOMICS - Interactive token distribution
  if (id === 'tokenomics') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-6">
        <span className="text-pyrax-400 font-semibold text-sm uppercase tracking-wider">Token Economics</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">$PYRAX Tokenomics</h2>
      </motion.div>

      <div className="grid lg:grid-cols-2 gap-6">
        {/* Token Overview */}
        <motion.div
          initial={{ opacity: 0, x: -30 }}
          animate={{ opacity: 1, x: 0 }}
          className="bg-gradient-to-br from-stone-800/60 to-stone-900/60 border border-stone-700 rounded-2xl p-6"
        >
          <h3 className="text-lg font-bold text-white mb-4">Token Overview</h3>
          <div className="grid grid-cols-2 gap-4 mb-6">
            {[
              { label: 'Name', value: 'PYRAX' },
              { label: 'Symbol', value: '$PYRAX' },
              { label: 'Total Supply', value: '100B' },
              { label: 'Decimals', value: '8' },
            ].map((item, i) => (
              <motion.div 
                key={i}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.3 + i * 0.1 }}
                className="bg-stone-900/50 rounded-xl p-3 text-center"
              >
                <div className="text-stone-500 text-xs">{item.label}</div>
                <div className="text-white font-bold text-lg">{item.value}</div>
              </motion.div>
            ))}
          </div>

          {/* Distribution Bars */}
          <h4 className="text-sm font-bold text-stone-400 mb-3">Distribution</h4>
          <div className="space-y-3">
            {[
              { label: 'Mining Rewards', pct: 35, color: 'bg-pyrax-500' },
              { label: 'Presale', pct: 15, color: 'bg-blue-500' },
              { label: 'BDAG Community', pct: 10, color: 'bg-purple-500' },
              { label: 'Ecosystem Fund', pct: 10, color: 'bg-green-500' },
              { label: 'Liquidity', pct: 10, color: 'bg-amber-500' },
              { label: 'Team & Other', pct: 20, color: 'bg-stone-500' },
            ].map((item, i) => (
              <div key={i}>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-stone-300">{item.label}</span>
                  <span className="text-white font-bold">{item.pct}%</span>
                </div>
                <div className="h-2 bg-stone-800 rounded-full overflow-hidden">
                  <motion.div 
                    className={`h-full ${item.color}`}
                    initial={{ width: 0 }}
                    animate={{ width: `${item.pct}%` }}
                    transition={{ delay: 0.5 + i * 0.1, duration: 0.8 }}
                  />
                </div>
              </div>
            ))}
          </div>
        </motion.div>

        {/* Vesting & Staking */}
        <motion.div
          initial={{ opacity: 0, x: 30 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.2 }}
          className="space-y-4"
        >
          {/* Vesting Schedule */}
          <div className="bg-gradient-to-br from-stone-800/60 to-stone-900/60 border border-stone-700 rounded-2xl p-6">
            <h3 className="text-lg font-bold text-white mb-4">Vesting Schedules</h3>
            <div className="space-y-3 text-sm">
              {[
                { cat: 'Presale', schedule: '30% TGE, 1mo cliff, 12mo vest' },
                { cat: 'Team', schedule: '12mo cliff, 48mo vest' },
                { cat: 'BDAG', schedule: '12mo cliff, 12mo vest' },
                { cat: 'Ecosystem', schedule: 'Milestone-based, DAO approval' },
              ].map((v, i) => (
                <div key={i} className="flex justify-between items-center p-2 bg-stone-900/50 rounded-lg">
                  <span className="text-stone-300">{v.cat}</span>
                  <span className="text-stone-400 text-xs">{v.schedule}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Staking Params */}
          <div className="bg-gradient-to-br from-pyrax-900/30 to-stone-900/60 border border-pyrax-500/40 rounded-2xl p-6">
            <h3 className="text-lg font-bold text-white mb-4">Staking Parameters</h3>
            <div className="grid grid-cols-2 gap-3 text-sm">
              {[
                { label: 'Validator Min', value: '100,000 PYRAX' },
                { label: 'Delegation Min', value: '100 PYRAX' },
                { label: 'Provider Stake', value: '1,000 PYRAX' },
                { label: 'Unbonding', value: '7 days' },
              ].map((s, i) => (
                <div key={i} className="text-center p-3 bg-stone-900/50 rounded-xl">
                  <div className="text-stone-500 text-xs">{s.label}</div>
                  <div className="text-pyrax-400 font-bold">{s.value}</div>
                </div>
              ))}
            </div>
          </div>
        </motion.div>
      </div>
    </div>
  );

  // SLIDE: TRACTION - Development progress & metrics
  if (id === 'traction') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-green-400 font-semibold text-sm uppercase tracking-wider">Development Progress</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Traction & Milestones</h2>
      </motion.div>

      <div className="grid lg:grid-cols-2 gap-6 mb-6">
        {/* Products Complete */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-gradient-to-br from-green-900/30 to-stone-900/60 border border-green-500/40 rounded-2xl p-6"
        >
          <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <CheckCircleIcon className="h-6 w-6 text-green-400" />
            Products Complete
          </h3>
          <div className="grid grid-cols-2 gap-3">
            {[
              { name: 'PYRAX Node', tech: 'Rust' },
              { name: 'GPU Miner', tech: 'C++/CUDA' },
              { name: 'Desktop Wallet', tech: 'Tauri' },
              { name: 'Block Explorer', tech: 'Next.js' },
              { name: 'Wallet Library', tech: 'Rust' },
              { name: 'AI Platform', tech: 'Rust' },
            ].map((p, i) => (
              <motion.div 
                key={i}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.3 + i * 0.1 }}
                className="flex items-center justify-between p-3 bg-stone-900/50 rounded-xl"
              >
                <span className="text-white font-medium text-sm">{p.name}</span>
                <span className="text-stone-500 text-xs">{p.tech}</span>
              </motion.div>
            ))}
          </div>
          <div className="mt-4 text-center">
            <span className="text-4xl font-black text-green-400">6/6</span>
            <span className="text-stone-400 text-sm block">Core Products</span>
          </div>
        </motion.div>

        {/* Metrics Dashboard */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="bg-gradient-to-br from-stone-800/60 to-stone-900/60 border border-stone-700 rounded-2xl p-6"
        >
          <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <ChartBarIcon className="h-6 w-6 text-pyrax-400" />
            Key Metrics
          </h3>
          <div className="space-y-4">
            {[
              { label: 'Q4 2025 Roadmap', value: 100, suffix: '%', color: 'text-green-400' },
              { label: 'Languages Supported', value: 20, suffix: '+', color: 'text-blue-400' },
              { label: 'Core Team Members', value: 8, suffix: '', color: 'text-amber-400' },
              { label: 'GitHub Commits', value: 500, suffix: '+', color: 'text-purple-400' },
            ].map((m, i) => (
              <div key={i} className="flex items-center justify-between p-3 bg-stone-900/50 rounded-xl">
                <span className="text-stone-300">{m.label}</span>
                <span className={`text-2xl font-black ${m.color}`}>
                  <AnimatedCounter value={m.value} suffix={m.suffix} duration={1.5} />
                </span>
              </div>
            ))}
          </div>
        </motion.div>
      </div>

      {/* Timeline */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.5 }}
        className="bg-stone-800/40 border border-stone-700 rounded-2xl p-6"
      >
        <div className="flex items-center justify-between">
          {[
            { phase: 'Q4 2025', label: 'Foundation', status: 'complete' },
            { phase: 'Q1-Q2 2026', label: 'Expansion', status: 'current' },
            { phase: 'Q3 2026', label: 'Mainnet', status: 'upcoming' },
            { phase: 'Q4 2026', label: 'Scale', status: 'upcoming' },
          ].map((p, i) => (
            <div key={i} className="text-center flex-1">
              <div className={`w-8 h-8 mx-auto mb-2 rounded-full flex items-center justify-center ${
                p.status === 'complete' ? 'bg-green-500' : p.status === 'current' ? 'bg-pyrax-500 animate-pulse' : 'bg-stone-700'
              }`}>
                {p.status === 'complete' ? '✓' : p.status === 'current' ? '●' : '○'}
              </div>
              <div className={`font-bold text-sm ${p.status === 'complete' ? 'text-green-400' : p.status === 'current' ? 'text-pyrax-400' : 'text-stone-500'}`}>
                {p.phase}
              </div>
              <div className="text-stone-500 text-xs">{p.label}</div>
            </div>
          ))}
        </div>
      </motion.div>
    </div>
  );

  // SLIDE: ROADMAP - Detailed timeline
  if (id === 'roadmap') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-blue-400 font-semibold text-sm uppercase tracking-wider">Development Timeline</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Roadmap to Mainnet</h2>
      </motion.div>

      <div className="grid lg:grid-cols-2 gap-6">
        {[
          {
            phase: 'Q4 2025',
            title: 'Foundation',
            status: 'complete',
            items: ['Core node with BLAKE3 PoW', 'P2P networking (libp2p)', 'Desktop wallet app', 'GPU miner (CUDA/OpenCL)']
          },
          {
            phase: 'Q1-Q2 2026',
            title: 'Platform Expansion',
            status: 'current',
            items: ['Public testnet launch', 'Block explorer deployment', 'EVM sidechain development', 'AI marketplace beta']
          },
          {
            phase: 'Q3 2026',
            title: 'Mainnet Launch',
            status: 'upcoming',
            items: ['Security audits completion', 'Genesis block generation', 'CEX/DEX listings', 'AI platform production']
          },
          {
            phase: 'Q4 2026',
            title: 'Scale & Growth',
            status: 'upcoming',
            items: ['Enterprise partnerships', 'Cross-chain bridges', 'DAO governance launch', 'Global node expansion']
          },
        ].map((phase, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.15 }}
            whileHover={{ scale: 1.02 }}
            className={`rounded-2xl p-6 border-2 ${
              phase.status === 'complete' ? 'bg-gradient-to-br from-green-900/30 to-stone-900/60 border-green-500/50' :
              phase.status === 'current' ? 'bg-gradient-to-br from-pyrax-900/30 to-stone-900/60 border-pyrax-500/50' :
              'bg-gradient-to-br from-stone-800/60 to-stone-900/60 border-stone-700'
            }`}
          >
            <div className="flex items-center gap-3 mb-4">
              <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
                phase.status === 'complete' ? 'bg-green-500' : phase.status === 'current' ? 'bg-pyrax-500 animate-pulse' : 'bg-stone-700'
              }`}>
                {phase.status === 'complete' ? '✓' : phase.status === 'current' ? '●' : '○'}
              </div>
              <div>
                <span className={`font-bold ${
                  phase.status === 'complete' ? 'text-green-400' : phase.status === 'current' ? 'text-pyrax-400' : 'text-stone-500'
                }`}>{phase.phase}</span>
                <h3 className="text-white font-bold text-lg">{phase.title}</h3>
              </div>
            </div>
            <div className="space-y-2">
              {phase.items.map((item, j) => (
                <motion.div 
                  key={j}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: 0.3 + j * 0.1 }}
                  className="flex items-center gap-2"
                >
                  <CheckCircleIcon className={`h-4 w-4 ${phase.status === 'complete' ? 'text-green-400' : 'text-stone-500'}`} />
                  <span className="text-stone-300 text-sm">{item}</span>
                </motion.div>
              ))}
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );

  // SLIDE: COMPETITIVE - Feature comparison matrix
  if (id === 'competitive') return (
    <div>
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-center mb-8">
        <span className="text-amber-400 font-semibold text-sm uppercase tracking-wider">Competitive Analysis</span>
        <h2 className="text-4xl md:text-5xl font-bold text-white mt-2">Why PYRAX Wins</h2>
      </motion.div>

      {/* Comparison Table */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-stone-800/40 border border-stone-700 rounded-2xl overflow-hidden mb-6"
      >
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-stone-700">
                <th className="text-left p-4 text-stone-400">Feature</th>
                <th className="p-4 text-pyrax-400 font-bold bg-pyrax-500/10">PYRAX</th>
                <th className="p-4 text-stone-400">Render</th>
                <th className="p-4 text-stone-400">Akash</th>
                <th className="p-4 text-stone-400">Ethereum</th>
              </tr>
            </thead>
            <tbody>
              {[
                { feature: 'Native L1 Blockchain', pyrax: true, render: false, akash: false, eth: true },
                { feature: 'AI Compute Marketplace', pyrax: true, render: true, akash: true, eth: false },
                { feature: 'GPU Mining Rewards', pyrax: true, render: false, akash: false, eth: false },
                { feature: 'EVM Compatibility', pyrax: true, render: false, akash: false, eth: true },
                { feature: '500K+ TPS (L2)', pyrax: true, render: false, akash: false, eth: false },
                { feature: 'Multi-Stream Consensus', pyrax: true, render: false, akash: false, eth: false },
                { feature: 'On-Chain Model Registry', pyrax: true, render: false, akash: false, eth: false },
                { feature: 'Dual-Purpose Mining', pyrax: true, render: false, akash: false, eth: false },
              ].map((row, i) => (
                <motion.tr 
                  key={i}
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ delay: 0.3 + i * 0.05 }}
                  className="border-b border-stone-800"
                >
                  <td className="p-4 text-stone-300">{row.feature}</td>
                  <td className="p-4 text-center bg-pyrax-500/5">
                    {row.pyrax ? <CheckCircleIcon className="h-5 w-5 text-green-400 mx-auto" /> : <span className="text-stone-600">—</span>}
                  </td>
                  <td className="p-4 text-center">
                    {row.render ? <CheckCircleIcon className="h-5 w-5 text-green-400 mx-auto" /> : <span className="text-stone-600">—</span>}
                  </td>
                  <td className="p-4 text-center">
                    {row.akash ? <CheckCircleIcon className="h-5 w-5 text-green-400 mx-auto" /> : <span className="text-stone-600">—</span>}
                  </td>
                  <td className="p-4 text-center">
                    {row.eth ? <CheckCircleIcon className="h-5 w-5 text-green-400 mx-auto" /> : <span className="text-stone-600">—</span>}
                  </td>
                </motion.tr>
              ))}
            </tbody>
          </table>
        </div>
      </motion.div>

      {/* Key Differentiators */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
        className="grid md:grid-cols-2 gap-4"
      >
        <div className="bg-gradient-to-r from-pyrax-900/30 to-stone-900/50 border border-pyrax-500/40 rounded-xl p-4">
          <h3 className="text-white font-bold mb-2">🎯 Only L1 with Native AI</h3>
          <p className="text-stone-400 text-sm">Not a layer on top of another chain - purpose-built from ground up</p>
        </div>
        <div className="bg-gradient-to-r from-green-900/30 to-stone-900/50 border border-green-500/40 rounded-xl p-4">
          <h3 className="text-white font-bold mb-2">💰 2x GPU Revenue</h3>
          <p className="text-stone-400 text-sm">Mine blocks AND execute AI jobs with the same hardware</p>
        </div>
      </motion.div>
    </div>
  );

  // SLIDE: CTA - Strong investor close
  if (id === 'cta') return (
    <div className="text-center">
      <motion.div
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        className="mb-6"
      >
        <Image 
          src="/pyrax-brand/pyrax-logo.svg" 
          alt="PYRAX" 
          width={250} 
          height={90} 
          className="h-20 w-auto mx-auto"
        />
      </motion.div>
      
      <motion.h2 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="text-4xl md:text-5xl font-bold text-white mb-4"
      >
        Join the AI x Blockchain Revolution
      </motion.h2>
      
      <motion.p 
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.3 }}
        className="text-xl text-stone-400 mb-8 max-w-2xl mx-auto"
      >
        Be part of the first Layer 1 blockchain with native AI compute integration
      </motion.p>

      {/* Investment Highlights */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-4xl mx-auto mb-8"
      >
        {[
          { value: '$344.5B', label: 'TAM by 2030' },
          { value: '6/6', label: 'Products Complete' },
          { value: 'Q3 2026', label: 'Mainnet Launch' },
          { value: '100B', label: 'Total Supply' },
        ].map((stat, i) => (
          <div key={i} className="bg-stone-800/50 border border-stone-700 rounded-xl p-4">
            <div className="text-pyrax-400 font-black text-2xl">{stat.value}</div>
            <div className="text-stone-500 text-sm">{stat.label}</div>
          </div>
        ))}
      </motion.div>

      {/* CTA Buttons */}
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        className="flex flex-wrap justify-center gap-4 mb-8"
      >
        <Link href={`/${locale}/whitepaper`}>
          <motion.span
            whileHover={{ scale: 1.05, y: -3 }}
            whileTap={{ scale: 0.98 }}
            className="inline-block px-10 py-5 bg-gradient-to-r from-pyrax-500 to-pyrax-600 text-white rounded-xl font-bold text-lg hover:shadow-xl hover:shadow-pyrax-500/30 transition-all"
          >
            Read Whitepaper
          </motion.span>
        </Link>
        <Link href={`/${locale}/technology`}>
          <motion.span
            whileHover={{ scale: 1.05, y: -3 }}
            whileTap={{ scale: 0.98 }}
            className="inline-block px-10 py-5 bg-stone-800 text-white rounded-xl font-bold text-lg hover:bg-stone-700 transition-all"
          >
            Explore Technology
          </motion.span>
        </Link>
      </motion.div>

      {/* Contact */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
        className="text-stone-500 text-sm"
      >
        <p>📧 Contact: investors@pyrax.org</p>
        <p className="mt-2">🌐 pyrax.org • testnet.pyrax.org • explorer.testnet.pyrax.org</p>
      </motion.div>
    </div>
  );

  return null;
}
