import { useState, useEffect } from 'react';
import { Search, Blocks, ArrowRight, Clock, Hash } from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import { useNodeStore } from '../stores/nodeStore';
import { truncateHash, formatTimeAgo } from '../lib/utils';

interface Block {
  number: string;
  hash: string;
  parentHash: string;
  timestamp: string;
  miner: string;
  transactionCount: number;
  size: string;
  gasUsed: string;
}

interface Transaction {
  hash: string;
  from: string;
  to?: string;
  value: string;
  blockNumber?: string;
}

export default function Explorer() {
  const { status } = useNodeStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [recentBlocks, setRecentBlocks] = useState<Block[]>([]);
  const [selectedBlock, setSelectedBlock] = useState<Block | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (status?.connected) {
      fetchRecentBlocks();
    }
  }, [status?.connected]);

  const fetchRecentBlocks = async () => {
    try {
      const blocks = await invoke<Block[]>('get_recent_blocks', { limit: 10 });
      setRecentBlocks(blocks);
    } catch (e) {
      console.error('Failed to fetch blocks:', e);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;
    setLoading(true);
    try {
      if (searchQuery.startsWith('0x') && searchQuery.length === 66) {
        // Block or transaction hash
        const block = await invoke<Block | null>('get_block', { hash: searchQuery });
        if (block) {
          setSelectedBlock(block);
        } else {
          const tx = await invoke<Transaction | null>('get_transaction', { hash: searchQuery });
          if (tx) {
            // Show transaction details
          }
        }
      } else if (/^\d+$/.test(searchQuery)) {
        // Block number
        const block = await invoke<Block | null>('get_block', { number: parseInt(searchQuery) });
        if (block) {
          setSelectedBlock(block);
        }
      }
    } catch (e) {
      console.error('Search failed:', e);
    } finally {
      setLoading(false);
    }
  };

  if (!status?.connected) {
    return (
      <div className="p-6">
        <div className="text-center py-12 text-gray-400">
          <Blocks size={64} className="mx-auto mb-4 opacity-50" />
          <p>Connect to a node to explore the blockchain</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Block Explorer</h1>

      {/* Search Bar */}
      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={20} />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search by block number, hash, or transaction..."
            className="w-full pl-10 pr-4 py-3 bg-gray-800 border border-gray-700 rounded-lg focus:border-purple-500 outline-none"
          />
        </div>
        <button
          onClick={handleSearch}
          disabled={loading}
          className="px-6 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
        >
          Search
        </button>
      </div>

      {/* Selected Block Details */}
      {selectedBlock && (
        <div className="bg-gray-800 rounded-xl p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold">Block #{parseInt(selectedBlock.number, 16).toLocaleString()}</h2>
            <button onClick={() => setSelectedBlock(null)} className="text-gray-400 hover:text-white">
              ✕
            </button>
          </div>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <div className="text-gray-500">Hash</div>
              <div className="font-mono">{truncateHash(selectedBlock.hash, 16)}</div>
            </div>
            <div>
              <div className="text-gray-500">Parent Hash</div>
              <div className="font-mono">{truncateHash(selectedBlock.parentHash, 16)}</div>
            </div>
            <div>
              <div className="text-gray-500">Timestamp</div>
              <div>{new Date(parseInt(selectedBlock.timestamp, 16) * 1000).toLocaleString()}</div>
            </div>
            <div>
              <div className="text-gray-500">Miner</div>
              <div className="font-mono">{truncateHash(selectedBlock.miner, 10)}</div>
            </div>
            <div>
              <div className="text-gray-500">Transactions</div>
              <div>{selectedBlock.transactionCount}</div>
            </div>
            <div>
              <div className="text-gray-500">Size</div>
              <div>{parseInt(selectedBlock.size, 16).toLocaleString()} bytes</div>
            </div>
          </div>
        </div>
      )}

      {/* Recent Blocks */}
      <div className="bg-gray-800 rounded-xl p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Blocks size={20} />
            Recent Blocks
          </h2>
          <button
            onClick={fetchRecentBlocks}
            className="text-sm text-purple-400 hover:text-purple-300"
          >
            Refresh
          </button>
        </div>
        
        {recentBlocks.length === 0 ? (
          <p className="text-center py-8 text-gray-400">No blocks found</p>
        ) : (
          <div className="space-y-2">
            {recentBlocks.map((block) => (
              <div
                key={block.hash}
                onClick={() => setSelectedBlock(block)}
                className="flex items-center justify-between p-4 bg-gray-700 rounded-lg cursor-pointer hover:bg-gray-650 transition-colors"
              >
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 bg-purple-600/20 rounded-lg flex items-center justify-center">
                    <Blocks className="text-purple-400" size={24} />
                  </div>
                  <div>
                    <div className="font-semibold">Block #{parseInt(block.number, 16).toLocaleString()}</div>
                    <div className="text-sm text-gray-400 flex items-center gap-2">
                      <Clock size={14} />
                      {formatTimeAgo(parseInt(block.timestamp, 16))}
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm">{block.transactionCount} txs</div>
                  <div className="text-xs text-gray-500 font-mono">{truncateHash(block.hash)}</div>
                </div>
                <ArrowRight className="text-gray-500" size={20} />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
