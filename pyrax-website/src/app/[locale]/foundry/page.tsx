'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Image from 'next/image';
import {
  CubeIcon,
  TagIcon,
  MagnifyingGlassIcon,
  ChartBarIcon,
  ClockIcon,
  ShieldCheckIcon,
  ServerIcon,
  CpuChipIcon,
  ArrowPathIcon,
  DocumentTextIcon,
  StarIcon,
  CheckBadgeIcon,
} from '@heroicons/react/24/outline';

export default function FoundryPage() {
  const t = useTranslations('foundryPage');

  const features = [
    {
      icon: CubeIcon,
      title: t('features.registration.title'),
      description: t('features.registration.description'),
      details: [
        t('features.registration.detail1'),
        t('features.registration.detail2'),
        t('features.registration.detail3'),
        t('features.registration.detail4'),
      ],
    },
    {
      icon: TagIcon,
      title: t('features.versioning.title'),
      description: t('features.versioning.description'),
      details: [
        t('features.versioning.detail1'),
        t('features.versioning.detail2'),
        t('features.versioning.detail3'),
        t('features.versioning.detail4'),
      ],
    },
    {
      icon: MagnifyingGlassIcon,
      title: t('features.discovery.title'),
      description: t('features.discovery.description'),
      details: [
        t('features.discovery.detail1'),
        t('features.discovery.detail2'),
        t('features.discovery.detail3'),
        t('features.discovery.detail4'),
      ],
    },
    {
      icon: ChartBarIcon,
      title: t('features.benchmarks.title'),
      description: t('features.benchmarks.description'),
      details: [
        t('features.benchmarks.detail1'),
        t('features.benchmarks.detail2'),
        t('features.benchmarks.detail3'),
        t('features.benchmarks.detail4'),
      ],
    },
  ];

  const workflow = [
    {
      step: '01',
      title: t('workflow.step1.title'),
      description: t('workflow.step1.description'),
      icon: DocumentTextIcon,
    },
    {
      step: '02',
      title: t('workflow.step2.title'),
      description: t('workflow.step2.description'),
      icon: ShieldCheckIcon,
    },
    {
      step: '03',
      title: t('workflow.step3.title'),
      description: t('workflow.step3.description'),
      icon: ServerIcon,
    },
    {
      step: '04',
      title: t('workflow.step4.title'),
      description: t('workflow.step4.description'),
      icon: StarIcon,
    },
  ];

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-purple-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/3 right-1/4 w-[500px] h-[500px] rounded-full bg-purple-500/10 blur-[128px]" />
        
        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5 }}
            >
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-purple-500/20 border border-purple-500/30 mb-6">
                <CubeIcon className="h-5 w-5 text-purple-400" />
                <span className="text-purple-400 font-medium">{t('hero.badge')}</span>
              </div>
              <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6">
                {t('hero.title')}
              </h1>
              <p className="text-xl text-stone-400 mb-8">
                {t('hero.subtitle')}
              </p>
              <div className="flex flex-wrap gap-4">
                <a href="#workflow" className="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold hover:scale-105 transition-transform">
                  {t('hero.cta1')}
                </a>
                <a href="#features" className="inline-flex items-center gap-2 px-6 py-3 rounded-xl border border-stone-700 text-white font-semibold hover:bg-stone-800 transition-colors">
                  {t('hero.cta2')}
                </a>
              </div>
            </motion.div>
            
            <motion.div
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.2 }}
              className="relative"
            >
              <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
                <div className="flex items-center gap-3 mb-4">
                  <div className="w-3 h-3 rounded-full bg-red-500" />
                  <div className="w-3 h-3 rounded-full bg-yellow-500" />
                  <div className="w-3 h-3 rounded-full bg-green-500" />
                  <span className="ml-2 text-stone-500 text-sm">foundry.pyrax.org</span>
                </div>
                <div className="space-y-3 font-mono text-sm">
                  <div className="text-stone-500"># Register a model</div>
                  <div className="text-purple-400">pyrax foundry register \</div>
                  <div className="text-stone-300 pl-4">--name "llama-3-70b" \</div>
                  <div className="text-stone-300 pl-4">--version "1.0.0" \</div>
                  <div className="text-stone-300 pl-4">--type "text-generation" \</div>
                  <div className="text-stone-300 pl-4">--min-vram "48GB" \</div>
                  <div className="text-stone-300 pl-4">--price "0.001 PYRAX/token"</div>
                  <div className="mt-4 text-green-400">✓ Model registered: 0x7a3b...</div>
                  <div className="text-stone-500">Transaction: 0xf8c2...</div>
                </div>
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* How it works on-chain */}
      <section className="py-20 bg-stone-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">{t('onchain.title')}</h2>
            <p className="text-xl text-stone-400 max-w-3xl mx-auto">{t('onchain.subtitle')}</p>
          </div>
          
          <div className="grid lg:grid-cols-3 gap-8">
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
              <div className="w-12 h-12 rounded-xl bg-purple-500/20 flex items-center justify-center mb-4">
                <DocumentTextIcon className="h-6 w-6 text-purple-400" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">{t('onchain.registry.title')}</h3>
              <p className="text-stone-400 mb-4">{t('onchain.registry.description')}</p>
              <ul className="space-y-2 text-sm text-stone-500">
                <li>• {t('onchain.registry.item1')}</li>
                <li>• {t('onchain.registry.item2')}</li>
                <li>• {t('onchain.registry.item3')}</li>
              </ul>
            </div>
            
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
              <div className="w-12 h-12 rounded-xl bg-blue-500/20 flex items-center justify-center mb-4">
                <ServerIcon className="h-6 w-6 text-blue-400" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">{t('onchain.storage.title')}</h3>
              <p className="text-stone-400 mb-4">{t('onchain.storage.description')}</p>
              <ul className="space-y-2 text-sm text-stone-500">
                <li>• {t('onchain.storage.item1')}</li>
                <li>• {t('onchain.storage.item2')}</li>
                <li>• {t('onchain.storage.item3')}</li>
              </ul>
            </div>
            
            <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6">
              <div className="w-12 h-12 rounded-xl bg-green-500/20 flex items-center justify-center mb-4">
                <CheckBadgeIcon className="h-6 w-6 text-green-400" />
              </div>
              <h3 className="text-xl font-bold text-white mb-3">{t('onchain.verification.title')}</h3>
              <p className="text-stone-400 mb-4">{t('onchain.verification.description')}</p>
              <ul className="space-y-2 text-sm text-stone-500">
                <li>• {t('onchain.verification.item1')}</li>
                <li>• {t('onchain.verification.item2')}</li>
                <li>• {t('onchain.verification.item3')}</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* Features */}
      <section id="features" className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">{t('features.title')}</h2>
            <p className="text-xl text-stone-400">{t('features.subtitle')}</p>
          </div>
          
          <div className="grid md:grid-cols-2 gap-8">
            {features.map((feature, index) => (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6"
              >
                <div className="flex items-start gap-4">
                  <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center flex-shrink-0">
                    <feature.icon className="h-6 w-6 text-white" />
                  </div>
                  <div>
                    <h3 className="text-xl font-bold text-white mb-2">{feature.title}</h3>
                    <p className="text-stone-400 mb-4">{feature.description}</p>
                    <ul className="grid grid-cols-2 gap-2">
                      {feature.details.map((detail, i) => (
                        <li key={i} className="text-sm text-stone-500 flex items-center gap-2">
                          <span className="w-1.5 h-1.5 rounded-full bg-purple-500" />
                          {detail}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Workflow */}
      <section id="workflow" className="py-20 bg-stone-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">{t('workflow.title')}</h2>
            <p className="text-xl text-stone-400">{t('workflow.subtitle')}</p>
          </div>
          
          <div className="grid md:grid-cols-4 gap-6">
            {workflow.map((step, index) => (
              <motion.div
                key={step.step}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="relative"
              >
                {index < workflow.length - 1 && (
                  <div className="hidden md:block absolute top-8 left-1/2 w-full h-0.5 bg-gradient-to-r from-purple-500/50 to-transparent" />
                )}
                <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6 relative">
                  <div className="text-4xl font-bold text-purple-500/30 mb-4">{step.step}</div>
                  <div className="w-12 h-12 rounded-xl bg-purple-500/20 flex items-center justify-center mb-4">
                    <step.icon className="h-6 w-6 text-purple-400" />
                  </div>
                  <h3 className="text-lg font-bold text-white mb-2">{step.title}</h3>
                  <p className="text-sm text-stone-400">{step.description}</p>
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
