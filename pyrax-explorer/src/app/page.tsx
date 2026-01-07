import Image from 'next/image';
import StatsGrid from '@/components/StatsGrid';
import RecentBlocks from '@/components/RecentBlocks';
import RecentTransactions from '@/components/RecentTransactions';
import NetworkOverview from '@/components/NetworkOverview';

export default function HomePage() {
  return (
    <div className="py-8 px-4 sm:px-6 lg:px-8">
      {/* Welcome Header */}
      <div className="mb-8">
        <div className="flex items-center gap-4 mb-2">
          <Image
            src="/pyrax-coin.svg"
            alt="PYRAX"
            width={48}
            height={48}
            className="w-12 h-12"
          />
          <div>
            <h1 className="text-2xl font-bold text-white">
              Welcome to <span className="pyrax-gradient-text">PYRAX Explorer</span>
            </h1>
            <p className="text-stone-400">
              GPU-mined, AI-powered blockchain explorer
            </p>
          </div>
        </div>
      </div>

      {/* Stats Grid - Async loaded */}
      <section className="py-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <StatsGrid />
        </div>
      </section>

      {/* Blocks & Transactions - Async loaded */}
      <section className="py-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-6">
            <RecentBlocks />
            <RecentTransactions />
          </div>
        </div>
      </section>

      {/* Network Stats - Async loaded */}
      <section className="py-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <NetworkOverview />
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-stone-800 mt-12 py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-6">
            <div className="flex items-center gap-3">
              <Image
                src="/pyrax-coin.svg"
                alt="PYRAX"
                width={32}
                height={32}
                className="w-8 h-8"
              />
              <span className="text-stone-400">PYRAX Explorer v0.1.0</span>
            </div>
            <div className="flex items-center gap-6 text-stone-500">
              <a href="https://pyrax.org" className="hover:text-pyrax-500 transition-colors">Website</a>
              <a href="https://github.com/pyrax-official/pyrax" className="hover:text-pyrax-500 transition-colors">GitHub</a>
              <a href="https://docs.pyrax.org" className="hover:text-pyrax-500 transition-colors">Docs</a>
              <a href="https://discord.gg/pyrax" className="hover:text-pyrax-500 transition-colors">Discord</a>
            </div>
          </div>
          <div className="mt-8 text-center text-stone-600 text-sm">
            © 2024 PYRAX. Built for GPU miners and AI innovation.
          </div>
        </div>
      </footer>
    </div>
  );
}
