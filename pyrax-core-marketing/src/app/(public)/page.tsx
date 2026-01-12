import Link from 'next/link';
import { FileText, Download, Image, ArrowRight } from 'lucide-react';

export default function PublicHomePage() {
  return (
    <div className="min-h-screen bg-stone-950">
      {/* Hero */}
      <div className="relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-pyrax-500/10 via-transparent to-transparent" />
        <div className="max-w-6xl mx-auto px-6 py-24 relative">
          <h1 className="text-5xl md:text-6xl font-bold text-stone-50 mb-6">
            PYRAX <span className="pyrax-gradient-text">Brand Hub</span>
          </h1>
          <p className="text-xl text-stone-400 max-w-2xl mb-8">
            Access official PYRAX brand assets, logos, guidelines, and approved marketing materials. 
            Everything you need for press coverage, partnerships, and community content.
          </p>
          <div className="flex flex-wrap gap-4">
            <Link
              href="/media-kit"
              className="inline-flex items-center gap-2 px-6 py-3 bg-pyrax-500 hover:bg-pyrax-600 text-white font-medium rounded-xl transition-colors"
            >
              <Image className="w-5 h-5" />
              View Media Kit
              <ArrowRight className="w-4 h-4" />
            </Link>
            <Link
              href="/public-library"
              className="inline-flex items-center gap-2 px-6 py-3 bg-stone-800 hover:bg-stone-700 text-stone-200 font-medium rounded-xl transition-colors"
            >
              <FileText className="w-5 h-5" />
              Browse Library
            </Link>
          </div>
        </div>
      </div>

      {/* Features */}
      <div className="max-w-6xl mx-auto px-6 py-16">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
            <div className="w-12 h-12 bg-pyrax-500/10 rounded-lg flex items-center justify-center mb-4">
              <Image className="w-6 h-6 text-pyrax-500" />
            </div>
            <h3 className="text-lg font-semibold text-stone-100 mb-2">Official Logos</h3>
            <p className="text-stone-400 text-sm">
              Download PYRAX logos in various formats and color variations for your projects.
            </p>
          </div>
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
            <div className="w-12 h-12 bg-blue-500/10 rounded-lg flex items-center justify-center mb-4">
              <FileText className="w-6 h-6 text-blue-400" />
            </div>
            <h3 className="text-lg font-semibold text-stone-100 mb-2">Brand Guidelines</h3>
            <p className="text-stone-400 text-sm">
              Access comprehensive brand guidelines to ensure consistent representation.
            </p>
          </div>
          <div className="bg-stone-900 border border-stone-800 rounded-xl p-6">
            <div className="w-12 h-12 bg-green-500/10 rounded-lg flex items-center justify-center mb-4">
              <Download className="w-6 h-6 text-green-400" />
            </div>
            <h3 className="text-lg font-semibold text-stone-100 mb-2">Easy Downloads</h3>
            <p className="text-stone-400 text-sm">
              All approved assets are available for immediate download in high quality.
            </p>
          </div>
        </div>
      </div>

      {/* CTA */}
      <div className="max-w-6xl mx-auto px-6 py-16">
        <div className="bg-gradient-to-r from-pyrax-500/20 to-pyrax-600/10 border border-pyrax-500/20 rounded-2xl p-8 md:p-12 text-center">
          <h2 className="text-2xl md:text-3xl font-bold text-stone-100 mb-4">
            Need Something Specific?
          </h2>
          <p className="text-stone-400 max-w-xl mx-auto mb-6">
            Can&apos;t find what you&apos;re looking for? Contact our marketing team for custom assets 
            or special requests.
          </p>
          <a
            href="mailto:marketing@pyrax.org"
            className="inline-flex items-center gap-2 px-6 py-3 bg-pyrax-500 hover:bg-pyrax-600 text-white font-medium rounded-xl transition-colors"
          >
            Contact Marketing Team
          </a>
        </div>
      </div>
    </div>
  );
}
