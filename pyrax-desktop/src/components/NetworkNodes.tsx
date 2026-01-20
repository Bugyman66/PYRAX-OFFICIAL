import { useEffect, useState } from 'react';
import { Server, Wifi, WifiOff, RefreshCw, Users } from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import { useNodeStore, BootnodeInfo, PeerInfo } from '../stores/nodeStore';

// Country code to flag emoji
function getFlag(countryCode: string): string {
  // Special case for local node
  if (countryCode === '🖥️') return '🖥️';
  if (!countryCode || countryCode.length !== 2) return '🌐';
  const codePoints = countryCode
    .toUpperCase()
    .split('')
    .map(char => 127397 + char.charCodeAt(0));
  return String.fromCodePoint(...codePoints);
}

// Stream label mapping
function getStreamLabel(stream: 'A' | 'B' | 'C'): { label: string; color: string } {
  switch (stream) {
    case 'A':
      return { label: 'FULL NODE / ASIC', color: 'text-pyrax-400' };
    case 'B':
      return { label: 'GPU / AI', color: 'text-blue-400' };
    case 'C':
      return { label: 'VALIDATOR', color: 'text-green-400' };
    default:
      return { label: 'Unknown', color: 'text-stone-400' };
  }
}

// Latency color based on ms
function getLatencyColor(latencyMs: number): string {
  if (latencyMs < 50) return 'text-green-400';
  if (latencyMs < 100) return 'text-yellow-400';
  if (latencyMs < 200) return 'text-orange-400';
  return 'text-red-400';
}

