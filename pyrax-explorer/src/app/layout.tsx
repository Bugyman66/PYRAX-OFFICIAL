import type { Metadata } from 'next';
import './globals.css';
import Sidebar from '@/components/Sidebar';
import { NetworkProvider } from '@/context/NetworkContext';

export const metadata: Metadata = {
  title: 'PYRAX Explorer | Block Explorer',
  description: 'Explore blocks, transactions, and accounts on the PYRAX blockchain - GPU-mined, AI-powered.',
  keywords: ['PYRAX', 'blockchain', 'explorer', 'cryptocurrency', 'GPU mining', 'KAWPOW'],
  icons: {
    icon: '/pyrax-favicon.svg',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="h-full dark">
      <body className="h-full antialiased bg-stone-950 text-white">
        <NetworkProvider>
          <Sidebar>{children}</Sidebar>
        </NetworkProvider>
      </body>
    </html>
  );
}
