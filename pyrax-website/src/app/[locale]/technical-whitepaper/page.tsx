'use client';

import { useTranslations } from 'next-intl';
import { motion } from 'framer-motion';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import Link from 'next/link';
import { 
  DocumentTextIcon,
  CpuChipIcon,
  CubeIcon,
  ShieldCheckIcon,
  ServerStackIcon,
  CodeBracketIcon,
  CircleStackIcon,
  BoltIcon,
  LockClosedIcon,
  ChartBarIcon,
  ArrowRightIcon,
} from '@heroicons/react/24/outline';

export default function TechnicalWhitepaperPage() {
  const t = useTranslations('technicalWhitepaper');

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />
      
      {/* Hero */}
      <section className="relative pt-32 pb-16 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-blue-500/5 via-transparent to-transparent" />
        <div className="relative z-10 max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}>
            <span className="inline-block px-4 py-2 rounded-full bg-blue-500/10 text-blue-400 text-sm font-medium mb-6">
              {t('badge')}
            </span>
            <h1 className="text-4xl sm:text-5xl font-bold text-white mb-4">{t('title')}</h1>
            <p className="text-lg text-stone-400 mb-6 max-w-2xl mx-auto">{t('subtitle')}</p>
            <div className="flex flex-wrap justify-center gap-4">
              <Link href="/whitepaper" className="px-5 py-2.5 bg-stone-800 text-white font-medium rounded-lg hover:bg-stone-700 transition-all border border-stone-700 text-sm">
                {t('simpleVersion')}
              </Link>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Version Info */}
      <section className="py-8 border-y border-stone-800 bg-stone-900/30">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6 text-center">
            <div><div className="text-stone-500 text-xs uppercase mb-1">{t('version')}</div><div className="text-white font-mono">1.0.0</div></div>
            <div><div className="text-stone-500 text-xs uppercase mb-1">{t('date')}</div><div className="text-white font-mono">2026-01</div></div>
            <div><div className="text-stone-500 text-xs uppercase mb-1">{t('status')}</div><div className="text-green-400 font-mono">Canonical</div></div>
            <div><div className="text-stone-500 text-xs uppercase mb-1">{t('network')}</div><div className="text-white font-mono">PYRAX L1</div></div>
          </div>
        </div>
      </section>

      {/* TOC */}
      <section className="py-12">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-lg font-semibold text-white mb-6">{t('toc.title')}</h2>
          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-3 text-sm">
            {['abstract','architecture','consensus','ghostdag','blockStructure','cryptography','smartContracts','aiPlatform','tokenomics','security','references'].map((id) => (
              <a key={id} href={`#${id}`} className="flex items-center gap-2 text-stone-400 hover:text-blue-400 transition-colors py-1">
                <ArrowRightIcon className="w-3 h-3" />{t(`toc.${id}`)}
              </a>
            ))}
          </div>
        </div>
      </section>

      {/* Abstract */}
      <section id="abstract" className="py-16 border-t border-stone-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={DocumentTextIcon} color="blue" label={t('sections.abstract.label')} title={t('sections.abstract.title')} />
          <div className="prose prose-invert max-w-none">
            <p className="text-stone-300 leading-relaxed mb-4">{t('sections.abstract.p1')}</p>
            <p className="text-stone-300 leading-relaxed mb-4">{t('sections.abstract.p2')}</p>
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6 my-6 font-mono text-sm">
              <div className="grid grid-cols-2 gap-4">
                <div><span className="text-stone-500">Ticker:</span> <span className="text-white">$PYRAX</span></div>
                <div><span className="text-stone-500">Consensus:</span> <span className="text-white">TriStream DAG</span></div>
                <div><span className="text-stone-500">L2 TPS:</span> <span className="text-white">500,000+</span></div>
                <div><span className="text-stone-500">Block Time:</span> <span className="text-white">10s / 60s</span></div>
                <div><span className="text-stone-500">Supply:</span> <span className="text-white">100,000,000,000</span></div>
                <div><span className="text-stone-500">Algorithms:</span> <span className="text-white">BLAKE3, KAWPOW, ZK-STARK</span></div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Architecture */}
      <section id="architecture" className="py-16 border-t border-stone-800 bg-stone-900/20">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={CubeIcon} color="purple" label={t('sections.architecture.label')} title={t('sections.architecture.title')} />
          <p className="text-stone-300 mb-8">{t('sections.architecture.intro')}</p>
          <div className="space-y-4">
            <LayerCard layer="3" title={t('sections.architecture.l3.title')} desc={t('sections.architecture.l3.desc')} specs={t('sections.architecture.l3.specs')} color="purple" />
            <LayerCard layer="2" title={t('sections.architecture.l2.title')} desc={t('sections.architecture.l2.desc')} specs={t('sections.architecture.l2.specs')} color="blue" />
            <LayerCard layer="1" title={t('sections.architecture.l1.title')} desc={t('sections.architecture.l1.desc')} specs={t('sections.architecture.l1.specs')} color="pyrax" />
          </div>
        </div>
      </section>

      {/* Consensus */}
      <section id="consensus" className="py-16 border-t border-stone-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={CpuChipIcon} color="pyrax" label={t('sections.consensus.label')} title={t('sections.consensus.title')} />
          <p className="text-stone-300 mb-8">{t('sections.consensus.intro')}</p>
          <div className="grid md:grid-cols-3 gap-6">
            <StreamCard stream="A" algorithm="BLAKE3" type={t('sections.consensus.streamA.type')} blockTime="10s" reward="50 PYRAX" feeShare="20%" desc={t('sections.consensus.streamA.desc')} color="pyrax" />
            <StreamCard stream="B" algorithm="KAWPOW" type={t('sections.consensus.streamB.type')} blockTime="60s" reward="100 PYRAX" feeShare="40%" desc={t('sections.consensus.streamB.desc')} color="blue" />
            <StreamCard stream="C" algorithm="ZK-STARK" type={t('sections.consensus.streamC.type')} blockTime="Variable" reward="10 PYRAX" feeShare="30%" desc={t('sections.consensus.streamC.desc')} color="purple" />
          </div>
        </div>
      </section>

      {/* GHOSTDAG */}
      <section id="ghostdag" className="py-16 border-t border-stone-800 bg-stone-900/20">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={CircleStackIcon} color="green" label={t('sections.ghostdag.label')} title={t('sections.ghostdag.title')} />
          <p className="text-stone-300 mb-6">{t('sections.ghostdag.intro')}</p>
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6 mb-6">
            <h4 className="text-white font-semibold mb-4">{t('sections.ghostdag.properties.title')}</h4>
            <ul className="space-y-3">
              <li className="flex gap-3 text-stone-300"><span className="text-green-400">•</span>{t('sections.ghostdag.properties.p1')}</li>
              <li className="flex gap-3 text-stone-300"><span className="text-green-400">•</span>{t('sections.ghostdag.properties.p2')}</li>
              <li className="flex gap-3 text-stone-300"><span className="text-green-400">•</span>{t('sections.ghostdag.properties.p3')}</li>
              <li className="flex gap-3 text-stone-300"><span className="text-green-400">•</span>{t('sections.ghostdag.properties.p4')}</li>
            </ul>
          </div>
        </div>
      </section>

      {/* Block Structure */}
      <section id="blockStructure" className="py-16 border-t border-stone-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={CubeIcon} color="yellow" label={t('sections.block.label')} title={t('sections.block.title')} />
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6 font-mono text-sm overflow-x-auto">
            <pre className="text-stone-300">{`BLOCK HEADER (112 bytes)
┌──────────────────┬────────────────────────────────────┐
│ Version          │ 4 bytes - Protocol version          │
│ Previous Hash    │ 32 bytes - SHA256 of prev header    │
│ Merkle Root      │ 32 bytes - Transaction tree root    │
│ Timestamp        │ 8 bytes - Unix timestamp            │
│ Difficulty       │ 4 bytes - Compact target            │
│ Nonce            │ 8 bytes - PoW solution              │
│ Height           │ 8 bytes - Block number              │
│ Extra Nonce      │ 8 bytes - Extended nonce            │
└──────────────────┴────────────────────────────────────┘`}</pre>
          </div>
        </div>
      </section>

      {/* Cryptography */}
      <section id="cryptography" className="py-16 border-t border-stone-800 bg-stone-900/20">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={LockClosedIcon} color="red" label={t('sections.crypto.label')} title={t('sections.crypto.title')} />
          <div className="grid md:grid-cols-2 gap-6">
            <CryptoCard title={t('sections.crypto.address.title')} value="Keccak-256" desc={t('sections.crypto.address.desc')} />
            <CryptoCard title={t('sections.crypto.blockHash.title')} value="BLAKE3 / KAWPOW" desc={t('sections.crypto.blockHash.desc')} />
            <CryptoCard title={t('sections.crypto.txHash.title')} value="Keccak-256" desc={t('sections.crypto.txHash.desc')} />
            <CryptoCard title={t('sections.crypto.signatures.title')} value="ECDSA secp256k1" desc={t('sections.crypto.signatures.desc')} />
            <CryptoCard title={t('sections.crypto.zkProofs.title')} value="STARK, SNARK, Plonk, Groth16" desc={t('sections.crypto.zkProofs.desc')} />
            <CryptoCard title={t('sections.crypto.keyDerivation.title')} value="BIP-32/39/44" desc={t('sections.crypto.keyDerivation.desc')} />
          </div>
        </div>
      </section>

      {/* Smart Contracts */}
      <section id="smartContracts" className="py-16 border-t border-stone-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={CodeBracketIcon} color="cyan" label={t('sections.contracts.label')} title={t('sections.contracts.title')} />
          <div className="grid md:grid-cols-2 gap-6">
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
              <h4 className="text-white font-semibold mb-3">{t('sections.contracts.evm.title')}</h4>
              <p className="text-stone-400 mb-4">{t('sections.contracts.evm.desc')}</p>
              <ul className="space-y-2 text-sm text-stone-400">
                <li>• Solidity 0.8.x</li><li>• 2s block time</li><li>• 1,000-5,000 TPS</li><li>• Full ERC-20/721/1155</li>
              </ul>
            </div>
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
              <h4 className="text-white font-semibold mb-3">{t('sections.contracts.wasm.title')}</h4>
              <p className="text-stone-400 mb-4">{t('sections.contracts.wasm.desc')}</p>
              <ul className="space-y-2 text-sm text-stone-400">
                <li>• Rust/WASM</li><li>• Wasmtime runtime</li><li>• EVM ↔ WASM interop</li><li>• Fuel-based gas</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* AI Platform */}
      <section id="aiPlatform" className="py-16 border-t border-stone-800 bg-stone-900/20">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={ServerStackIcon} color="cyan" label={t('sections.ai.label')} title={t('sections.ai.title')} />
          <p className="text-stone-300 mb-8">{t('sections.ai.intro')}</p>
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6 font-mono text-sm mb-6">
            <pre className="text-stone-300">{`SUBMIT → PENDING → MATCHED → RUNNING → VERIFY → SETTLE
   │                                              │
   └──── escrow ──────────────────────────────────┘
                                                  │
                              ┌───────────────────┴───────────────────┐
                              ▼                                       ▼
                          SUCCESS                                 DISPUTE
                        (payment)                               (arbitration)`}</pre>
          </div>
        </div>
      </section>

      {/* Tokenomics */}
      <section id="tokenomics" className="py-16 border-t border-stone-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={ChartBarIcon} color="green" label={t('sections.tokenomics.label')} title={t('sections.tokenomics.title')} />
          <div className="grid md:grid-cols-2 gap-6 mb-8">
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
              <h4 className="text-white font-semibold mb-4">{t('sections.tokenomics.distribution.title')}</h4>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between"><span className="text-stone-400">Mining Rewards</span><span className="text-white">35%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Presale</span><span className="text-white">15%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">BDAG Community</span><span className="text-white">10%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Ecosystem</span><span className="text-white">10%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Liquidity</span><span className="text-white">10%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">ZK Prover</span><span className="text-white">5%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Marketing</span><span className="text-white">5%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Team</span><span className="text-white">4%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Advisors</span><span className="text-white">3%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Treasury + Reserve</span><span className="text-white">3%</span></div>
              </div>
            </div>
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
              <h4 className="text-white font-semibold mb-4">{t('sections.tokenomics.fees.title')}</h4>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between"><span className="text-stone-400">Stream A Miners</span><span className="text-white">20%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Stream B Miners</span><span className="text-white">40%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Stream C Validators</span><span className="text-white">30%</span></div>
                <div className="flex justify-between"><span className="text-stone-400">Protocol Treasury</span><span className="text-white">10%</span></div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Security */}
      <section id="security" className="py-16 border-t border-stone-800 bg-stone-900/20">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={ShieldCheckIcon} color="red" label={t('sections.security.label')} title={t('sections.security.title')} />
          <div className="grid md:grid-cols-2 gap-6">
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
              <h4 className="text-white font-semibold mb-3">{t('sections.security.slashing.title')}</h4>
              <ul className="space-y-2 text-sm text-stone-400">
                <li>• Double-sign: 5% slash</li><li>• Downtime: 0.1% slash</li><li>• Unbonding: 7 days</li>
              </ul>
            </div>
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
              <h4 className="text-white font-semibold mb-3">{t('sections.security.verification.title')}</h4>
              <ul className="space-y-2 text-sm text-stone-400">
                <li>• Hash check verification</li><li>• N-of-M redundant execution</li><li>• ZK proof verification</li><li>• TEE attestation (future)</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* References */}
      <section id="references" className="py-16 border-t border-stone-800">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <SectionHeader icon={DocumentTextIcon} color="stone" label={t('sections.references.label')} title={t('sections.references.title')} />
          <ul className="space-y-2 text-sm text-stone-400">
            <li>[1] Sompolinsky, Y., Zohar, A. - PHANTOM GHOSTDAG (2018)</li>
            <li>[2] Buterin, V. - Ethereum Yellow Paper (2014)</li>
            <li>[3] KAWPOW Algorithm Specification - Ravencoin</li>
            <li>[4] BLAKE3 Cryptographic Hash Function</li>
            <li>[5] BIP-32, BIP-39, BIP-44 - HD Wallet Specifications</li>
            <li>[6] StarkWare - STARK Proof Systems</li>
          </ul>
        </div>
      </section>

      <Footer />
    </main>
  );
}

