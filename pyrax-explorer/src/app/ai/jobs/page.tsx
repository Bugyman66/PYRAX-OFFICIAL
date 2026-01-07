'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';

interface Job {
  id: string;
  job_type: string;
  model_id: string;
  requester: string;
  provider: string | null;
  status: string;
  input: string;
  output: string | null;
  max_price: number;
  actual_price: number | null;
  collateral: number;
  submitted_at: number;
  assigned_at: number | null;
  completed_at: number | null;
  error: string | null;
  timeout_secs: number;
}

export default function JobsPage() {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState<string>('');
  const [filterType, setFilterType] = useState<string>('');

  const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:8545';

  const rpcCall = async (method: string, params: any[] = []) => {
    const response = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method,
        params,
        id: Date.now(),
      }),
    });
    const data = await response.json();
    if (data.error) throw new Error(data.error.message);
    return data.result;
  };

  useEffect(() => {
    const fetchJobs = async () => {
      try {
        setLoading(true);
        const result = await rpcCall('ai_listPendingJobs', [100]);
        setJobs(result);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch jobs');
      } finally {
        setLoading(false);
      }
    };

    fetchJobs();
    const interval = setInterval(fetchJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const formatPyrax = (amount: number) => (amount / 100_000_000).toFixed(4);

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'pending': return 'bg-yellow-900 text-yellow-300';
      case 'assigned': return 'bg-blue-900 text-blue-300';
      case 'running': return 'bg-purple-900 text-purple-300';
      case 'completed': return 'bg-green-900 text-green-300';
      case 'failed': return 'bg-red-900 text-red-300';
      case 'cancelled': return 'bg-gray-700 text-gray-400';
      case 'expired': return 'bg-orange-900 text-orange-300';
      default: return 'bg-gray-700 text-gray-400';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'inference': return '🔮';
      case 'batch_inference': return '📦';
      case 'training': return '🎓';
      case 'fine_tuning': return '🔧';
      default: return '⚙️';
    }
  };

  const filteredJobs = jobs
    .filter((j) => !filterStatus || j.status === filterStatus)
    .filter((j) => !filterType || j.job_type === filterType)
    .filter((j) =>
      j.id.toLowerCase().includes(searchTerm.toLowerCase()) ||
      j.requester.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (j.provider && j.provider.toLowerCase().includes(searchTerm.toLowerCase()))
    );

  const statuses = ['pending', 'assigned', 'running', 'completed', 'failed', 'cancelled', 'expired'];
  const jobTypes = ['inference', 'batch_inference', 'training', 'fine_tuning'];

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <div>
            <Link href="/ai" className="text-gray-400 hover:text-white text-sm mb-2 block">
              ← Back to AI Platform
            </Link>
            <h1 className="text-3xl font-bold">AI Jobs</h1>
          </div>
        </div>

        {/* Filters */}
        <div className="bg-gray-800 rounded-lg p-4 mb-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-gray-400 text-sm mb-1">Search</label>
              <input
                type="text"
                placeholder="Search by ID, requester, provider..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-gray-400 text-sm mb-1">Status</label>
              <select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value)}
                className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">All Statuses</option>
                {statuses.map((status) => (
                  <option key={status} value={status}>{status}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-gray-400 text-sm mb-1">Job Type</label>
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">All Types</option>
                {jobTypes.map((type) => (
                  <option key={type} value={type}>{type.replace('_', ' ')}</option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {error && (
          <div className="bg-red-900/50 border border-red-500 rounded-lg p-4 mb-6">
            <p className="text-red-300">{error}</p>
          </div>
        )}

        {loading ? (
          <div className="text-gray-400">Loading jobs...</div>
        ) : filteredJobs.length === 0 ? (
          <div className="bg-gray-800 rounded-lg p-8 text-center">
            <p className="text-gray-400">No jobs found</p>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredJobs.map((job) => (
              <div
                key={job.id}
                className="bg-gray-800 rounded-lg p-6 border border-gray-700"
              >
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span className="text-2xl">{getTypeIcon(job.job_type)}</span>
                      <h2 className="text-lg font-bold font-mono">{job.id.slice(0, 18)}...</h2>
                      <span className={`px-2 py-1 rounded text-xs ${getStatusColor(job.status)}`}>
                        {job.status}
                      </span>
                    </div>
                    <div className="space-y-1 text-sm">
                      <p className="text-gray-400">
                        <span className="text-gray-500">Model:</span>{' '}
                        <span className="font-mono">{job.model_id.slice(0, 18)}...</span>
                      </p>
                      <p className="text-gray-400">
                        <span className="text-gray-500">Requester:</span>{' '}
                        <span className="font-mono">{job.requester.slice(0, 14)}...{job.requester.slice(-8)}</span>
                      </p>
                      {job.provider && (
                        <p className="text-gray-400">
                          <span className="text-gray-500">Provider:</span>{' '}
                          <span className="font-mono">{job.provider.slice(0, 14)}...{job.provider.slice(-8)}</span>
                        </p>
                      )}
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-xl font-bold text-blue-400">
                      {formatPyrax(job.actual_price || job.max_price)} PYRAX
                    </p>
                    <p className="text-gray-500 text-sm">
                      {job.actual_price ? 'paid' : 'max price'}
                    </p>
                  </div>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mt-4 pt-4 border-t border-gray-700">
                  <div>
                    <p className="text-gray-500 text-xs">Type</p>
                    <p className="font-semibold">{job.job_type.replace('_', ' ')}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Collateral</p>
                    <p className="font-semibold">{formatPyrax(job.collateral)} PYRAX</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Timeout</p>
                    <p className="font-semibold">{job.timeout_secs}s</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Submitted</p>
                    <p className="font-semibold">{formatDate(job.submitted_at)}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">
                      {job.completed_at ? 'Completed' : job.assigned_at ? 'Assigned' : 'Waiting'}
                    </p>
                    <p className="font-semibold">
                      {job.completed_at 
                        ? formatDate(job.completed_at) 
                        : job.assigned_at 
                          ? formatDate(job.assigned_at)
                          : '-'}
                    </p>
                  </div>
                </div>

                {job.error && (
                  <div className="mt-4 p-3 bg-red-900/30 border border-red-800 rounded">
                    <p className="text-red-300 text-sm">
                      <span className="font-semibold">Error:</span> {job.error}
                    </p>
                  </div>
                )}

                {job.output && (
                  <div className="mt-4 p-3 bg-green-900/30 border border-green-800 rounded">
                    <p className="text-green-300 text-sm font-mono truncate">
                      <span className="font-semibold">Output:</span> {job.output.slice(0, 200)}...
                    </p>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
