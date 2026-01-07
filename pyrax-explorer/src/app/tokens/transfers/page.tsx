'use client'

import { useState, useEffect, useCallback } from 'react'
import Link from 'next/link'
import { 
  ArrowsRightLeftIcon, 
  ChevronLeftIcon, 
  ChevronRightIcon,
  ChevronDoubleLeftIcon,
  ChevronDoubleRightIcon,
  ArrowPathIcon,
  ChevronUpIcon,
  ChevronDownIcon,
  FunnelIcon,
  MagnifyingGlassIcon,
  XMarkIcon,
  ArrowLongRightIcon,
} from '@heroicons/react/24/outline'

interface TokenTransfer {
  txHash: string
  blockNumber: number
  timestamp: number
  tokenAddress: string
  tokenName: string
  tokenSymbol: string
  tokenType: string
  from: string
  to: string
  amount: string
  valueUsd: number
  method: string
}

interface Pagination {
  page: number
  pageSize: number
  totalTransfers: number
  totalPages: number
}

interface Filters {
  sortBy: string
  sortOrder: string
  tokenType: string
  minValue: number
  tokenAddress: string
  address: string
}

const PAGE_SIZE_OPTIONS = [10, 25, 50, 100]
const TOKEN_TYPES = [
  { value: 'all', label: 'All Types' },
  { value: 'ERC20', label: 'ERC-20 (EVM)' },
  { value: 'ERC721', label: 'ERC-721 NFT (EVM)' },
  { value: 'ERC1155', label: 'ERC-1155 (EVM)' },
  { value: 'WASM20', label: 'Fungible (WASM)' },
  { value: 'WASM721', label: 'NFT (WASM)' },
]

function formatNumber(num: number): string {
  if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`
  if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`
  if (num >= 1e3) return `${(num / 1e3).toFixed(2)}K`
  return new Intl.NumberFormat().format(num)
}

function formatValue(value: number): string {
  if (value === 0) return '-'
  if (value < 0.01) return '< $0.01'
  if (value >= 1e6) return `$${(value / 1e6).toFixed(2)}M`
  if (value >= 1e3) return `$${(value / 1e3).toFixed(2)}K`
  return `$${value.toFixed(2)}`
}

function formatAddress(address: string): string {
  if (!address || address.length < 16) return address || 'Unknown'
  return `${address.slice(0, 8)}...${address.slice(-6)}`
}

function formatTxHash(hash: string): string {
  if (!hash || hash.length < 20) return hash || 'Unknown'
  return `${hash.slice(0, 10)}...${hash.slice(-6)}`
}

function formatAmount(amount: string, decimals: number = 18): string {
  try {
    const num = parseFloat(amount)
    if (isNaN(num)) return amount
    if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`
    if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`
    if (num >= 1e3) return `${(num / 1e3).toFixed(2)}K`
    if (num < 0.0001 && num > 0) return '< 0.0001'
    return num.toFixed(4)
  } catch {
    return amount
  }
}

