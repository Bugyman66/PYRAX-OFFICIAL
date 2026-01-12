'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import {
  Download,
  FileText,
  Image as ImageIcon,
  Video,
  Loader2,
  Search,
  Grid,
  List,
  ChevronRight,
  Calendar,
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

export default function PublicLibraryPage() {
  const [assets, setAssets] = useState<PublicAsset[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

  useEffect(() => {
    fetchAssets();
  }, []);

  async function fetchAssets() {
    try {
      const res = await fetch('/api/public-assets?public=true');
      if (!res.ok) throw new Error('Failed to fetch assets');
      const data = await res.json();
      setAssets(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load assets');
    } finally {
      setLoading(false);
    }
  }

  const filteredAssets = assets.filter(asset =>
    asset.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    asset.description?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const getFileIcon = (mimeType: string) => {
    if (mimeType.startsWith('image/')) return ImageIcon;
    if (mimeType.startsWith('video/')) return Video;
    return FileText;
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
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
            <span className="text-stone-300">Public Library</span>
          </div>
          <h1 className="text-4xl md:text-5xl font-bold text-stone-50 mb-4">
            Public <span className="pyrax-gradient-text">Library</span>
          </h1>
          <p className="text-lg text-stone-400 max-w-2xl">
            Browse and download approved PYRAX assets for community use, press coverage, 
            and partnership materials.
          </p>
        </div>
      </div>

      {/* Toolbar */}
      <div className="max-w-6xl mx-auto px-6 py-6">
        <div className="flex flex-col sm:flex-row gap-4 items-center justify-between">
          <div className="relative w-full sm:w-96">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search assets..."
              className="w-full pl-10 pr-4 py-2 bg-stone-900 border border-stone-700 rounded-lg text-stone-200 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
            />
          </div>
          <div className="flex items-center gap-2">
            <span className="text-sm text-stone-500">{filteredAssets.length} assets</span>
            <div className="flex items-center bg-stone-900 border border-stone-700 rounded-lg overflow-hidden">
              <button
                onClick={() => setViewMode('grid')}
                className={`p-2 ${viewMode === 'grid' ? 'bg-stone-800 text-stone-200' : 'text-stone-500 hover:text-stone-300'}`}
              >
                <Grid className="w-5 h-5" />
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`p-2 ${viewMode === 'list' ? 'bg-stone-800 text-stone-200' : 'text-stone-500 hover:text-stone-300'}`}
              >
                <List className="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="max-w-6xl mx-auto px-6 pb-12">
        {error ? (
          <div className="text-center py-12">
            <p className="text-red-400">{error}</p>
          </div>
        ) : filteredAssets.length === 0 ? (
          <div className="text-center py-16">
            <FileText className="w-16 h-16 text-stone-700 mx-auto mb-4" />
            <h2 className="text-xl font-semibold text-stone-300 mb-2">
              {searchQuery ? 'No matching assets' : 'No assets available'}
            </h2>
            <p className="text-stone-500">
              {searchQuery ? 'Try a different search term.' : 'Public assets will be published here soon.'}
            </p>
          </div>
        ) : viewMode === 'grid' ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredAssets.map(asset => {
              const Icon = asset.version ? getFileIcon(asset.version.mimeType) : FileText;
              return (
                <Link
                  key={asset.id}
                  href={`/public-library/${asset.slug}`}
                  className="bg-stone-900 border border-stone-800 rounded-xl overflow-hidden hover:border-stone-700 transition-colors group"
                >
                  <div className="aspect-video bg-stone-800 flex items-center justify-center relative overflow-hidden">
                    {asset.version?.mimeType.startsWith('image/') ? (
                      <img
                        src={`/api/files/${asset.version.id}/stream`}
                        alt={asset.title}
                        className="w-full h-full object-contain group-hover:scale-105 transition-transform duration-300"
                      />
                    ) : (
                      <Icon className="w-16 h-16 text-stone-600" />
                    )}
                  </div>
                  <div className="p-4">
                    <h3 className="font-medium text-stone-100 mb-1 truncate group-hover:text-pyrax-400 transition-colors">
                      {asset.title}
                    </h3>
                    {asset.description && (
                      <p className="text-sm text-stone-400 line-clamp-2 mb-2">
                        {asset.description}
                      </p>
                    )}
                    <div className="flex items-center gap-2 text-xs text-stone-500">
                      <Calendar className="w-3 h-3" />
                      {formatDate(asset.publishedAt)}
                      {asset.downloadAllowed && (
                        <>
                          <span className="text-stone-600">•</span>
                          <Download className="w-3 h-3" />
                          Download available
                        </>
                      )}
                    </div>
                  </div>
                </Link>
              );
            })}
          </div>
        ) : (
          <div className="space-y-3">
            {filteredAssets.map(asset => {
              const Icon = asset.version ? getFileIcon(asset.version.mimeType) : FileText;
              return (
                <Link
                  key={asset.id}
                  href={`/public-library/${asset.slug}`}
                  className="flex items-center gap-4 p-4 bg-stone-900 border border-stone-800 rounded-xl hover:border-stone-700 transition-colors group"
                >
                  <div className="w-16 h-16 bg-stone-800 rounded-lg flex items-center justify-center flex-shrink-0 overflow-hidden">
                    {asset.version?.mimeType.startsWith('image/') ? (
                      <img
                        src={`/api/files/${asset.version.id}/stream`}
                        alt={asset.title}
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <Icon className="w-8 h-8 text-stone-600" />
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-medium text-stone-100 truncate group-hover:text-pyrax-400 transition-colors">
                      {asset.title}
                    </h3>
                    {asset.description && (
                      <p className="text-sm text-stone-400 truncate">
                        {asset.description}
                      </p>
                    )}
                    <div className="flex items-center gap-2 text-xs text-stone-500 mt-1">
                      <Calendar className="w-3 h-3" />
                      {formatDate(asset.publishedAt)}
                    </div>
                  </div>
                  {asset.downloadAllowed && (
                    <div className="flex items-center gap-1 text-xs text-stone-500">
                      <Download className="w-4 h-4" />
                    </div>
                  )}
                </Link>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
