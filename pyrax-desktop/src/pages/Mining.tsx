import { useState, useEffect } from 'react';
import { Hammer, Play, Square, Cpu, Gauge, Award, Thermometer } from 'lucide-react';
import { useMinerStore } from '../stores/minerStore';
import { useWalletStore } from '../stores/walletStore';
import { useNodeStore } from '../stores/nodeStore';
import { formatHashrate } from '../lib/utils';

export default function Mining() {
  const { status: nodeStatus } = useNodeStore();
  const { status, loading, startMiner, stopMiner, fetchStatus } = useMinerStore();
  const { addresses, selectedAddress } = useWalletStore();
  const [threads, setThreads] = useState(0);

  // PERFORMANCE FIX: Increased polling interval from 2s to 5s to reduce CPU overhead
  useEffect(() => {
    fetchStatus(); // Initial fetch
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const handleStartMining = () => {
    if (selectedAddress) {
      startMiner(selectedAddress, threads);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Mining</h1>
        {status?.running ? (
          <button
            onClick={() => stopMiner()}
            disabled={loading}
            className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors disabled:opacity-50"
          >
            <Square size={16} />
            Stop Mining
          </button>
        ) : (
          <button
            onClick={handleStartMining}
            disabled={loading || !nodeStatus?.connected || !selectedAddress}
            className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors disabled:opacity-50"
          >
            <Play size={16} />
            Start Mining
          </button>
        )}
      </div>

      {!nodeStatus?.connected && (
        <div className="bg-yellow-600/20 border border-yellow-500 rounded-lg p-4 text-yellow-400">
          Node must be connected to start mining
        </div>
      )}

      {/* Mining Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon={<Gauge className="text-blue-400" />}
          label="Hashrate"
          value={status?.running ? formatHashrate(status.hashrate) : '0 H/s'}
        />
        <StatCard
          icon={<Award className="text-green-400" />}
          label="Accepted Shares"
          value={status?.acceptedShares?.toString() || '0'}
        />
        <StatCard
          icon={<Hammer className="text-purple-400" />}
          label="Blocks Found"
          value={status?.blocksFound?.toString() || '0'}
        />
        <StatCard
          icon={<Thermometer className="text-red-400" />}
          label="Temperature"
          value={status?.temperature ? `${status.temperature}°C` : 'N/A'}
        />
      </div>

      {/* Mining Configuration */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Cpu size={20} />
          Mining Configuration
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Mining Address</label>
            <select
              value={selectedAddress || ''}
              disabled={status?.running}
              className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none disabled:opacity-50"
            >
              {addresses.map((addr) => (
                <option key={addr.address} value={addr.address}>
                  {addr.label || addr.address.slice(0, 20)}...
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">CPU Threads (0 = auto)</label>
            <input
              type="number"
              value={threads}
              onChange={(e) => setThreads(parseInt(e.target.value) || 0)}
              min={0}
              max={64}
              disabled={status?.running}
              className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none disabled:opacity-50"
            />
          </div>
        </div>
      </div>

      {/* Mining Status */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Mining Status</h2>
        {status?.running ? (
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse" />
              <span className="text-green-400">Mining Active</span>
            </div>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <div className="text-gray-500">Algorithm</div>
                <div>{status.algorithm || 'BLAKE3'}</div>
              </div>
              <div>
                <div className="text-gray-500">Pool</div>
                <div>{status.pool || 'Solo Mining'}</div>
              </div>
              <div>
                <div className="text-gray-500">Rejected</div>
                <div className="text-red-400">{status.rejectedShares || 0}</div>
              </div>
              <div>
                <div className="text-gray-500">Power</div>
                <div>{status.powerUsage ? `${status.powerUsage}W` : 'N/A'}</div>
              </div>
            </div>
          </div>
        ) : (
          <div className="text-center py-8 text-gray-400">
            <Hammer size={48} className="mx-auto mb-4 opacity-50" />
            <p>Mining is not active</p>
            <p className="text-sm mt-2">Click "Start Mining" to begin earning PYRAX</p>
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className="bg-gray-800 rounded-xl p-4">
      <div className="flex items-center gap-3 mb-2">
        {icon}
        <span className="text-sm text-gray-400">{label}</span>
      </div>
      <div className="text-2xl font-bold">{value}</div>
    </div>
  );
}
