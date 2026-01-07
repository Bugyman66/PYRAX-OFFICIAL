import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import { 
  Activity, 
  Blocks, 
  Users, 
  Zap,
  Play,
  Square,
  RefreshCw
} from 'lucide-react';
import { useNodeStore } from '../stores/nodeStore';
import { useWalletStore } from '../stores/walletStore';
import { useMinerStore } from '../stores/minerStore';
import { formatBalance, formatHashrate } from '../lib/utils';

export default function Dashboard() {
  const { status, chainInfo, error: nodeError, startNode, stopNode, fetchChainInfo, loading: nodeLoading } = useNodeStore();
  const { addresses } = useWalletStore();
  const { status: minerStatus } = useMinerStore();

  useEffect(() => {
    if (status?.running && status?.connected) {
      fetchChainInfo();
    }
  }, [status?.running, status?.connected, fetchChainInfo]);

  const totalBalance = addresses.reduce((sum, addr) => {
    return sum + parseFloat(addr.balance || '0');
  }, 0);

  const handleStartNode = async () => {
    try {
      await startNode();
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
        <div className="flex items-center gap-2">
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
          icon={<Blocks className="text-purple-400" />}
          label="Block Height"
          value={status && typeof status.blockHeight === 'number' ? status.blockHeight.toLocaleString() : '0'}
          subtext={status?.syncing ? `Syncing: ${(status?.syncProgress ?? 0).toFixed(1)}%` : 'Synced'}
        />
        <StatCard
          icon={<Users className="text-blue-400" />}
          label="Connected Peers"
          value={status && typeof status.peerCount === 'number' ? status.peerCount.toString() : '0'}
          subtext={status?.network || 'Not connected'}
        />
        <StatCard
          icon={<Activity className="text-green-400" />}
          label="Wallet Balance"
          value={formatBalance(totalBalance)}
          subtext="PYRAX"
        />
        <StatCard
          icon={<Zap className="text-yellow-400" />}
          label="Hashrate"
          value={minerStatus?.running ? formatHashrate(minerStatus.hashrate) : '0 H/s'}
          subtext={minerStatus?.running ? 'Mining' : 'Idle'}
        />
      </div>

      {/* Node Status Panel */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Activity size={20} />
          Node Status
        </h2>
        
        {nodeError && (
          <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded-lg text-red-300 text-sm">
            <strong>Error:</strong> {nodeError}
          </div>
        )}
        {status?.running ? (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <InfoItem label="Status" value={status.connected ? 'Connected' : 'Connecting...'} />
            <InfoItem label="Network" value={status.network || 'Unknown'} />
            <InfoItem label="Version" value={status.version || '0.1.0'} />
            <InfoItem label="Peers" value={String(status.peerCount || 0)} />
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
        ) : (
          <div className="text-center py-8 text-gray-400">
            <p>Node is not running</p>
            <button
              onClick={handleStartNode}
              disabled={nodeLoading}
              className="mt-4 px-6 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
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
    <div className="bg-gray-800 rounded-xl p-4">
      <div className="flex items-center gap-3 mb-2">
        {icon}
        <span className="text-sm text-gray-400">{label}</span>
      </div>
      <div className="text-2xl font-bold">{value}</div>
      <div className="text-xs text-gray-500 mt-1">{subtext}</div>
    </div>
  );
}

function InfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="text-xs text-gray-500">{label}</div>
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
      className="block bg-gray-800 hover:bg-gray-750 rounded-xl p-4 transition-colors border border-gray-700 hover:border-purple-500"
    >
      <h3 className="font-semibold">{title}</h3>
      <p className="text-sm text-gray-400 mt-1">{description}</p>
    </Link>
  );
}
