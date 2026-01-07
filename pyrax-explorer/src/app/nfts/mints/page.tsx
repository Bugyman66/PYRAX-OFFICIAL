'use client'

import { useState, useEffect, useCallback } from 'react'
import Link from 'next/link'
import { 
  SparklesIcon, 
  ChevronLeftIcon, 
  ChevronRightIcon,
  ChevronDoubleLeftIcon,
  ChevronDoubleRightIcon,
  ArrowPathIcon,
  ChevronUpIcon,
  ChevronDownIcon,
  FunnelIcon,
  XMarkIcon,
  PhotoIcon,
} from '@heroicons/react/24/outline'

interface NFTMint {
  txHash: string
  blockNumber: number
  timestamp: number
  collectionAddress: string
  collectionName: string
  tokenId: string
  tokenType: string
  creator: string
  owner: string
  mintPrice: number
  mintPriceUsd: number
  holders: number
  imageUrl: string | null
  name: string | null
}

interface Pagination {
  page: number
  pageSize: number
  totalMints: number
  totalPages: number
}

interface Filters {
  sortBy: string
  sortOrder: string
  nftType: string
  minValue: number
  minHolders: number
  days: number
  collectionAddress: string
}

const PAGE_SIZE_OPTIONS = [10, 25, 50, 100]
const NFT_TYPES = [
  { value: 'all', label: 'All Types' },
  { value: 'ERC721', label: 'ERC-721 (EVM)' },
  { value: 'ERC1155', label: 'ERC-1155 (EVM)' },
  { value: 'WASM721', label: 'NFT (WASM)' },
]
const TIME_RANGES = [
  { value: 1, label: 'Last 24 hours' },
  { value: 3, label: 'Last 3 days' },
  { value: 5, label: 'Last 5 days' },
  { value: 7, label: 'Last 7 days' },
  { value: 14, label: 'Last 14 days' },
  { value: 30, label: 'Last 30 days' },
]

function formatNumber(num: number): string {
  if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`
  if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`
  if (num >= 1e3) return `${(num / 1e3).toFixed(2)}K`
  return new Intl.NumberFormat().format(num)
}

