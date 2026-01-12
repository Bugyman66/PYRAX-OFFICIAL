'use client';

import { useState, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import { Mail, Loader2, CheckCircle, AlertCircle } from 'lucide-react';

function LoginForm() {
  const searchParams = useSearchParams();
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loginType, setLoginType] = useState<'internal' | 'external'>('internal');

  const urlError = searchParams.get('error');
  const errorMessages: Record<string, string> = {
    invalid: 'Invalid login link. Please request a new one.',
    expired: 'Your login link has expired. Please request a new one.',
    server: 'A server error occurred. Please try again.',
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/auth/request-link', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, type: loginType }),
      });

      const data = await response.json();

      if (!response.ok) {
        setError(data.error || 'Failed to send login link');
        return;
      }

      setSuccess(true);
    } catch {
      setError('An error occurred. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  if (success) {
    return (
      <div className="min-h-screen bg-stone-950 flex items-center justify-center p-4">
        <div className="w-full max-w-md">
          <div className="bg-stone-900 rounded-2xl p-8 border border-stone-800">
            <div className="text-center">
              <div className="w-16 h-16 bg-pyrax-500/10 rounded-full flex items-center justify-center mx-auto mb-6">
                <CheckCircle className="w-8 h-8 text-pyrax-500" />
              </div>
              <h2 className="text-2xl font-bold text-stone-50 mb-2">Check your email</h2>
              <p className="text-stone-400 mb-6">
                We sent a login link to <span className="text-stone-50 font-medium">{email}</span>
              </p>
              <p className="text-sm text-stone-500">
                The link will expire in 15 minutes. If you don&apos;t see the email, check your spam folder.
              </p>
              <button
                onClick={() => {
                  setSuccess(false);
                  setEmail('');
                }}
                className="mt-6 text-pyrax-500 hover:text-pyrax-400 text-sm font-medium transition-colors"
              >
                Use a different email
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-stone-950 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold pyrax-gradient-text mb-2">PYRAX</h1>
          <p className="text-stone-500">Proofing Hub</p>
        </div>

        <div className="bg-stone-900 rounded-2xl p-8 border border-stone-800">
          <h2 className="text-2xl font-bold text-stone-50 mb-2">Welcome back</h2>
          <p className="text-stone-400 mb-6">Sign in to your account</p>

          {(urlError || error) && (
            <div className="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg flex items-start gap-3">
              <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
              <p className="text-red-400 text-sm">
                {error || (urlError && errorMessages[urlError]) || 'An error occurred'}
              </p>
            </div>
          )}

          <div className="flex gap-2 mb-6 p-1 bg-stone-950 rounded-lg">
            <button
              type="button"
              onClick={() => setLoginType('internal')}
              className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors ${
                loginType === 'internal'
                  ? 'pyrax-gradient text-white'
                  : 'text-stone-400 hover:text-stone-50'
              }`}
            >
              Internal Team
            </button>
            <button
              type="button"
              onClick={() => setLoginType('external')}
              className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors ${
                loginType === 'external'
                  ? 'pyrax-gradient text-white'
                  : 'text-stone-400 hover:text-stone-50'
              }`}
            >
              External Submitter
            </button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-stone-400 mb-2">
                Email address
              </label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-600" />
                <input
                  type="email"
                  id="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder={loginType === 'internal' ? 'you@pyrax.org' : 'you@example.com'}
                  required
                  className="w-full pl-11 pr-4 py-3 bg-stone-950 border border-stone-700 rounded-lg text-stone-50 placeholder-stone-600 focus:outline-none focus:ring-2 focus:ring-pyrax-500 focus:border-transparent transition-all"
                />
              </div>
              {loginType === 'internal' && (
                <p className="mt-2 text-xs text-stone-500">
                  Internal team members must use their @pyrax.org email
                </p>
              )}
            </div>

            <button
              type="submit"
              disabled={isLoading || !email}
              className="w-full py-3 px-4 pyrax-gradient text-white font-semibold rounded-lg hover:opacity-90 transition-all pyrax-glow disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none flex items-center justify-center gap-2"
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin" />
                  Sending link...
                </>
              ) : (
                'Send magic link'
              )}
            </button>
          </form>

          <div className="mt-6 pt-6 border-t border-stone-800">
            <p className="text-center text-sm text-stone-500">
              {loginType === 'internal' ? (
                <>Need access? Contact your administrator.</>
              ) : (
                <>
                  Don&apos;t have an account?{' '}
                  <a href="/auth/register" className="text-pyrax-500 hover:text-pyrax-400 transition-colors">
                    Sign up
                  </a>
                </>
              )}
            </p>
          </div>
        </div>

        <div className="mt-8 text-center">
          <a href="/media-kit" className="text-sm text-stone-500 hover:text-stone-300 transition-colors">
            View public media kit →
          </a>
        </div>
      </div>
    </div>
  );
}

function LoginLoading() {
  return (
    <div className="min-h-screen bg-stone-950 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold pyrax-gradient-text mb-2">PYRAX</h1>
          <p className="text-stone-500">Proofing Hub</p>
        </div>
        <div className="bg-stone-900 rounded-2xl p-8 border border-stone-800 animate-pulse">
          <div className="h-8 bg-stone-800 rounded w-3/4 mb-4"></div>
          <div className="h-4 bg-stone-800 rounded w-1/2 mb-6"></div>
          <div className="h-12 bg-stone-800 rounded mb-4"></div>
          <div className="h-12 bg-stone-800 rounded"></div>
        </div>
      </div>
    </div>
  );
}

export default function LoginPage() {
  return (
    <Suspense fallback={<LoginLoading />}>
      <LoginForm />
    </Suspense>
  );
}
