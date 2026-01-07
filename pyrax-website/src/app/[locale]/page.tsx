'use client';

import { useTranslations } from 'next-intl';
import Navbar from '@/components/Navbar';
import Hero from '@/components/Hero';
import Features from '@/components/Features';
import AIPlatform from '@/components/AIPlatform';
import Tokenomics from '@/components/Tokenomics';
import Roadmap from '@/components/Roadmap';
import Technology from '@/components/Technology';
import Ecosystem from '@/components/Ecosystem';
import CTA from '@/components/CTA';
import Footer from '@/components/Footer';

export default function Home() {
  return (
    <main className="min-h-screen">
      <Navbar />
      <Hero />
      <Features />
      <Technology />
      <AIPlatform />
      <Tokenomics />
      <Roadmap />
      <Ecosystem />
      <CTA />
      <Footer />
    </main>
  );
}
