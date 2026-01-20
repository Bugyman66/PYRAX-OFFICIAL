import { useState, useEffect } from 'react';
import { writeText } from '@tauri-apps/api/clipboard';
import { 
  Wallet as WalletIcon, 
  Plus, 
  Send, 
  Copy, 
  Check,
  RefreshCw,
  Lock,
  Unlock,
  Key,
  ArrowDownLeft,
  ArrowUpRight,
  Hammer,
  Clock,
  ExternalLink
} from 'lucide-react';
import { useWalletStore } from '../stores/walletStore';
import { useNodeStore } from '../stores/nodeStore';
import { formatBalance, truncateHash, formatTimeAgo } from '../lib/utils';

export default function Wallet() {
  const { status } = useNodeStore();
  const {
    info,
    addresses,
    transactions,
    selectedAddress,
    loading,
    error,
    unlockWallet,
    lockWallet,
    createWallet,
    fetchAddresses,
    fetchTransactions,
    sendTransaction,
    createAddress,
    setSelectedAddress,
  } = useWalletStore();

  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showSendModal, setShowSendModal] = useState(false);
  const [showUnlockModal, setShowUnlockModal] = useState(false);
  const [password, setPassword] = useState('');
  const [sendTo, setSendTo] = useState('');
  const [sendAmount, setSendAmount] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  const [copied, setCopied] = useState(false);
  const [activeTab, setActiveTab] = useState<'addresses' | 'transactions'>('addresses');

  useEffect(() => {
    if (info && !info.locked) {
      fetchAddresses();
      fetchTransactions();
    }
  }, [info?.locked, fetchAddresses, fetchTransactions]);

  const handleCreateWallet = async () => {
    if (!password || password.length < 4) {
      return;
    }
    try {
      const newMnemonic = await createWallet(password);
      if (newMnemonic) {
        setMnemonic(newMnemonic);
      }
    } catch (e) {
      console.error('Failed to create wallet:', e);
    }
  };

  const handleUnlock = async () => {
    try {
      await unlockWallet(password);
      setShowUnlockModal(false);
      setPassword('');
    } catch (e) {
      console.error(e);
    }
  };

  const handleSend = async () => {
    try {
      await sendTransaction(sendTo, sendAmount);
      setShowSendModal(false);
      setSendTo('');
      setSendAmount('');
    } catch (e) {
      console.error(e);
    }
  };

  const copyAddress = async (address: string) => {
    try {
      // CLIPBOARD FIX: Use Tauri clipboard API
      await writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      // Fallback to navigator.clipboard
      try {
        await navigator.clipboard.writeText(address);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (_) {
        console.error('Failed to copy:', e);
      }
    }
  };

  const selectedAddr = addresses.find(a => a.address === selectedAddress);

  // Render create wallet modal - must be outside early returns
  const createWalletModal = showCreateModal && (
    <Modal onClose={() => { setShowCreateModal(false); setMnemonic(''); setPassword(''); }}>
      <h2 className="text-xl font-semibold mb-4">
        {mnemonic ? 'Backup Your Seed Phrase' : 'Create New Wallet'}
      </h2>
      {error && (
        <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded-lg text-red-300 text-sm">
          {error}
        </div>
      )}
      {mnemonic ? (
        <div className="space-y-4">
          <p className="text-sm text-gray-400">
            Write down these 12 words in order. This is the only way to recover your wallet.
          </p>
          <div className="p-4 bg-gray-700 rounded-lg font-mono text-sm break-all">
            {mnemonic}
          </div>
          <button
            onClick={() => { setShowCreateModal(false); setMnemonic(''); setPassword(''); }}
            className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors"
          >
            I've Saved My Seed Phrase
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Password (min 4 characters)</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Create a strong password"
              className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
            />
          </div>
          <button
            onClick={handleCreateWallet}
            disabled={loading || !password || password.length < 4}
            className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
          >
            {loading ? <RefreshCw className="animate-spin inline mr-2" size={16} /> : null}
            Create Wallet
          </button>
        </div>
      )}
    </Modal>
  );

  if (!info) {
    return (
      <div className="p-6">
        <div className="max-w-md mx-auto text-center py-12">
          <WalletIcon size={64} className="mx-auto text-gray-600 mb-4" />
          <h2 className="text-xl font-semibold mb-2">No Wallet Found</h2>
          <p className="text-gray-400 mb-6">Create a new wallet or import an existing one</p>
          {error && (
            <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded-lg text-red-300 text-sm">
              {error}
            </div>
          )}
          <div className="space-y-3">
            <button
              onClick={() => setShowCreateModal(true)}
              disabled={loading}
              className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
            >
              <Plus className="inline mr-2" size={20} />
              Create New Wallet
            </button>
            <button
              onClick={() => setShowUnlockModal(true)}
              disabled={loading}
              className="w-full px-4 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-50"
            >
              <Key className="inline mr-2" size={20} />
              Import Wallet
            </button>
          </div>
        </div>
        {createWalletModal}
      </div>
    );
  }

  if (info.locked) {
    return (
      <div className="p-6">
        <div className="max-w-md mx-auto text-center py-12">
          <Lock size={64} className="mx-auto text-gray-600 mb-4" />
          <h2 className="text-xl font-semibold mb-2">Wallet Locked</h2>
          <p className="text-gray-400 mb-6">Enter your password to unlock</p>
          <div className="space-y-3">
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Password"
              className="w-full px-4 py-3 bg-gray-800 border border-gray-700 rounded-lg focus:border-purple-500 outline-none"
            />
            <button
              onClick={handleUnlock}
              disabled={loading || !password}
              className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {loading ? <RefreshCw className="animate-spin inline mr-2" size={16} /> : <Unlock className="inline mr-2" size={16} />}
              Unlock Wallet
            </button>
          </div>
          {error && <p className="text-red-400 mt-4 text-sm">{error}</p>}
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Wallet</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={() => createAddress()}
            className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            <Plus size={16} />
            New Address
          </button>
          <button
            onClick={() => setShowSendModal(true)}
            disabled={!status?.connected}
            className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
          >
            <Send size={16} />
            Send
          </button>
          <button
            onClick={() => lockWallet()}
            className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            <Lock size={16} />
          </button>
        </div>
      </div>

      {/* Balance Card */}
      <div className="bg-gradient-to-r from-purple-600 to-pink-600 rounded-xl p-6">
        <div className="text-sm opacity-80">Total Balance</div>
        <div className="text-4xl font-bold mt-2">
          {formatBalance(info.totalBalance)} PYRAX
        </div>
        <div className="mt-4 text-sm opacity-80">
          {addresses.length} address{addresses.length !== 1 ? 'es' : ''}
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="bg-gray-800 rounded-xl overflow-hidden">
        <div className="flex border-b border-gray-700">
          <button
            onClick={() => setActiveTab('addresses')}
            className={`flex-1 px-6 py-4 text-sm font-medium transition-colors ${
              activeTab === 'addresses'
                ? 'bg-purple-600/20 text-purple-400 border-b-2 border-purple-500'
                : 'text-gray-400 hover:text-white hover:bg-gray-700/50'
            }`}
          >
            <WalletIcon size={16} className="inline mr-2" />
            Addresses ({addresses.length})
          </button>
          <button
            onClick={() => setActiveTab('transactions')}
            className={`flex-1 px-6 py-4 text-sm font-medium transition-colors ${
              activeTab === 'transactions'
                ? 'bg-purple-600/20 text-purple-400 border-b-2 border-purple-500'
                : 'text-gray-400 hover:text-white hover:bg-gray-700/50'
            }`}
          >
            <Clock size={16} className="inline mr-2" />
            Transactions ({transactions.length})
          </button>
        </div>

        <div className="p-6">
          {/* Addresses Tab */}
          {activeTab === 'addresses' && (
            <div className="space-y-2">
              {addresses.length === 0 ? (
                <p className="text-gray-400 text-center py-8">No addresses yet. Create one to get started.</p>
              ) : (
                addresses.map((addr) => (
                  <div
                    key={addr.address}
                    onClick={() => setSelectedAddress(addr.address)}
                    className={`p-4 rounded-lg cursor-pointer transition-colors ${
                      selectedAddress === addr.address
                        ? 'bg-purple-600/20 border border-purple-500'
                        : 'bg-gray-700 hover:bg-gray-650'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <div>
                        <div className="font-mono text-sm">{truncateHash(addr.address, 12)}</div>
                        {addr.label && <div className="text-xs text-gray-400 mt-1">{addr.label}</div>}
                      </div>
                      <div className="text-right">
                        <div className="font-semibold">{formatBalance(addr.balance)} PYRAX</div>
                        <button
                          onClick={(e) => { e.stopPropagation(); copyAddress(addr.address); }}
                          className="text-xs text-gray-400 hover:text-white mt-1"
                        >
                          {copied ? <Check size={14} className="inline" /> : <Copy size={14} className="inline" />}
                          {' '}Copy
                        </button>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          )}

          {/* Transactions Tab */}
          {activeTab === 'transactions' && (
            <div className="space-y-2">
              {transactions.length === 0 ? (
                <div className="text-center py-12">
                  <Clock size={48} className="mx-auto text-gray-600 mb-4" />
                  <p className="text-gray-400">No transactions yet</p>
                  <p className="text-sm text-gray-500 mt-2">
                    Receive mining rewards or use the faucet to get started
                  </p>
                </div>
              ) : (
                transactions.map((tx) => {
                  const isMining = tx.txType === 'mining' || tx.txType === 'coinbase';
                  const isReceive = tx.txType === 'receive' || isMining;
                  const isSend = tx.txType === 'send';
                  
                  return (
                    <div key={tx.hash} className="p-4 bg-gray-700 rounded-lg hover:bg-gray-650 transition-colors">
                      <div className="flex items-center gap-4">
                        {/* Icon */}
                        <div className={`p-2 rounded-full ${
                          isMining ? 'bg-yellow-500/20 text-yellow-400' :
                          isReceive ? 'bg-green-500/20 text-green-400' :
                          'bg-red-500/20 text-red-400'
                        }`}>
                          {isMining ? <Hammer size={20} /> :
                           isReceive ? <ArrowDownLeft size={20} /> :
                           <ArrowUpRight size={20} />}
                        </div>
                        
                        {/* Details */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="font-medium capitalize">
                              {isMining ? 'Mining Reward' : tx.txType}
                            </span>
                            <span className={`text-xs px-2 py-0.5 rounded-full ${
                              tx.status === 'confirmed' ? 'bg-green-500/20 text-green-400' : 'bg-yellow-500/20 text-yellow-400'
                            }`}>
                              {tx.status}
                            </span>
                          </div>
                          <div className="font-mono text-xs text-gray-400 truncate mt-1">
                            {truncateHash(tx.hash, 16)}
                          </div>
                          {tx.blockNumber && (
                            <div className="text-xs text-gray-500 mt-1">
                              Block #{tx.blockNumber}
                              {tx.timestamp && ` • ${formatTimeAgo(tx.timestamp)}`}
                            </div>
                          )}
                        </div>
                        
                        {/* Amount */}
                        <div className="text-right">
                          <div className={`font-semibold ${
                            isMining ? 'text-yellow-400' :
                            isReceive ? 'text-green-400' :
                            'text-red-400'
                          }`}>
                            {isReceive ? '+' : '-'}{formatBalance(tx.value)} PYRAX
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          )}
        </div>
      </div>

      {/* Send Modal */}
      {showSendModal && (
        <Modal onClose={() => setShowSendModal(false)}>
          <h2 className="text-xl font-semibold mb-4">Send PYRAX</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-1">To Address</label>
              <input
                type="text"
                value={sendTo}
                onChange={(e) => setSendTo(e.target.value)}
                placeholder="0x..."
                className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Amount</label>
              <input
                type="text"
                value={sendAmount}
                onChange={(e) => setSendAmount(e.target.value)}
                placeholder="0.0"
                className="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
              />
            </div>
            <button
              onClick={handleSend}
              disabled={loading || !sendTo || !sendAmount}
              className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {loading ? <RefreshCw className="animate-spin inline mr-2" size={16} /> : <Send className="inline mr-2" size={16} />}
              Send Transaction
            </button>
          </div>
        </Modal>
      )}

      {/* Create Wallet Modal - reuse the one defined earlier */}
      {createWalletModal}
    </div>
  );
}

function Modal({ children, onClose }: { children: React.ReactNode; onClose: () => void }) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-gray-800 rounded-xl p-6 max-w-md w-full mx-4" onClick={(e) => e.stopPropagation()}>
        {children}
      </div>
    </div>
  );
}