function SectionHeader({ icon: Icon, color, label, title }: { icon: React.ComponentType<{className?: string}>, color: string, label: string, title: string }) {
  const colorMap: Record<string, string> = { blue: 'bg-blue-500/20 text-blue-400', purple: 'bg-purple-500/20 text-purple-400', pyrax: 'bg-pyrax-500/20 text-pyrax-400', green: 'bg-green-500/20 text-green-400', yellow: 'bg-yellow-500/20 text-yellow-400', red: 'bg-red-500/20 text-red-400', cyan: 'bg-cyan-500/20 text-cyan-400', stone: 'bg-stone-500/20 text-stone-400' };
  return (
    <div className="mb-8">
      <div className="flex items-center gap-3 mb-4">
        <div className={`p-2 rounded-lg ${colorMap[color]?.split(' ')[0]}`}><Icon className={`w-5 h-5 ${colorMap[color]?.split(' ')[1]}`} /></div>
        <span className={colorMap[color]?.split(' ')[1]}>{label}</span>
      </div>
      <h2 className="text-3xl font-bold text-white">{title}</h2>
    </div>
  );
}

function LayerCard({ layer, title, desc, specs, color }: { layer: string, title: string, desc: string, specs: string, color: string }) {
  const colorMap: Record<string, string> = { purple: 'border-purple-500/30 from-purple-500/10', blue: 'border-blue-500/30 from-blue-500/10', pyrax: 'border-pyrax-500/30 from-pyrax-500/10' };
  return (
    <div className={`bg-gradient-to-r ${colorMap[color]} to-stone-900 border ${colorMap[color]?.split(' ')[0]} rounded-xl p-6`}>
      <div className="flex items-center gap-3 mb-2"><span className="px-2 py-0.5 bg-stone-800 text-white text-xs font-mono rounded">L{layer}</span><h4 className="text-white font-semibold">{title}</h4></div>
      <p className="text-stone-400 text-sm mb-2">{desc}</p><p className="text-stone-500 text-xs font-mono">{specs}</p>
    </div>
  );
}

