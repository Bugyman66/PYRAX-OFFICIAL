'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Link from 'next/link';
import { useLocale } from 'next-intl';
import {
  CalendarIcon,
  ArrowLeftIcon,
  ShareIcon,
  CheckCircleIcon,
  ClockIcon,
  CpuChipIcon,
  CubeIcon,
  ServerStackIcon,
  ComputerDesktopIcon,
  WrenchScrewdriverIcon,
  ShieldCheckIcon,
  RocketLaunchIcon,
  UserGroupIcon,
  ChartBarIcon,
  BeakerIcon,
} from '@heroicons/react/24/outline';

export default function DevelopmentStatusJanuary2026Page() {
  const t = useTranslations('releasesPage');
  const locale = useLocale();

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />

      {/* Article Header */}
      <section className="relative pt-32 pb-12 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-green-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-green-500/5 blur-[128px]" />

        <div className="relative z-10 max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <Link
              href={`/${locale}/releases/dev`}
              className="inline-flex items-center gap-2 text-stone-400 hover:text-white transition-colors mb-8"
            >
              <ArrowLeftIcon className="w-4 h-4" />
              <span>{t('backToReleases')}</span>
            </Link>

            <div className="flex items-center gap-2 text-sm text-stone-500 mb-4">
              <CalendarIcon className="w-4 h-4" />
              <span>January 7, 2026</span>
              <span className="mx-2">•</span>
              <span className="text-green-500">DEV Release</span>
            </div>

            <h1 className="text-4xl sm:text-5xl font-bold text-white mb-6">
              Development Status Update
            </h1>

            <p className="text-xl text-stone-400">
              A comprehensive look at where PYRAX stands today, explained in plain language for everyone in our community.
            </p>
          </motion.div>
        </div>
      </section>

      {/* Article Content */}
      <section className="py-12">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.article
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="prose prose-invert prose-lg max-w-none"
          >
            {/* Introduction */}
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-8 mb-8">
              <p className="text-stone-300 leading-relaxed">
                To the PYRAX community,
              </p>
              <p className="text-stone-300 leading-relaxed mt-4">
                We&apos;re excited to share our first comprehensive DEV Release, providing full transparency into the technical progress of PYRAX. This update details what has been built, what&apos;s currently in testing, and what remains on our roadmap as we prepare for mainnet.
              </p>
              <p className="text-stone-300 leading-relaxed mt-4">
                <strong className="text-white">The headline:</strong> Our Devnet is preparing to transition into a <strong className="text-green-400">&quot;production-grade environment&quot;</strong> where the entire platform will undergo rigorous testing by the core and internal PYRAX team. Upon confirming stability across all systems, we will migrate to the <strong className="text-green-400">Public Testnet</strong> for community engagement.
              </p>
            </div>

            {/* What does this mean? */}
            <div className="bg-gradient-to-r from-green-500/10 to-transparent border-l-4 border-green-500 rounded-r-xl p-6 my-8">
              <h3 className="text-lg font-semibold text-white mb-2 flex items-center gap-2">
                <BeakerIcon className="w-5 h-5 text-green-500" />
                What does &quot;production-grade environment&quot; mean?
              </h3>
              <p className="text-stone-300 text-base">
                Think of it like this: we&apos;ve been building and testing PYRAX in a &quot;lab&quot; setting (Devnet). Now we&apos;re moving to a &quot;dress rehearsal&quot; where we simulate real-world conditions — lots of users, transactions, and even intentional failures — to make sure everything works perfectly before opening it up to the public.
              </p>
            </div>

            {/* Progress Overview */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <ChartBarIcon className="w-7 h-7 text-green-500" />
              Current Development Status
            </h2>

            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 mb-8">
              <div className="flex items-center justify-between mb-4">
                <span className="text-white font-semibold">Overall Progress</span>
                <span className="text-green-400 font-bold text-2xl">~85%</span>
              </div>
              <div className="w-full bg-stone-800 rounded-full h-4">
                <div className="bg-gradient-to-r from-green-600 to-green-400 h-4 rounded-full" style={{ width: '85%' }}></div>
              </div>
              <p className="text-stone-400 text-sm mt-2">Core infrastructure complete</p>
            </div>

            {/* Status Table */}
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="border-b border-stone-800">
                    <th className="py-3 px-4 text-stone-400 font-medium">Phase</th>
                    <th className="py-3 px-4 text-stone-400 font-medium">Status</th>
                  </tr>
                </thead>
                <tbody className="text-stone-300">
                  {[
                    { phase: 'Engineering Ground Truth', status: 'complete' },
                    { phase: 'Stream A (BLAKE3 Mining)', status: 'complete' },
                    { phase: 'P2P Networking', status: 'complete' },
                    { phase: 'Node Integration', status: 'complete' },
                    { phase: 'Desktop App + Miner', status: 'complete' },
                    { phase: 'AI Platforms', status: 'complete' },
                    { phase: 'EVM Sidechain', status: 'complete' },
                    { phase: 'Dual State Integration', status: 'complete' },
                    { phase: 'Rust/WASM Contracts', status: 'complete' },
                    { phase: 'AI/ML Infrastructure', status: 'complete' },
                    { phase: 'ZK-Rollups', status: 'complete' },
                    { phase: 'Mainnet Infrastructure', status: 'complete' },
                    { phase: 'Post-Launch Services', status: 'complete' },
                    { phase: 'Multi-Environment Deployment', status: 'complete' },
                    { phase: 'External Security Audits', status: 'pending' },
                    { phase: 'Public Testnet Launch', status: 'pending' },
                    { phase: 'Mainnet Genesis', status: 'pending' },
                  ].map((item, index) => (
                    <tr key={index} className="border-b border-stone-800/50">
                      <td className="py-3 px-4">{item.phase}</td>
                      <td className="py-3 px-4">
                        {item.status === 'complete' ? (
                          <span className="inline-flex items-center gap-1 text-green-400">
                            <CheckCircleIcon className="w-4 h-4" />
                            Complete
                          </span>
                        ) : (
                          <span className="inline-flex items-center gap-1 text-yellow-400">
                            <ClockIcon className="w-4 h-4" />
                            Pending
                          </span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* PYRAX Desktop - Special Section */}
            <div className="my-12 bg-gradient-to-br from-pyrax-500/20 via-stone-900/50 to-stone-900/50 border border-pyrax-500/30 rounded-3xl p-8">
              <div className="flex items-center gap-3 mb-6">
                <div className="p-3 bg-pyrax-500/20 rounded-xl">
                  <ComputerDesktopIcon className="w-8 h-8 text-pyrax-400" />
                </div>
                <div>
                  <h2 className="text-2xl font-bold text-white">PYRAX Desktop</h2>
                  <p className="text-pyrax-400">The All-in-One Blockchain Experience</p>
                </div>
              </div>

              <p className="text-stone-300 leading-relaxed mb-6">
                One of the most exciting parts of PYRAX is our <strong className="text-white">Desktop Application</strong> — and it&apos;s fundamentally different from anything else in the blockchain space.
              </p>

              <h3 className="text-xl font-semibold text-white mb-4">Why is PYRAX Desktop Different?</h3>
              
              <p className="text-stone-300 leading-relaxed mb-4">
                Traditionally, if you wanted to participate in a blockchain network, you&apos;d need to:
              </p>
              
              <ul className="list-none space-y-3 mb-6">
                <li className="flex items-start gap-3 text-stone-400">
                  <span className="text-red-400 mt-1">✗</span>
                  <span>Download a <strong className="text-white">separate node software</strong> (often command-line only)</span>
                </li>
                <li className="flex items-start gap-3 text-stone-400">
                  <span className="text-red-400 mt-1">✗</span>
                  <span>Install a <strong className="text-white">different wallet application</strong></span>
                </li>
                <li className="flex items-start gap-3 text-stone-400">
                  <span className="text-red-400 mt-1">✗</span>
                  <span>Set up <strong className="text-white">yet another mining program</strong> with complex configuration</span>
                </li>
                <li className="flex items-start gap-3 text-stone-400">
                  <span className="text-red-400 mt-1">✗</span>
                  <span>Use a <strong className="text-white">web browser</strong> to check the block explorer</span>
                </li>
                <li className="flex items-start gap-3 text-stone-400">
                  <span className="text-red-400 mt-1">✗</span>
                  <span>Figure out how to make all these pieces <strong className="text-white">talk to each other</strong></span>
                </li>
              </ul>

              <p className="text-stone-300 leading-relaxed mb-4">
                <strong className="text-white">PYRAX Desktop changes all of this.</strong> It&apos;s a single, beautiful application that includes:
              </p>

              <div className="grid md:grid-cols-2 gap-4 mb-6">
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <ServerStackIcon className="w-5 h-5 text-green-400" />
                    <span className="font-semibold text-white">Built-in Node</span>
                  </div>
                  <p className="text-stone-400 text-sm">
                    Run a full PYRAX node with one click. No terminal commands, no configuration files. Just click &quot;Start&quot; and you&apos;re supporting the network.
                  </p>
                </div>
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <CubeIcon className="w-5 h-5 text-blue-400" />
                    <span className="font-semibold text-white">Integrated Wallet</span>
                  </div>
                  <p className="text-stone-400 text-sm">
                    Your keys, your coins. Generate wallets, send and receive PYRAX, and manage multiple addresses — all secured with industry-standard encryption.
                  </p>
                </div>
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <CpuChipIcon className="w-5 h-5 text-yellow-400" />
                    <span className="font-semibold text-white">One-Click Mining</span>
                  </div>
                  <p className="text-stone-400 text-sm">
                    Have a GPU? Start mining with a single button. The app automatically detects your graphics card and optimizes settings for you.
                  </p>
                </div>
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <ChartBarIcon className="w-5 h-5 text-purple-400" />
                    <span className="font-semibold text-white">Live Explorer</span>
                  </div>
                  <p className="text-stone-400 text-sm">
                    Watch blocks being created in real-time, search for transactions, and explore the entire blockchain — right from your desktop.
                  </p>
                </div>
              </div>

              <h3 className="text-xl font-semibold text-white mb-4">Built with Modern Technology</h3>
              
              <p className="text-stone-300 leading-relaxed mb-4">
                PYRAX Desktop is built using <strong className="text-white">Tauri</strong>, a modern framework that combines the power of Rust (the same language our node is written in) with a sleek web-based interface. This means:
              </p>

              <ul className="list-none space-y-2 mb-6">
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span><strong className="text-white">Lightweight</strong> — Uses far less memory than Electron-based apps</span>
                </li>
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span><strong className="text-white">Secure</strong> — Rust&apos;s memory safety protects your keys and data</span>
                </li>
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span><strong className="text-white">Cross-Platform</strong> — Works on Windows, macOS, and Linux</span>
                </li>
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span><strong className="text-white">Beautiful</strong> — A modern, intuitive interface anyone can use</span>
                </li>
              </ul>

              <div className="bg-stone-900/80 border border-stone-700 rounded-xl p-4">
                <p className="text-stone-400 text-sm italic">
                  <strong className="text-white">Bottom line:</strong> Whether you&apos;re a complete beginner or an experienced crypto user, PYRAX Desktop lets you participate in the network without needing to be a technical expert. Download, install, and you&apos;re ready to go.
                </p>
              </div>
            </div>

            {/* What We've Built - Simplified */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <WrenchScrewdriverIcon className="w-7 h-7 text-green-500" />
              What We&apos;ve Built (In Plain English)
            </h2>

            {/* TriStream Explained */}
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 mb-6">
              <h3 className="text-xl font-semibold text-white mb-4">🔗 The TriStream System</h3>
              <p className="text-stone-300 leading-relaxed mb-4">
                Most blockchains have one type of &quot;miner&quot; or &quot;validator.&quot; PYRAX has <strong className="text-white">three different ways</strong> to participate in securing the network, running simultaneously:
              </p>
              
              <div className="space-y-4">
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <h4 className="font-semibold text-white mb-2">Stream A: For Specialized Hardware (ASICs)</h4>
                  <p className="text-stone-400 text-sm">
                    Uses the BLAKE3 algorithm with 10-second blocks. This is for users who have dedicated mining machines. Think of it as the &quot;professional lane&quot; for high-performance mining.
                  </p>
                </div>
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <h4 className="font-semibold text-white mb-2">Stream B: For Graphics Cards (GPUs)</h4>
                  <p className="text-stone-400 text-sm">
                    Uses the KAWPOW algorithm with 60-second blocks. This is for anyone with a gaming computer or graphics card. It&apos;s designed to be ASIC-resistant, meaning regular people can compete. <strong className="text-white">Bonus:</strong> Your GPU can also help run AI tasks!
                  </p>
                </div>
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <h4 className="font-semibold text-white mb-2">Stream C: For Stakers (No Hardware Needed)</h4>
                  <p className="text-stone-400 text-sm">
                    Uses zero-knowledge proofs for finality. If you don&apos;t want to mine, you can &quot;stake&quot; your PYRAX tokens to help validate the network and earn rewards. This is like putting your tokens to work for you.
                  </p>
                </div>
              </div>

              <div className="mt-4 p-4 bg-green-500/10 border border-green-500/20 rounded-xl">
                <p className="text-stone-300 text-sm">
                  <strong className="text-green-400">Why three streams?</strong> Diversity makes the network stronger. If one type of hardware becomes unavailable or compromised, the other two keep the network running smoothly. It also means more people can participate in ways that work for them.
                </p>
              </div>
            </div>

            {/* EVM Explained */}
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 mb-6">
              <h3 className="text-xl font-semibold text-white mb-4">⚡ Smart Contracts (Like Ethereum, But Faster)</h3>
              <p className="text-stone-300 leading-relaxed mb-4">
                PYRAX includes an <strong className="text-white">EVM-compatible sidechain</strong>. In plain terms, this means:
              </p>
              <ul className="list-none space-y-2">
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span>Developers can copy their Ethereum apps directly to PYRAX</span>
                </li>
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span>Transactions confirm in <strong className="text-white">2 seconds</strong> instead of 12+ seconds</span>
                </li>
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span>Lower fees because we&apos;re more efficient</span>
                </li>
                <li className="flex items-start gap-3 text-stone-300">
                  <span className="text-green-400 mt-1">✓</span>
                  <span>Works with MetaMask and other Ethereum wallets</span>
                </li>
              </ul>
            </div>

            {/* AI Platform Explained */}
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 mb-6">
              <h3 className="text-xl font-semibold text-white mb-4">🤖 Decentralized AI</h3>
              <p className="text-stone-300 leading-relaxed mb-4">
                This is where PYRAX gets really exciting. We&apos;ve built a complete <strong className="text-white">AI marketplace</strong> right into the blockchain:
              </p>
              
              <div className="grid md:grid-cols-2 gap-4">
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <h4 className="font-semibold text-white mb-2">Foundry (Model Registry)</h4>
                  <p className="text-stone-400 text-sm">
                    A library where anyone can publish AI models. Think of it like an app store, but for AI — and the creators get paid when people use their models.
                  </p>
                </div>
                <div className="bg-stone-800/50 rounded-xl p-4">
                  <h4 className="font-semibold text-white mb-2">Crucible (Job Execution)</h4>
                  <p className="text-stone-400 text-sm">
                    Need to run an AI task but don&apos;t have a powerful computer? Submit a job, and someone else&apos;s GPU will do the work. They get paid in PYRAX, you get your results.
                  </p>
                </div>
              </div>

              <div className="mt-4 p-4 bg-blue-500/10 border border-blue-500/20 rounded-xl">
                <p className="text-stone-300 text-sm">
                  <strong className="text-blue-400">The big picture:</strong> GPU miners don&apos;t just secure the network — they can also rent out their computing power for AI tasks. This creates a second income stream and makes PYRAX mining more profitable than traditional crypto mining.
                </p>
              </div>
            </div>

            {/* Reality Gates */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <ShieldCheckIcon className="w-7 h-7 text-green-500" />
              Quality Checkpoints — All Passed ✅
            </h2>

            <p className="text-stone-300 leading-relaxed mb-6">
              Before we call anything &quot;done,&quot; it has to pass our <strong className="text-white">Reality Gates</strong> — strict tests that prove the system actually works in real conditions, not just in theory.
            </p>

            <div className="grid gap-3">
              {[
                { gate: 'A', desc: 'Two computers can connect and stay synchronized for 24 hours', status: 'PASS' },
                { gate: 'B', desc: 'Sending tokens actually works — send, confirm, receive', status: 'PASS' },
                { gate: 'C', desc: 'Miners can find valid blocks that the network accepts', status: 'PASS' },
                { gate: 'D', desc: '10-computer network survives being split apart and reconnected', status: 'PASS' },
                { gate: 'E', desc: 'A brand new computer can build the software and get identical results', status: 'PASS' },
                { gate: 'F', desc: 'Someone outside our team can follow the docs and join the network', status: 'PASS' },
              ].map((item, index) => (
                <div key={index} className="flex items-center gap-4 bg-stone-900/50 border border-stone-800 rounded-xl p-4">
                  <div className="flex-shrink-0 w-10 h-10 bg-green-500/20 rounded-full flex items-center justify-center">
                    <span className="text-green-400 font-bold">{item.gate}</span>
                  </div>
                  <div className="flex-grow">
                    <p className="text-stone-300">{item.desc}</p>
                  </div>
                  <div className="flex-shrink-0">
                    <span className="inline-flex items-center gap-1 text-green-400 font-semibold">
                      <CheckCircleIcon className="w-5 h-5" />
                      {item.status}
                    </span>
                  </div>
                </div>
              ))}
            </div>

            {/* What's Next */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <RocketLaunchIcon className="w-7 h-7 text-green-500" />
              What&apos;s Coming Next
            </h2>

            <div className="space-y-6">
              <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
                <h3 className="text-lg font-semibold text-white mb-3">📍 Phase 1: Internal Production Testing (Now - February 2026)</h3>
                <ul className="list-none space-y-2 text-stone-400">
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Deploy everything to our cloud servers
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Run thousands of test transactions
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Intentionally break things to find weaknesses
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Fix any issues we discover
                  </li>
                </ul>
              </div>

              <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
                <h3 className="text-lg font-semibold text-white mb-3">📍 Phase 2: Public Testnet (Q1 2026)</h3>
                <ul className="list-none space-y-2 text-stone-400">
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    <strong className="text-white">Community can join!</strong> Download the app and participate
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Free test tokens from our faucet
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Bug bounty program with real rewards
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Full documentation and tutorials
                  </li>
                </ul>
              </div>

              <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
                <h3 className="text-lg font-semibold text-white mb-3">📍 Phase 3: Security Audits (Q1-Q2 2026)</h3>
                <ul className="list-none space-y-2 text-stone-400">
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Professional security firms review our code
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Independent verification of our systems
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Address any findings before mainnet
                  </li>
                </ul>
              </div>

              <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
                <h3 className="text-lg font-semibold text-white mb-3">📍 Phase 4: Mainnet Launch (Q3 2026)</h3>
                <ul className="list-none space-y-2 text-stone-400">
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    <strong className="text-white">The real thing!</strong> Live network with real value
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Exchange listings
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-stone-500">•</span>
                    Full ecosystem launch
                  </li>
                </ul>
              </div>
            </div>

            {/* How to Participate */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <UserGroupIcon className="w-7 h-7 text-green-500" />
              How You Can Participate
            </h2>

            <div className="grid md:grid-cols-2 gap-4 mb-8">
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-6">
                <h3 className="font-semibold text-white mb-3">👨‍💻 Developers</h3>
                <ul className="list-none space-y-2 text-stone-400 text-sm">
                  <li>• Review our documentation at docs.testnet.pyrax.org</li>
                  <li>• Explore the open-source code</li>
                  <li>• Start building your apps now</li>
                </ul>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-6">
                <h3 className="font-semibold text-white mb-3">⛏️ Miners</h3>
                <ul className="list-none space-y-2 text-stone-400 text-sm">
                  <li>• Update your GPU drivers</li>
                  <li>• Join Discord for announcements</li>
                  <li>• Get ready for testnet mining</li>
                </ul>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-6">
                <h3 className="font-semibold text-white mb-3">🏛️ Validators</h3>
                <ul className="list-none space-y-2 text-stone-400 text-sm">
                  <li>• Learn about staking requirements</li>
                  <li>• Prepare your server infrastructure</li>
                  <li>• Study the validator documentation</li>
                </ul>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-6">
                <h3 className="font-semibold text-white mb-3">🌍 Community</h3>
                <ul className="list-none space-y-2 text-stone-400 text-sm">
                  <li>• Follow us on social media</li>
                  <li>• Join Discord for discussions</li>
                  <li>• Share feedback and suggestions</li>
                </ul>
              </div>
            </div>

            {/* Timeline */}
            <div className="bg-gradient-to-r from-pyrax-500/20 to-transparent border border-pyrax-500/30 rounded-2xl p-6 mb-8">
              <h3 className="text-xl font-semibold text-white mb-4">📅 Timeline Summary</h3>
              <div className="space-y-3">
                <div className="flex items-center gap-4">
                  <span className="text-pyrax-400 font-mono text-sm w-32">Jan-Feb 2026</span>
                  <span className="text-stone-300">Internal Production Testing</span>
                </div>
                <div className="flex items-center gap-4">
                  <span className="text-pyrax-400 font-mono text-sm w-32">Q1 2026</span>
                  <span className="text-stone-300">Public Testnet Launch</span>
                </div>
                <div className="flex items-center gap-4">
                  <span className="text-pyrax-400 font-mono text-sm w-32">Q1-Q2 2026</span>
                  <span className="text-stone-300">External Security Audits</span>
                </div>
                <div className="flex items-center gap-4">
                  <span className="text-pyrax-400 font-mono text-sm w-32">Q3 2026</span>
                  <span className="text-stone-300 font-semibold">Mainnet Launch 🚀</span>
                </div>
              </div>
              <p className="text-stone-500 text-sm mt-4">
                Note: These are estimates. We prioritize security and stability over speed.
              </p>
            </div>

            {/* Disclaimer */}
            <div className="bg-stone-800/30 border border-stone-700 rounded-xl p-6 my-8 text-sm">
              <h3 className="text-base font-semibold text-stone-400 mb-2">Disclaimer</h3>
              <p className="text-stone-500">
                This document is for informational purposes only. Development timelines and features are subject to change. This is not financial advice. Always conduct your own research before participating in any blockchain project.
              </p>
            </div>

            {/* Signatures */}
            <div className="mt-12 pt-8 border-t border-stone-800">
              <p className="text-stone-300 mb-8">Sincerely,<br />The PYRAX Development Team</p>
              
              <div className="grid md:grid-cols-3 gap-6">
                <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4">
                  <p className="font-bold text-white">Shawn Wilson</p>
                  <p className="text-stone-400 text-sm">President & Lead Developer</p>
                </div>
                <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4">
                  <p className="font-bold text-white">Gabriel Mascioli</p>
                  <p className="text-stone-400 text-sm">Senior Vice President</p>
                </div>
                <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4">
                  <p className="font-bold text-white">Tekky Natale</p>
                  <p className="text-stone-400 text-sm">Chief Financial Officer</p>
                </div>
              </div>
            </div>
          </motion.article>

          {/* Share */}
          <div className="mt-12 pt-8 border-t border-stone-800 flex items-center justify-between">
            <Link
              href={`/${locale}/releases/dev`}
              className="inline-flex items-center gap-2 text-stone-400 hover:text-white transition-colors"
            >
              <ArrowLeftIcon className="w-4 h-4" />
              <span>{t('backToReleases')}</span>
            </Link>
            <button
              onClick={() => {
                if (navigator.share) {
                  navigator.share({
                    title: 'PYRAX - Development Status Update January 2026',
                    url: window.location.href,
                  });
                } else {
                  navigator.clipboard.writeText(window.location.href);
                }
              }}
              className="inline-flex items-center gap-2 px-4 py-2 bg-stone-800 text-stone-300 rounded-lg hover:bg-stone-700 hover:text-white transition-colors"
            >
              <ShareIcon className="w-4 h-4" />
              <span>{t('share')}</span>
            </button>
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
