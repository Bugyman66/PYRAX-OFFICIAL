'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import {
  ComputerDesktopIcon,
  CpuChipIcon,
  WalletIcon,
  ServerIcon,
  ArrowDownTrayIcon,
  CheckCircleIcon,
  ShieldCheckIcon,
  BoltIcon,
  ChartBarIcon,
  CogIcon,
  CommandLineIcon,
  GlobeAltIcon,
} from '@heroicons/react/24/outline';

export default function DownloadsPage() {
  const t = useTranslations('downloadsPage');

  const features = [
    {
      icon: WalletIcon,
      title: t('features.wallet.title'),
      description: t('features.wallet.description'),
      details: [
        t('features.wallet.detail1'),
        t('features.wallet.detail2'),
        t('features.wallet.detail3'),
        t('features.wallet.detail4'),
      ],
    },
    {
      icon: CpuChipIcon,
      title: t('features.mining.title'),
      description: t('features.mining.description'),
      details: [
        t('features.mining.detail1'),
        t('features.mining.detail2'),
        t('features.mining.detail3'),
        t('features.mining.detail4'),
      ],
    },
    {
      icon: ServerIcon,
      title: t('features.node.title'),
      description: t('features.node.description'),
      details: [
        t('features.node.detail1'),
        t('features.node.detail2'),
        t('features.node.detail3'),
        t('features.node.detail4'),
      ],
    },
    {
      icon: ChartBarIcon,
      title: t('features.dashboard.title'),
      description: t('features.dashboard.description'),
      details: [
        t('features.dashboard.detail1'),
        t('features.dashboard.detail2'),
        t('features.dashboard.detail3'),
        t('features.dashboard.detail4'),
      ],
    },
  ];

  const requirements = [
    { label: t('requirements.os'), value: 'Windows 10+, macOS 12+, Ubuntu 20.04+' },
    { label: t('requirements.ram'), value: '8 GB RAM (16 GB recommended)' },
    { label: t('requirements.storage'), value: '50 GB SSD (for full node)' },
    { label: t('requirements.gpu'), value: 'NVIDIA RTX 20 series+ (for mining)' },
  ];

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/3 right-1/4 w-[500px] h-[500px] rounded-full bg-pyrax-500/10 blur-[128px]" />
        
        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-pyrax-500/20 border border-pyrax-500/30 mb-6">
              <ComputerDesktopIcon className="h-5 w-5 text-pyrax-400" />
              <span className="text-pyrax-400 font-medium">{t('hero.badge')}</span>
            </div>
            <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6">
              {t('hero.title')}
            </h1>
            <p className="text-xl text-stone-400 max-w-3xl mx-auto mb-10">
              {t('hero.subtitle')}
            </p>
            
            {/* Download Buttons */}
            <div className="flex flex-wrap justify-center gap-4 mb-8">
              <a
                href="https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases/download/desktop-v0.1.0/PYRAX.Desktop_0.1.0_x64-setup.exe"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-3 px-8 py-4 rounded-xl bg-gradient-to-r from-pyrax-500 to-orange-500 text-white font-semibold hover:from-pyrax-600 hover:to-orange-600 transition-all shadow-lg shadow-pyrax-500/25 hover:shadow-pyrax-500/40"
              >
                <ArrowDownTrayIcon className="h-6 w-6" />
                {t('hero.downloadWindows')}
              </a>
              <button
                disabled
                className="inline-flex items-center gap-3 px-8 py-4 rounded-xl border border-stone-700 bg-stone-900/50 text-stone-400 font-semibold cursor-not-allowed opacity-70"
              >
                <ArrowDownTrayIcon className="h-6 w-6" />
                {t('hero.downloadMac')} (Coming Soon)
              </button>
              <button
                disabled
                className="inline-flex items-center gap-3 px-8 py-4 rounded-xl border border-stone-700 bg-stone-900/50 text-stone-400 font-semibold cursor-not-allowed opacity-70"
              >
                <ArrowDownTrayIcon className="h-6 w-6" />
                {t('hero.downloadLinux')} (Coming Soon)
              </button>
            </div>
            
            <p className="text-sm text-stone-500">v0.1.0 • Windows 10+ (64-bit)</p>
          </motion.div>
        </div>
      </section>

      {/* Features Grid */}
      <section className="py-20">
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
                  <div className="w-14 h-14 rounded-xl bg-gradient-to-br from-pyrax-500 to-orange-500 flex items-center justify-center flex-shrink-0">
                    <feature.icon className="h-7 w-7 text-white" />
                  </div>
                  <div className="flex-1">
                    <h3 className="text-xl font-bold text-white mb-2">{feature.title}</h3>
                    <p className="text-stone-400 mb-4">{feature.description}</p>
                    <ul className="space-y-2">
                      {feature.details.map((detail, i) => (
                        <li key={i} className="flex items-center gap-2 text-sm text-stone-500">
                          <CheckCircleIcon className="h-4 w-4 text-pyrax-500" />
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

      {/* App Screenshot/Preview */}
      <section className="py-20 bg-stone-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">{t('preview.title')}</h2>
            <p className="text-xl text-stone-400">{t('preview.subtitle')}</p>
          </div>
          
          <div className="bg-stone-900 border border-stone-800 rounded-2xl p-4 max-w-4xl mx-auto">
            <div className="flex items-center gap-2 mb-4">
              <div className="w-3 h-3 rounded-full bg-red-500" />
              <div className="w-3 h-3 rounded-full bg-yellow-500" />
              <div className="w-3 h-3 rounded-full bg-green-500" />
              <span className="ml-3 text-stone-500 text-sm">PYRAX Desktop v1.0.0</span>
            </div>
            <div className="bg-stone-950 rounded-xl p-8 min-h-[400px] flex items-center justify-center">
              <div className="text-center">
                <ComputerDesktopIcon className="h-24 w-24 text-stone-700 mx-auto mb-4" />
                <p className="text-stone-500">{t('preview.placeholder')}</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* System Requirements */}
      <section className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">{t('requirements.title')}</h2>
          </div>
          
          <div className="bg-stone-900 border border-stone-800 rounded-2xl overflow-hidden">
            {requirements.map((req, index) => (
              <div
                key={req.label}
                className={`flex items-center justify-between p-4 ${
                  index !== requirements.length - 1 ? 'border-b border-stone-800' : ''
                }`}
              >
                <span className="text-stone-400">{req.label}</span>
                <span className="text-white font-medium">{req.value}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
