'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import {
  Download,
  FileText,
  Image as ImageIcon,
  Video,
  Loader2,
  ExternalLink,
  ChevronRight,
} from 'lucide-react';

interface PublicAsset {
  id: string;
  slug: string;
  title: string;
  description: string | null;
  downloadAllowed: boolean;
  mediaKitSection: string | null;
  publishedAt: string;
  version: {
    id: string;
    fileName: string;
    mimeType: string;
  } | null;
}

const SECTION_ORDER = ['logos', 'brand-guidelines', 'imagery', 'videos', 'other'];

const SECTION_LABELS: Record<string, string> = {
  'logos': 'Logos & Wordmarks',
  'brand-guidelines': 'Brand Guidelines',
  'imagery': 'Imagery & Graphics',
  'videos': 'Videos & Motion',
  'other': 'Other Assets',
};

export default function MediaKitPage() {
  const [assets, setAssets] = useState<PublicAsset[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchAssets();
  }, []);

  async function fetchAssets() {
    try {
      const res = await fetch('/api/public-assets?public=true');
      if (!res.ok) throw new Error('Failed to fetch assets');
      const data = await res.json();
      setAssets(data.filter((a: PublicAsset) => a.mediaKitSection));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load assets');
    } finally {
      setLoading(false);
    }
  }

  const groupedAssets = assets.reduce((acc, asset) => {
    const section = asset.mediaKitSection || 'other';
    if (!acc[section]) {
      acc[section] = [];
    }
    acc[section].push(asset);
    return acc;
  }, {} as Record<string, PublicAsset[]>);

  const sortedSections = Object.keys(groupedAssets).sort((a, b) => {
    const aIndex = SECTION_ORDER.indexOf(a);
    const bIndex = SECTION_ORDER.indexOf(b);
    if (aIndex === -1 && bIndex === -1) return a.localeCompare(b);
    if (aIndex === -1) return 1;
    if (bIndex === -1) return -1;
    return aIndex - bIndex;
  });

  const getFileIcon = (mimeType: string) => {
    if (mimeType.startsWith('image/')) return ImageIcon;
    if (mimeType.startsWith('video/')) return Video;
    return FileText;
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-stone-950 flex items-center justify-center">
        <Loader2 className="w-8 h-8 text-pyrax-500 animate-spin" />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-stone-950">
      {/* Hero */}
      <div className="bg-gradient-to-b from-stone-900 to-stone-950 border-b border-stone-800">
        <div className="max-w-6xl mx-auto px-6 py-16">
          <div className="flex items-center gap-2 text-stone-500 text-sm mb-4">
            <Link href="/" className="hover:text-stone-300">Home</Link>
            <ChevronRight className="w-4 h-4" />
            <span className="text-stone-300">Media Kit</span>
          </div>
          <h1 className="text-4xl md:text-5xl font-bold text-stone-50 mb-4">
            PYRAX <span className="pyrax-gradient-text">Media Kit</span>
          </h1>
          <p className="text-lg text-stone-400 max-w-2xl">
            Official brand assets for PYRAX. Download logos, brand guidelines, and approved imagery 
            for use in press, partnerships, and community content.
          </p>
        </div>
      </div>

      {/* Content */}
      <div className="max-w-6xl mx-auto px-6 py-12">
        {error ? (
          <div className="text-center py-12">
            <p className="text-red-400">{error}</p>
          </div>
        ) : sortedSections.length === 0 ? (
          <div className="text-center py-16">
            <FileText className="w-16 h-16 text-stone-700 mx-auto mb-4" />
            <h2 className="text-xl font-semibold text-stone-300 mb-2">No assets available</h2>
            <p className="text-stone-500">Media kit assets will be published here soon.</p>
          </div>
        ) : (
          <div className="space-y-12">
            {sortedSections.map(section => (
              <section key={section}>
                <h2 className="text-2xl font-semibold text-stone-100 mb-6">
                  {SECTION_LABELS[section] || section}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                  {groupedAssets[section].map(asset => {
                    const Icon = asset.version ? getFileIcon(asset.version.mimeType) : FileText;
                    return (
                      <div
                        key={asset.id}
                        className="bg-stone-900 border border-stone-800 rounded-xl overflow-hidden hover:border-stone-700 transition-colors group"
                      >
                        {/* Preview */}
                        <div className="aspect-video bg-stone-800 flex items-center justify-center relative overflow-hidden">
                          {asset.version?.mimeType.startsWith('image/') ? (
                            <img
                              src={`/api/files/${asset.version.id}/stream`}
                              alt={asset.title}
                              className="w-full h-full object-contain"
                            />
                          ) : (
                            <Icon className="w-16 h-16 text-stone-600" />
                          )}
                        </div>

                        {/* Info */}
                        <div className="p-4">
                          <h3 className="font-medium text-stone-100 mb-1 truncate">
                            {asset.title}
                          </h3>
                          {asset.description && (
                            <p className="text-sm text-stone-400 line-clamp-2 mb-3">
                              {asset.description}
                            </p>
                          )}
                          <div className="flex items-center gap-2">
                            <Link
                              href={`/public-library/${asset.slug}`}
                              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-stone-800 hover:bg-stone-700 text-stone-200 rounded-lg text-sm transition-colors"
                            >
                              <ExternalLink className="w-4 h-4" />
                              View
                            </Link>
                            {asset.downloadAllowed && asset.version && (
                              <a
                                href={`/api/files/${asset.version.id}/stream`}
                                download={asset.version.fileName}
                                className="flex items-center justify-center gap-2 px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 text-white rounded-lg text-sm transition-colors"
                              >
                                <Download className="w-4 h-4" />
                                Download
                              </a>
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </section>
            ))}
          </div>
        )}

        {/* Usage Guidelines */}
        <div className="mt-16 bg-stone-900 border border-stone-800 rounded-xl p-8">
          <h2 className="text-xl font-semibold text-stone-100 mb-4">Usage Guidelines</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8 text-stone-400">
            <div>
              <h3 className="font-medium text-stone-200 mb-2">Do</h3>
              <ul className="space-y-2 text-sm">
                <li>• Use official logos from this media kit</li>
                <li>• Maintain proper spacing around logos</li>
                <li>• Use approved color variations</li>
                <li>• Credit PYRAX when using our assets</li>
              </ul>
            </div>
            <div>
              <h3 className="font-medium text-stone-200 mb-2">Don&apos;t</h3>
              <ul className="space-y-2 text-sm">
                <li>• Modify or distort the logo</li>
                <li>• Change brand colors</li>
                <li>• Use assets for misleading purposes</li>
                <li>• Claim ownership of PYRAX assets</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
