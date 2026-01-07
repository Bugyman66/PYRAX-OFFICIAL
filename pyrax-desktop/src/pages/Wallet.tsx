import { useState, useEffect } from 'react';
import { 
  Wallet as WalletIcon, 
  Plus, 
  Send, 
  Copy, 
  Check,
  RefreshCw,
  Lock,
  Unlock,
  Key
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

  const copyAddress = (address: string) => {
    navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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

      {/* Addresses */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Addresses</h2>
        <div className="space-y-2">
          {addresses.map((addr) => (
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
          ))}
        </div>
      </div>

      {/* Transactions */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Recent Transactions</h2>
        {transactions.length === 0 ? (
          <p className="text-gray-400 text-center py-8">No transactions yet</p>
        ) : (
          <div className="space-y-2">
            {transactions.slice(0, 10).map((tx) => (
              <div key={tx.hash} className="p-4 bg-gray-700 rounded-lg">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-mono text-sm">{truncateHash(tx.hash)}</div>
                    <div className="text-xs text-gray-400 mt-1">
                      {tx.txType} • {tx.status}
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={tx.txType === 'receive' ? 'text-green-400' : 'text-red-400'}>
                      {tx.txType === 'receive' ? '+' : '-'}{formatBalance(tx.value)} PYRAX
                    </div>
                    {tx.timestamp && (
                      <div className="text-xs text-gray-500">{formatTimeAgo(tx.timestamp)}</div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
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
