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
} from '@heroicons/react/24/outline';

export default function LegalLaunchFundingPathPage() {
  const t = useTranslations('releasesPage');
  const locale = useLocale();

  return (
    <main className="min-h-screen bg-stone-950">
      <Navbar />

      {/* Article Header */}
      <section className="relative pt-32 pb-12 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-pyrax-500/10 via-transparent to-transparent" />
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-pyrax-500/5 blur-[128px]" />

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
              <span>January 7, 2026</span>
              <span className="mx-2">•</span>
              <span className="text-pyrax-500">Community Release</span>
            </div>

            <h1 className="text-4xl sm:text-5xl font-bold text-white mb-6">
              Legal launch & funding path
            </h1>

            <p className="text-xl text-stone-400">
              A candid project update about what&apos;s moving quickly, what&apos;s moving slowly, and why.
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
            <div className="bg-stone-900/50 border border-stone-800 rounded-2xl p-8 mb-8">
              <p className="text-stone-300 leading-relaxed">
                To the PYRAX community,
              </p>
              <p className="text-stone-300 leading-relaxed mt-4">
                We&apos;re building PYRAX with a simple (and stubborn) principle: if we&apos;re going to ship a Layer 1 meant to last decades, we should launch it in a way that&apos;s legal, proper, and ethical from day one — even when that path is slower and more expensive.
              </p>
              <p className="text-stone-300 leading-relaxed mt-4">
                This letter is a candid project update about what&apos;s moving quickly, what&apos;s moving slowly, and why. The headline is that our primary bottleneck right now is not engineering — it&apos;s <strong className="text-white">structuring a compliant cross-border launch</strong> for a Canada/U.S.-founded project in a regulatory environment that treats many token sales as securities offerings.
              </p>
            </div>

            {/* What's going well */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6">What&apos;s going well</h2>
            <p className="text-stone-300 leading-relaxed">
              Protocol engineering and design work continues. We are keeping the core team focused on building a robust base layer and the tooling needed to support a healthy ecosystem. The &quot;hard part&quot; here is not writing code — it&apos;s making sure the network can be launched and supported without putting the community (or the project) on the wrong side of securities law.
            </p>

            {/* Why a typical crypto presale is a problem */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6">Why a &quot;typical crypto presale&quot; is a problem for Canada & the U.S.</h2>
            <p className="text-stone-300 leading-relaxed">
              In the U.S., the baseline rule is that offers and sales of securities are regulated — they generally must be registered, or fit within an exemption.<sup>1</sup> If a token sale is treated as a securities offering, that framework applies.
            </p>

            <div className="bg-stone-800/50 border-l-4 border-pyrax-500 rounded-r-xl p-6 my-8">
              <h3 className="text-lg font-semibold text-white mb-4">Three &quot;why this matters&quot; quotes</h3>
              <blockquote className="text-stone-300 italic border-none pl-0">
                &quot;Every offer and sale of securities must either be registered… or rely on an available exemption…&quot;<sup>1</sup>
              </blockquote>
              <blockquote className="text-stone-300 italic border-none pl-0 mt-4">
                &quot;Many of these cryptocurrency offerings involve sales of securities.&quot;<sup>5</sup>
              </blockquote>
              <blockquote className="text-stone-300 italic border-none pl-0 mt-4">
                &quot;Most of these [token] offerings have involved securities.&quot;<sup>6</sup>
              </blockquote>
            </div>

            <p className="text-stone-300 leading-relaxed">
              Regulators have repeatedly emphasized that many digital-asset sales meet the legal definition of an &quot;investment contract.&quot; The SEC&apos;s digital-asset framework summarizes the classic test as an investment of money in a common enterprise with an expectation of profits to be derived from the efforts of others.
            </p>

            <div className="bg-stone-800/50 border border-stone-700 rounded-xl p-6 my-8">
              <h3 className="text-lg font-semibold text-white mb-2">Howey test (U.S. Supreme Court)</h3>
              <p className="text-stone-300 italic">
                &quot;…a person invests money in a common enterprise and is led to expect profits solely from efforts of the promoter or a third party.&quot;<sup>3</sup>
              </p>
            </div>

            <p className="text-stone-300 leading-relaxed">
              The SEC&apos;s 2017 DAO Report (focused on token fundraising) reiterates the same baseline: &quot;All securities offered and sold in the United States must be registered… or must qualify for an exemption.&quot;<sup>4</sup>
            </p>

            <p className="text-stone-300 leading-relaxed mt-4">
              In Canada, the Canadian Securities Administrators (CSA) have warned that many crypto offerings are securities offerings, and that Canadian securities laws apply if an issuer is doing business from within Canada or there are Canadian investors.<sup>5</sup> The CSA has also observed (after engaging with market participants) that most token offerings they have seen involved securities, and that many businesses sell tokens to raise capital for development — even when the token has &quot;utility&quot; features.<sup>6</sup>
            </p>

            <p className="text-stone-300 leading-relaxed mt-4 font-medium text-white">
              Put plainly: if you raise money today by selling a token that is not yet fully functional, and the buyer&apos;s thesis is &quot;the team will build this and the token will go up,&quot; that looks a lot like the fact pattern regulators and courts analyze under securities laws.
            </p>

            {/* How do we fund PYRAX */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6">So how do we fund PYRAX without doing something we can&apos;t defend?</h2>
            <p className="text-stone-300 leading-relaxed">
              We&apos;re deliberately avoiding a retail-facing token presale. Not because we dislike community participation, but because a public presale is one of the fastest ways for a serious project to accidentally run an unregistered securities offering (or the Canadian equivalent).
            </p>

            <p className="text-stone-300 leading-relaxed mt-4">
              Instead, if we raise outside capital, we&apos;re planning around the <strong className="text-white">accredited investor route</strong> and <strong className="text-white">institutional fundraising</strong> — the boring, paperwork-heavy path that exists specifically for private offerings.
            </p>

            <p className="text-stone-300 leading-relaxed mt-4">
              In the U.S., many private offerings rely on Regulation D. The SEC explains that Rule 506(c) permits general solicitation only where all purchasers are accredited investors and the issuer takes reasonable steps to verify that status; the SEC also notes that self-certification alone (for example, checking a box) is not enough.<sup>1,9</sup> The federal definition of &quot;accredited investor&quot; includes (among other categories) individuals with net worth over $1,000,000 (excluding primary residence) or income thresholds (e.g., $200,000 individual / $300,000 joint, subject to conditions).<sup>8</sup>
            </p>

            <p className="text-stone-300 leading-relaxed mt-4">
              In Canada, National Instrument 45-106 includes a parallel concept: &quot;The prospectus requirement does not apply to a distribution of a security if the purchaser… is an accredited investor.&quot;<sup>7</sup> In other words, Canada also provides exemptions for private capital raising — but they come with rules, forms, resale restrictions, and compliance obligations.
            </p>

            <p className="text-stone-300 leading-relaxed mt-4">
              Practically, that means any compliant fundraising may look like some combination of: (i) traditional equity financing (or SAFE-style instruments), (ii) token warrants or token purchase rights structured under applicable exemptions, or (iii) a mix of both — typically with transfer restrictions, investor suitability limits, and careful disclosure. We can&apos;t discuss deal terms publicly, and we won&apos;t pretend the paperwork is &quot;fun,&quot; but this is the route that aligns with our goal of a legitimate launch.
            </p>

            {/* What this means for the community */}
            <h2 className="text-2xl font-bold text-white mt-12 mb-6">What this means for the community (and why we&apos;re telling you now)</h2>
            
            <ul className="list-none space-y-4 my-6">
              <li className="flex items-start gap-3">
                <span className="text-pyrax-500 font-bold">First:</span>
                <span className="text-stone-300"><strong className="text-white">Transparency.</strong> We&apos;d rather tell you the truth than market a fantasy. The regulatory landscape isn&apos;t a footnote — it&apos;s a design constraint.</span>
              </li>
              <li className="flex items-start gap-3">
                <span className="text-pyrax-500 font-bold">Second:</span>
                <span className="text-stone-300"><strong className="text-white">Credibility.</strong> If PYRAX is going to be a serious base layer, it has to be able to survive scrutiny from regulators, partners, exchanges, and institutions.</span>
              </li>
              <li className="flex items-start gap-3">
                <span className="text-pyrax-500 font-bold">Third:</span>
                <span className="text-stone-300"><strong className="text-white">Momentum.</strong> While we structure a compliant path, community support still matters — through testing, building, feedback, and signal-boosting what we&apos;re doing.</span>
              </li>
            </ul>

            <p className="text-stone-300 leading-relaxed mt-4">
              This update is also written so that the right readers can understand the subtext: we are building something ambitious, and we are pursuing a compliant capital path that fits that ambition. If you&apos;re a Web3 fund or an accredited investor who values patient engineering and regulatory discipline, you already know what to do next — and we will only engage in any capital-raising discussions in a manner consistent with applicable law.
            </p>

            {/* Disclaimer */}
            <div className="bg-stone-800/30 border border-stone-700 rounded-xl p-6 my-8 text-sm">
              <h3 className="text-base font-semibold text-stone-400 mb-2">A quick (non-legal) disclaimer</h3>
              <p className="text-stone-500">
                This letter is for informational purposes only and is not an offer to sell, or a solicitation of an offer to buy, any securities or tokens in any jurisdiction. Nothing here is legal, tax, or investment advice. Any fundraising, if pursued, would be conducted pursuant to applicable exemptions, documentation, and compliance procedures.
              </p>
            </div>

            {/* Signatures */}
            <div className="mt-12 pt-8 border-t border-stone-800">
              <p className="text-stone-300 mb-8">Sincerely,<br />The PYRAX Founding Team</p>
              
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

            {/* References */}
            <div className="mt-12 pt-8 border-t border-stone-800">
              <h3 className="text-lg font-semibold text-white mb-4">References (selected)</h3>
              <ol className="list-decimal list-inside text-stone-500 text-sm space-y-2">
                <li>U.S. Securities and Exchange Commission (SEC), &quot;Exempt Offerings&quot; (June 21, 2024; last reviewed Nov. 6, 2024), sec.gov.</li>
                <li>SEC (FinHub), &quot;Framework for &apos;Investment Contract&apos; Analysis of Digital Assets&quot; (April 2019), sec.gov (dlt-framework.pdf).</li>
                <li>SEC v. W.J. Howey Co., 328 U.S. 293 (1946) (U.S. Supreme Court), via Cornell Legal Information Institute.</li>
                <li>SEC, Report of Investigation Pursuant to Section 21(a) of the Securities Exchange Act of 1934: The DAO (Release No. 81207, July 25, 2017), sec.gov.</li>
                <li>Canadian Securities Administrators (CSA), Staff Notice 46-307 Cryptocurrency Offerings (Aug. 24, 2017), securities-administrators.ca (PDF).</li>
                <li>CSA, Staff Notice 46-308 Securities Law Implications for Offerings of Tokens (June 11, 2018), asc.ca (PDF).</li>
                <li>CSA, National Instrument 45-106 Prospectus Exemptions (consolidation effective Sept. 19, 2025), asc.ca (PDF).</li>
                <li>17 C.F.R. § 230.501 (Regulation D — accredited investor definition), via Cornell Legal Information Institute.</li>
                <li>SEC, &quot;Assessing Accredited Investors under Regulation D&quot; (March 21, 2025; last reviewed Aug. 8, 2025), sec.gov.</li>
              </ol>
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
                    title: 'PYRAX - Legal launch & funding path',
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
