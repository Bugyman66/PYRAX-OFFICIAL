'use client'

import { useState, Suspense } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { 
  MagnifyingGlassIcon,
  WalletIcon,
  ArrowDownLeftIcon,
  ArrowUpRightIcon,
  CubeIcon,
  ClockIcon,
  DocumentDuplicateIcon,
  CheckIcon,
} from '@heroicons/react/24/outline'
import { getAddressBalance, getAddressTransactions, type AddressInfo, type AddressTransactions, type AddressTx } from '@/lib/rpc'
import { useNetwork } from '@/context/NetworkContext'

function formatPyrax(value: number): string {
  const pyrax = value / 100_000_000
  return pyrax.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })
}

function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor(Date.now() / 1000 - timestamp)
  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
  return `${Math.floor(seconds / 86400)}d ago`
}

function truncateHash(hash: string, chars: number = 8): string {
  if (hash.length <= chars * 2) return hash
  return `${hash.slice(0, chars)}...${hash.slice(-chars)}`
}

function AccountPageContent() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const { networkState } = useNetwork()
  
  const [searchInput, setSearchInput] = useState(searchParams.get('address') || '')
  const [address, setAddress] = useState(searchParams.get('address') || '')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [accountInfo, setAccountInfo] = useState<AddressInfo | null>(null)
  const [transactions, setTransactions] = useState<AddressTransactions | null>(null)
  const [copied, setCopied] = useState(false)

  const handleSearch = async (e?: React.FormEvent) => {
    e?.preventDefault()
    
    if (!searchInput.trim()) {
      setError('Please enter an address')
      return
    }

    // Validate address format (0x + 40 hex chars)
    const cleanAddress = searchInput.trim()
    if (!/^0x[a-fA-F0-9]{40}$/.test(cleanAddress)) {
      setError('Invalid address format. Address should be 0x followed by 40 hex characters.')
      return
    }

    setLoading(true)
    setError(null)
    setAddress(cleanAddress)

    try {
      const [balanceResult, txResult] = await Promise.all([
        getAddressBalance(cleanAddress, networkState.network.id),
        getAddressTransactions(cleanAddress, 50, networkState.network.id),
      ])

      if (!balanceResult) {
        setError('Address not found or no activity')
        setAccountInfo(null)
        setTransactions(null)
      } else {
        setAccountInfo(balanceResult)
        setTransactions(txResult)
        // Update URL with address
        router.push(`/account?address=${cleanAddress}`, { scroll: false })
      }
    } catch (err) {
      setError('Failed to fetch account data')
      console.error(err)
    } finally {
      setLoading(false)
    }
  }

  const copyAddress = () => {
    navigator.clipboard.writeText(address)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  // Auto-search if address is in URL
  useState(() => {
    const urlAddress = searchParams.get('address')
    if (urlAddress) {
      setSearchInput(urlAddress)
      setAddress(urlAddress)
      handleSearch()
    }
  })

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white flex items-center gap-3">
          <WalletIcon className="h-8 w-8 text-pyrax-500" />
          Account Lookup
        </h1>
        <p className="text-stone-400 mt-2">
          Search for any address to view balance, UTXOs, and transaction history
        </p>
      </div>

      {/* Search Box */}
      <form onSubmit={handleSearch} className="mb-8">
        <div className="flex gap-4">
          <div className="flex-1 relative">
            <MagnifyingGlassIcon className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500" />
            <input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Enter address (0x...)"
              className="w-full pl-12 pr-4 py-4 bg-stone-900 border border-stone-700 rounded-xl text-white placeholder:text-stone-500 focus:border-pyrax-500 focus:outline-none font-mono"
            />
          </div>
          <button
            type="submit"
            disabled={loading}
            className="px-8 py-4 pyrax-gradient rounded-xl font-semibold text-white hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            {loading ? 'Searching...' : 'Search'}
          </button>
        </div>
        {error && (
          <p className="mt-3 text-red-400 text-sm">{error}</p>
        )}
      </form>

      {/* Account Info */}
      {accountInfo && (
        <div className="space-y-6">
          {/* Balance Card */}
          <div className="bg-gradient-to-r from-pyrax-600 to-pink-600 rounded-2xl p-6">
            <div className="flex items-start justify-between">
              <div>
                <p className="text-white/80 text-sm">Address</p>
                <div className="flex items-center gap-2 mt-1">
                  <p className="font-mono text-white text-lg">{truncateHash(address, 12)}</p>
                  <button
                    onClick={copyAddress}
                    className="p-1.5 rounded-lg bg-white/20 hover:bg-white/30 transition-colors"
                  >
                    {copied ? (
                      <CheckIcon className="h-4 w-4 text-white" />
                    ) : (
                      <DocumentDuplicateIcon className="h-4 w-4 text-white" />
                    )}
                  </button>
                </div>
              </div>
              <div className="text-right">
                <p className="text-white/80 text-sm">Balance</p>
                <p className="text-3xl font-bold text-white mt-1">
                  {formatPyrax(accountInfo.balance)} PYRAX
                </p>
              </div>
            </div>
            
            <div className="grid grid-cols-3 gap-4 mt-6 pt-6 border-t border-white/20">
              <div>
                <p className="text-white/80 text-sm">UTXOs</p>
                <p className="text-xl font-semibold text-white">{accountInfo.utxo_count}</p>
              </div>
              <div>
                <p className="text-white/80 text-sm">Total Received</p>
                <p className="text-xl font-semibold text-green-300">
                  {transactions ? formatPyrax(transactions.total_received) : '0'} PYRAX
                </p>
              </div>
              <div>
                <p className="text-white/80 text-sm">Total Sent</p>
                <p className="text-xl font-semibold text-red-300">
                  {transactions ? formatPyrax(transactions.total_sent) : '0'} PYRAX
                </p>
              </div>
            </div>
          </div>

          {/* Transactions */}
          <div className="bg-stone-900 rounded-2xl border border-stone-800 overflow-hidden">
            <div className="px-6 py-4 border-b border-stone-800">
              <h2 className="text-lg font-semibold text-white flex items-center gap-2">
                <ClockIcon className="h-5 w-5 text-pyrax-500" />
                Transaction History
                {transactions && (
                  <span className="text-sm font-normal text-stone-400">
                    ({transactions.tx_count} transactions)
                  </span>
                )}
              </h2>
            </div>
            
            <div className="divide-y divide-stone-800">
              {!transactions || transactions.transactions.length === 0 ? (
                <div className="px-6 py-12 text-center">
                  <ClockIcon className="h-12 w-12 text-stone-600 mx-auto mb-4" />
                  <p className="text-stone-400">No transactions found</p>
                </div>
              ) : (
                transactions.transactions.map((tx) => (
                  <TransactionRow key={tx.txid} tx={tx} />
                ))
              )}
            </div>
          </div>

          {/* UTXOs */}
          {accountInfo.utxos.length > 0 && (
            <div className="bg-stone-900 rounded-2xl border border-stone-800 overflow-hidden">
              <div className="px-6 py-4 border-b border-stone-800">
                <h2 className="text-lg font-semibold text-white flex items-center gap-2">
                  <CubeIcon className="h-5 w-5 text-pyrax-500" />
                  Unspent Outputs (UTXOs)
                </h2>
              </div>
              
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-stone-800/50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-stone-400 uppercase">TXID</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-stone-400 uppercase">Output</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-stone-400 uppercase">Block</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-stone-400 uppercase">Type</th>
                      <th className="px-6 py-3 text-right text-xs font-medium text-stone-400 uppercase">Value</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-stone-800">
                    {accountInfo.utxos.map((utxo) => (
                      <tr key={`${utxo.txid}:${utxo.vout}`} className="hover:bg-stone-800/50 transition-colors">
                        <td className="px-6 py-4 font-mono text-sm text-stone-300">
                          {truncateHash(utxo.txid, 10)}
                        </td>
                        <td className="px-6 py-4 text-stone-400">
                          #{utxo.vout}
                        </td>
                        <td className="px-6 py-4 text-stone-400">
                          {utxo.height.toLocaleString()}
                        </td>
                        <td className="px-6 py-4">
                          {utxo.coinbase ? (
                            <span className="px-2 py-1 text-xs rounded-full bg-yellow-500/20 text-yellow-400">
                              Coinbase
                            </span>
                          ) : (
                            <span className="px-2 py-1 text-xs rounded-full bg-blue-500/20 text-blue-400">
                              Regular
                            </span>
                          )}
                        </td>
                        <td className="px-6 py-4 text-right font-semibold text-green-400">
                          {formatPyrax(utxo.value)} PYRAX
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Empty State */}
      {!accountInfo && !loading && !error && (
        <div className="text-center py-20">
          <WalletIcon className="h-16 w-16 text-stone-600 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-white mb-2">Search for an Account</h2>
          <p className="text-stone-400">
            Enter an address above to view balance and transaction history
          </p>
        </div>
      )}
    </div>
  )
}

function TransactionRow({ tx }: { tx: AddressTx }) {
  const isMining = tx.direction === 'mining' || tx.is_coinbase
  const isReceive = tx.direction === 'receive' || isMining
  
  return (
    <div className="px-6 py-4 hover:bg-stone-800/50 transition-colors">
      <div className="flex items-center gap-4">
        {/* Icon */}
        <div className={`p-2 rounded-full ${
          isMining ? 'bg-yellow-500/20 text-yellow-400' :
          isReceive ? 'bg-green-500/20 text-green-400' :
          'bg-red-500/20 text-red-400'
        }`}>
          {isMining ? (
            <CubeIcon className="h-5 w-5" />
          ) : isReceive ? (
            <ArrowDownLeftIcon className="h-5 w-5" />
          ) : (
            <ArrowUpRightIcon className="h-5 w-5" />
          )}
        </div>
        
        {/* Details */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-white capitalize">
              {isMining ? 'Mining Reward' : tx.direction}
            </span>
            <span className={`text-xs px-2 py-0.5 rounded-full ${
              tx.confirmations > 0 ? 'bg-green-500/20 text-green-400' : 'bg-yellow-500/20 text-yellow-400'
            }`}>
              {tx.confirmations > 0 ? 'Confirmed' : 'Pending'}
            </span>
          </div>
          <div className="font-mono text-xs text-stone-400 truncate mt-1">
            {truncateHash(tx.txid, 16)}
          </div>
          <div className="text-xs text-stone-500 mt-1">
            Block #{tx.block_height.toLocaleString()}
            {tx.timestamp > 0 && ` • ${formatTimeAgo(tx.timestamp)}`}
          </div>
        </div>
        
        {/* Amount */}
        <div className="text-right">
          <div className={`font-semibold ${
            isMining ? 'text-yellow-400' :
            isReceive ? 'text-green-400' :
            'text-red-400'
          }`}>
            {isReceive ? '+' : '-'}{formatPyrax(tx.value)} PYRAX
          </div>
          <div className="text-xs text-stone-500">
            {tx.confirmations} confirmations
          </div>
        </div>
      </div>
    </div>
  )
}

export default function AccountPage() {
  return (
    <Suspense fallback={<div className="flex items-center justify-center min-h-[400px]"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-pyrax-500"></div></div>}>
      <AccountPageContent />
    </Suspense>
  )
}
