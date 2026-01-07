'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';

interface RegistryStats {
  total_models: number;
  active_models: number;
  total_providers: number;
  active_providers: number;
  total_inferences: number;
  total_gpu_memory_gb: number;
}

interface JobStats {
  total_jobs: number;
  pending_jobs: number;
  running_jobs: number;
  completed_jobs: number;
  failed_jobs: number;
  total_paid_pyrax: number;
}

interface Model {
  id: string;
  name: string;
  version: string;
  owner: string;
  framework: string;
  model_type: string;
  size_bytes: number;
  price_per_1k: number;
  total_inferences: number;
  is_active: boolean;
}

interface Provider {
  address: string;
  name: string;
  stake: number;
  total_gpu_memory_mb: number;
  jobs_completed: number;
  reputation: number;
  is_active: boolean;
}

export default function AIPage() {
  const [registryStats, setRegistryStats] = useState<RegistryStats | null>(null);
  const [jobStats, setJobStats] = useState<JobStats | null>(null);
  const [models, setModels] = useState<Model[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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
    const fetchData = async () => {
      try {
        setLoading(true);
        const [regStats, jStats, modelList, providerList] = await Promise.all([
          rpcCall('ai_getRegistryStats'),
          rpcCall('ai_getJobStats'),
          rpcCall('ai_listModels', [10, 0]),
          rpcCall('ai_listProviders', [10, 0]),
        ]);
        setRegistryStats(regStats);
        setJobStats(jStats);
        setModels(modelList);
        setProviders(providerList);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch AI data');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 10000);
    return () => clearInterval(interval);
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  };

  const formatPyrax = (amount: number) => {
    return (amount / 100_000_000).toFixed(2);
  };

  if (loading && !registryStats) {
    return (
      <div className="min-h-screen bg-gray-900 text-white p-8">
        <div className="max-w-7xl mx-auto">
          <h1 className="text-3xl font-bold mb-8">AI Services Platform</h1>
          <div className="animate-pulse text-gray-400">Loading AI platform data...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-3xl font-bold">AI Services Platform</h1>
          <div className="flex gap-4">
            <Link href="/ai/models" className="px-4 py-2 bg-orange-600 hover:bg-orange-700 rounded-lg">
              All Models
            </Link>
            <Link href="/ai/providers" className="px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg">
              All Providers
            </Link>
            <Link href="/ai/jobs" className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg">
              All Jobs
            </Link>
          </div>
        </div>

        {error && (
          <div className="bg-red-900/50 border border-red-500 rounded-lg p-4 mb-6">
            <p className="text-red-300">{error}</p>
          </div>
        )}

        {/* Stats Overview */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="bg-gray-800 rounded-lg p-6">
            <h3 className="text-gray-400 text-sm mb-2">Active Models</h3>
            <p className="text-3xl font-bold text-orange-400">
              {registryStats?.active_models || 0}
            </p>
            <p className="text-gray-500 text-sm mt-1">
              of {registryStats?.total_models || 0} total
            </p>
          </div>

          <div className="bg-gray-800 rounded-lg p-6">
            <h3 className="text-gray-400 text-sm mb-2">Active Providers</h3>
            <p className="text-3xl font-bold text-purple-400">
              {registryStats?.active_providers || 0}
            </p>
            <p className="text-gray-500 text-sm mt-1">
              {registryStats?.total_gpu_memory_gb || 0} GB GPU
            </p>
          </div>

          <div className="bg-gray-800 rounded-lg p-6">
            <h3 className="text-gray-400 text-sm mb-2">Total Inferences</h3>
            <p className="text-3xl font-bold text-green-400">
              {(registryStats?.total_inferences || 0).toLocaleString()}
            </p>
            <p className="text-gray-500 text-sm mt-1">
              {jobStats?.running_jobs || 0} running now
            </p>
          </div>

          <div className="bg-gray-800 rounded-lg p-6">
            <h3 className="text-gray-400 text-sm mb-2">Total Paid</h3>
            <p className="text-3xl font-bold text-yellow-400">
              {formatPyrax(jobStats?.total_paid_pyrax || 0)} PYRAX
            </p>
            <p className="text-gray-500 text-sm mt-1">
              {(jobStats?.completed_jobs || 0).toLocaleString()} jobs completed
            </p>
          </div>
        </div>

        {/* Job Stats */}
        <div className="bg-gray-800 rounded-lg p-6 mb-8">
          <h2 className="text-xl font-bold mb-4">Job Status</h2>
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
            <div className="text-center p-4 bg-gray-700 rounded-lg">
              <p className="text-2xl font-bold text-yellow-400">{jobStats?.pending_jobs || 0}</p>
              <p className="text-gray-400 text-sm">Pending</p>
            </div>
            <div className="text-center p-4 bg-gray-700 rounded-lg">
              <p className="text-2xl font-bold text-blue-400">{jobStats?.running_jobs || 0}</p>
              <p className="text-gray-400 text-sm">Running</p>
            </div>
            <div className="text-center p-4 bg-gray-700 rounded-lg">
              <p className="text-2xl font-bold text-green-400">{jobStats?.completed_jobs || 0}</p>
              <p className="text-gray-400 text-sm">Completed</p>
            </div>
            <div className="text-center p-4 bg-gray-700 rounded-lg">
              <p className="text-2xl font-bold text-red-400">{jobStats?.failed_jobs || 0}</p>
              <p className="text-gray-400 text-sm">Failed</p>
            </div>
            <div className="text-center p-4 bg-gray-700 rounded-lg">
              <p className="text-2xl font-bold text-gray-300">{jobStats?.total_jobs || 0}</p>
              <p className="text-gray-400 text-sm">Total</p>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Recent Models */}
          <div className="bg-gray-800 rounded-lg p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-bold">Recent Models</h2>
              <Link href="/ai/models" className="text-orange-400 hover:text-orange-300 text-sm">
                View All →
              </Link>
            </div>
            <div className="space-y-3">
              {models.length === 0 ? (
                <p className="text-gray-500">No models registered yet</p>
              ) : (
                models.slice(0, 5).map((model) => (
                  <Link
                    key={model.id}
                    href={`/ai/models/${model.id}`}
                    className="block bg-gray-700 rounded-lg p-4 hover:bg-gray-600 transition"
                  >
                    <div className="flex justify-between items-start">
                      <div>
                        <h3 className="font-semibold">{model.name}</h3>
                        <p className="text-gray-400 text-sm">v{model.version} • {model.framework}</p>
                      </div>
                      <div className="text-right">
                        <span className={`px-2 py-1 rounded text-xs ${
                          model.is_active ? 'bg-green-900 text-green-300' : 'bg-gray-600 text-gray-400'
                        }`}>
                          {model.is_active ? 'Active' : 'Inactive'}
                        </span>
                      </div>
                    </div>
                    <div className="mt-2 flex gap-4 text-sm text-gray-400">
                      <span>{formatBytes(model.size_bytes)}</span>
                      <span>{model.total_inferences.toLocaleString()} inferences</span>
                      <span>{formatPyrax(model.price_per_1k)} PYRAX/1k</span>
                    </div>
                  </Link>
                ))
              )}
            </div>
          </div>

          {/* Active Providers */}
          <div className="bg-gray-800 rounded-lg p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-bold">Active Providers</h2>
              <Link href="/ai/providers" className="text-purple-400 hover:text-purple-300 text-sm">
                View All →
              </Link>
            </div>
            <div className="space-y-3">
              {providers.length === 0 ? (
                <p className="text-gray-500">No providers registered yet</p>
              ) : (
                providers.slice(0, 5).map((provider) => (
                  <Link
                    key={provider.address}
                    href={`/ai/providers/${provider.address}`}
                    className="block bg-gray-700 rounded-lg p-4 hover:bg-gray-600 transition"
                  >
                    <div className="flex justify-between items-start">
                      <div>
                        <h3 className="font-semibold">{provider.name}</h3>
                        <p className="text-gray-400 text-sm font-mono">
                          {provider.address.slice(0, 10)}...{provider.address.slice(-8)}
                        </p>
                      </div>
                      <div className="text-right">
                        <span className={`px-2 py-1 rounded text-xs ${
                          provider.is_active ? 'bg-green-900 text-green-300' : 'bg-gray-600 text-gray-400'
                        }`}>
                          {provider.is_active ? 'Online' : 'Offline'}
                        </span>
                      </div>
                    </div>
                    <div className="mt-2 flex gap-4 text-sm text-gray-400">
                      <span>{(provider.total_gpu_memory_mb / 1024).toFixed(1)} GB VRAM</span>
                      <span>{provider.jobs_completed.toLocaleString()} jobs</span>
                      <span>Rep: {provider.reputation}/1000</span>
                    </div>
                  </Link>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
