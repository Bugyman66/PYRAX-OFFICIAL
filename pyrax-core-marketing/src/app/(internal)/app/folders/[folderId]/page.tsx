'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  FolderOpen,
  Plus,
  ArrowLeft,
  FileText,
  MoreVertical,
  Edit2,
  Trash2,
  Loader2,
  Clock,
  User,
  Eye,
  Upload,
} from 'lucide-react';

interface Proof {
  id: string;
  title: string;
  description: string | null;
  status: string;
  createdBy: { id: string; name: string | null; email: string };
  versions: Array<{
    id: string;
    versionNumber: number;
    fileName: string;
    mimeType: string;
  }>;
  _count: { versions: number };
  createdAt: string;
  updatedAt: string;
}

interface Folder {
  id: string;
  name: string;
  description: string | null;
  department: { id: string; name: string };
  createdBy: { id: string; name: string | null; email: string };
  proofs: Proof[];
  createdAt: string;
}

const statusColors: Record<string, string> = {
  Draft: 'bg-stone-500/20 text-stone-300',
  InReview: 'bg-blue-500/20 text-blue-400',
  Approved: 'bg-green-500/20 text-green-400',
  Rejected: 'bg-red-500/20 text-red-400',
  Published: 'bg-purple-500/20 text-purple-400',
};

