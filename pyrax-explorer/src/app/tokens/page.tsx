'use client'

import { useState, useEffect, useCallback } from 'react'
import Link from 'next/link'
import { 
  CurrencyDollarIcon, 
  ChevronLeftIcon, 
  ChevronRightIcon,
  ChevronDoubleLeftIcon,
  ChevronDoubleRightIcon,
  ArrowPathIcon,
  CheckBadgeIcon,
  ChevronUpIcon,
  ChevronDownIcon,
  FunnelIcon,
  MagnifyingGlassIcon,
  XMarkIcon,
} from '@heroicons/react/24/outline'

interface Token {
  address: string
  name: string
  symbol: string
  decimals: number
  totalSupply: string
  holders: number
  transfers: number
  marketCap: number
  price: number
  change24h: number
  volume24h: number
  type: string
  verified: boolean
  logo: string | null
  website: string | null
  createdAt: number
  creator: string
}

interface Pagination {
  page: number
  pageSize: number
  totalTokens: number
  totalPages: number
}

interface Filters {
  sortBy: string
  sortOrder: string
  tokenType: string
  minHolders: number
  minValue: number
  search: string
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

function formatPrice(price: number): string {
  if (price === 0) return '$0.00'
  if (price < 0.0001) return '< $0.0001'
  if (price < 1) return `$${price.toFixed(6)}`
  return `$${price.toFixed(2)}`
}

function formatMarketCap(cap: number): string {
  if (cap === 0) return '-'
  if (cap >= 1e9) return `$${(cap / 1e9).toFixed(2)}B`
  if (cap >= 1e6) return `$${(cap / 1e6).toFixed(2)}M`
  if (cap >= 1e3) return `$${(cap / 1e3).toFixed(2)}K`
  return `$${cap.toFixed(2)}`
}

function formatAddress(address: string): string {
  if (!address || address.length < 16) return address || 'Unknown'
  return `${address.slice(0, 10)}...${address.slice(-6)}`
}

function formatChange(change: number): { text: string; color: string } {
  if (change === 0) return { text: '0.00%', color: 'text-stone-400' }
  if (change > 0) return { text: `+${change.toFixed(2)}%`, color: 'text-green-400' }
  return { text: `${change.toFixed(2)}%`, color: 'text-red-400' }
}

export default function TokensPage() {
  const [tokens, setTokens] = useState<Token[]>([])
  const [pagination, setPagination] = useState<Pagination>({
    page: 1,
    pageSize: 25,
    totalTokens: 0,
    totalPages: 0,
  })
  const [filters, setFilters] = useState<Filters>({
    sortBy: 'market_cap',
    sortOrder: 'desc',
    tokenType: 'all',
    minHolders: 0,
    minValue: 0,
    search: '',
  })
  const [showFilters, setShowFilters] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [searchInput, setSearchInput] = useState('')

  const fetchTokens = useCallback(async (page: number, pageSize: number, currentFilters: Filters) => {
    setLoading(true)
    setError(null)
    
    try {
      const params = new URLSearchParams({
        page: page.toString(),
        pageSize: pageSize.toString(),
        sortBy: currentFilters.sortBy,
        sortOrder: currentFilters.sortOrder,
        type: currentFilters.tokenType,
        minHolders: currentFilters.minHolders.toString(),
        minValue: currentFilters.minValue.toString(),
        search: currentFilters.search,
      })
      
      const response = await fetch(`/api/tokens?${params}`)
      const data = await response.json()
      
      if (data.error) {
        setError(data.error)
      } else {
        setTokens(data.tokens || [])
        setPagination(data.pagination || { page: 1, pageSize, totalTokens: 0, totalPages: 0 })
      }
    } catch (err) {
      setError('Failed to fetch tokens')
      console.error(err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchTokens(pagination.page, pagination.pageSize, filters)
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= pagination.totalPages) {
      fetchTokens(newPage, pagination.pageSize, filters)
    }
  }

  const handlePageSizeChange = (newPageSize: number) => {
    fetchTokens(1, newPageSize, filters)
  }

  const handleSort = (field: string) => {
    const newOrder = filters.sortBy === field && filters.sortOrder === 'desc' ? 'asc' : 'desc'
    const newFilters = { ...filters, sortBy: field, sortOrder: newOrder }
    setFilters(newFilters)
    fetchTokens(1, pagination.pageSize, newFilters)
  }

  const handleFilterChange = (field: keyof Filters, value: string | number) => {
    const newFilters = { ...filters, [field]: value }
    setFilters(newFilters)
    fetchTokens(1, pagination.pageSize, newFilters)
  }

  const handleSearch = () => {
    const newFilters = { ...filters, search: searchInput }
    setFilters(newFilters)
    fetchTokens(1, pagination.pageSize, newFilters)
  }

  const clearSearch = () => {
    setSearchInput('')
    const newFilters = { ...filters, search: '' }
    setFilters(newFilters)
    fetchTokens(1, pagination.pageSize, newFilters)
  }

  const handleRefresh = () => {
    fetchTokens(pagination.page, pagination.pageSize, filters)
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
          <div className="p-2 rounded-lg bg-yellow-500/10">
            <CurrencyDollarIcon className="h-6 w-6 text-yellow-500" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Tokens</h1>
            <p className="text-sm text-stone-400">
              {pagination.totalTokens > 0 
                ? `${formatNumber(pagination.totalTokens)} tokens on the chain`
                : 'Browse all tokens on the chain'}
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
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
            {/* Search */}
            <div className="lg:col-span-2">
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Search</label>
              <div className="relative">
                <input
                  type="text"
                  value={searchInput}
                  onChange={(e) => setSearchInput(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                  placeholder="Token name or symbol..."
                  className="w-full rounded-lg bg-stone-800 border border-stone-700 pl-9 pr-9 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                />
                <MagnifyingGlassIcon className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-stone-500" />
                {searchInput && (
                  <button
                    onClick={clearSearch}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-stone-500 hover:text-white"
                  >
                    <XMarkIcon className="h-4 w-4" />
                  </button>
                )}
              </div>
            </div>

            {/* Token Type */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Type</label>
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

            {/* Min Holders */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Min Holders</label>
              <input
                type="number"
                value={filters.minHolders || ''}
                onChange={(e) => handleFilterChange('minHolders', parseInt(e.target.value) || 0)}
                placeholder="0"
                min={0}
                className="w-full rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-pyrax-500"
              />
            </div>

            {/* Min Market Cap */}
            <div>
              <label className="block text-xs font-medium text-stone-400 mb-1.5">Min Market Cap ($)</label>
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

      {/* Tokens Table */}
      <div className="overflow-hidden rounded-xl bg-stone-900 border border-stone-800">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-800">
            <thead className="bg-stone-800/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  #
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Token
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Type
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('market_cap')}
                >
                  Market Cap <SortIcon field="market_cap" />
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Price
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('change_24h')}
                >
                  24h % <SortIcon field="change_24h" />
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('holders')}
                >
                  Holders <SortIcon field="holders" />
                </th>
                <th 
                  className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider cursor-pointer hover:text-white"
                  onClick={() => handleSort('transfers')}
                >
                  Transfers <SortIcon field="transfers" />
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {loading ? (
                // Loading skeletons
                Array.from({ length: pagination.pageSize || 10 }).map((_, i) => (
                  <tr key={i}>
                    <td className="px-4 py-4">
                      <div className="h-5 w-6 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="flex items-center gap-3">
                        <div className="h-8 w-8 bg-stone-800 rounded-full animate-pulse" />
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
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-14 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-14 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-14 bg-stone-800 rounded animate-pulse" />
                    </td>
                  </tr>
                ))
              ) : tokens.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-4 py-12 text-center text-stone-500">
                    No tokens found
                  </td>
                </tr>
              ) : (
                tokens.map((token, index) => {
                  const change = formatChange(token.change24h)
                  return (
                    <tr 
                      key={token.address} 
                      className="hover:bg-stone-800/50 transition-colors"
                    >
                      <td className="px-4 py-4 whitespace-nowrap text-sm text-stone-500">
                        {((pagination.page - 1) * pagination.pageSize) + index + 1}
                      </td>
                      <td className="px-4 py-4 whitespace-nowrap">
                        <Link 
                          href={`/token/${token.address}`}
                          className="flex items-center gap-3 group"
                        >
                          <div className="h-8 w-8 rounded-full bg-gradient-to-br from-yellow-400 to-orange-500 flex items-center justify-center text-white font-bold text-xs">
                            {token.symbol.slice(0, 2).toUpperCase()}
                          </div>
                          <div>
                            <div className="flex items-center gap-1.5">
                              <span className="text-sm font-medium text-white group-hover:text-pyrax-400 transition-colors">
                                {token.name}
                              </span>
                              {token.verified && (
                                <CheckBadgeIcon className="h-4 w-4 text-blue-400" title="Verified" />
                              )}
                            </div>
                            <div className="text-xs text-stone-500">{token.symbol}</div>
                          </div>
                        </Link>
                      </td>
                      <td className="px-4 py-4 whitespace-nowrap">
                        <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${
                          token.type === 'ERC20' ? 'bg-blue-500/10 text-blue-400' :
                          token.type === 'ERC721' ? 'bg-purple-500/10 text-purple-400' :
                          token.type === 'ERC1155' ? 'bg-green-500/10 text-green-400' :
                          token.type === 'WASM20' ? 'bg-orange-500/10 text-orange-400' :
                          token.type === 'WASM721' ? 'bg-pink-500/10 text-pink-400' :
                          'bg-stone-500/10 text-stone-400'
                        }`}>
                          {token.type}
                        </span>
                      </td>
                      <td className="px-4 py-4 whitespace-nowrap">
                        <span className="text-sm font-medium text-white">
                          {formatMarketCap(token.marketCap)}
                        </span>
                      </td>
                      <td className="px-4 py-4 whitespace-nowrap">
                        <span className="text-sm text-white font-mono">
                          {formatPrice(token.price)}
                        </span>
                      </td>
                      <td className="px-4 py-4 whitespace-nowrap">
                        <span className={`text-sm font-medium ${change.color}`}>
                          {change.text}
                        </span>
                      </td>
                      <td className="px-4 py-4 whitespace-nowrap">
                        <span className="text-sm text-stone-300">
                          {formatNumber(token.holders)}
                        </span>
                      </td>
                      <td className="px-4 py-4 whitespace-nowrap">
                        <span className="text-sm text-stone-300">
                          {formatNumber(token.transfers)}
                        </span>
                      </td>
                    </tr>
                  )
                })
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
                {Math.min(pagination.page * pagination.pageSize, pagination.totalTokens)}
              </span>
              of{' '}
              <span className="font-medium text-white mx-1">
                {formatNumber(pagination.totalTokens)}
              </span>
              tokens
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
