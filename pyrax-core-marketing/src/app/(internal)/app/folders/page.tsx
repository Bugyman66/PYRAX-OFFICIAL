'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import {
  FolderOpen,
  Plus,
  Search,
  MoreVertical,
  Edit2,
  Trash2,
  Loader2,
  FileText,
  Building2,
} from 'lucide-react';

interface Department {
  id: string;
  name: string;
}

interface Folder {
  id: string;
  name: string;
  description: string | null;
  department: Department;
  createdBy: { id: string; name: string | null; email: string };
  proofCount: number;
  createdAt: string;
  updatedAt: string;
}

export default function FoldersPage() {
  const [folders, setFolders] = useState<Folder[]>([]);
  const [departments, setDepartments] = useState<Department[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [selectedDepartment, setSelectedDepartment] = useState<string>('');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newFolder, setNewFolder] = useState({ name: '', description: '', departmentId: '' });
  const [menuOpen, setMenuOpen] = useState<string | null>(null);

  useEffect(() => {
    fetchDepartments();
    fetchFolders();
  }, []);

  useEffect(() => {
    fetchFolders();
  }, [selectedDepartment]);

  const fetchDepartments = async () => {
    try {
      const res = await fetch('/api/departments');
      const data = await res.json();
      // API returns array directly
      setDepartments(Array.isArray(data) ? data : (data.departments || []));
    } catch (error) {
      console.error('Failed to fetch departments:', error);
    }
  };

  const fetchFolders = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      if (selectedDepartment) params.set('departmentId', selectedDepartment);
      
      const res = await fetch(`/api/folders?${params}`);
      const data = await res.json();
      setFolders(data.folders || []);
    } catch (error) {
      console.error('Failed to fetch folders:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateFolder = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newFolder.name.trim()) return;

    setCreating(true);
    try {
      const res = await fetch('/api/folders', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newFolder),
      });

      if (res.ok) {
        setShowCreateModal(false);
        setNewFolder({ name: '', description: '', departmentId: '' });
        fetchFolders();
      } else {
        const data = await res.json();
        alert(data.error || 'Failed to create folder');
      }
    } catch (error) {
      console.error('Failed to create folder:', error);
      alert('Failed to create folder');
    } finally {
      setCreating(false);
    }
  };

  const handleDeleteFolder = async (folderId: string) => {
    if (!confirm('Are you sure you want to delete this folder?')) return;

    try {
      const res = await fetch(`/api/folders/${folderId}`, { method: 'DELETE' });
      if (res.ok) {
        fetchFolders();
      } else {
        const data = await res.json();
        alert(data.error || 'Failed to delete folder');
      }
    } catch (error) {
      console.error('Failed to delete folder:', error);
      alert('Failed to delete folder');
    }
    setMenuOpen(null);
  };

  const filteredFolders = folders.filter((folder) =>
    folder.name.toLowerCase().includes(search.toLowerCase()) ||
    folder.description?.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-stone-50">Folders</h1>
          <p className="text-stone-400 mt-1">Organize your proofs into folders</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="inline-flex items-center gap-2 px-4 py-2 pyrax-gradient text-white font-medium rounded-lg hover:opacity-90 transition-opacity"
        >
          <Plus className="w-5 h-5" />
          New Folder
        </button>
      </div>

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
          <input
            type="text"
            placeholder="Search folders..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-stone-900 border border-stone-700 rounded-lg text-stone-50 placeholder:text-stone-500 focus:outline-none focus:border-pyrax-500"
          />
        </div>
        <select
          value={selectedDepartment}
          onChange={(e) => setSelectedDepartment(e.target.value)}
          className="px-4 py-2 bg-stone-900 border border-stone-700 rounded-lg text-stone-50 focus:outline-none focus:border-pyrax-500"
        >
          <option value="">All Departments</option>
          {departments.map((dept) => (
            <option key={dept.id} value={dept.id}>{dept.name}</option>
          ))}
        </select>
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
        </div>
      ) : filteredFolders.length === 0 ? (
        <div className="text-center py-12 bg-stone-900 rounded-xl border border-stone-800">
          <FolderOpen className="w-12 h-12 text-stone-600 mx-auto mb-4" />
          <h3 className="text-stone-300 font-medium mb-2">No folders found</h3>
          <p className="text-stone-500 text-sm mb-4">
            {search ? 'Try a different search term' : 'Create your first folder to get started'}
          </p>
          {!search && (
            <button
              onClick={() => setShowCreateModal(true)}
              className="inline-flex items-center gap-2 px-4 py-2 bg-stone-800 hover:bg-stone-700 text-stone-50 rounded-lg transition-colors"
            >
              <Plus className="w-4 h-4" />
              Create Folder
            </button>
          )}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredFolders.map((folder) => (
            <div
              key={folder.id}
              className="bg-stone-900 rounded-xl border border-stone-800 p-5 hover:border-stone-700 transition-colors group"
            >
              <div className="flex items-start justify-between mb-3">
                <Link href={`/app/folders/${folder.id}`} className="flex items-center gap-3 flex-1 min-w-0">
                  <div className="w-10 h-10 bg-pyrax-500/10 rounded-lg flex items-center justify-center flex-shrink-0">
                    <FolderOpen className="w-5 h-5 text-pyrax-500" />
                  </div>
                  <div className="min-w-0">
                    <h3 className="text-stone-50 font-medium truncate group-hover:text-pyrax-400 transition-colors">
                      {folder.name}
                    </h3>
                    <div className="flex items-center gap-2 text-xs text-stone-500">
                      <Building2 className="w-3 h-3" />
                      {folder.department.name}
                    </div>
                  </div>
                </Link>
                <div className="relative">
                  <button
                    onClick={() => setMenuOpen(menuOpen === folder.id ? null : folder.id)}
                    className="p-1.5 text-stone-500 hover:text-stone-300 rounded-lg hover:bg-stone-800 transition-colors"
                  >
                    <MoreVertical className="w-4 h-4" />
                  </button>
                  {menuOpen === folder.id && (
                    <div className="absolute right-0 top-full mt-1 w-36 bg-stone-800 border border-stone-700 rounded-lg shadow-xl z-10">
                      <Link
                        href={`/app/folders/${folder.id}/edit`}
                        className="flex items-center gap-2 px-3 py-2 text-sm text-stone-300 hover:bg-stone-700 rounded-t-lg"
                        onClick={() => setMenuOpen(null)}
                      >
                        <Edit2 className="w-4 h-4" />
                        Edit
                      </Link>
                      <button
                        onClick={() => handleDeleteFolder(folder.id)}
                        className="flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-stone-700 w-full rounded-b-lg"
                      >
                        <Trash2 className="w-4 h-4" />
                        Delete
                      </button>
                    </div>
                  )}
                </div>
              </div>
              {folder.description && (
                <p className="text-sm text-stone-400 mb-3 line-clamp-2">{folder.description}</p>
              )}
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-1.5 text-stone-500">
                  <FileText className="w-4 h-4" />
                  {folder.proofCount} {folder.proofCount === 1 ? 'proof' : 'proofs'}
                </div>
                <span className="text-stone-600 text-xs">
                  {new Date(folder.createdAt).toLocaleDateString()}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-stone-900 rounded-xl border border-stone-700 w-full max-w-md">
            <div className="p-6 border-b border-stone-800">
              <h2 className="text-lg font-semibold text-stone-50">Create New Folder</h2>
            </div>
            <form onSubmit={handleCreateFolder} className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-2">
                  Folder Name *
                </label>
                <input
                  type="text"
                  value={newFolder.name}
                  onChange={(e) => setNewFolder({ ...newFolder, name: e.target.value })}
                  placeholder="e.g., Q1 2025 Campaigns"
                  className="w-full px-4 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-50 placeholder:text-stone-500 focus:outline-none focus:border-pyrax-500"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-2">
                  Description
                </label>
                <textarea
                  value={newFolder.description}
                  onChange={(e) => setNewFolder({ ...newFolder, description: e.target.value })}
                  placeholder="Optional description..."
                  rows={3}
                  className="w-full px-4 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-50 placeholder:text-stone-500 focus:outline-none focus:border-pyrax-500 resize-none"
                />
              </div>
              {departments.length > 0 && (
                <div>
                  <label className="block text-sm font-medium text-stone-300 mb-2">
                    Department *
                  </label>
                  <select
                    value={newFolder.departmentId}
                    onChange={(e) => setNewFolder({ ...newFolder, departmentId: e.target.value })}
                    className="w-full px-4 py-2 bg-stone-800 border border-stone-700 rounded-lg text-stone-50 focus:outline-none focus:border-pyrax-500"
                    required
                  >
                    <option value="">Select a department</option>
                    {departments.map((dept) => (
                      <option key={dept.id} value={dept.id}>{dept.name}</option>
                    ))}
                  </select>
                </div>
              )}
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
                  disabled={creating || !newFolder.name.trim()}
                  className="inline-flex items-center gap-2 px-4 py-2 pyrax-gradient text-white font-medium rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50"
                >
                  {creating && <Loader2 className="w-4 h-4 animate-spin" />}
                  Create Folder
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
