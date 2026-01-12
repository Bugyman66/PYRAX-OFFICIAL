'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  Download,
  FileText,
  Image as ImageIcon,
  Video,
  Loader2,
  ChevronRight,
  Calendar,
  ArrowLeft,
  ExternalLink,
  ZoomIn,
  ZoomOut,
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
    fileSize: number | null;
  } | null;
}

export default function AssetDetailPage({ params }: { params: { slug: string } }) {
  const { slug } = params;
  const router = useRouter();
  const [asset, setAsset] = useState<PublicAsset | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [zoom, setZoom] = useState(100);

  useEffect(() => {
    fetchAsset();
  }, [slug]);

  async function fetchAsset() {
    try {
      const res = await fetch(`/api/public-assets/${slug}?public=true`);
      if (res.status === 404) {
        router.push('/public-library');
        return;
      }
      if (!res.ok) throw new Error('Failed to fetch asset');
      const data = await res.json();
      setAsset(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load asset');
    } finally {
      setLoading(false);
    }
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  const formatFileSize = (bytes: number | null) => {
    if (!bytes) return 'Unknown size';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
  };

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

  if (error || !asset) {
    return (
      <div className="min-h-screen bg-stone-950 flex flex-col items-center justify-center gap-4">
        <FileText className="w-16 h-16 text-stone-700" />
        <p className="text-stone-400">{error || 'Asset not found'}</p>
        <Link href="/public-library" className="text-pyrax-400 hover:text-pyrax-300">
          Back to Library
        </Link>
      </div>
    );
  }

  const isImage = asset.version?.mimeType.startsWith('image/');
  const isVideo = asset.version?.mimeType.startsWith('video/');
  const Icon = asset.version ? getFileIcon(asset.version.mimeType) : FileText;
  const streamUrl = asset.version ? `/api/files/${asset.version.id}/stream` : null;

  return (
    <div className="min-h-screen bg-stone-950">
      {/* Header */}
      <div className="bg-stone-900 border-b border-stone-800">
        <div className="max-w-6xl mx-auto px-6 py-4">
          <div className="flex items-center gap-2 text-stone-500 text-sm">
            <Link href="/" className="hover:text-stone-300">Home</Link>
            <ChevronRight className="w-4 h-4" />
            <Link href="/public-library" className="hover:text-stone-300">Public Library</Link>
            <ChevronRight className="w-4 h-4" />
            <span className="text-stone-300 truncate max-w-[200px]">{asset.title}</span>
          </div>
        </div>
      </div>

      <div className="max-w-6xl mx-auto px-6 py-8">
        <Link
          href="/public-library"
          className="inline-flex items-center gap-2 text-stone-400 hover:text-stone-200 mb-6"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Library
        </Link>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Preview */}
          <div className="lg:col-span-2">
            <div className="bg-stone-900 border border-stone-800 rounded-xl overflow-hidden">
              {/* Preview Controls */}
              {isImage && (
                <div className="flex items-center justify-end gap-2 p-3 border-b border-stone-800">
                  <button
                    onClick={() => setZoom(Math.max(50, zoom - 25))}
                    className="p-2 text-stone-400 hover:text-stone-200 hover:bg-stone-800 rounded-lg"
                  >
                    <ZoomOut className="w-4 h-4" />
                  </button>
                  <span className="text-sm text-stone-400 min-w-[50px] text-center">{zoom}%</span>
                  <button
                    onClick={() => setZoom(Math.min(200, zoom + 25))}
                    className="p-2 text-stone-400 hover:text-stone-200 hover:bg-stone-800 rounded-lg"
                  >
                    <ZoomIn className="w-4 h-4" />
                  </button>
                </div>
              )}

              {/* Preview Content */}
              <div className="aspect-video bg-stone-800 flex items-center justify-center overflow-auto">
                {isImage && streamUrl ? (
                  <img
                    src={streamUrl}
                    alt={asset.title}
                    style={{ transform: `scale(${zoom / 100})` }}
                    className="max-w-full max-h-full object-contain transition-transform"
                  />
                ) : isVideo && streamUrl ? (
                  <video
                    src={streamUrl}
                    controls
                    className="w-full h-full"
                  />
                ) : (
                  <div className="text-center">
                    <Icon className="w-24 h-24 text-stone-600 mx-auto mb-4" />
                    <p className="text-stone-500">Preview not available</p>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Details */}
          <div className="space-y-6">
            <div>
              <h1 className="text-2xl font-bold text-stone-100 mb-2">{asset.title}</h1>
              {asset.description && (
                <p className="text-stone-400">{asset.description}</p>
              )}
            </div>

            {/* Meta */}
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-4 space-y-3">
              <div className="flex items-center justify-between text-sm">
                <span className="text-stone-500">Published</span>
                <span className="text-stone-300 flex items-center gap-1">
                  <Calendar className="w-4 h-4" />
                  {formatDate(asset.publishedAt)}
                </span>
              </div>
              {asset.version && (
                <>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-stone-500">File Name</span>
                    <span className="text-stone-300 truncate max-w-[180px]" title={asset.version.fileName}>
                      {asset.version.fileName}
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-stone-500">Type</span>
                    <span className="text-stone-300">{asset.version.mimeType}</span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-stone-500">Size</span>
                    <span className="text-stone-300">{formatFileSize(asset.version.fileSize)}</span>
                  </div>
                </>
              )}
              {asset.mediaKitSection && (
                <div className="flex items-center justify-between text-sm">
                  <span className="text-stone-500">Category</span>
                  <span className="text-stone-300 capitalize">{asset.mediaKitSection.replace(/-/g, ' ')}</span>
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="space-y-3">
              {asset.downloadAllowed && streamUrl ? (
                <a
                  href={streamUrl}
                  download={asset.version?.fileName}
                  className="w-full flex items-center justify-center gap-2 px-6 py-3 bg-pyrax-500 hover:bg-pyrax-600 text-white font-medium rounded-xl transition-colors"
                >
                  <Download className="w-5 h-5" />
                  Download Asset
                </a>
              ) : (
                <div className="w-full flex items-center justify-center gap-2 px-6 py-3 bg-stone-800 text-stone-500 font-medium rounded-xl cursor-not-allowed">
                  <Download className="w-5 h-5" />
                  Download Not Available
                </div>
              )}

              {streamUrl && (
                <a
                  href={streamUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full flex items-center justify-center gap-2 px-6 py-3 bg-stone-800 hover:bg-stone-700 text-stone-200 font-medium rounded-xl transition-colors"
                >
                  <ExternalLink className="w-5 h-5" />
                  Open in New Tab
                </a>
              )}
            </div>

            {/* Usage Notice */}
            <div className="bg-stone-900/50 border border-stone-800 rounded-xl p-4">
              <h3 className="text-sm font-medium text-stone-300 mb-2">Usage Guidelines</h3>
              <p className="text-xs text-stone-500">
                This asset is provided for approved use cases only. Please review our 
                <Link href="/media-kit" className="text-pyrax-400 hover:text-pyrax-300 ml-1">
                  brand guidelines
                </Link> before using.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
