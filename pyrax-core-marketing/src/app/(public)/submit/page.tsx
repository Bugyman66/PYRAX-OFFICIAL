'use client';

import { useState } from 'react';
import Link from 'next/link';
import {
  Upload,
  FileText,
  Send,
  Loader2,
  CheckCircle,
  AlertCircle,
  ChevronRight,
  User,
  Mail,
  Tag,
  MessageSquare,
} from 'lucide-react';

const SUBMISSION_CATEGORIES = [
  { value: 'logo-design', label: 'Logo Design' },
  { value: 'marketing-material', label: 'Marketing Material' },
  { value: 'social-media', label: 'Social Media Content' },
  { value: 'video-content', label: 'Video Content' },
  { value: 'brand-assets', label: 'Brand Assets' },
  { value: 'other', label: 'Other' },
];

export default function SubmitPage() {
  const [email, setEmail] = useState('');
  const [name, setName] = useState('');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [category, setCategory] = useState('');
  const [message, setMessage] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [submissionId, setSubmissionId] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    
    if (!email.trim()) {
      setError('Email is required');
      return;
    }
    
    if (!title.trim()) {
      setError('Title is required');
      return;
    }

    try {
      setSubmitting(true);
      setError(null);

      const res = await fetch('/api/submissions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: email.trim(),
          name: name.trim() || null,
          title: title.trim(),
          description: description.trim() || null,
          category: category || null,
          message: message.trim() || null,
        }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Failed to submit');
      }

      const data = await res.json();
      setSubmissionId(data.id);
      setSuccess(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit');
    } finally {
      setSubmitting(false);
    }
  }

  if (success) {
    return (
      <div className="min-h-screen bg-stone-950 flex items-center justify-center p-4">
        <div className="max-w-md w-full text-center">
          <div className="w-20 h-20 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-6">
            <CheckCircle className="w-10 h-10 text-green-400" />
          </div>
          <h1 className="text-2xl font-bold text-stone-100 mb-3">Submission Received!</h1>
          <p className="text-stone-400 mb-6">
            Thank you for your submission. Our team will review it and get back to you soon.
          </p>
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-4 mb-6">
            <p className="text-sm text-stone-500 mb-1">Submission Reference</p>
            <p className="text-stone-200 font-mono text-sm">{submissionId}</p>
          </div>
          <p className="text-sm text-stone-500 mb-6">
            You can track the status of your submission using the reference above.
          </p>
          <div className="flex flex-col gap-3">
            <Link
              href={`/submit/status?ref=${submissionId}`}
              className="w-full px-6 py-3 bg-pyrax-500 hover:bg-pyrax-600 text-white font-medium rounded-xl transition-colors"
            >
              Track Submission
            </Link>
            <Link
              href="/submit"
              onClick={() => {
                setSuccess(false);
                setEmail('');
                setName('');
                setTitle('');
                setDescription('');
                setCategory('');
                setMessage('');
              }}
              className="w-full px-6 py-3 bg-stone-800 hover:bg-stone-700 text-stone-200 font-medium rounded-xl transition-colors"
            >
              Submit Another
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-stone-950">
      {/* Hero */}
      <div className="bg-gradient-to-b from-stone-900 to-stone-950 border-b border-stone-800">
        <div className="max-w-4xl mx-auto px-6 py-12">
          <div className="flex items-center gap-2 text-stone-500 text-sm mb-4">
            <Link href="/" className="hover:text-stone-300">Home</Link>
            <ChevronRight className="w-4 h-4" />
            <span className="text-stone-300">Submit Content</span>
          </div>
          <h1 className="text-3xl md:text-4xl font-bold text-stone-50 mb-4">
            Submit <span className="pyrax-gradient-text">Content for Review</span>
          </h1>
          <p className="text-lg text-stone-400 max-w-2xl">
            Share your designs, graphics, or marketing materials with the PYRAX team for review and approval.
          </p>
        </div>
      </div>

      {/* Form */}
      <div className="max-w-2xl mx-auto px-6 py-12">
        <form onSubmit={handleSubmit} className="space-y-6">
          {error && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 flex items-center gap-3">
              <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0" />
              <span className="text-red-400">{error}</span>
            </div>
          )}

          {/* Contact Info */}
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-stone-100 mb-4 flex items-center gap-2">
              <User className="w-5 h-5 text-pyrax-500" />
              Your Information
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-1">
                  Email Address <span className="text-red-400">*</span>
                </label>
                <div className="relative">
                  <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
                  <input
                    type="email"
                    value={email}
                    onChange={e => setEmail(e.target.value)}
                    placeholder="your@email.com"
                    required
                    className="w-full pl-10 pr-4 py-2.5 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-1">
                  Your Name
                </label>
                <input
                  type="text"
                  value={name}
                  onChange={e => setName(e.target.value)}
                  placeholder="John Doe"
                  className="w-full px-4 py-2.5 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                />
              </div>
            </div>
          </div>

          {/* Submission Details */}
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-stone-100 mb-4 flex items-center gap-2">
              <FileText className="w-5 h-5 text-pyrax-500" />
              Submission Details
            </h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-1">
                  Title <span className="text-red-400">*</span>
                </label>
                <input
                  type="text"
                  value={title}
                  onChange={e => setTitle(e.target.value)}
                  placeholder="e.g., PYRAX Logo Design v2"
                  required
                  className="w-full px-4 py-2.5 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-stone-300 mb-1">
                  Description
                </label>
                <textarea
                  value={description}
                  onChange={e => setDescription(e.target.value)}
                  placeholder="Describe your submission..."
                  rows={3}
                  className="w-full px-4 py-2.5 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 resize-none"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-stone-300 mb-1">
                  <span className="flex items-center gap-2">
                    <Tag className="w-4 h-4" />
                    Category
                  </span>
                </label>
                <select
                  value={category}
                  onChange={e => setCategory(e.target.value)}
                  className="w-full px-4 py-2.5 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                >
                  <option value="">Select a category</option>
                  {SUBMISSION_CATEGORIES.map(cat => (
                    <option key={cat.value} value={cat.value}>{cat.label}</option>
                  ))}
                </select>
              </div>
            </div>
          </div>

          {/* Message */}
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-stone-100 mb-4 flex items-center gap-2">
              <MessageSquare className="w-5 h-5 text-pyrax-500" />
              Additional Message
            </h2>
            <textarea
              value={message}
              onChange={e => setMessage(e.target.value)}
              placeholder="Any additional context or notes for the review team..."
              rows={4}
              className="w-full px-4 py-2.5 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 resize-none"
            />
          </div>

          {/* Note */}
          <div className="bg-stone-800/50 border border-stone-700 rounded-xl p-4">
            <div className="flex items-start gap-3">
              <Upload className="w-5 h-5 text-pyrax-500 flex-shrink-0 mt-0.5" />
              <div>
                <p className="text-sm text-stone-300 font-medium">File Upload Coming Soon</p>
                <p className="text-xs text-stone-500 mt-1">
                  After submitting this form, you will receive instructions on how to upload your files. 
                  We support images, videos, PDFs, and other common file formats.
                </p>
              </div>
            </div>
          </div>

          {/* Submit Button */}
          <button
            type="submit"
            disabled={submitting}
            className="w-full flex items-center justify-center gap-2 px-6 py-3 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white font-medium rounded-xl transition-colors"
          >
            {submitting ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Send className="w-5 h-5" />
            )}
            {submitting ? 'Submitting...' : 'Submit for Review'}
          </button>

          <p className="text-center text-xs text-stone-500">
            By submitting, you agree to our terms of use and allow PYRAX to review your content.
          </p>
        </form>
      </div>
    </div>
  );
}
