'use client';

import { useState } from 'react';
import { useTranslations, useLocale } from 'next-intl';
import { motion, AnimatePresence } from 'framer-motion';
import Link from 'next/link';
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  HomeIcon,
} from '@heroicons/react/24/outline';

const slideIds = ['title', 'problem', 'solution', 'market', 'technology', 'aiPlatform', 'tokenomics', 'roadmap', 'ecosystem', 'cta'];

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
          <Link href={`/${locale}`} className="flex items-center gap-2 text-stone-400 hover:text-white">
            <HomeIcon className="h-5 w-5" />
            <span className="text-sm">{t('backToHome')}</span>
          </Link>
          <span className="text-pyrax-500 font-bold">PYRAX {t('title')}</span>
          <span className="text-stone-500">{current + 1}/{slideIds.length}</span>
        </div>
      </div>

      <div className="fixed top-[53px] left-0 right-0 z-40 h-1 bg-stone-800">
        <motion.div className="h-full bg-pyrax-500" animate={{ width: `${((current + 1) / slideIds.length) * 100}%` }} />
      </div>

      <div className="pt-24 pb-28 px-4 min-h-screen flex items-center justify-center">
        <AnimatePresence mode="wait">
          <motion.div
            key={current}
            initial={{ opacity: 0, x: 50 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -50 }}
            className="max-w-5xl w-full"
          >
            <Slide id={slideIds[current]} t={t} locale={locale} />
          </motion.div>
        </AnimatePresence>
      </div>

      <div className="fixed bottom-0 left-0 right-0 z-50 bg-stone-950/80 backdrop-blur-xl border-t border-stone-800">
        <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
          <button onClick={prev} disabled={current === 0} className={`flex items-center gap-2 px-6 py-3 rounded-xl ${current === 0 ? 'bg-stone-800/50 text-stone-600' : 'bg-stone-800 text-white hover:bg-stone-700'}`}>
            <ChevronLeftIcon className="h-5 w-5" />{t('previous')}
          </button>
          <div className="flex gap-1">
            {slideIds.map((_, i) => (
              <button key={i} onClick={() => setCurrent(i)} className={`w-2 h-2 rounded-full ${current === i ? 'bg-pyrax-500 w-4' : 'bg-stone-600'}`} />
            ))}
          </div>
          <button onClick={next} disabled={current === slideIds.length - 1} className={`flex items-center gap-2 px-6 py-3 rounded-xl ${current === slideIds.length - 1 ? 'bg-stone-800/50 text-stone-600' : 'bg-pyrax-500 text-white hover:bg-pyrax-600'}`}>
            {t('next')}<ChevronRightIcon className="h-5 w-5" />
          </button>
        </div>
      </div>
    </div>
  );
}

