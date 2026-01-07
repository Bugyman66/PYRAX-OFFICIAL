'use client';

import { useState, useEffect } from 'react';
import Image from 'next/image';

export default function Home() {
  const [address, setAddress] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [message, setMessage] = useState('');
  const [txHash, setTxHash] = useState('');
  const [networkInfo, setNetworkInfo] = useState<{ network?: string; status?: string } | null>(null);

  useEffect(() => {
    fetch('/api/faucet')
      .then(res => res.json())
      .then(data => setNetworkInfo(data))
      .catch(() => setNetworkInfo(null));
  }, []);

  const isValidAddress = (addr: string) => {
    return /^0x[a-fA-F0-9]{40}$/.test(addr) || /^[a-zA-Z0-9]{40,}$/.test(addr);
  };

  const requestTokens = async () => {
    if (!address) {
      setStatus('error');
      setMessage('Please enter a wallet address');
      return;
    }

    if (!isValidAddress(address)) {
      setStatus('error');
      setMessage('Please enter a valid wallet address');
      return;
    }

    setStatus('loading');
    setMessage('');

    try {
      const response = await fetch('/api/faucet', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ address }),
      });

      const data = await response.json();

      if (response.ok) {
        setStatus('success');
        setMessage(data.message || 'Tokens sent successfully!');
        setTxHash(data.txHash || '');
      } else {
        setStatus('error');
        setMessage(data.error || 'Failed to request tokens');
      }
    } catch {
      setStatus('error');
      setMessage('Network error. Please try again.');
    }
  };

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="border-b border-stone-800 bg-stone-900/50 backdrop-blur-sm">
        <div className="max-w-5xl mx-auto px-4 py-4 flex items-center justify-between">
          <a href="https://pyrax.org" className="flex items-center">
            <Image src="/pyrax-logo.svg" alt="PYRAX" width={96} height={96} />
          </a>
          <nav className="flex items-center gap-4">
            <a
              href="https://pyrax.org"
              className="text-stone-400 hover:text-white transition-colors text-sm"
            >
              ← Back to pyrax.org
            </a>
          </nav>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 flex items-center justify-center p-4">
        <div className="w-full max-w-lg">
          {/* Hero */}
          <div className="text-center mb-8">
            <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-gradient-to-br from-[#ff6b35] to-[#ff8c42] mb-6 animate-pulse-glow">
              <svg className="w-10 h-10 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <h1 className="text-3xl font-bold mb-2">PYRAX {networkInfo?.network === 'devnet' ? 'Devnet' : 'Testnet'} Faucet</h1>
            <p className="text-stone-400">
              Get free testnet PYRAX tokens for development and testing
            </p>
          </div>

          {/* Faucet Card */}
          <div className="bg-stone-900 border border-stone-800 rounded-2xl p-6 shadow-xl">
            {/* Network Badge */}
            <div className="flex items-center justify-between mb-6">
              <span className="text-sm text-stone-400">Network</span>
              <span className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 text-emerald-400 text-sm font-medium">
                <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                PYRAX {networkInfo?.network === 'devnet' ? 'Devnet' : 'Testnet'}
              </span>
            </div>

            {/* Amount */}
            <div className="bg-stone-950 rounded-xl p-4 mb-6">
              <div className="text-sm text-stone-400 mb-1">Amount per request</div>
              <div className="text-2xl font-bold text-[#ff8c42]">100 PYRAX</div>
            </div>

            {/* Address Input */}
            <div className="mb-4">
              <label htmlFor="address" className="block text-sm text-stone-400 mb-2">
                Wallet Address
              </label>
              <input
                id="address"
                type="text"
                value={address}
                onChange={(e) => setAddress(e.target.value)}
                placeholder="0x... or PYRAX address"
                className="w-full px-4 py-3 rounded-xl bg-stone-950 border border-stone-700 text-white placeholder-stone-500 focus:outline-none focus:border-[#ff6b35] focus:ring-1 focus:ring-[#ff6b35] transition-all"
                disabled={status === 'loading'}
              />
            </div>

            {/* Request Button */}
            <button
              onClick={requestTokens}
              disabled={status === 'loading'}
              className="w-full py-4 rounded-xl font-semibold text-white bg-gradient-to-r from-[#ff6b35] to-[#ff8c42] hover:from-[#ff8c42] hover:to-[#ff6b35] disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-lg shadow-[#ff6b35]/20 hover:shadow-[#ff6b35]/30"
            >
              {status === 'loading' ? (
                <span className="flex items-center justify-center gap-2">
                  <svg className="w-5 h-5 animate-spin" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                  Requesting...
                </span>
              ) : (
                'Request Tokens'
              )}
            </button>

            {/* Status Message */}
            {message && (
              <div className={`mt-4 p-4 rounded-xl ${
                status === 'success' 
                  ? 'bg-emerald-500/10 border border-emerald-500/20 text-emerald-400' 
                  : 'bg-red-500/10 border border-red-500/20 text-red-400'
              }`}>
                <p>{message}</p>
                {txHash && (
                  <a
                    href={`https://explorer.testnet.pyrax.org/tx/${txHash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1 mt-2 text-sm underline hover:no-underline"
                  >
                    View transaction →
                  </a>
                )}
              </div>
            )}

            {/* Info */}
            <div className="mt-6 pt-6 border-t border-stone-800">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <div className="text-stone-500">Cooldown</div>
                  <div className="text-stone-300">1 hour</div>
                </div>
                <div>
                  <div className="text-stone-500">Per request</div>
                  <div className="text-stone-300">100 PYRAX</div>
                </div>
              </div>
            </div>
          </div>

          {/* Additional Info */}
          <div className="mt-6 text-center text-sm text-stone-500">
            <p>Need help? Join our <a href="https://discord.gg/sS7kaacRwU" className="text-[#ff8c42] hover:underline">Discord</a></p>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-stone-800 py-6">
        <div className="max-w-5xl mx-auto px-4 flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-stone-500">
          <div>© {new Date().getFullYear()} PYRAX Network. All rights reserved.</div>
          <div className="flex items-center gap-4">
            <a href="https://docs.pyrax.org" className="hover:text-white transition-colors">Docs</a>
            <a href="https://explorer.testnet.pyrax.org" className="hover:text-white transition-colors">Explorer</a>
            <a href="https://github.com/PYRAX-Chain" className="hover:text-white transition-colors">GitHub</a>
          </div>
        </div>
      </footer>
    </div>
  );
}