function StreamCard({ stream, algorithm, type, blockTime, reward, feeShare, desc, color }: { stream: string, algorithm: string, type: string, blockTime: string, reward: string, feeShare: string, desc: string, color: string }) {
  const colorMap: Record<string, string> = { pyrax: 'from-pyrax-500/20 border-pyrax-500/30 text-pyrax-400', blue: 'from-blue-500/20 border-blue-500/30 text-blue-400', purple: 'from-purple-500/20 border-purple-500/30 text-purple-400' };
  return (
    <div className={`bg-gradient-to-b ${colorMap[color]?.split(' ')[0]} to-stone-900 border ${colorMap[color]?.split(' ')[1]} rounded-xl p-6`}>
      <div className={`text-2xl font-bold ${colorMap[color]?.split(' ')[2]} mb-1`}>Stream {stream}</div>
      <div className="text-white font-semibold mb-1">{algorithm}</div><div className="text-stone-400 text-sm mb-3">{type}</div>
      <p className="text-stone-400 text-xs mb-4">{desc}</p>
      <div className="grid grid-cols-3 gap-2 text-xs"><div><div className="text-stone-500">Block</div><div className="text-white">{blockTime}</div></div><div><div className="text-stone-500">Reward</div><div className="text-white">{reward}</div></div><div><div className="text-stone-500">Fees</div><div className="text-white">{feeShare}</div></div></div>
    </div>
  );
}

function CryptoCard({ title, value, desc }: { title: string, value: string, desc: string }) {
  return <div className="bg-stone-900 border border-stone-800 rounded-xl p-4"><h4 className="text-white font-medium mb-1">{title}</h4><div className="text-pyrax-400 font-mono text-sm mb-2">{value}</div><p className="text-stone-500 text-xs">{desc}</p></div>;
}