function Slide({ id, t, locale }: { id: string; t: any; locale: string }) {
  if (id === 'title') return (
    <div className="text-center">
      <div className="inline-flex items-center gap-2 px-4 py-2 bg-pyrax-500/20 border border-pyrax-500/30 rounded-full mb-6">
        <span className="w-2 h-2 bg-pyrax-500 rounded-full animate-pulse" />
        <span className="text-pyrax-400 text-sm">{t('slides.title.badge')}</span>
      </div>
      <h1 className="text-6xl md:text-8xl font-black mb-4">
        <span className="bg-gradient-to-r from-pyrax-400 to-pyrax-600 bg-clip-text text-transparent">PYRAX</span>
      </h1>
      <p className="text-2xl text-stone-300 mb-8">{t('slides.title.tagline')}</p>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl mx-auto">
        {['l1', 'ai', 'tristream', 'mainnet'].map((k) => (
          <div key={k} className="bg-stone-800/50 border border-stone-700 rounded-xl p-4">
            <div className="text-pyrax-500 font-bold text-lg">{t(`slides.title.stats.${k}.value`)}</div>
            <div className="text-stone-400 text-sm">{t(`slides.title.stats.${k}.label`)}</div>
          </div>
        ))}
      </div>
    </div>
  );

  if (id === 'problem') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.problem.title')}</h2>
      <div className="grid md:grid-cols-2 gap-6">
        {['expensive', 'centralized', 'wasted', 'slow'].map((p) => (
          <div key={p} className="bg-red-900/20 border border-red-900/30 rounded-2xl p-6">
            <h3 className="text-xl font-bold text-white mb-2">{t(`slides.problem.items.${p}.title`)}</h3>
            <p className="text-stone-400">{t(`slides.problem.items.${p}.desc`)}</p>
          </div>
        ))}
      </div>
    </div>
  );

  if (id === 'solution') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.solution.title')}</h2>
      <div className="space-y-6">
        {['tristream', 'dualPurpose', 'marketplace'].map((s, i) => (
          <div key={s} className="bg-pyrax-900/30 border border-pyrax-500/30 rounded-2xl p-6 flex gap-4">
            <div className="w-12 h-12 bg-pyrax-500/20 rounded-xl flex items-center justify-center flex-shrink-0">
              <span className="text-pyrax-400 font-bold text-xl">{i + 1}</span>
            </div>
            <div>
              <h3 className="text-2xl font-bold text-white mb-2">{t(`slides.solution.items.${s}.title`)}</h3>
              <p className="text-stone-400">{t(`slides.solution.items.${s}.desc`)}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  if (id === 'market') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.market.title')}</h2>
      <div className="grid md:grid-cols-2 gap-8">
        <div className="bg-stone-800/50 border border-stone-700 rounded-2xl p-6">
          <h3 className="text-xl font-bold text-white mb-4">{t('slides.market.aiCompute')}</h3>
          <div className="text-3xl font-bold text-pyrax-400 mb-2">$344.5B</div>
          <div className="text-stone-400">{t('slides.market.by2030')}</div>
          <div className="text-pyrax-500 mt-2">32.8% CAGR</div>
        </div>
        <div className="bg-stone-800/50 border border-stone-700 rounded-2xl p-6">
          <h3 className="text-xl font-bold text-white mb-4">{t('slides.market.advantage')}</h3>
          <ul className="space-y-2 text-stone-400">
            <li>• {t('slides.market.adv1')}</li>
            <li>• {t('slides.market.adv2')}</li>
            <li>• {t('slides.market.adv3')}</li>
          </ul>
        </div>
      </div>
    </div>
  );

  if (id === 'technology') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.technology.title')}</h2>
      <div className="space-y-4">
        {[
          { layer: 'L3', name: 'ZK-Rollups', tps: '500K+ TPS', color: 'purple' },
          { layer: 'L2', name: 'EVM Sidechain', tps: '1K-5K TPS', color: 'blue' },
          { layer: 'L1', name: 'TriStream DAG', tps: 'GHOSTDAG', color: 'pyrax' },
        ].map((l) => (
          <div key={l.layer} className={`rounded-2xl p-6 border bg-${l.color}-900/30 border-${l.color}-500/30`}>
            <div className="flex justify-between items-center">
              <div>
                <span className="text-stone-500">{l.layer}</span>
                <h3 className="text-xl font-bold text-white">{l.name}</h3>
              </div>
              <div className="text-pyrax-400 font-bold">{l.tps}</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  if (id === 'aiPlatform') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.aiPlatform.title')}</h2>
      <div className="grid md:grid-cols-2 gap-6">
        <div className="bg-blue-900/20 border border-blue-500/30 rounded-2xl p-6">
          <h3 className="text-2xl font-bold text-white mb-2">Foundry</h3>
          <p className="text-stone-400 mb-4">{t('slides.aiPlatform.foundry')}</p>
          <div className="text-blue-400 text-sm">{t('slides.aiPlatform.modelRegistry')}</div>
        </div>
        <div className="bg-amber-900/20 border border-amber-500/30 rounded-2xl p-6">
          <h3 className="text-2xl font-bold text-white mb-2">Crucible</h3>
          <p className="text-stone-400 mb-4">{t('slides.aiPlatform.crucible')}</p>
          <div className="text-amber-400 text-sm">{t('slides.aiPlatform.jobExecution')}</div>
        </div>
      </div>
    </div>
  );

  if (id === 'tokenomics') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.tokenomics.title')}</h2>
      <div className="bg-stone-800/50 border border-stone-700 rounded-2xl p-6 mb-6">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
          <div><div className="text-stone-500 text-sm">{t('slides.tokenomics.name')}</div><div className="text-white font-bold">PYRAX</div></div>
          <div><div className="text-stone-500 text-sm">{t('slides.tokenomics.symbol')}</div><div className="text-white font-bold">$PYRAX</div></div>
          <div><div className="text-stone-500 text-sm">{t('slides.tokenomics.supply')}</div><div className="text-white font-bold">100B</div></div>
          <div><div className="text-stone-500 text-sm">{t('slides.tokenomics.decimals')}</div><div className="text-white font-bold">8</div></div>
        </div>
      </div>
      <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
        {[{ k: 'mining', p: '35%' }, { k: 'presale', p: '15%' }, { k: 'ecosystem', p: '10%' }, { k: 'liquidity', p: '10%' }, { k: 'bdag', p: '10%' }, { k: 'other', p: '20%' }].map((d) => (
          <div key={d.k} className="bg-stone-900/50 rounded-xl p-3 flex justify-between">
            <span className="text-stone-400">{t(`slides.tokenomics.${d.k}`)}</span>
            <span className="text-pyrax-400 font-bold">{d.p}</span>
          </div>
        ))}
      </div>
    </div>
  );

  if (id === 'roadmap') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.roadmap.title')}</h2>
      <div className="space-y-4">
        {[
          { q: 'Q4 2025', title: t('slides.roadmap.q4_2025'), done: true },
          { q: 'Q1-Q2 2026', title: t('slides.roadmap.q1q2_2026'), done: false },
          { q: 'Q3 2026', title: t('slides.roadmap.q3_2026'), done: false },
          { q: 'Q4 2026', title: t('slides.roadmap.q4_2026'), done: false },
        ].map((m) => (
          <div key={m.q} className={`rounded-xl p-4 border ${m.done ? 'bg-green-900/20 border-green-500/30' : 'bg-stone-800/50 border-stone-700'}`}>
            <div className="flex items-center gap-3">
              {m.done && <span className="px-2 py-1 bg-green-500/20 text-green-400 text-xs font-bold rounded">✓</span>}
              <span className="text-pyrax-400 font-bold">{m.q}</span>
              <span className="text-white">{m.title}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  if (id === 'ecosystem') return (
    <div>
      <h2 className="text-4xl font-bold text-white text-center mb-8">{t('slides.ecosystem.title')}</h2>
      <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
        {['node', 'miner', 'desktop', 'explorer', 'wallet', 'ai'].map((p) => (
          <div key={p} className="bg-stone-800/50 border border-stone-700 rounded-xl p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-white font-bold">{t(`slides.ecosystem.${p}.name`)}</span>
              <span className="text-green-400 text-xs">✓</span>
            </div>
            <p className="text-stone-500 text-sm">{t(`slides.ecosystem.${p}.tech`)}</p>
          </div>
        ))}
      </div>
    </div>
  );

  if (id === 'cta') return (
    <div className="text-center">
      <h2 className="text-4xl md:text-5xl font-bold text-white mb-6">{t('slides.cta.title')}</h2>
      <p className="text-xl text-stone-400 mb-8 max-w-2xl mx-auto">{t('slides.cta.subtitle')}</p>
      <div className="flex flex-wrap justify-center gap-4">
        <Link href={`/${locale}/whitepaper`} className="px-8 py-4 bg-pyrax-500 text-white rounded-xl font-bold hover:bg-pyrax-600">{t('slides.cta.whitepaper')}</Link>
        <Link href={`/${locale}/technology`} className="px-8 py-4 bg-stone-800 text-white rounded-xl font-bold hover:bg-stone-700">{t('slides.cta.technology')}</Link>
      </div>
    </div>
  );

  return null;
}
