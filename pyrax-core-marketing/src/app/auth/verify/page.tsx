'use client';

import { useEffect, Suspense } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import { Loader2, Shield } from 'lucide-react';

function VerifyForm() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const token = searchParams.get('token');

  useEffect(() => {
    if (token) {
      window.location.href = `/api/auth/verify?token=${token}`;
    } else {
      router.push('/auth/login?error=invalid');
    }
  }, [token, router]);

  return (
    <div className="min-h-screen bg-stone-950 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold pyrax-gradient-text mb-2">PYRAX</h1>
          <p className="text-stone-500 text-sm uppercase tracking-widest">Proofing Hub</p>
        </div>

        {/* Card */}
        <div className="bg-stone-900 border border-stone-800 rounded-2xl p-8 text-center shadow-xl">
          <div className="w-20 h-20 bg-gradient-to-br from-pyrax-500/20 to-pyrax-600/10 rounded-full flex items-center justify-center mx-auto mb-6 relative">
            <div className="absolute inset-0 rounded-full bg-gradient-to-br from-pyrax-500/20 to-transparent animate-pulse" />
            <Loader2 className="w-10 h-10 text-pyrax-500 animate-spin" />
          </div>
          
          <h2 className="text-2xl font-semibold text-stone-100 mb-3">Verifying your identity</h2>
          <p className="text-stone-400 mb-6">Please wait while we securely sign you in...</p>
          
          <div className="flex items-center justify-center gap-2 text-stone-500 text-sm">
            <Shield className="w-4 h-4" />
            <span>Secure authentication in progress</span>
          </div>
        </div>

        {/* Footer */}
        <p className="text-center text-stone-600 text-xs mt-8">
          © 2026 PYRAX Blockchain. All rights reserved.
        </p>
      </div>
    </div>
  );
}

function LoadingFallback() {
  return (
    <div className="min-h-screen bg-stone-950 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold pyrax-gradient-text mb-2">PYRAX</h1>
          <p className="text-stone-500 text-sm uppercase tracking-widest">Proofing Hub</p>
        </div>

        {/* Card */}
        <div className="bg-stone-900 border border-stone-800 rounded-2xl p-8 text-center shadow-xl">
          <div className="w-20 h-20 bg-gradient-to-br from-pyrax-500/20 to-pyrax-600/10 rounded-full flex items-center justify-center mx-auto mb-6">
            <Loader2 className="w-10 h-10 text-pyrax-500 animate-spin" />
          </div>
          <h2 className="text-2xl font-semibold text-stone-100 mb-3">Loading...</h2>
        </div>

        {/* Footer */}
        <p className="text-center text-stone-600 text-xs mt-8">
          © 2026 PYRAX Blockchain. All rights reserved.
        </p>
      </div>
    </div>
  );
}

export default function VerifyPage() {
  return (
    <Suspense fallback={<LoadingFallback />}>
      <VerifyForm />
    </Suspense>
  );
}
