'use client';

import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  ArrowLeft,
  FileText,
  Upload,
  Loader2,
  Clock,
  User,
  FolderOpen,
  Download,
  Eye,
  Trash2,
  MoreVertical,
  ChevronDown,
  MessageSquare,
  PenTool,
} from 'lucide-react';

interface Version {
  id: string;
  versionNumber: number;
  driveFileId: string;
  fileName: string;
  mimeType: string;
  fileSize: number | null;
  kind: string;
  createdBy: { id: string; name: string | null; email: string };
  createdAt: string;
  _count: { annotations: number; comments: number };
}

interface Proof {
  id: string;
  title: string;
  description: string | null;
  status: string;
  folder: { id: string; name: string; departmentId: string };
  createdBy: { id: string; name: string | null; email: string };
  versions: Version[];
  workflowInstance: {
    id: string;
    currentStepIndex: number;
    status: string;
    template: { id: string; name: string };
  } | null;
  createdAt: string;
  updatedAt: string;
}

const statusColors: Record<string, string> = {
  Draft: 'bg-stone-500/20 text-stone-300 border-stone-500/30',
  InReview: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  Approved: 'bg-green-500/20 text-green-400 border-green-500/30',
  Rejected: 'bg-red-500/20 text-red-400 border-red-500/30',
  Published: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
};

