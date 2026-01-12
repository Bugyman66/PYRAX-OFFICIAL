'use client';

import { useState, useEffect, Suspense } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import {
  Search,
  Loader2,
  CheckCircle,
  Clock,
  XCircle,
  AlertTriangle,
  ChevronRight,
  FileText,
  Calendar,
  User,
  Tag,
  ArrowLeft,
  RefreshCw,
} from 'lucide-react';

interface SubmissionStatus {
  id: string;
  message: string | null;
  category: string | null;
  createdAt: string;
  proof: {
    id: string;
    title: string;
    status: string;
  };
}

function StatusContent() {
  const searchParams = useSearchParams();
  const refFromUrl = searchParams.get('ref');
  
  const [reference, setReference] = useState(refFromUrl || '');
  const [submission, setSubmission] = useState<SubmissionStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  useEffect(() => {
    if (refFromUrl) {
      fetchSubmission(refFromUrl);
    }
  }, [refFromUrl]);

  async function fetchSubmission(ref: string) {
    try {
      setLoading(true);
      setError(null);
      setSearched(true);

      const res = await fetch(`/api/submissions/${ref}`);
      if (res.status === 404) {
        setSubmission(null);
        setError('Submission not found. Please check your reference number.');
        return;
      }
      if (!res.ok) throw new Error('Failed to fetch submission');
      
      const data = await res.json();
      setSubmission(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch submission');
      setSubmission(null);
    } finally {
      setLoading(false);
    }
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (reference.trim()) {
      fetchSubmission(reference.trim());
    }
  }

  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'Draft':
        return {
          icon: FileText,
          color: 'text-stone-400',
          bgColor: 'bg-stone-500/20',
          label: 'Pending Review',
          description: 'Your submission is waiting to be reviewed by our team.',
        };
      case 'InReview':
        return {
          icon: Clock,
          color: 'text-blue-400',
          bgColor: 'bg-blue-500/20',
          label: 'Under Review',
          description: 'Your submission is currently being reviewed by our team.',
        };
      case 'Approved':
        return {
          icon: CheckCircle,
          color: 'text-green-400',
          bgColor: 'bg-green-500/20',
          label: 'Approved',
          description: 'Your submission has been approved!',
        };
      case 'Rejected':
        return {
          icon: XCircle,
          color: 'text-red-400',
          bgColor: 'bg-red-500/20',
          label: 'Not Approved',
          description: 'Your submission was not approved. Please check your email for feedback.',
        };
      default:
        return {
          icon: AlertTriangle,
          color: 'text-yellow-400',
          bgColor: 'bg-yellow-500/20',
          label: status,
          description: 'Status information is being processed.',
        };
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="min-h-screen bg-stone-950">
      {/* Hero */}
      <div className="bg-gradient-to-b from-stone-900 to-stone-950 border-b border-stone-800">
        <div className="max-w-4xl mx-auto px-6 py-12">
          <div className="flex items-center gap-2 text-stone-500 text-sm mb-4">
            <Link href="/" className="hover:text-stone-300">Home</Link>
            <ChevronRight className="w-4 h-4" />
            <Link href="/submit" className="hover:text-stone-300">Submit</Link>
            <ChevronRight className="w-4 h-4" />
            <span className="text-stone-300">Track Status</span>
          </div>
          <h1 className="text-3xl md:text-4xl font-bold text-stone-50 mb-4">
            Track Your <span className="pyrax-gradient-text">Submission</span>
          </h1>
          <p className="text-lg text-stone-400 max-w-2xl">
            Enter your submission reference number to check the current status of your review.
          </p>
        </div>
      </div>

      {/* Search Form */}
      <div className="max-w-2xl mx-auto px-6 py-12">
        <form onSubmit={handleSubmit} className="mb-8">
          <div className="flex gap-3">
            <div className="flex-1 relative">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
              <input
                type="text"
                value={reference}
                onChange={e => setReference(e.target.value)}
                placeholder="Enter your submission reference..."
                className="w-full pl-12 pr-4 py-3 bg-stone-900 border border-stone-700 rounded-xl text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
              />
            </div>
            <button
              type="submit"
              disabled={loading || !reference.trim()}
              className="px-6 py-3 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white font-medium rounded-xl transition-colors"
            >
              {loading ? <Loader2 className="w-5 h-5 animate-spin" /> : 'Track'}
            </button>
          </div>
        </form>

        {/* Error State */}
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-6 text-center mb-8">
            <AlertTriangle className="w-12 h-12 text-red-400 mx-auto mb-4" />
            <p className="text-red-400">{error}</p>
          </div>
        )}

        {/* Submission Status */}
        {submission && !error && (
          <div className="space-y-6">
            {/* Status Card */}
            {(() => {
              const config = getStatusConfig(submission.proof.status);
              const StatusIcon = config.icon;
              return (
                <div className={`${config.bgColor} border border-stone-800 rounded-xl p-6`}>
                  <div className="flex items-center gap-4 mb-4">
                    <div className={`w-14 h-14 rounded-full ${config.bgColor} flex items-center justify-center`}>
                      <StatusIcon className={`w-7 h-7 ${config.color}`} />
                    </div>
                    <div>
                      <h2 className={`text-2xl font-bold ${config.color}`}>{config.label}</h2>
                      <p className="text-stone-400 text-sm">{config.description}</p>
                    </div>
                  </div>
                </div>
              );
            })()}

            {/* Details */}
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
              <h3 className="font-semibold text-stone-100 mb-4">Submission Details</h3>
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <FileText className="w-5 h-5 text-stone-500" />
                  <div>
                    <p className="text-xs text-stone-500">Title</p>
                    <p className="text-stone-200">{submission.proof.title}</p>
                  </div>
                </div>

                {submission.category && (
                  <div className="flex items-center gap-3">
                    <Tag className="w-5 h-5 text-stone-500" />
                    <div>
                      <p className="text-xs text-stone-500">Category</p>
                      <p className="text-stone-200 capitalize">{submission.category.replace(/-/g, ' ')}</p>
                    </div>
                  </div>
                )}

                <div className="flex items-center gap-3">
                  <Calendar className="w-5 h-5 text-stone-500" />
                  <div>
                    <p className="text-xs text-stone-500">Submitted</p>
                    <p className="text-stone-200">{formatDate(submission.createdAt)}</p>
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  <Search className="w-5 h-5 text-stone-500" />
                  <div>
                    <p className="text-xs text-stone-500">Reference</p>
                    <p className="text-stone-200 font-mono text-sm">{submission.id}</p>
                  </div>
                </div>

                {submission.message && (
                  <div className="pt-4 border-t border-stone-800">
                    <p className="text-xs text-stone-500 mb-2">Your Message</p>
                    <p className="text-stone-400 text-sm">{submission.message}</p>
                  </div>
                )}
              </div>
            </div>

            {/* Actions */}
            <div className="flex flex-col sm:flex-row gap-3">
              <button
                onClick={() => fetchSubmission(submission.id)}
                disabled={loading}
                className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-stone-800 hover:bg-stone-700 text-stone-200 rounded-xl transition-colors"
              >
                <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
                Refresh Status
              </button>
              <Link
                href="/submit"
                className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-pyrax-500 hover:bg-pyrax-600 text-white rounded-xl transition-colors"
              >
                Submit Another
              </Link>
            </div>
          </div>
        )}

        {/* Empty State */}
        {!submission && !error && searched && (
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-12 text-center">
            <Search className="w-16 h-16 text-stone-700 mx-auto mb-4" />
            <h3 className="text-xl font-medium text-stone-300 mb-2">No submission found</h3>
            <p className="text-stone-500">
              We couldn&apos;t find a submission with that reference. Please check and try again.
            </p>
          </div>
        )}

        {/* Initial State */}
        {!submission && !error && !searched && (
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-12 text-center">
            <Search className="w-16 h-16 text-stone-700 mx-auto mb-4" />
            <h3 className="text-xl font-medium text-stone-300 mb-2">Enter your reference</h3>
            <p className="text-stone-500">
              Enter the submission reference you received after submitting your content.
            </p>
          </div>
        )}

        {/* Back Link */}
        <div className="mt-8 text-center">
          <Link
            href="/submit"
            className="inline-flex items-center gap-2 text-stone-400 hover:text-stone-200 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Submit
          </Link>
        </div>
      </div>
    </div>
  );
}

export default function SubmissionStatusPage() {
  return (
    <Suspense fallback={
      <div className="min-h-screen bg-stone-950 flex items-center justify-center">
        <Loader2 className="w-8 h-8 text-pyrax-500 animate-spin" />
      </div>
    }>
      <StatusContent />
    </Suspense>
  );
}
