'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import {
  ServerIcon,
  CpuChipIcon,
  ComputerDesktopIcon,
  MagnifyingGlassIcon,
  WalletIcon,
  SparklesIcon,
} from '@heroicons/react/24/outline';

export default function Ecosystem() {
  const t = useTranslations('ecosystem');

  const products = [
    {
      icon: ServerIcon,
      title: t('products.node.title'),
      description: t('products.node.description'),
      language: t('products.node.language'),
      status: t('products.node.status'),
      color: 'from-pyrax-500 to-orange-500',
      href: 'https://github.com/pyrax-official/pyrax-node',
    },
    {
      icon: CpuChipIcon,
      title: t('products.miner.title'),
      description: t('products.miner.description'),
      language: t('products.miner.language'),
      status: t('products.miner.status'),
      color: 'from-green-500 to-emerald-500',
      href: 'https://github.com/pyrax-official/pyrax-miner',
    },
    {
      icon: ComputerDesktopIcon,
      title: t('products.desktop.title'),
      description: t('products.desktop.description'),
      language: t('products.desktop.language'),
      status: t('products.desktop.status'),
      color: 'from-blue-500 to-cyan-500',
      href: 'https://github.com/pyrax-official/pyrax-desktop',
    },
    {
      icon: MagnifyingGlassIcon,
      title: t('products.explorer.title'),
      description: t('products.explorer.description'),
      language: t('products.explorer.language'),
      status: t('products.explorer.status'),
      color: 'from-purple-500 to-pink-500',
      href: 'https://explorer.testnet.pyrax.org',
    },
    {
      icon: WalletIcon,
      title: t('products.wallet.title'),
      description: t('products.wallet.description'),
      language: t('products.wallet.language'),
      status: t('products.wallet.status'),
      color: 'from-amber-500 to-yellow-500',
      href: 'https://github.com/pyrax-official/pyrax-wallet',
    },
    {
      icon: SparklesIcon,
      title: t('products.ai.title'),
      description: t('products.ai.description'),
      language: t('products.ai.language'),
      status: t('products.ai.status'),
      color: 'from-rose-500 to-red-500',
      href: 'https://github.com/pyrax-official/pyrax-ai',
    },
  ];

  return (
    <section id="ecosystem" className="relative py-24 overflow-hidden">
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

        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {products.map((product, index) => (
            <motion.a
              key={product.title}
              href={product.href}
              target="_blank"
              rel="noopener noreferrer"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
              className="group relative rounded-2xl bg-stone-900/50 border border-stone-800 p-6 hover:border-stone-700 transition-all"
            >
              {/* Glow */}
              <div className={`absolute inset-0 bg-gradient-to-r ${product.color} rounded-2xl blur-xl opacity-0 group-hover:opacity-10 transition-opacity`} />
              
              <div className="relative">
                {/* Icon */}
                <div className={`inline-flex items-center justify-center w-12 h-12 rounded-xl bg-gradient-to-r ${product.color} mb-4`}>
                  <product.icon className="h-6 w-6 text-white" />
                </div>

                {/* Title & Status */}
                <div className="flex items-center justify-between mb-2">
                  <h3 className="text-xl font-semibold text-white group-hover:text-pyrax-400 transition-colors">
                    {product.title}
                  </h3>
                  <span className="text-xs px-2 py-1 rounded-full bg-green-500/10 text-green-400 border border-green-500/20">
                    {product.status}
                  </span>
                </div>

                {/* Description */}
                <p className="text-stone-400 text-sm mb-4">
                  {product.description}
                </p>

                {/* Language */}
                <div className="flex items-center gap-2">
                  <span className="text-xs text-stone-500">Built with</span>
                  <span className="text-xs font-mono text-stone-300 px-2 py-0.5 rounded bg-stone-800">
                    {product.language}
                  </span>
                </div>
              </div>
            </motion.a>
          ))}
        </div>
      </div>
    </section>
  );
}
