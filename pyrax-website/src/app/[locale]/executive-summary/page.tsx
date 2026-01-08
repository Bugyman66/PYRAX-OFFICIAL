'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import { useLocale } from 'next-intl';
import {
  DocumentTextIcon,
  UserGroupIcon,
  RocketLaunchIcon,
  ShieldCheckIcon,
  CpuChipIcon,
  ChartBarIcon,
  ClockIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  BuildingLibraryIcon,
  CurrencyDollarIcon,
  ServerStackIcon,
  SparklesIcon,
  ArrowTrendingUpIcon,
} from '@heroicons/react/24/outline';

export default function ExecutiveSummaryPage() {
  const t = useTranslations('nav');
  const locale = useLocale();

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />

      {/* Header */}
      <section className="relative pt-32 pb-12 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-pyrax-500/5 blur-[128px]" />

        <div className="relative z-10 max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="text-center"
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-pyrax-500/10 border border-pyrax-500/20 mb-6">
              <DocumentTextIcon className="w-5 h-5 text-pyrax-500" />
              <span className="text-pyrax-400 font-medium">Executive Summary</span>
            </div>
            <h1 className="text-4xl sm:text-5xl font-bold text-white mb-4">
              PYRAX Executive Summary
            </h1>
            <div className="flex items-center justify-center gap-4 text-sm text-stone-500">
              <span>Version: 1.0</span>
              <span>•</span>
              <span>Updated: 2026-01-08</span>
              <span>•</span>
              <span className="text-green-400 font-medium">Status: ACTIVE</span>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Content */}
      <section className="py-12">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          
          {/* What & Who */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="grid md:grid-cols-2 gap-6 mb-12"
          >
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
              <h2 className="text-lg font-bold text-pyrax-400 mb-3 flex items-center gap-2">
                <RocketLaunchIcon className="w-5 h-5" />
                What PYRAX is Building
              </h2>
              <p className="text-stone-300 leading-relaxed">
                A high-performance Layer 1 blockchain optimized for AI workloads, plus a decentralized AI compute marketplace that settles training and inference jobs on-chain.
              </p>
            </div>
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
              <h2 className="text-lg font-bold text-pyrax-400 mb-3 flex items-center gap-2">
                <UserGroupIcon className="w-5 h-5" />
                Who It Serves
              </h2>
              <p className="text-stone-300 leading-relaxed">
                AI developers needing affordable GPU compute; GPU miners seeking better ROI; enterprise AI teams needing scalable infrastructure; and dApp builders who want AI-enhanced smart contracts.
              </p>
            </div>
          </motion.div>

          {/* Opportunity and Vision */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="bg-gradient-to-br from-pyrax-900/30 to-stone-900/50 border border-pyrax-500/30 rounded-2xl p-8 mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-4">Opportunity and Vision</h2>
            <p className="text-stone-300 leading-relaxed mb-6">
              PYRAX targets the convergence of blockchain infrastructure and AI compute, positioning the network as an execution and settlement layer for AI jobs with cryptographic security. The plan assumes a <span className="text-pyrax-400 font-semibold">$500B+ combined market opportunity by 2030</span> across blockchain infrastructure, AI/ML cloud services, and decentralized AI.
            </p>
            <div className="grid sm:grid-cols-2 gap-4">
              <div className="bg-stone-800/50 rounded-xl p-4">
                <p className="text-pyrax-400 font-semibold mb-1">Vision</p>
                <p className="text-stone-300 text-sm">The infrastructure layer where blockchain security meets AI compute.</p>
              </div>
              <div className="bg-stone-800/50 rounded-xl p-4">
                <p className="text-pyrax-400 font-semibold mb-1">Mission</p>
                <p className="text-stone-300 text-sm">Democratize access to AI compute through an open GPU marketplace settled on a secure blockchain.</p>
              </div>
            </div>
          </motion.div>

          {/* Key Differentiators */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <SparklesIcon className="w-6 h-6 text-pyrax-400" />
              Key Differentiators
            </h2>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-stone-700">
                    <th className="text-left py-3 px-4 text-stone-400 font-medium">Capability</th>
                    <th className="text-left py-3 px-4 text-pyrax-400 font-medium">PYRAX Approach</th>
                    <th className="text-left py-3 px-4 text-stone-500 font-medium">Typical Alternatives</th>
                  </tr>
                </thead>
                <tbody className="text-sm">
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-white font-medium">Consensus</td>
                    <td className="py-3 px-4 text-stone-300">TriStream DAG (BLAKE3 + KAWPOW + ZK streams)</td>
                    <td className="py-3 px-4 text-stone-500">Single algorithm / single stream</td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-white font-medium">AI Integration</td>
                    <td className="py-3 px-4 text-stone-300">Native job settlement and verification on-chain</td>
                    <td className="py-3 px-4 text-stone-500">External bridges or off-chain settlement</td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-white font-medium">Mining Model</td>
                    <td className="py-3 px-4 text-stone-300">Dual-purpose (network security + compute economy)</td>
                    <td className="py-3 px-4 text-stone-500">Security-focused mining only</td>
                  </tr>
                  <tr>
                    <td className="py-3 px-4 text-white font-medium">Finality</td>
                    <td className="py-3 px-4 text-stone-300">Sub-second finality target (fast stream)</td>
                    <td className="py-3 px-4 text-stone-500">10-60 seconds typical</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </motion.div>

          {/* Current Status */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="bg-green-500/10 border border-green-500/30 rounded-2xl p-6 mb-12"
          >
            <h2 className="text-xl font-bold text-white mb-4 flex items-center gap-2">
              <CheckCircleIcon className="w-6 h-6 text-green-400" />
              Current Status
            </h2>
            <ul className="space-y-2 text-stone-300">
              <li className="flex items-start gap-2">
                <CheckCircleIcon className="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" />
                <span>Reality Gates A-F completed</span>
              </li>
              <li className="flex items-start gap-2">
                <CheckCircleIcon className="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" />
                <span>Core node implementation complete; 10-node testnet validated</span>
              </li>
              <li className="flex items-start gap-2">
                <CheckCircleIcon className="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" />
                <span>AI Services Platform (Crucible + Foundry) in active development</span>
              </li>
            </ul>
          </motion.div>

          {/* Platform Overview */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <ServerStackIcon className="w-6 h-6 text-pyrax-400" />
              Platform Overview
            </h2>
            
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6 mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">Core Network</h3>
              <p className="text-stone-300 leading-relaxed">
                TriStream DAG consensus unifies three block streams into one chain state: a <span className="text-pyrax-400">fast stream</span> for near-real-time finality, a <span className="text-amber-400">GPU-mining stream</span> as the security anchor, and a <span className="text-purple-400">ZK stream</span> for privacy / settlement.
              </p>
            </div>

            <div className="grid md:grid-cols-2 gap-6">
              <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="w-10 h-10 bg-amber-500/20 rounded-xl flex items-center justify-center">
                    <CpuChipIcon className="w-5 h-5 text-amber-400" />
                  </div>
                  <h3 className="text-lg font-semibold text-white">Crucible (Training)</h3>
                </div>
                <p className="text-stone-400 text-sm leading-relaxed">
                  Matches training jobs to GPU providers; results are verified (planned) using ZK proofs; payments settle automatically.
                </p>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="w-10 h-10 bg-blue-500/20 rounded-xl flex items-center justify-center">
                    <SparklesIcon className="w-5 h-5 text-blue-400" />
                  </div>
                  <h3 className="text-lg font-semibold text-white">Foundry (Inference)</h3>
                </div>
                <p className="text-stone-400 text-sm leading-relaxed">
                  Model registry, provider load balancing, per-request billing, and API-based access to hosted models.
                </p>
              </div>
            </div>
          </motion.div>

          {/* Token Economics */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <CurrencyDollarIcon className="w-6 h-6 text-pyrax-400" />
              Token Economics Snapshot
            </h2>
            
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl overflow-hidden">
              <table className="w-full">
                <tbody className="text-sm">
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-stone-400 font-medium w-1/3">Token</td>
                    <td className="py-3 px-4 text-white">PYRAX ($PYRAX)</td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-stone-400 font-medium">Max Supply</td>
                    <td className="py-3 px-4 text-white">100,000,000,000 (100B)</td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-stone-400 font-medium">Decimals</td>
                    <td className="py-3 px-4 text-white">8</td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-stone-400 font-medium">Distribution</td>
                    <td className="py-3 px-4 text-stone-300 text-xs leading-relaxed">
                      Mining 35% • Presale 15% • BDAG Community 10% • Ecosystem 10% • Liquidity 10% • ZK Provers 5% • Marketing 5% • Team 4% • Advisors 3% • Treasury 2% • Reserve 1%
                    </td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-stone-400 font-medium">Utility</td>
                    <td className="py-3 px-4 text-stone-300">Fees, AI job payments, staking, governance, EVM gas, collateral</td>
                  </tr>
                  <tr>
                    <td className="py-3 px-4 text-stone-400 font-medium">Fee Distribution</td>
                    <td className="py-3 px-4 text-stone-300 text-xs">
                      Stream B (GPU Mining) 40% • Stream C (ZK Provers) 30% • Stream A (Validators) 20% • Treasury 10%
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </motion.div>

          {/* Roadmap */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <ClockIcon className="w-6 h-6 text-pyrax-400" />
              Go-to-Market and 2026 Roadmap
            </h2>
            <p className="text-stone-400 mb-6">
              Initial go-to-market targets developers and GPU providers through devrel (hackathons, grants), mining partnerships, and AI community benchmarks.
            </p>
            
            <div className="space-y-4">
              {[
                { phase: 'Foundation', timing: 'Q4 2025', focus: 'Core node, P2P networking, EVM sidechain, desktop wallet, GPU miner', status: 'complete' },
                { phase: 'Platform Expansion', timing: 'Q1-Q2 2026', focus: 'Public testnet, block explorer, bug bounty, AI marketplace beta, mobile wallet', status: 'current' },
                { phase: 'Mainnet Launch', timing: 'Q3 2026', focus: 'Security audits, genesis block, seed nodes, exchange listings, AI platform production', status: 'upcoming' },
                { phase: 'Scale & Growth', timing: 'Q4 2026+', focus: 'Enterprise partnerships, cross-chain bridges (ETH, BSC), DAO governance', status: 'upcoming' },
              ].map((item) => (
                <div key={item.phase} className={`flex items-start gap-4 p-4 rounded-xl ${item.status === 'complete' ? 'bg-green-500/10 border border-green-500/30' : item.status === 'current' ? 'bg-pyrax-500/10 border border-pyrax-500/30' : 'bg-stone-800/50 border border-stone-700'}`}>
                  <div className={`w-20 flex-shrink-0 text-sm font-medium ${item.status === 'complete' ? 'text-green-400' : item.status === 'current' ? 'text-pyrax-400' : 'text-stone-500'}`}>
                    {item.timing}
                  </div>
                  <div>
                    <p className="text-white font-semibold">{item.phase}</p>
                    <p className="text-stone-400 text-sm">{item.focus}</p>
                  </div>
                </div>
              ))}
            </div>
          </motion.div>

          {/* Risks */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <ExclamationTriangleIcon className="w-6 h-6 text-amber-400" />
              Top Risks and Controls
            </h2>
            
            <div className="grid md:grid-cols-3 gap-4">
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4">
                <p className="text-white font-medium mb-2">Smart Contracts</p>
                <p className="text-stone-400 text-sm">Audits, bug bounties, formal verification</p>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4">
                <p className="text-white font-medium mb-2">Network Security</p>
                <p className="text-stone-400 text-sm">Hybrid PoW streams, checkpointing, continuous monitoring</p>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4">
                <p className="text-white font-medium mb-2">Regulatory</p>
                <p className="text-stone-400 text-sm">Compliance-first posture and ongoing legal counsel</p>
              </div>
            </div>
          </motion.div>

          {/* Success Metrics */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <ArrowTrendingUpIcon className="w-6 h-6 text-pyrax-400" />
              Success Metrics (Targets)
            </h2>
            
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-stone-700">
                    <th className="text-left py-3 px-4 text-stone-400 font-medium">Metric</th>
                    <th className="text-center py-3 px-4 text-stone-400 font-medium">Q2 2026</th>
                    <th className="text-center py-3 px-4 text-stone-400 font-medium">Q4 2026</th>
                    <th className="text-center py-3 px-4 text-pyrax-400 font-medium">Q4 2027</th>
                  </tr>
                </thead>
                <tbody className="text-sm">
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-white">Active Nodes</td>
                    <td className="py-3 px-4 text-stone-300 text-center">500</td>
                    <td className="py-3 px-4 text-stone-300 text-center">1,000</td>
                    <td className="py-3 px-4 text-pyrax-400 text-center font-medium">5,000</td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-white">Daily Transactions</td>
                    <td className="py-3 px-4 text-stone-300 text-center">50K</td>
                    <td className="py-3 px-4 text-stone-300 text-center">100K</td>
                    <td className="py-3 px-4 text-pyrax-400 text-center font-medium">1M</td>
                  </tr>
                  <tr className="border-b border-stone-800">
                    <td className="py-3 px-4 text-white">AI Jobs / Month</td>
                    <td className="py-3 px-4 text-stone-300 text-center">5K</td>
                    <td className="py-3 px-4 text-stone-300 text-center">10K</td>
                    <td className="py-3 px-4 text-pyrax-400 text-center font-medium">100K</td>
                  </tr>
                  <tr>
                    <td className="py-3 px-4 text-white">Monthly Revenue</td>
                    <td className="py-3 px-4 text-stone-300 text-center">$50K</td>
                    <td className="py-3 px-4 text-stone-300 text-center">$100K</td>
                    <td className="py-3 px-4 text-pyrax-400 text-center font-medium">$2M</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </motion.div>

          {/* Governance */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="mb-12"
          >
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <BuildingLibraryIcon className="w-6 h-6 text-pyrax-400" />
              Governance Trajectory
            </h2>
            
            <div className="grid md:grid-cols-3 gap-4 mb-6">
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4 text-center">
                <p className="text-pyrax-400 font-bold text-lg">2026</p>
                <p className="text-stone-400 text-sm">Foundation-led with structured community input</p>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4 text-center">
                <p className="text-pyrax-400 font-bold text-lg">2027</p>
                <p className="text-stone-400 text-sm">Hybrid governance with token voting</p>
              </div>
              <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4 text-center">
                <p className="text-pyrax-400 font-bold text-lg">2028+</p>
                <p className="text-stone-400 text-sm">Full DAO with working groups</p>
              </div>
            </div>

            <div className="bg-stone-800/50 rounded-xl p-4">
              <p className="text-stone-400 text-sm">
                <span className="text-white font-medium">Proposal flow:</span> Draft (7d) → Review (7d) → Vote (7d); 5% quorum; 66% approval; time-lock 24h to 7d.
              </p>
            </div>
          </motion.div>

          {/* Footer Note */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center pt-8 border-t border-stone-800"
          >
            <p className="text-stone-500 text-sm">
              For detailed technical documentation, visit <a href="https://docs.testnet.pyrax.org" target="_blank" rel="noopener noreferrer" className="text-pyrax-400 hover:text-pyrax-300">docs.testnet.pyrax.org</a>
            </p>
          </motion.div>

        </div>
      </section>

      <Footer />
    </main>
  );
}
