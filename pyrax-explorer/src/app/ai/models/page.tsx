'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';

interface Model {
  id: string;
  name: string;
  version: string;
  owner: string;
  framework: string;
  model_type: string;
  size_bytes: number;
  ipfs_cid: string;
  min_gpu_memory_mb: number;
  price_per_1k: number;
  registered_at: number;
  is_active: boolean;
  total_inferences: number;
  avg_latency_ms: number;
}

export default function ModelsPage() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState<string>('');
  const [filterFramework, setFilterFramework] = useState<string>('');

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
    const fetchModels = async () => {
      try {
        setLoading(true);
        const result = await rpcCall('ai_searchModels', [
          filterType || null,
          filterFramework || null,
          100,
        ]);
        setModels(result);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch models');
      } finally {
        setLoading(false);
      }
    };

    fetchModels();
  }, [filterType, filterFramework]);

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  };

  const formatPyrax = (amount: number) => (amount / 100_000_000).toFixed(4);

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const filteredModels = models.filter((model) =>
    model.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    model.id.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const modelTypes = ['llm', 'image_generation', 'image_classification', 'embedding', 'speech_to_text', 'text_to_speech'];
  const frameworks = ['pytorch', 'tensorflow', 'onnx', 'jax', 'huggingface'];

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <div>
            <Link href="/ai" className="text-gray-400 hover:text-white text-sm mb-2 block">
              ← Back to AI Platform
            </Link>
            <h1 className="text-3xl font-bold">AI Models</h1>
          </div>
        </div>

        {/* Filters */}
        <div className="bg-gray-800 rounded-lg p-4 mb-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-gray-400 text-sm mb-1">Search</label>
              <input
                type="text"
                placeholder="Search by name or ID..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
            </div>
            <div>
              <label className="block text-gray-400 text-sm mb-1">Model Type</label>
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-orange-500"
              >
                <option value="">All Types</option>
                {modelTypes.map((type) => (
                  <option key={type} value={type}>{type.replace('_', ' ')}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-gray-400 text-sm mb-1">Framework</label>
              <select
                value={filterFramework}
                onChange={(e) => setFilterFramework(e.target.value)}
                className="w-full bg-gray-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-orange-500"
              >
                <option value="">All Frameworks</option>
                {frameworks.map((fw) => (
                  <option key={fw} value={fw}>{fw}</option>
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
          <div className="text-gray-400">Loading models...</div>
        ) : filteredModels.length === 0 ? (
          <div className="bg-gray-800 rounded-lg p-8 text-center">
            <p className="text-gray-400">No models found</p>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredModels.map((model) => (
              <Link
                key={model.id}
                href={`/ai/models/${model.id}`}
                className="block bg-gray-800 rounded-lg p-6 hover:bg-gray-750 transition border border-gray-700 hover:border-orange-500"
              >
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h2 className="text-xl font-bold">{model.name}</h2>
                      <span className="text-gray-400">v{model.version}</span>
                      <span className={`px-2 py-1 rounded text-xs ${
                        model.is_active ? 'bg-green-900 text-green-300' : 'bg-gray-600 text-gray-400'
                      }`}>
                        {model.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </div>
                    <p className="text-gray-400 font-mono text-sm mb-3">
                      {model.id}
                    </p>
                    <div className="flex flex-wrap gap-2 mb-3">
                      <span className="px-2 py-1 bg-orange-900/50 text-orange-300 rounded text-sm">
                        {model.model_type.replace('_', ' ')}
                      </span>
                      <span className="px-2 py-1 bg-blue-900/50 text-blue-300 rounded text-sm">
                        {model.framework}
                      </span>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-2xl font-bold text-orange-400">
                      {formatPyrax(model.price_per_1k)} PYRAX
                    </p>
                    <p className="text-gray-400 text-sm">per 1,000 tokens</p>
                  </div>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mt-4 pt-4 border-t border-gray-700">
                  <div>
                    <p className="text-gray-500 text-xs">Size</p>
                    <p className="font-semibold">{formatBytes(model.size_bytes)}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Min GPU Memory</p>
                    <p className="font-semibold">{(model.min_gpu_memory_mb / 1024).toFixed(1)} GB</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Total Inferences</p>
                    <p className="font-semibold">{model.total_inferences.toLocaleString()}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Avg Latency</p>
                    <p className="font-semibold">{model.avg_latency_ms} ms</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Registered</p>
                    <p className="font-semibold">{formatDate(model.registered_at)}</p>
                  </div>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