function formatFileSize(bytes: number | null): string {
  if (!bytes) return 'Unknown';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function ProofDetailPage({ params }: { params: { proofId: string } }) {
  const { proofId } = params;
  const router = useRouter();
  const [proof, setProof] = useState<Proof | null>(null);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [menuOpen, setMenuOpen] = useState<string | null>(null);
  const [dragActive, setDragActive] = useState(false);

  const fetchProof = useCallback(async () => {
    try {
      const res = await fetch(`/api/proofs/${proofId}`);
      if (res.ok) {
        const data = await res.json();
        setProof(data.proof);
      } else if (res.status === 404) {
        router.push('/app/folders');
      }
    } catch (error) {
      console.error('Failed to fetch proof:', error);
    } finally {
      setLoading(false);
    }
  }, [proofId, router]);

  useEffect(() => {
    fetchProof();
  }, [fetchProof]);

  const handleUpload = async (file: File) => {
    if (!file) return;

    setUploading(true);
    try {
      const formData = new FormData();
      formData.append('file', file);

      const res = await fetch(`/api/proofs/${proofId}/versions`, {
        method: 'POST',
        body: formData,
      });

      if (res.ok) {
        fetchProof();
      } else {
        const data = await res.json();
        alert(data.error || 'Failed to upload version');
      }
    } catch (error) {
      console.error('Failed to upload:', error);
      alert('Failed to upload file');
    } finally {
      setUploading(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragActive(false);
    const file = e.dataTransfer.files[0];
    if (file) handleUpload(file);
  };

  const handleDeleteVersion = async (versionId: string) => {
    if (!confirm('Are you sure you want to delete this version?')) return;

    try {
      const res = await fetch(`/api/proofs/${proofId}/versions/${versionId}`, {
        method: 'DELETE',
      });
      if (res.ok) {
        fetchProof();
      } else {
        const data = await res.json();
        alert(data.error || 'Failed to delete version');
      }
    } catch (error) {
      console.error('Failed to delete version:', error);
      alert('Failed to delete version');
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

  if (!proof) {
    return (
      <div className="text-center py-12">
        <p className="text-stone-400">Proof not found</p>
      </div>
    );
  }

  const latestVersion = proof.versions[0];

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <Link
            href={`/app/folders/${proof.folder.id}`}
            className="inline-flex items-center gap-1.5 text-sm text-stone-400 hover:text-stone-50 mb-3 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to {proof.folder.name}
          </Link>
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 bg-pyrax-500/10 rounded-xl flex items-center justify-center">
              <FileText className="w-6 h-6 text-pyrax-500" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-stone-50">{proof.title}</h1>
              <div className="flex items-center gap-3 mt-1">
                <span className={`inline-flex px-2.5 py-0.5 text-xs font-medium rounded-full border ${statusColors[proof.status]}`}>
                  {proof.status}
                </span>
                <span className="text-stone-500 text-sm">
                  {proof.versions.length} {proof.versions.length === 1 ? 'version' : 'versions'}
                </span>
              </div>
            </div>
          </div>
          {proof.description && (
            <p className="text-stone-400 mt-3 max-w-2xl">{proof.description}</p>
          )}
        </div>
        <div className="flex items-center gap-3">
          {latestVersion && (
            <Link
              href={`/app/viewer/${latestVersion.id}`}
              className="inline-flex items-center gap-2 px-4 py-2 bg-stone-800 hover:bg-stone-700 text-stone-50 rounded-lg transition-colors"
            >
              <Eye className="w-5 h-5" />
              View Latest
            </Link>
          )}
          <label className="inline-flex items-center gap-2 px-4 py-2 pyrax-gradient text-white font-medium rounded-lg hover:opacity-90 transition-opacity cursor-pointer">
            <Upload className="w-5 h-5" />
            Upload Version
            <input
              type="file"
              className="hidden"
              onChange={(e) => e.target.files?.[0] && handleUpload(e.target.files[0])}
              disabled={uploading}
            />
          </label>
        </div>
      </div>

      <div className="flex items-center gap-6 text-sm text-stone-500">
        <div className="flex items-center gap-1.5">
          <FolderOpen className="w-4 h-4" />
          <Link href={`/app/folders/${proof.folder.id}`} className="hover:text-stone-300 transition-colors">
            {proof.folder.name}
          </Link>
        </div>
        <div className="flex items-center gap-1.5">
          <User className="w-4 h-4" />
          {proof.createdBy.name || proof.createdBy.email}
        </div>
        <div className="flex items-center gap-1.5">
          <Clock className="w-4 h-4" />
          Updated {new Date(proof.updatedAt).toLocaleDateString()}
        </div>
      </div>

      <div
        className={`border-2 border-dashed rounded-xl p-8 text-center transition-colors ${
          dragActive
            ? 'border-pyrax-500 bg-pyrax-500/10'
            : 'border-stone-700 hover:border-stone-600'
        }`}
        onDragOver={(e) => { e.preventDefault(); setDragActive(true); }}
        onDragLeave={() => setDragActive(false)}
        onDrop={handleDrop}
      >
        {uploading ? (
          <div className="flex items-center justify-center gap-3">
            <Loader2 className="w-6 h-6 text-pyrax-500 animate-spin" />
            <span className="text-stone-300">Uploading...</span>
          </div>
        ) : (
          <>
            <Upload className="w-10 h-10 text-stone-500 mx-auto mb-3" />
            <p className="text-stone-300 mb-1">Drag and drop a file to upload a new version</p>
            <p className="text-stone-500 text-sm">or click the Upload Version button above</p>
          </>
        )}
      </div>

      <div className="bg-stone-900 rounded-xl border border-stone-800">
        <div className="p-4 border-b border-stone-800">
          <h2 className="text-lg font-semibold text-stone-50">Version History</h2>
        </div>
        {proof.versions.length === 0 ? (
          <div className="p-8 text-center">
            <FileText className="w-12 h-12 text-stone-600 mx-auto mb-4" />
            <h3 className="text-stone-300 font-medium mb-2">No versions yet</h3>
            <p className="text-stone-500 text-sm">Upload your first file to create version 1</p>
          </div>
        ) : (
          <div className="divide-y divide-stone-800">
            {proof.versions.map((version) => (
              <div
                key={version.id}
                className="p-4 hover:bg-stone-800/50 transition-colors flex items-center justify-between"
              >
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 bg-stone-800 rounded-lg flex items-center justify-center text-stone-400 font-mono text-sm">
                    v{version.versionNumber}
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-stone-50 font-medium">{version.fileName}</span>
                      {version.kind === 'Rendition' && (
                        <span className="px-1.5 py-0.5 text-xs bg-blue-500/20 text-blue-400 rounded">
                          Rendition
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-3 text-sm text-stone-500 mt-0.5">
                      <span>{formatFileSize(version.fileSize)}</span>
                      <span>•</span>
                      <span>{version.createdBy.name || version.createdBy.email}</span>
                      <span>•</span>
                      <span>{new Date(version.createdAt).toLocaleString()}</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <div className="flex items-center gap-3 text-sm text-stone-500 mr-4">
                    <span className="flex items-center gap-1">
                      <PenTool className="w-3.5 h-3.5" />
                      {version._count.annotations}
                    </span>
                    <span className="flex items-center gap-1">
                      <MessageSquare className="w-3.5 h-3.5" />
                      {version._count.comments}
                    </span>
                  </div>
                  <Link
                    href={`/app/viewer/${version.id}`}
                    className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-700 rounded-lg transition-colors"
                    title="View"
                  >
                    <Eye className="w-4 h-4" />
                  </Link>
                  <a
                    href={`/api/drive/stream/${version.driveFileId}`}
                    download={version.fileName}
                    className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-700 rounded-lg transition-colors"
                    title="Download"
                  >
                    <Download className="w-4 h-4" />
                  </a>
                  <div className="relative">
                    <button
                      onClick={() => setMenuOpen(menuOpen === version.id ? null : version.id)}
                      className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-700 rounded-lg transition-colors"
                    >
                      <MoreVertical className="w-4 h-4" />
                    </button>
                    {menuOpen === version.id && (
                      <div className="absolute right-0 top-full mt-1 w-36 bg-stone-800 border border-stone-700 rounded-lg shadow-xl z-10">
                        <button
                          onClick={() => handleDeleteVersion(version.id)}
                          className="flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-stone-700 w-full rounded-lg"
                        >
                          <Trash2 className="w-4 h-4" />
                          Delete
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {proof.workflowInstance && (
        <div className="bg-stone-900 rounded-xl border border-stone-800 p-6">
          <h2 className="text-lg font-semibold text-stone-50 mb-4">Workflow</h2>
          <div className="flex items-center gap-4">
            <span className="text-stone-400">Template:</span>
            <span className="text-stone-50">{proof.workflowInstance.template.name}</span>
            <span className="text-stone-600">•</span>
            <span className="text-stone-400">Status:</span>
            <span className={`px-2 py-0.5 text-xs rounded-full ${
              proof.workflowInstance.status === 'Active' 
                ? 'bg-blue-500/20 text-blue-400' 
                : proof.workflowInstance.status === 'Completed'
                ? 'bg-green-500/20 text-green-400'
                : 'bg-stone-500/20 text-stone-400'
            }`}>
              {proof.workflowInstance.status}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