export default function FolderDetailPage({ params }: { params: { folderId: string } }) {
  const { folderId } = params;
  const router = useRouter();
  const [folder, setFolder] = useState<Folder | null>(null);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newProof, setNewProof] = useState({ title: '', description: '' });
  const [menuOpen, setMenuOpen] = useState<string | null>(null);

  useEffect(() => {
    fetchFolder();
  }, [folderId]);

  const fetchFolder = async () => {
    try {
      const res = await fetch(`/api/folders/${folderId}`);
      if (res.ok) {
        const data = await res.json();
        setFolder(data.folder);
      } else if (res.status === 404) {
        router.push('/app/folders');
      }
    } catch (error) {
      console.error('Failed to fetch folder:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateProof = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newProof.title.trim()) return;

    setCreating(true);
    try {
      const res = await fetch('/api/proofs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...newProof, folderId }),
      });

      if (res.ok) {
        const data = await res.json();
        setShowCreateModal(false);
        setNewProof({ title: '', description: '' });
        router.push(`/app/proofs/${data.proof.id}`);
      } else {
        const data = await res.json();
        alert(data.error || 'Failed to create proof');
      }
    } catch (error) {
      console.error('Failed to create proof:', error);
      alert('Failed to create proof');
    } finally {
      setCreating(false);
    }
  };

  const handleDeleteProof = async (proofId: string) => {
    if (!confirm('Are you sure you want to delete this proof? All versions will be deleted.')) return;

    try {
      const res = await fetch(`/api/proofs/${proofId}`, { method: 'DELETE' });
      if (res.ok) {
        fetchFolder();
      } else {
        const data = await res.json();
        alert(data.error || 'Failed to delete proof');
      }
    } catch (error) {
      console.error('Failed to delete proof:', error);
      alert('Failed to delete proof');
    }
    setMenuOpen(null);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
      </div>
    );
  }

  if (!folder) {
    return (
      <div className="text-center py-12">
        <p className="text-stone-400">Folder not found</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <Link
            href="/app/folders"
            className="inline-flex items-center gap-1.5 text-sm text-stone-400 hover:text-stone-50 mb-3 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Folders
          </Link>
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 bg-pyrax-500/10 rounded-xl flex items-center justify-center">
              <FolderOpen className="w-6 h-6 text-pyrax-500" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-stone-50">{folder.name}</h1>
              <p className="text-stone-400 text-sm">{folder.department.name}</p>
            </div>
          </div>
          {folder.description && (
            <p className="text-stone-400 mt-3 max-w-2xl">{folder.description}</p>
          )}
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="inline-flex items-center gap-2 px-4 py-2 pyrax-gradient text-white font-medium rounded-lg hover:opacity-90 transition-opacity"
        >
          <Plus className="w-5 h-5" />
          New Proof
        </button>
      </div>

      <div className="flex items-center gap-4 text-sm text-stone-500">
        <div className="flex items-center gap-1.5">
          <FileText className="w-4 h-4" />
          {folder.proofs.length} {folder.proofs.length === 1 ? 'proof' : 'proofs'}
        </div>
        <div className="flex items-center gap-1.5">
          <User className="w-4 h-4" />
          Created by {folder.createdBy.name || folder.createdBy.email}
        </div>
        <div className="flex items-center gap-1.5">
          <Clock className="w-4 h-4" />
          {new Date(folder.createdAt).toLocaleDateString()}
        </div>
      </div>

      {folder.proofs.length === 0 ? (
        <div className="text-center py-12 bg-stone-900 rounded-xl border border-stone-800">
          <FileText className="w-12 h-12 text-stone-600 mx-auto mb-4" />
          <h3 className="text-stone-300 font-medium mb-2">No proofs yet</h3>
          <p className="text-stone-500 text-sm mb-4">
            Create your first proof in this folder
          </p>
          <button
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center gap-2 px-4 py-2 bg-stone-800 hover:bg-stone-700 text-stone-50 rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" />
            Create Proof
          </button>
        </div>
      ) : (
        <div className="bg-stone-900 rounded-xl border border-stone-800 overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-stone-800">
                <th className="text-left text-xs font-medium text-stone-400 uppercase tracking-wider px-6 py-3">
                  Proof
                </th>
                <th className="text-left text-xs font-medium text-stone-400 uppercase tracking-wider px-6 py-3">
                  Status
                </th>
                <th className="text-left text-xs font-medium text-stone-400 uppercase tracking-wider px-6 py-3">
                  Versions
                </th>
                <th className="text-left text-xs font-medium text-stone-400 uppercase tracking-wider px-6 py-3">
                  Updated
                </th>
                <th className="w-12"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {folder.proofs.map((proof) => (
                <tr key={proof.id} className="hover:bg-stone-800/50 transition-colors">
                  <td className="px-6 py-4">
                    <Link href={`/app/proofs/${proof.id}`} className="group">
                      <div className="font-medium text-stone-50 group-hover:text-pyrax-400 transition-colors">
                        {proof.title}
                      </div>
                      {proof.description && (
                        <div className="text-sm text-stone-500 truncate max-w-xs">
                          {proof.description}
                        </div>
                      )}
                    </Link>
                  </td>
                  <td className="px-6 py-4">
                    <span className={`inline-flex px-2.5 py-1 text-xs font-medium rounded-full ${statusColors[proof.status] || statusColors.Draft}`}>
                      {proof.status}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-stone-400">
                    {proof._count.versions}
                  </td>
                  <td className="px-6 py-4 text-stone-500 text-sm">
                    {new Date(proof.updatedAt).toLocaleDateString()}
                  </td>
                  <td className="px-6 py-4">
                    <div className="relative">
                      <button
                        onClick={() => setMenuOpen(menuOpen === proof.id ? null : proof.id)}
                        className="p-1.5 text-stone-500 hover:text-stone-300 rounded-lg hover:bg-stone-700 transition-colors"
                      >
                        <MoreVertical className="w-4 h-4" />
                      </button>
                      {menuOpen === proof.id && (
                        <div className="absolute right-0 top-full mt-1 w-40 bg-stone-800 border border-stone-700 rounded-lg shadow-xl z-10">
                          <Link
                            href={`/app/proofs/${proof.id}`}
                            className="flex items-center gap-2 px-3 py-2 text-sm text-stone-300 hover:bg-stone-700 rounded-t-lg"
                            onClick={() => setMenuOpen(null)}
                          >
                            <Eye className="w-4 h-4" />
                            View
                          </Link>
                          <Link
                            href={`/app/proofs/${proof.id}/upload`}
                            className="flex items-center gap-2 px-3 py-2 text-sm text-stone-300 hover:bg-stone-700"
                            onClick={() => setMenuOpen(null)}
                          >
                            <Upload className="w-4 h-4" />
                            Upload Version
                          </Link>
                          <Link
                            href={`/app/proofs/${proof.id}/edit`}
                            className="flex items-center gap-2 px-3 py-2 text-sm text-stone-300 hover:bg-stone-700"
                            onClick={() => setMenuOpen(null)}
                          >
                            <Edit2 className="w-4 h-4" />
                            Edit
                          </Link>
                          <button
                            onClick={() => handleDeleteProof(proof.id)}
                            className="flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-stone-700 w-full rounded-b-lg"
                          >
                            <Trash2 className="w-4 h-4" />
                            Delete
                          </button>
                        </div>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-stone-900 rounded-xl border border-stone-700 w-full max-w-md">
            <div className="p-6 border-b border-stone-800">
              <h2 className="text-lg font-semibold text-stone-50">Create New Proof</h2>
              <p className="text-sm text-stone-400 mt-1">in {folder.name}</p>
            </div>
            <form onSubmit={handleCreateProof} className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-2">
                  Proof Title *
                </label>
                <input
                  type="text"
                  value={newProof.title}
                  onChange={(e) => setNewProof({ ...newProof, title: e.target.value })}
                  placeholder="e.g., Homepage Banner v2"
                  className="w-full px-4 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-50 placeholder:text-stone-500 focus:outline-none focus:border-pyrax-500"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-2">
                  Description
                </label>
                <textarea
                  value={newProof.description}
                  onChange={(e) => setNewProof({ ...newProof, description: e.target.value })}
                  placeholder="Optional description..."
                  rows={3}
                  className="w-full px-4 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-50 placeholder:text-stone-500 focus:outline-none focus:border-pyrax-500 resize-none"
                />
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-stone-400 hover:text-stone-50 transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creating || !newProof.title.trim()}
                  className="inline-flex items-center gap-2 px-4 py-2 pyrax-gradient text-white font-medium rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50"
                >
                  {creating && <Loader2 className="w-4 h-4 animate-spin" />}
                  Create Proof
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
