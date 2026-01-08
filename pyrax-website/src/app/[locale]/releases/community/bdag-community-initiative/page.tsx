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
  GiftIcon,
  UserGroupIcon,
  ClockIcon,
  CheckCircleIcon,
  HeartIcon,
} from '@heroicons/react/24/outline';

export default function BDAGCommunityInitiativePage() {
  const t = useTranslations('releasesPage');
  const locale = useLocale();

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />

      {/* Article Header */}
      <section className="relative pt-32 pb-12 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-blue-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-blue-500/5 blur-[128px]" />

        <div className="relative z-10 max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <Link
              href={`/${locale}/releases/community`}
              className="inline-flex items-center gap-2 text-stone-400 hover:text-white transition-colors mb-8"
            >
              <ArrowLeftIcon className="w-4 h-4" />
              <span>{t('backToReleases')}</span>
            </Link>

            <div className="flex items-center gap-2 text-sm text-stone-500 mb-4">
              <CalendarIcon className="w-4 h-4" />
              <span>January 8, 2026</span>
              <span className="mx-2">•</span>
              <span className="text-blue-500">Community Release</span>
            </div>

            <h1 className="text-4xl sm:text-5xl font-bold text-white mb-6">
              The BDAG Community Initiative
            </h1>

            <p className="text-xl text-stone-400">
              Our commitment to the BlockDAG community — how the 10% allocation works, and why we&apos;re seeing this through to the finish line.
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
            {/* Opening Letter */}
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-8 mb-8">
              <p className="text-stone-300 leading-relaxed">
                To the BlockDAG community,
              </p>
              <p className="text-stone-300 leading-relaxed mt-4">
                When we first announced the BDAG Community Initiative, we made a promise: <strong className="text-white">10% of the total PYRAX supply would be reserved for BlockDAG token holders</strong>. That promise stands, and this letter explains exactly how we plan to deliver on it — even as our funding path has evolved.
              </p>
              <p className="text-stone-300 leading-relaxed mt-4">
                We understand that many of you have been waiting patiently. We also understand that trust is earned through action, not words. This update is meant to give you clarity on the mechanics, the timeline, and our unwavering commitment to this initiative.
              </p>
            </div>

            {/* Quick Stats */}
            <div className="grid sm:grid-cols-3 gap-4 my-8">
              <div className="bg-blue-500/10 border border-blue-500/30 rounded-xl p-6 text-center">
                <GiftIcon className="w-8 h-8 text-blue-400 mx-auto mb-2" />
                <p className="text-3xl font-bold text-white">10B</p>
                <p className="text-stone-400 text-sm">PYRAX Reserved</p>
              </div>
              <div className="bg-blue-500/10 border border-blue-500/30 rounded-xl p-6 text-center">
                <UserGroupIcon className="w-8 h-8 text-blue-400 mx-auto mb-2" />
                <p className="text-3xl font-bold text-white">10%</p>
                <p className="text-stone-400 text-sm">Total Supply</p>
              </div>
              <div className="bg-blue-500/10 border border-blue-500/30 rounded-xl p-6 text-center">
                <ClockIcon className="w-8 h-8 text-blue-400 mx-auto mb-2" />
                <p className="text-3xl font-bold text-white">12mo</p>
                <p className="text-stone-400 text-sm">Cliff Period</p>
              </div>
            </div>

            {/* How It Works */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <span className="w-8 h-8 bg-blue-500/20 rounded-full flex items-center justify-center text-blue-400 text-sm font-bold">1</span>
              How the BDAG Allocation Works
            </h2>

            <p className="text-stone-300 leading-relaxed">
              The BDAG Community Initiative reserves <strong className="text-white">10 billion PYRAX tokens</strong> (10% of the 100 billion total supply) for verified BlockDAG token holders. Here&apos;s the high-level mechanics:
            </p>

            <div className="bg-stone-800/50 border border-stone-700 rounded-xl p-6 my-6">
              <ul className="list-none space-y-4 m-0 p-0">
                <li className="flex items-start gap-3">
                  <CheckCircleIcon className="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" />
                  <span className="text-stone-300"><strong className="text-white">Pro-rata Distribution:</strong> PYRAX tokens will be allocated proportionally based on each holder&apos;s BDAG balance.</span>
                </li>
                <li className="flex items-start gap-3">
                  <CheckCircleIcon className="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" />
                  <div className="text-stone-300">
                    <strong className="text-white">Claim Process:</strong> Eligible holders will be able to claim their allocation through a dedicated portal on the PYRAX website after mainnet launch.
                    <ul className="list-disc list-inside mt-3 space-y-2 text-sm">
                      <li>Users can submit entire screenshots of their BDAG dashboard. This <strong className="text-white">MUST</strong> include: the wallet address the dashboard is connected to, and screenshots containing all transactions that include the USD value of the purchase amount.</li>
                      <li>If transactions are in foreign currency, screenshots must be submitted for manual review and claiming, which may take between 3-10 business days for processing.</li>
                      <li>Alternatively, users may submit transaction hashes via a downloadable Excel spreadsheet — complete it in full and upload for automatic processing.</li>
                      <li>More details on the claim process will be announced closer to portal release date.</li>
                    </ul>
                  </div>
                </li>
                <li className="flex items-start gap-3">
                  <CheckCircleIcon className="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0" />
                  <span className="text-stone-300"><strong className="text-white">Vesting Schedule:</strong> 12-month cliff, followed by 12-month linear vesting to ensure long-term alignment.</span>
                </li>
              </ul>
            </div>

            <p className="text-stone-300 leading-relaxed">
              The vesting schedule is designed to reward holders who believe in the long-term vision of PYRAX, not short-term speculation. We believe this aligns incentives between the BDAG community and the PYRAX ecosystem.
            </p>

            {/* Why Private Funding */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <span className="w-8 h-8 bg-blue-500/20 rounded-full flex items-center justify-center text-blue-400 text-sm font-bold">2</span>
              Why Private Funding Is Our Path Forward
            </h2>

            <p className="text-stone-300 leading-relaxed">
              As detailed in our previous community release, the regulatory landscape for token launches in Canada and the United States makes a traditional public presale extremely challenging. After extensive legal review, <strong className="text-white">private institutional funding has emerged as the most viable path to launch PYRAX properly</strong>.
            </p>

            <p className="text-stone-300 leading-relaxed mt-4">
              This means:
            </p>

            <ul className="list-none space-y-3 my-6">
              <li className="flex items-start gap-3 text-stone-300">
                <span className="text-blue-400 font-bold">→</span>
                We are pursuing funding from accredited investors and institutional partners under applicable exemptions.
              </li>
              <li className="flex items-start gap-3 text-stone-300">
                <span className="text-blue-400 font-bold">→</span>
                We are not conducting a public token presale that could be classified as an unregistered securities offering.
              </li>
              <li className="flex items-start gap-3 text-stone-300">
                <span className="text-blue-400 font-bold">→</span>
                We are building PYRAX to last — which means launching it in a way that can withstand regulatory scrutiny.
              </li>
            </ul>

            <div className="bg-blue-500/10 border border-blue-500/30 rounded-xl p-6 my-8">
              <p className="text-white font-semibold mb-2">Important Note:</p>
              <p className="text-stone-300 m-0">
                The BDAG Community Initiative is <strong className="text-white">not</strong> affected by our funding path. The 10% allocation remains locked and reserved regardless of how we fund development. This allocation is a commitment, not a reward for investment.
              </p>
            </div>

            {/* Our Commitment */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <span className="w-8 h-8 bg-blue-500/20 rounded-full flex items-center justify-center text-blue-400 text-sm font-bold">3</span>
              Our Commitment to the BDAG Community
            </h2>

            <p className="text-stone-300 leading-relaxed">
              We want to be absolutely clear about something:
            </p>

            <div className="bg-gradient-to-r from-blue-900/30 to-stone-900/50 border-2 border-blue-500/50 rounded-2xl p-8 my-8">
              <HeartIcon className="w-12 h-12 text-blue-400 mx-auto mb-4" />
              <p className="text-xl text-white text-center font-semibold mb-4">
                The founding team is committed to seeing the BDAG Community Initiative through to completion.
              </p>
              <p className="text-stone-300 text-center m-0">
                Regardless of the funding path we take, regardless of the challenges we face, the 10 billion PYRAX reserved for BDAG holders will be distributed as promised. This is a core commitment of the PYRAX project.
              </p>
            </div>

            <p className="text-stone-300 leading-relaxed">
              We understand that the crypto space is full of broken promises. We also understand that you have every reason to be skeptical. Our response to that skepticism is simple: <strong className="text-white">we will deliver</strong>.
            </p>

            <p className="text-stone-300 leading-relaxed mt-4">
              The BDAG community believed in decentralized infrastructure before it was mainstream. Many of you have been through the ups and downs of the BlockDAG project. We see the BDAG Community Initiative as a way to reward that patience and belief — and to bring you along on the PYRAX journey.
            </p>

            {/* Timeline */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6 flex items-center gap-3">
              <span className="w-8 h-8 bg-blue-500/20 rounded-full flex items-center justify-center text-blue-400 text-sm font-bold">4</span>
              What Comes Next
            </h2>

            <div className="space-y-4 my-6">
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 bg-green-500/20 rounded-full flex items-center justify-center flex-shrink-0">
                  <CheckCircleIcon className="w-5 h-5 text-green-400" />
                </div>
                <div>
                  <p className="text-white font-semibold">Allocation Reserved</p>
                  <p className="text-stone-400 text-sm">10 billion PYRAX locked in tokenomics</p>
                </div>
              </div>
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 bg-blue-500/20 rounded-full flex items-center justify-center flex-shrink-0">
                  <ClockIcon className="w-5 h-5 text-blue-400" />
                </div>
                <div>
                  <p className="text-white font-semibold">Mainnet Launch</p>
                  <p className="text-stone-400 text-sm">Target: Q3 2026</p>
                </div>
              </div>
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 bg-stone-700 rounded-full flex items-center justify-center flex-shrink-0">
                  <span className="text-stone-400 text-sm font-bold">3</span>
                </div>
                <div>
                  <p className="text-white font-semibold">Claim Portal Opens</p>
                  <p className="text-stone-400 text-sm">Shortly after testnet launch</p>
                </div>
              </div>
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 bg-stone-700 rounded-full flex items-center justify-center flex-shrink-0">
                  <span className="text-stone-400 text-sm font-bold">4</span>
                </div>
                <div>
                  <p className="text-white font-semibold">Vesting Begins</p>
                  <p className="text-stone-400 text-sm">12-month cliff, then 12-month linear release (starting on 1st day of mainnet & TGE)</p>
                </div>
              </div>
            </div>

            {/* Stay Connected */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6">Stay Connected</h2>
            
            <p className="text-stone-300 leading-relaxed">
              The best way to stay informed about the BDAG Community Initiative is to join our official channels:
            </p>

            <div className="grid sm:grid-cols-3 gap-4 my-6">
              <a href="https://discord.gg/z9kjrE9q" target="_blank" rel="noopener noreferrer" className="bg-stone-800/50 border border-stone-700 rounded-xl p-4 text-center hover:border-blue-500/50 transition-colors no-underline">
                <p className="text-white font-semibold">Discord</p>
                <p className="text-stone-400 text-sm">Real-time updates</p>
              </a>
              <a href="https://twitter.com/pyrax_org" target="_blank" rel="noopener noreferrer" className="bg-stone-800/50 border border-stone-700 rounded-xl p-4 text-center hover:border-blue-500/50 transition-colors no-underline">
                <p className="text-white font-semibold">Twitter/X</p>
                <p className="text-stone-400 text-sm">Announcements</p>
              </a>
              <a href="https://t.me/pyrax_official" target="_blank" rel="noopener noreferrer" className="bg-stone-800/50 border border-stone-700 rounded-xl p-4 text-center hover:border-blue-500/50 transition-colors no-underline">
                <p className="text-white font-semibold">Telegram</p>
                <p className="text-stone-400 text-sm">Community chat</p>
              </a>
            </div>

            {/* Closing */}
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-8 my-8">
              <p className="text-stone-300 leading-relaxed">
                To the BDAG community: thank you for your patience and your belief in what we&apos;re building. The road to mainnet is long, but not as long as that other project..., but we&apos;re glad we&apos;re walking it together.
              </p>
              <p className="text-stone-300 leading-relaxed mt-4">
                We will continue to provide updates as we hit milestones. In the meantime, know that your allocation is reserved, the commitment is real, and the team is working every day to make PYRAX a reality.
              </p>
            </div>

            {/* Signatures */}
            <div className="mt-12 pt-8 border-t border-stone-800">
              <p className="text-stone-300 mb-8">With gratitude,<br />The PYRAX Founding Team</p>
              
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

            {/* Disclaimer */}
            <div className="bg-stone-800/30 border border-stone-700 rounded-xl p-6 mt-12 text-sm">
              <h3 className="text-base font-semibold text-stone-400 mb-2">Disclaimer</h3>
              <p className="text-stone-500 m-0">
                This communication is for informational purposes only and does not constitute an offer to sell or a solicitation to buy any securities or tokens. The BDAG Community Initiative allocation is subject to the terms and conditions that will be published prior to the snapshot. Eligibility requirements and distribution mechanics may be updated as we finalize technical implementation. Always verify information through official PYRAX channels.
              </p>
            </div>
          </motion.article>

          {/* Share */}
          <div className="mt-12 pt-8 border-t border-stone-800 flex items-center justify-between">
            <Link
              href={`/${locale}/releases/community`}
              className="inline-flex items-center gap-2 text-stone-400 hover:text-white transition-colors"
            >
              <ArrowLeftIcon className="w-4 h-4" />
              <span>{t('backToReleases')}</span>
            </Link>
            <button
              onClick={() => {
                if (navigator.share) {
                  navigator.share({
                    title: 'PYRAX - The BDAG Community Initiative',
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