export default function NetworkNodes() {
  const { bootnodes, fetchBootnodes, peers, fetchPeers, status } = useNodeStore();
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [averageLatency, setAverageLatency] = useState<number | null>(null);
  const [appVersion, setAppVersion] = useState<string>('...');
  const [showPeers, setShowPeers] = useState(true);

  // Fetch app version on mount and every 10 seconds for real-time updates
  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const version = await invoke<string>('get_app_version');
        setAppVersion(version);
      } catch (e) {
        console.error('Failed to fetch app version:', e);
        setAppVersion('0.2.11'); // Fallback
      }
    };
    
    fetchVersion();
    const versionInterval = setInterval(fetchVersion, 10000);
    return () => clearInterval(versionInterval);
  }, []);

  useEffect(() => {
    if (status?.connected) {
      fetchBootnodes();
      fetchPeers();
      // Refresh bootnode and peer info every 10 seconds
      const interval = setInterval(() => {
        fetchBootnodes();
        fetchPeers();
      }, 10000);
      return () => clearInterval(interval);
    }
  }, [status?.connected, fetchBootnodes, fetchPeers]);

  useEffect(() => {
    if (bootnodes.length > 0) {
      const onlineNodes = bootnodes.filter(n => n.online);
      if (onlineNodes.length > 0) {
        const avg = onlineNodes.reduce((sum, n) => sum + n.latencyMs, 0) / onlineNodes.length;
        setAverageLatency(Math.round(avg));
      }
    }
  }, [bootnodes]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await Promise.all([fetchBootnodes(), fetchPeers()]);
    setIsRefreshing(false);
  };

  if (!status?.connected) {
    return (
      <div className="bg-dark-800 rounded-xl p-6">
        <div className="text-center py-8 text-stone-400">
          <Server size={48} className="mx-auto mb-4 opacity-50" />
          <p>Connect to a node to view network nodes</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Connected Peers Section */}
      <div className="bg-dark-800 rounded-xl p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Users size={20} className="text-green-400" />
            Connected Peers
            <span className="text-sm font-normal text-stone-400">({peers.length})</span>
          </h2>
          <button
            onClick={handleRefresh}
            disabled={isRefreshing}
            className="text-sm text-pyrax-400 hover:text-pyrax-300 flex items-center gap-1"
          >
            <RefreshCw size={14} className={isRefreshing ? 'animate-spin' : ''} />
            Refresh
          </button>
        </div>

        {peers.length === 0 ? (
          <div className="text-center py-6 text-stone-400">
            <Users size={32} className="mx-auto mb-2 opacity-50" />
            <p>No peers connected yet...</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="text-left text-sm text-stone-500 border-b border-dark-600">
                  <th className="pb-3 font-medium">Peer ID</th>
                  <th className="pb-3 font-medium">Address</th>
                  <th className="pb-3 font-medium">Direction</th>
                  <th className="pb-3 font-medium">Block Height</th>
                  <th className="pb-3 font-medium text-right">Latency</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-dark-700">
                {peers.map((peer) => (
                  <tr key={peer.id} className="hover:bg-dark-700/50">
                    <td className="py-3">
                      <div className="flex items-center gap-2">
                        <Wifi size={14} className="text-green-400" />
                        <span className="font-mono text-sm text-stone-300" title={peer.id}>
                          {peer.id.slice(0, 16)}...
                        </span>
                      </div>
                    </td>
                    <td className="py-3">
                      <span className="text-sm text-stone-400">{peer.address}</span>
                    </td>
                    <td className="py-3">
                      <span className={`text-xs px-2 py-1 rounded ${
                        peer.direction === 'Inbound' 
                          ? 'bg-blue-500/20 text-blue-400' 
                          : 'bg-purple-500/20 text-purple-400'
                      }`}>
                        {peer.direction}
                      </span>
                    </td>
                    <td className="py-3">
                      <span className="text-sm">{peer.bestHeight.toLocaleString()}</span>
                    </td>
                    <td className="py-3 text-right">
                      <span className={`font-mono ${getLatencyColor(peer.latencyMs)}`}>
                        {peer.latencyMs}ms
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Bootnodes Section */}
      <div className="bg-dark-800 rounded-xl p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Server size={20} className="text-pyrax-400" />
            Network Nodes
          </h2>
          <div className="flex items-center gap-4">
            {averageLatency !== null && (
              <span className="text-sm text-stone-400">
                Avg Latency: <span className={getLatencyColor(averageLatency)}>{averageLatency}ms</span>
              </span>
            )}
          </div>
        </div>

        {bootnodes.length === 0 ? (
          <div className="text-center py-8 text-stone-400">
            <p>No nodes discovered yet...</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="text-left text-sm text-stone-500 border-b border-dark-600">
                  <th className="pb-3 font-medium">Status</th>
                  <th className="pb-3 font-medium">Location</th>
                  <th className="pb-3 font-medium">Stream</th>
                  <th className="pb-3 font-medium">Version</th>
                  <th className="pb-3 font-medium text-right">Latency</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-dark-700">
                {bootnodes.map((node) => {
                  const streamInfo = getStreamLabel(node.stream);
                  const isLocalNode = node.id === 'this-node';
                  return (
                    <tr key={node.id} className={`hover:bg-dark-700/50 ${isLocalNode ? 'bg-pyrax-900/20 border-l-2 border-pyrax-500' : ''}`}>
                      <td className="py-3">
                        <div className="flex items-center gap-2">
                          {node.online ? (
                            <Wifi size={16} className="text-green-400" />
                          ) : (
                            <WifiOff size={16} className="text-red-400" />
                          )}
                          <span className={node.online ? 'text-green-400' : 'text-red-400'}>
                            {node.online ? 'Online' : 'Offline'}
                          </span>
                          {isLocalNode && (
                            <span className="text-xs bg-pyrax-500/20 text-pyrax-400 px-2 py-0.5 rounded-full">
                              YOU
                            </span>
                          )}
                        </div>
                      </td>
                      <td className="py-3">
                        <div className="flex items-center gap-2">
                          <span className="flag-emoji text-lg">{getFlag(node.countryCode)}</span>
                          <div>
                            <div className="text-sm">
                              {node.city}{node.region ? `, ${node.region}` : ''}
                            </div>
                            <div className="text-xs text-stone-500">{node.country}</div>
                          </div>
                        </div>
                      </td>
                      <td className="py-3">
                        <span className={`text-sm font-medium ${streamInfo.color}`}>
                          {streamInfo.label}
                        </span>
                      </td>
                      <td className="py-3">
                        <span className="text-sm font-mono text-pyrax-400">
                          v{appVersion}
                        </span>
                      </td>
                      <td className="py-3 text-right">
                        <span className={`font-mono ${getLatencyColor(node.latencyMs)}`}>
                          {node.online ? `${node.latencyMs}ms` : '—'}
                        </span>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
