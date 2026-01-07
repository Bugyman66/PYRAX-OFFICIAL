'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';

interface GpuSpec {
  model: string;
  vram_mb: number;
  count: number;
}

interface Provider {
  address: string;
  name: string;
  stake: number;
  gpu_specs: GpuSpec[];
  total_gpu_memory_mb: number;
  supported_models: string[];
  is_active: boolean;
  jobs_completed: number;
  jobs_failed: number;
  avg_response_ms: number;
  reputation: number;
  endpoint: string;
}

export default function ProvidersPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [showOnlyActive, setShowOnlyActive] = useState(true);

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
    const fetchProviders = async () => {
      try {
        setLoading(true);
        const result = await rpcCall('ai_listProviders', [100, 0]);
        setProviders(result);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch providers');
      } finally {
        setLoading(false);
      }
    };

    fetchProviders();
    const interval = setInterval(fetchProviders, 15000);
    return () => clearInterval(interval);
  }, []);

  const formatPyrax = (amount: number) => (amount / 100_000_000).toFixed(2);

  const getReputationColor = (rep: number) => {
    if (rep >= 800) return 'text-green-400';
    if (rep >= 500) return 'text-yellow-400';
    if (rep >= 200) return 'text-orange-400';
    return 'text-red-400';
  };

  const getSuccessRate = (provider: Provider) => {
    const total = provider.jobs_completed + provider.jobs_failed;
    if (total === 0) return 0;
    return ((provider.jobs_completed / total) * 100).toFixed(1);
  };

  const filteredProviders = providers
    .filter((p) => !showOnlyActive || p.is_active)
    .filter((p) =>
      p.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      p.address.toLowerCase().includes(searchTerm.toLowerCase())
    );

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <div>
            <Link href="/ai" className="text-gray-400 hover:text-white text-sm mb-2 block">
              ← Back to AI Platform
            </Link>
            <h1 className="text-3xl font-bold">AI Providers</h1>
          </div>
        </div>

        {/* Filters */}
        <div className="bg-gray-800 rounded-lg p-4 mb-6">
          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1">
              <input
                type="text"
                placeholder="Search by name or address..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
              />
            </div>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={showOnlyActive}
                onChange={(e) => setShowOnlyActive(e.target.checked)}
                className="w-4 h-4 rounded bg-gray-700 border-gray-600 text-purple-500 focus:ring-purple-500"
              />
              <span className="text-gray-300">Show only active</span>
            </label>
          </div>
        </div>

        {error && (
          <div className="bg-red-900/50 border border-red-500 rounded-lg p-4 mb-6">
            <p className="text-red-300">{error}</p>
          </div>
        )}

        {loading ? (
          <div className="text-gray-400">Loading providers...</div>
        ) : filteredProviders.length === 0 ? (
          <div className="bg-gray-800 rounded-lg p-8 text-center">
            <p className="text-gray-400">No providers found</p>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredProviders.map((provider) => (
              <Link
                key={provider.address}
                href={`/ai/providers/${provider.address}`}
                className="block bg-gray-800 rounded-lg p-6 hover:bg-gray-750 transition border border-gray-700 hover:border-purple-500"
              >
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h2 className="text-xl font-bold">{provider.name}</h2>
                      <span className={`px-2 py-1 rounded text-xs ${
                        provider.is_active ? 'bg-green-900 text-green-300' : 'bg-gray-600 text-gray-400'
                      }`}>
                        {provider.is_active ? 'Online' : 'Offline'}
                      </span>
                    </div>
                    <p className="text-gray-400 font-mono text-sm mb-3">
                      {provider.address}
                    </p>
                    {/* GPU Specs */}
                    <div className="flex flex-wrap gap-2">
                      {provider.gpu_specs.map((gpu, idx) => (
                        <span key={idx} className="px-2 py-1 bg-purple-900/50 text-purple-300 rounded text-sm">
                          {gpu.count}x {gpu.model} ({(gpu.vram_mb / 1024).toFixed(0)}GB)
                        </span>
                      ))}
                    </div>
                  </div>
                  <div className="text-right">
                    <p className={`text-3xl font-bold ${getReputationColor(provider.reputation)}`}>
                      {provider.reputation}
                    </p>
                    <p className="text-gray-400 text-sm">reputation</p>
                  </div>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mt-4 pt-4 border-t border-gray-700">
                  <div>
                    <p className="text-gray-500 text-xs">Staked</p>
                    <p className="font-semibold">{formatPyrax(provider.stake)} PYRAX</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Total GPU Memory</p>
                    <p className="font-semibold">{(provider.total_gpu_memory_mb / 1024).toFixed(1)} GB</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Jobs Completed</p>
                    <p className="font-semibold">{provider.jobs_completed.toLocaleString()}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Success Rate</p>
                    <p className="font-semibold">{getSuccessRate(provider)}%</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Avg Response</p>
                    <p className="font-semibold">{provider.avg_response_ms} ms</p>
                  </div>
                </div>

                {provider.supported_models.length > 0 && (
                  <div className="mt-4 pt-4 border-t border-gray-700">
                    <p className="text-gray-500 text-xs mb-2">Supported Models ({provider.supported_models.length})</p>
                    <div className="flex flex-wrap gap-1">
                      {provider.supported_models.slice(0, 5).map((modelId) => (
                        <span key={modelId} className="px-2 py-1 bg-gray-700 rounded text-xs font-mono">
                          {modelId.slice(0, 10)}...
                        </span>
                      ))}
                      {provider.supported_models.length > 5 && (
                        <span className="px-2 py-1 bg-gray-700 rounded text-xs">
                          +{provider.supported_models.length - 5} more
                        </span>
                      )}
                    </div>
                  </div>
                )}
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