function formatValue(value: number): string {
  if (value === 0) return 'Free'
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

function formatTokenId(tokenId: string): string {
  if (!tokenId) return '#?'
  if (tokenId.length <= 8) return `#${tokenId}`
  return `#${tokenId.slice(0, 6)}...`
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

export default function NFTMintsPage() {
  const [mints, setMints] = useState<NFTMint[]>([])
  const [pagination, setPagination] = useState<Pagination>({
    page: 1,
    pageSize: 25,
    totalMints: 0,
    totalPages: 0,
  })
  const [filters, setFilters] = useState<Filters>({
    sortBy: 'timestamp',
    sortOrder: 'desc',
    nftType: 'all',
    minValue: 0,
    minHolders: 0,
    days: 5,
    collectionAddress: '',
  })
  const [showFilters, setShowFilters] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [collectionInput, setCollectionInput] = useState('')

  const fetchMints = useCallback(async (page: number, pageSize: number, currentFilters: Filters) => {
    setLoading(true)
    setError(null)
    
    try {
      const params = new URLSearchParams({
        page: page.toString(),
        pageSize: pageSize.toString(),
        sortBy: currentFilters.sortBy,
        sortOrder: currentFilters.sortOrder,
        type: currentFilters.nftType,
        minValue: currentFilters.minValue.toString(),
        minHolders: currentFilters.minHolders.toString(),
        days: currentFilters.days.toString(),
        collection: currentFilters.collectionAddress,
      })
      
      const response = await fetch(`/api/nfts/mints?${params}`)
      const data = await response.json()
      
      if (data.error) {
        setError(data.error)
      } else {
        setMints(data.mints || [])
        setPagination(data.pagination || { page: 1, pageSize, totalMints: 0, totalPages: 0 })
      }
    } catch (err) {
      setError('Failed to fetch NFT mints')
      console.error(err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchMints(pagination.page, pagination.pageSize, filters)
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= pagination.totalPages) {
      fetchMints(newPage, pagination.pageSize, filters)
    }
  }

  const handlePageSizeChange = (newPageSize: number) => {
    fetchMints(1, newPageSize, filters)
  }

  const handleSort = (field: string) => {
    const newOrder = filters.sortBy === field && filters.sortOrder === 'desc' ? 'asc' : 'desc'
    const newFilters = { ...filters, sortBy: field, sortOrder: newOrder }
    setFilters(newFilters)
    fetchMints(1, pagination.pageSize, newFilters)
  }

  const handleFilterChange = (field: keyof Filters, value: string | number) => {
    const newFilters = { ...filters, [field]: value }
    setFilters(newFilters)
    fetchMints(1, pagination.pageSize, newFilters)
  }

  const handleCollectionSearch = () => {
    const newFilters = { ...filters, collectionAddress: collectionInput }
    setFilters(newFilters)
    fetchMints(1, pagination.pageSize, newFilters)
  }

  const clearCollectionFilter = () => {
    setCollectionInput('')
    const newFilters = { ...filters, collectionAddress: '' }
    setFilters(newFilters)
    fetchMints(1, pagination.pageSize, newFilters)
  }

  const handleRefresh = () => {
    fetchMints(pagination.page, pagination.pageSize, filters)
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
          <div className="p-2 rounded-lg bg-pink-500/10">
            <SparklesIcon className="h-6 w-6 text-pink-500" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Recent NFT Mints</h1>
            <p className="text-sm text-stone-400">
              {pagination.totalMints > 0 
                ? `${formatNumber(pagination.totalMints)} NFTs minted in the last ${filters.days} days`
                : `Browse NFTs minted in the last ${filters.days} days`}
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
              showFilters ? 'bg-pink-500 text-white' : 'bg-stone-800 text-white hover:bg-stone-700'
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
              className="rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pink-500"
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
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
            {/* Time Range */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Time Range</label>
              <select
                value={filters.days}
                onChange={(e) => handleFilterChange('days', parseInt(e.target.value))}
                className="w-full rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pink-500"
              >
                {TIME_RANGES.map((range) => (
                  <option key={range.value} value={range.value}>
                    {range.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Collection Address Filter */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Collection</label>
              <div className="relative">
                <input
                  type="text"
                  value={collectionInput}
                  onChange={(e) => setCollectionInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleCollectionSearch()}
                  placeholder="0x..."
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 pl-3 pr-9 py-2 text-sm text-white font-mono focus:outline-none focus:ring-2 focus:ring-pink-500"
                />
                {collectionInput && (
                  <button
                    onClick={clearCollectionFilter}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-stone-500 hover:text-white"
                  >
                    <XMarkIcon className="h-4 w-4" />
                  </button>
                )}
              </div>
            </div>

            {/* NFT Type */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">NFT Type</label>
              <select
                value={filters.nftType}
                onChange={(e) => handleFilterChange('nftType', e.target.value)}
                className="w-full rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pink-500"
              >
                {NFT_TYPES.map((type) => (
                  <option key={type.value} value={type.value}>
                    {type.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Min Value */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Min Mint Price (USD)</label>
              <input
                type="number"
                value={filters.minValue || ''}
                onChange={(e) => handleFilterChange('minValue', parseFloat(e.target.value) || 0)}
                placeholder="0"
                min={0}
                className="w-full rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pink-500"
              />
            </div>

            {/* Min Holders */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Min Holders</label>
              <input
                type="number"
                value={filters.minHolders || ''}
                onChange={(e) => handleFilterChange('minHolders', parseInt(e.target.value) || 0)}
                placeholder="0"
                min={0}
                className="w-full rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pink-500"
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

      {/* Mints Table */}
      <div className="overflow-hidden rounded-xl bg-stone-900 border border-stone-800">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-800">
            <thead className="bg-stone-800/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Item
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('timestamp')}
                >
                  Minted <SortIcon field="timestamp" />
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Creator
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Owner
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Type
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('mint_price_usd')}
                >
                  Mint Price <SortIcon field="mint_price_usd" />
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('holders')}
                >
                  Holders <SortIcon field="holders" />
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Txn
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {loading ? (
                Array.from({ length: pagination.pageSize || 10 }).map((_, i) => (
                  <tr key={i}>
                    <td className="px-4 py-4">
                      <div className="flex items-center gap-3">
                        <div className="h-12 w-12 bg-stone-800 rounded-lg animate-pulse" />
                        <div className="space-y-1">
                          <div className="h-4 w-24 bg-stone-800 rounded animate-pulse" />
                          <div className="h-3 w-16 bg-stone-800 rounded animate-pulse" />
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-24 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-24 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-12 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                  </tr>
                ))
              ) : mints.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-4 py-12 text-center text-stone-500">
                    No NFT mints found in the last {filters.days} days
                  </td>
                </tr>
              ) : (
                mints.map((mint, index) => (
                  <tr 
                    key={`${mint.txHash}-${index}`} 
                    className="hover:bg-stone-800/50 transition-colors"
                  >
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/nft/${mint.collectionAddress}/${mint.tokenId}`}
                        className="flex items-center gap-3 group"
                      >
                        <div className="h-12 w-12 rounded-lg bg-gradient-to-br from-pink-400 to-purple-500 flex items-center justify-center overflow-hidden">
                          {mint.imageUrl ? (
                            <img 
                              src={mint.imageUrl} 
                              alt={mint.name || `${mint.collectionName} #${mint.tokenId}`}
                              className="h-full w-full object-cover"
                            />
                          ) : (
                            <PhotoIcon className="h-6 w-6 text-white/70" />
                          )}
                        </div>
                        <div>
                          <div className="text-sm font-medium text-white group-hover:text-pink-400 transition-colors">
                            {mint.name || mint.collectionName || 'Unknown'}
                          </div>
                          <div className="text-xs text-stone-500">
                            {formatTokenId(mint.tokenId)}
                          </div>
                        </div>
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm text-stone-400" title={new Date(mint.timestamp * 1000).toLocaleString()}>
                        {timeAgo(mint.timestamp)}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/address/${mint.creator}`}
                        className="text-sm font-mono text-stone-300 hover:text-white transition-colors"
                      >
                        {formatAddress(mint.creator)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/address/${mint.owner}`}
                        className="text-sm font-mono text-stone-300 hover:text-white transition-colors"
                      >
                        {formatAddress(mint.owner)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${
                        mint.tokenType === 'ERC721' ? 'bg-purple-500/10 text-purple-400' :
                        mint.tokenType === 'ERC1155' ? 'bg-green-500/10 text-green-400' :
                        mint.tokenType === 'WASM721' ? 'bg-pink-500/10 text-pink-400' :
                        'bg-stone-500/10 text-stone-400'
                      }`}>
                        {mint.tokenType}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm font-medium text-white">
                        {formatValue(mint.mintPriceUsd)}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm text-stone-300">
                        {formatNumber(mint.holders)}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/tx/${mint.txHash}`}
                        className="text-sm font-mono text-pyrax-400 hover:text-pyrax-300 transition-colors"
                      >
                        {formatTxHash(mint.txHash)}
                      </Link>
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
                {Math.min(pagination.page * pagination.pageSize, pagination.totalMints)}
              </span>
              of{' '}
              <span className="font-medium text-white mx-1">
                {formatNumber(pagination.totalMints)}
              </span>
              mints
            </div>
            
            <div className="flex items-center gap-1">
              <button
                onClick={() => handlePageChange(1)}
                disabled={pagination.page === 1 || loading}
                className="p-2 rounded-lg text-stone-400 hover:text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                title="First page"
              >
                <ChevronDoubleLeftIcon className="h-4 w-4" />
              </button>
              
              <button
                onClick={() => handlePageChange(pagination.page - 1)}
                disabled={pagination.page === 1 || loading}
                className="p-2 rounded-lg text-stone-400 hover:text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                title="Previous page"
              >
                <ChevronLeftIcon className="h-4 w-4" />
              </button>
              
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
                          ? 'bg-pink-500 text-white'
                          : 'text-stone-400 hover:text-white hover:bg-stone-700'
                      }`}
                    >
                      {pageNum}
                    </button>
                  )
                })}
              </div>
              
              <button
                onClick={() => handlePageChange(pagination.page + 1)}
                disabled={pagination.page === pagination.totalPages || loading}
                className="p-2 rounded-lg text-stone-400 hover:text-white hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                title="Next page"
              >
                <ChevronRightIcon className="h-4 w-4" />
              </button>
              
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