function timeAgo(timestamp: number): string {
  const seconds = Math.floor(Date.now() / 1000 - timestamp)
  if (seconds < 60) return `${seconds}s ago`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days}d ago`
  return new Date(timestamp * 1000).toLocaleDateString()
}

export default function TokenTransfersPage() {
  const [transfers, setTransfers] = useState<TokenTransfer[]>([])
  const [pagination, setPagination] = useState<Pagination>({
    page: 1,
    pageSize: 25,
    totalTransfers: 0,
    totalPages: 0,
  })
  const [filters, setFilters] = useState<Filters>({
    sortBy: 'timestamp',
    sortOrder: 'desc',
    tokenType: 'all',
    minValue: 0,
    tokenAddress: '',
    address: '',
  })
  const [showFilters, setShowFilters] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [addressInput, setAddressInput] = useState('')
  const [tokenInput, setTokenInput] = useState('')

  const fetchTransfers = useCallback(async (page: number, pageSize: number, currentFilters: Filters) => {
    setLoading(true)
    setError(null)
    
    try {
      const params = new URLSearchParams({
        page: page.toString(),
        pageSize: pageSize.toString(),
        sortBy: currentFilters.sortBy,
        sortOrder: currentFilters.sortOrder,
        type: currentFilters.tokenType,
        minValue: currentFilters.minValue.toString(),
        token: currentFilters.tokenAddress,
        address: currentFilters.address,
      })
      
      const response = await fetch(`/api/tokens/transfers?${params}`)
      const data = await response.json()
      
      if (data.error) {
        setError(data.error)
      } else {
        setTransfers(data.transfers || [])
        setPagination(data.pagination || { page: 1, pageSize, totalTransfers: 0, totalPages: 0 })
      }
    } catch (err) {
      setError('Failed to fetch token transfers')
      console.error(err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchTransfers(pagination.page, pagination.pageSize, filters)
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= pagination.totalPages) {
      fetchTransfers(newPage, pagination.pageSize, filters)
    }
  }

  const handlePageSizeChange = (newPageSize: number) => {
    fetchTransfers(1, newPageSize, filters)
  }

  const handleSort = (field: string) => {
    const newOrder = filters.sortBy === field && filters.sortOrder === 'desc' ? 'asc' : 'desc'
    const newFilters = { ...filters, sortBy: field, sortOrder: newOrder }
    setFilters(newFilters)
    fetchTransfers(1, pagination.pageSize, newFilters)
  }

  const handleFilterChange = (field: keyof Filters, value: string | number) => {
    const newFilters = { ...filters, [field]: value }
    setFilters(newFilters)
    fetchTransfers(1, pagination.pageSize, newFilters)
  }

  const handleAddressSearch = () => {
    const newFilters = { ...filters, address: addressInput }
    setFilters(newFilters)
    fetchTransfers(1, pagination.pageSize, newFilters)
  }

  const handleTokenSearch = () => {
    const newFilters = { ...filters, tokenAddress: tokenInput }
    setFilters(newFilters)
    fetchTransfers(1, pagination.pageSize, newFilters)
  }

  const clearAddressFilter = () => {
    setAddressInput('')
    const newFilters = { ...filters, address: '' }
    setFilters(newFilters)
    fetchTransfers(1, pagination.pageSize, newFilters)
  }

  const clearTokenFilter = () => {
    setTokenInput('')
    const newFilters = { ...filters, tokenAddress: '' }
    setFilters(newFilters)
    fetchTransfers(1, pagination.pageSize, newFilters)
  }

  const handleRefresh = () => {
    fetchTransfers(pagination.page, pagination.pageSize, filters)
  }

  const SortIcon = ({ field }: { field: string }) => {
    if (filters.sortBy !== field) return null
    return filters.sortOrder === 'desc' 
      ? <ChevronDownIcon className="h-4 w-4 inline ml-1" />
      : <ChevronUpIcon className="h-4 w-4 inline ml-1" />
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="sm:flex sm:items-center sm:justify-between mb-6">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-green-500/10">
            <ArrowsRightLeftIcon className="h-6 w-6 text-green-500" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Token Transfers</h1>
            <p className="text-sm text-stone-400">
              {pagination.totalTransfers > 0 
                ? `${formatNumber(pagination.totalTransfers)} token transfers`
                : 'Browse all token transfers on the chain'}
            </p>
          </div>
        </div>
        
        <div className="mt-4 sm:mt-0 flex items-center gap-4">
          {/* Refresh Button */}
          <button
            onClick={handleRefresh}
            disabled={loading}
            className="inline-flex items-center gap-2 rounded-lg bg-stone-800 px-3 py-2 text-sm font-medium text-white hover:bg-stone-700 disabled:opacity-50 transition-colors"
          >
            <ArrowPathIcon className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </button>

          {/* Filter Toggle */}
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`inline-flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${
              showFilters ? 'bg-pyrax-500 text-white' : 'bg-stone-800 text-white hover:bg-stone-700'
            }`}
          >
            <FunnelIcon className="h-4 w-4" />
            Filters
          </button>
          
          {/* Page Size Selector */}
          <div className="flex items-center gap-2">
            <label htmlFor="pageSize" className="text-sm text-stone-400">Show:</label>
            <select
              id="pageSize"
              value={pagination.pageSize}
              onChange={(e) => handlePageSizeChange(Number(e.target.value))}
              className="rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pyrax-500"
            >
              {PAGE_SIZE_OPTIONS.map((size) => (
                <option key={size} value={size}>
                  {size}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className="mb-6 p-4 rounded-xl bg-stone-900 border border-stone-800">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* Address Filter */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">From/To Address</label>
              <div className="relative">
                <input
                  type="text"
                  value={addressInput}
                  onChange={(e) => setAddressInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAddressSearch()}
                  placeholder="0x..."
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 pl-3 pr-9 py-2 text-sm text-white font-mono focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                />
                {addressInput && (
                  <button
                    onClick={clearAddressFilter}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-stone-500 hover:text-white"
                  >
                    <XMarkIcon className="h-4 w-4" />
                  </button>
                )}
              </div>
            </div>

            {/* Token Address Filter */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Token Contract</label>
              <div className="relative">
                <input
                  type="text"
                  value={tokenInput}
                  onChange={(e) => setTokenInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleTokenSearch()}
                  placeholder="0x..."
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 pl-3 pr-9 py-2 text-sm text-white font-mono focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                />
                {tokenInput && (
                  <button
                    onClick={clearTokenFilter}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-stone-500 hover:text-white"
                  >
                    <XMarkIcon className="h-4 w-4" />
                  </button>
                )}
              </div>
            </div>

            {/* Token Type */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Token Type</label>
              <select
                value={filters.tokenType}
                onChange={(e) => handleFilterChange('tokenType', e.target.value)}
                className="w-full rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pyrax-500"
              >
                {TOKEN_TYPES.map((type) => (
                  <option key={type.value} value={type.value}>
                    {type.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Min Value */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Min Value (USD)</label>
              <input
                type="number"
                value={filters.minValue || ''}
                onChange={(e) => handleFilterChange('minValue', parseFloat(e.target.value) || 0)}
                placeholder="0"
                min={0}
                className="w-full rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pyrax-500"
              />
            </div>
          </div>
        </div>
      )}

      {/* Error Message */}
      {error && (
        <div className="mb-6 rounded-lg bg-red-500/10 border border-red-500/20 p-4 text-red-400">
          {error}
        </div>
      )}

      {/* Transfers Table */}
      <div className="overflow-hidden rounded-xl bg-stone-900 border border-stone-800">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-800">
            <thead className="bg-stone-800/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Txn Hash
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('timestamp')}
                >
                  Age <SortIcon field="timestamp" />
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Token
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  From
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  To
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('amount')}
                >
                  Amount <SortIcon field="amount" />
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('value_usd')}
                >
                  Value <SortIcon field="value_usd" />
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {loading ? (
                // Loading skeletons
                Array.from({ length: pagination.pageSize || 10 }).map((_, i) => (
                  <tr key={i}>
                    <td className="px-4 py-4">
                      <div className="h-5 w-24 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="flex items-center gap-2">
                        <div className="h-6 w-6 bg-stone-800 rounded-full animate-pulse" />
                        <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                      </div>
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-24 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-4 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-24 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                  </tr>
                ))
              ) : transfers.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-4 py-12 text-center text-stone-500">
                    No token transfers found
                  </td>
                </tr>
              ) : (
                transfers.map((transfer, index) => (
                  <tr 
                    key={`${transfer.txHash}-${index}`} 
                    className="hover:bg-stone-800/50 transition-colors"
                  >
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/tx/${transfer.txHash}`}
                        className="text-sm font-mono text-pyrax-400 hover:text-pyrax-300 transition-colors"
                      >
                        {formatTxHash(transfer.txHash)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm text-stone-400" title={new Date(transfer.timestamp * 1000).toLocaleString()}>
                        {timeAgo(transfer.timestamp)}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/token/${transfer.tokenAddress}`}
                        className="flex items-center gap-2 group"
                      >
                        <div className={`h-6 w-6 rounded-full flex items-center justify-center text-white font-bold text-[10px] ${
                          transfer.tokenType.startsWith('ERC') ? 'bg-gradient-to-br from-blue-400 to-indigo-500' :
                          'bg-gradient-to-br from-orange-400 to-red-500'
                        }`}>
                          {transfer.tokenSymbol.slice(0, 2).toUpperCase()}
                        </div>
                        <span className="text-sm text-white group-hover:text-pyrax-400 transition-colors">
                          {transfer.tokenSymbol}
                        </span>
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/address/${transfer.from}`}
                        className="text-sm font-mono text-stone-300 hover:text-white transition-colors"
                      >
                        {formatAddress(transfer.from)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <ArrowLongRightIcon className="h-4 w-4 text-green-500" />
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/address/${transfer.to}`}
                        className="text-sm font-mono text-stone-300 hover:text-white transition-colors"
                      >
                        {formatAddress(transfer.to)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm font-medium text-white">
                        {formatAmount(transfer.amount)}
                      </span>
                      <span className="text-xs text-stone-500 ml-1">
                        {transfer.tokenSymbol}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm text-stone-300">
                        {formatValue(transfer.valueUsd)}
                      </span>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        {pagination.totalPages > 1 && (
          <div className="flex items-center justify-between border-t border-stone-800 bg-stone-800/30 px-4 py-3">
            <div className="flex items-center text-sm text-stone-400">
              Showing{' '}
              <span className="font-medium text-white mx-1">
                {((pagination.page - 1) * pagination.pageSize) + 1}
              </span>
              to{' '}
              <span className="font-medium text-white mx-1">
                {Math.min(pagination.page * pagination.pageSize, pagination.totalTransfers)}
              </span>
              of{' '}
              <span className="font-medium text-white mx-1">
                {formatNumber(pagination.totalTransfers)}
              </span>
              transfers
            </div>
            
            <div className="flex items-center gap-1">
              {/* First Page */}
              <button
                onClick={() => handlePageChange(1)}
                disabled={pagination.page === 1 || loading}
                className="p-2 rounded-lg text-stone-400 hover:text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                title="First page"
              >
                <ChevronDoubleLeftIcon className="h-4 w-4" />
              </button>
              
              {/* Previous Page */}
              <button
                onClick={() => handlePageChange(pagination.page - 1)}
                disabled={pagination.page === 1 || loading}
                className="p-2 rounded-lg text-stone-400 hover:text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                title="Previous page"
              >
                <ChevronLeftIcon className="h-4 w-4" />
              </button>
              
              {/* Page Numbers */}
              <div className="flex items-center gap-1 mx-2">
                {Array.from({ length: Math.min(5, pagination.totalPages) }, (_, i) => {
                  let pageNum: number
                  if (pagination.totalPages <= 5) {
                    pageNum = i + 1
                  } else if (pagination.page <= 3) {
                    pageNum = i + 1
                  } else if (pagination.page >= pagination.totalPages - 2) {
                    pageNum = pagination.totalPages - 4 + i
                  } else {
                    pageNum = pagination.page - 2 + i
                  }
                  
                  return (
                    <button
                      key={pageNum}
                      onClick={() => handlePageChange(pageNum)}
                      disabled={loading}
                      className={`min-w-[2.5rem] px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                        pagination.page === pageNum
                          ? 'bg-pyrax-500 text-white'
                          : 'text-stone-400 hover:text-white hover:bg-stone-700'
                      }`}
                    >
                      {pageNum}
                    </button>
                  )
                })}
              </div>
              
              {/* Next Page */}
              <button
                onClick={() => handlePageChange(pagination.page + 1)}
                disabled={pagination.page === pagination.totalPages || loading}
                className="p-2 rounded-lg text-stone-400 hover:text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                title="Next page"
              >
                <ChevronRightIcon className="h-4 w-4" />
              </button>
              
              {/* Last Page */}
              <button
                onClick={() => handlePageChange(pagination.totalPages)}
                disabled={pagination.page === pagination.totalPages || loading}
                className="p-2 rounded-lg text-stone-400 hover:text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                title="Last page"
              >
                <ChevronDoubleRightIcon className="h-4 w-4" />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
