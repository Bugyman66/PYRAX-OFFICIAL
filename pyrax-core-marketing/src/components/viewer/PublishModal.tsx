'use client';

import { useState } from 'react';
import {
  X,
  Globe,
  Download,
  Loader2,
  AlertCircle,
  CheckCircle,
  Link as LinkIcon,
} from 'lucide-react';

interface PublishModalProps {
  proofId: string;
  proofTitle: string;
  onClose: () => void;
  onPublished: () => void;
}

const MEDIA_KIT_SECTIONS = [
  { value: '', label: 'None (Public Library only)' },
  { value: 'logos', label: 'Logos & Wordmarks' },
  { value: 'brand-guidelines', label: 'Brand Guidelines' },
  { value: 'imagery', label: 'Imagery & Graphics' },
  { value: 'videos', label: 'Videos & Motion' },
  { value: 'other', label: 'Other Assets' },
];

export default function PublishModal({ proofId, proofTitle, onClose, onPublished }: PublishModalProps) {
  const [title, setTitle] = useState(proofTitle);
  const [description, setDescription] = useState('');
  const [slug, setSlug] = useState(generateSlug(proofTitle));
  const [downloadAllowed, setDownloadAllowed] = useState(true);
  const [mediaKitSection, setMediaKitSection] = useState('');
  const [publishNow, setPublishNow] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  function generateSlug(text: string): string {
    return text
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .substring(0, 50);
  }

  function handleTitleChange(value: string) {
    setTitle(value);
    setSlug(generateSlug(value));
  }

  async function handlePublish() {
    if (!title.trim()) {
      setError('Title is required');
      return;
    }

    if (!slug.trim()) {
      setError('Slug is required');
      return;
    }

    try {
      setSaving(true);
      setError(null);

      const res = await fetch('/api/public-assets', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          proofId,
          title: title.trim(),
          description: description.trim() || null,
          slug: slug.trim(),
          downloadAllowed,
          mediaKitSection: mediaKitSection || null,
          isPublished: publishNow,
        }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Failed to publish');
      }

      setSuccess(true);
      setTimeout(() => {
        onPublished();
        onClose();
      }, 1500);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to publish');
    } finally {
      setSaving(false);
    }
  }

  if (success) {
    return (
      <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
        <div className="bg-stone-900 border border-stone-800 rounded-xl w-full max-w-md p-8 text-center">
          <div className="w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
            <CheckCircle className="w-8 h-8 text-green-400" />
          </div>
          <h2 className="text-xl font-semibold text-stone-100 mb-2">Published Successfully!</h2>
          <p className="text-stone-400 text-sm">
            Your asset is now available at{' '}
            <span className="text-pyrax-400">/public-library/{slug}</span>
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
      <div className="bg-stone-900 border border-stone-800 rounded-xl w-full max-w-lg max-h-[90vh] overflow-hidden flex flex-col">
        <div className="p-4 border-b border-stone-800 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-pyrax-500/20 rounded-lg flex items-center justify-center">
              <Globe className="w-5 h-5 text-pyrax-400" />
            </div>
            <div>
              <h2 className="font-semibold text-stone-100">Publish to Public Library</h2>
              <p className="text-xs text-stone-500">Make this asset publicly available</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-stone-800 rounded-lg"
          >
            <X className="w-5 h-5 text-stone-400" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {error && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3 flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-red-400 flex-shrink-0" />
              <span className="text-red-400 text-sm">{error}</span>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-stone-300 mb-1">Title</label>
            <input
              type="text"
              value={title}
              onChange={e => handleTitleChange(e.target.value)}
              placeholder="Asset title"
              className="w-full px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-stone-300 mb-1">Description</label>
            <textarea
              value={description}
              onChange={e => setDescription(e.target.value)}
              placeholder="Brief description of the asset..."
              rows={2}
              className="w-full px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 resize-none"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-stone-300 mb-1">
              URL Slug
            </label>
            <div className="flex items-center gap-2">
              <span className="text-stone-500 text-sm">/public-library/</span>
              <input
                type="text"
                value={slug}
                onChange={e => setSlug(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ''))}
                placeholder="asset-slug"
                className="flex-1 px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
              />
            </div>
            <p className="text-xs text-stone-500 mt-1">
              This will be the public URL for accessing this asset
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-stone-300 mb-1">
              Media Kit Section
            </label>
            <select
              value={mediaKitSection}
              onChange={e => setMediaKitSection(e.target.value)}
              className="w-full px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-100 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
            >
              {MEDIA_KIT_SECTIONS.map(section => (
                <option key={section.value} value={section.value}>
                  {section.label}
                </option>
              ))}
            </select>
            <p className="text-xs text-stone-500 mt-1">
              If set, the asset will appear in the Media Kit page
            </p>
          </div>

          <div className="space-y-3 pt-2">
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={downloadAllowed}
                onChange={e => setDownloadAllowed(e.target.checked)}
                className="w-4 h-4 rounded border-stone-600 bg-stone-800 text-pyrax-500 focus:ring-pyrax-500"
              />
              <div className="flex items-center gap-2">
                <Download className="w-4 h-4 text-stone-400" />
                <span className="text-sm text-stone-300">Allow public downloads</span>
              </div>
            </label>

            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={publishNow}
                onChange={e => setPublishNow(e.target.checked)}
                className="w-4 h-4 rounded border-stone-600 bg-stone-800 text-pyrax-500 focus:ring-pyrax-500"
              />
              <div className="flex items-center gap-2">
                <Globe className="w-4 h-4 text-stone-400" />
                <span className="text-sm text-stone-300">Publish immediately</span>
              </div>
            </label>
          </div>
        </div>

        <div className="p-4 border-t border-stone-800 flex items-center justify-between">
          <div className="flex items-center gap-2 text-xs text-stone-500">
            <LinkIcon className="w-3 h-3" />
            <span>Public URL will be generated</span>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-stone-400 hover:text-stone-300"
            >
              Cancel
            </button>
            <button
              onClick={handlePublish}
              disabled={saving}
              className="flex items-center gap-2 px-4 py-2 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white rounded-lg transition-colors"
            >
              {saving && <Loader2 className="w-4 h-4 animate-spin" />}
              {publishNow ? 'Publish Now' : 'Save as Draft'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
