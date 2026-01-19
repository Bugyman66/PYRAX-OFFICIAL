import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { 
  Activity, 
  Blocks, 
  Users, 
  Zap,
  Play,
  Square,
  RefreshCw,
  Globe
} from 'lucide-react';
import { useNodeStore } from '../stores/nodeStore';
import { useWalletStore } from '../stores/walletStore';
import { useMinerStore } from '../stores/minerStore';
import { useLogStore } from '../stores/logStore';
import { formatBalance, formatHashrate } from '../lib/utils';
import LogViewer from '../components/LogViewer';

export default function Dashboard() {
  const { status, chainInfo, error: nodeError, startNode, stopNode, fetchChainInfo, loading: nodeLoading } = useNodeStore();
  const { addresses } = useWalletStore();
  const { status: minerStatus } = useMinerStore();
  const { addLog } = useLogStore();
  const [selectedNetwork, setSelectedNetwork] = useState<'testnet' | 'devnet'>('testnet');

  // Listen for log events from backend
  useEffect(() => {
    const unlisten = listen<{ level: string; category: string; message: string }>('node-log', (event) => {
      const { level, category, message } = event.payload;
      addLog(
        level as 'info' | 'warn' | 'error' | 'debug',
        category as 'node' | 'block' | 'p2p' | 'rpc' | 'mining' | 'staking' | 'system',
        message
      );
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addLog]);

  // Listen for watchdog events (auto-reconnect)
  useEffect(() => {
    const unlistenDisconnected = listen<{ reason: string; will_restart: boolean }>('node-disconnected', (event) => {
      addLog('warn', 'node', `Node disconnected: ${event.payload.reason}`);
      if (event.payload.will_restart) {
        addLog('info', 'node', 'Auto-restart pending...');
      }
    });

    const unlistenRestart = listen<{ reason: string }>('node-restart-requested', async (event) => {
      addLog('info', 'node', `Auto-restart triggered: ${event.payload.reason}`);
      try {
        await startNode();
        addLog('info', 'node', 'Node restarted successfully');
      } catch (e) {
        addLog('error', 'node', `Failed to restart node: ${e}`);
      }
    });

    const unlistenNetworkError = listen<{ reason: string; duration_seconds: number }>('node-network-error', (event) => {
      addLog('error', 'node', `Network error: ${event.payload.reason} (${event.payload.duration_seconds}s)`);
    });

    return () => {
      unlistenDisconnected.then((fn) => fn());
      unlistenRestart.then((fn) => fn());
      unlistenNetworkError.then((fn) => fn());
    };
  }, [startNode, addLog]);

  useEffect(() => {
    // Load current network setting
    invoke<{ network: string }>('get_settings').then((settings) => {
      if (settings.network === 'devnet' || settings.network === 'testnet') {
        setSelectedNetwork(settings.network);
      }
    }).catch(console.error);
  }, []);

  useEffect(() => {
    if (status?.running && status?.connected) {
      fetchChainInfo();
    }
  }, [status?.running, status?.connected, fetchChainInfo]);

  const handleNetworkChange = async (network: 'testnet' | 'devnet') => {
    setSelectedNetwork(network);
    try {
      const currentSettings = await invoke<any>('get_settings');
      await invoke('save_settings', { 
        settings: { ...currentSettings, network } 
      });
    } catch (e) {
      console.error('Failed to save network:', e);
    }
  };

  const totalBalance = addresses.reduce((sum, addr) => {
    return sum + parseFloat(addr.balance || '0');
  }, 0);

  const handleStartNode = async () => {
    try {
      await startNode();
      // Also try to connect to remote bootnode for logs
      invoke('start_remote_log_stream').catch(console.error);
    } catch (e) {
      console.error('Failed to start node:', e);
    }
  };

  const handleStopNode = async () => {
    try {
      await stopNode();
    } catch (e) {
      console.error('Failed to stop node:', e);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <div className="flex items-center gap-4">
          {/* Network Selector */}
          <div className="flex items-center gap-2 bg-dark-800 rounded-lg p-1">
            <Globe size={16} className="ml-2 text-stone-400" />
            <button
              onClick={() => handleNetworkChange('testnet')}
              disabled={status?.running}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                selectedNetwork === 'testnet'
                  ? 'bg-pyrax-600 text-white'
                  : 'text-stone-400 hover:text-white hover:bg-dark-700'
              } ${status?.running ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              Testnet
            </button>
            <button
              onClick={() => handleNetworkChange('devnet')}
              disabled={status?.running}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                selectedNetwork === 'devnet'
                  ? 'bg-pyrax-600 text-white'
                  : 'text-stone-400 hover:text-white hover:bg-dark-700'
              } ${status?.running ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              Devnet
            </button>
          </div>

          {/* Node Control */}
          {status?.running ? (
            <button
              onClick={handleStopNode}
              disabled={nodeLoading}
              className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors disabled:opacity-50"
            >
              <Square size={16} />
              Stop Node
            </button>
          ) : (
            <button
              onClick={handleStartNode}
              disabled={nodeLoading}
              className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors disabled:opacity-50"
            >
              <Play size={16} />
              Start Node
            </button>
          )}
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon={<Blocks className="text-pyrax-400" />}
          label="Block Height"
          value={status && typeof status.blockHeight === 'number' ? status.blockHeight.toLocaleString() : '0'}
          subtext={status?.syncing ? `Syncing: ${(status?.syncProgress ?? 0).toFixed(1)}%` : 'Synced'}
        />
        <StatCard
          icon={<Users className="text-blue-400" />}
          label="Connected Peers"
          value={`${status && typeof status.peerCount === 'number' ? status.peerCount : 0}/50`}
          subtext={status?.network || 'Not connected'}
        />
        <StatCard
          icon={<Activity className="text-green-400" />}
          label="Wallet Balance"
          value={formatBalance(totalBalance)}
          subtext="PYRAX"
        />
        <StatCard
          icon={<Zap className="text-pyrax-400" />}
          label="Hashrate"
          value={minerStatus?.running ? formatHashrate(minerStatus.hashrate) : '0 H/s'}
          subtext={minerStatus?.running ? 'Mining' : 'Idle'}
        />
      </div>

      {/* Node Status Panel */}
      <div className="bg-dark-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Activity size={20} className="text-pyrax-400" />
          Node Status
        </h2>
        
        {status?.running ? (
          <div className="space-y-4">
            {/* Basic Node Info */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <InfoItem label="Status" value={status.connected ? 'Connected' : 'Connecting...'} />
              <InfoItem label="Network" value={status.network || 'Unknown'} />
              <InfoItem label="Version" value={status.version || '0.1.0'} />
              <InfoItem label="Peers" value={`${status.peerCount || 0}/${status.targetPeers || 50}`} />
              {chainInfo && (
                <>
                  <InfoItem label="Chain ID" value={String(chainInfo.chainId)} />
                  <InfoItem label="Difficulty" value={chainInfo.difficulty || '0'} />
                  <InfoItem 
                    label="Best Block" 
                    value={chainInfo.bestBlockHash ? `${chainInfo.bestBlockHash.slice(0, 10)}...` : 'N/A'} 
                  />
                  <InfoItem 
                    label="Genesis" 
                    value={chainInfo.genesisHash ? `${chainInfo.genesisHash.slice(0, 10)}...` : 'N/A'} 
                  />
                </>
              )}
            </div>
            
            {/* Realtime P2P Connection Stats */}
            {status.connected && (
              <div className="mt-4 pt-4 border-t border-dark-600">
                <h3 className="text-sm font-medium text-stone-400 mb-3 flex items-center gap-2">
                  <Users size={14} />
                  P2P Mesh Status
                </h3>
                <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-3">
                  <InfoItem 
                    label="Network State" 
                    value={status.networkState || 'Unknown'} 
                  />
                  <InfoItem 
                    label="NAT Status" 
                    value={status.natStatus?.split(' ')[0] || 'Unknown'} 
                  />
                  <InfoItem 
                    label="Inbound" 
                    value={`${status.inboundPeers ?? 0}`} 
                  />
                  <InfoItem 
                    label="Outbound" 
                    value={`${status.outboundPeers ?? 0}`} 
                  />
                  <InfoItem 
                    label="Mesh Peers" 
                    value={`${status.meshPeers ?? 0}`} 
                  />
                  <InfoItem 
                    label="Gossip Peers" 
                    value={`${status.gossipPeers ?? 0}`} 
                  />
                  <InfoItem 
                    label="Dial Success" 
                    value={`${status.dialSuccesses ?? 0}/${(status.dialSuccesses ?? 0) + (status.dialFailures ?? 0)}`} 
                  />
                  <InfoItem 
                    label="Avg RTT" 
                    value={status.averageRttMs ? `${status.averageRttMs}ms` : 'N/A'} 
                  />
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="text-center py-8 text-stone-400">
            <p>Node is not running</p>
            <button
              onClick={handleStartNode}
              disabled={nodeLoading}
              className="mt-4 px-6 py-2 bg-pyrax-600 hover:bg-pyrax-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {nodeLoading ? (
                <RefreshCw className="animate-spin inline mr-2" size={16} />
              ) : (
                <Play className="inline mr-2" size={16} />
              )}
              Start Node
            </button>
          </div>
        )}
      </div>

      {/* Real-time Log Viewer */}
      <LogViewer />

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <QuickAction
          title="View Wallet"
          description="Check balances and send transactions"
          to="/wallet"
        />
        <QuickAction
          title="Start Mining"
          description="Earn PYRAX by mining blocks"
          to="/mining"
        />
        <QuickAction
          title="Explore Blocks"
          description="Browse the blockchain"
          to="/explorer"
        />
      </div>
    </div>
  );
}

function StatCard({ icon, label, value, subtext }: {
  icon: React.ReactNode;
  label: string;
  value: string;
  subtext: string;
}) {
  return (
    <div className="bg-dark-800 rounded-xl p-4">
      <div className="flex items-center gap-3 mb-2">
        {icon}
        <span className="text-sm text-stone-400">{label}</span>
      </div>
      <div className="text-2xl font-bold">{value}</div>
      <div className="text-xs text-stone-500 mt-1">{subtext}</div>
    </div>
  );
}

function InfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="text-xs text-stone-500">{label}</div>
      <div className="text-sm font-medium">{value}</div>
    </div>
  );
}

function QuickAction({ title, description, to }: {
  title: string;
  description: string;
  to: string;
}) {
  return (
    <Link
      to={to}
      className="block bg-dark-800 hover:bg-dark-700 rounded-xl p-4 transition-colors border border-dark-600 hover:border-pyrax-500"
    >
      <h3 className="font-semibold">{title}</h3>
      <p className="text-sm text-stone-400 mt-1">{description}</p>
    </Link>
  );
}
