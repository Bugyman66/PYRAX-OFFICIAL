'use client'

import { useState, useEffect, useCallback } from 'react'
import Link from 'next/link'
import { 
  CubeIcon, 
  ChevronLeftIcon, 
  ChevronRightIcon,
  ChevronDoubleLeftIcon,
  ChevronDoubleRightIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline'

interface Block {
  height: number
  hash: string
  timestamp: number
  txCount: number
  miner: string
  reward: number
  size: number
  difficulty: number
  prevHash: string
  nonce: number
}

interface Pagination {
  page: number
  pageSize: number
  totalBlocks: number
  totalPages: number
}

const PAGE_SIZE_OPTIONS = [10, 25, 50, 100]

function formatTimeAgo(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp * 1000
  
  if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
  return `${Math.floor(diff / 86400000)}d ago`
}

function formatHash(hash: string): string {
  if (!hash || hash.length < 16) return hash || '-'
  return `${hash.slice(0, 10)}...${hash.slice(-8)}`
}

function formatAddress(address: string): string {
  if (!address || address.length < 16) return address || 'Unknown'
  return `${address.slice(0, 8)}...${address.slice(-6)}`
}

function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num)
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

export default function BlocksPage() {
  const [blocks, setBlocks] = useState<Block[]>([])
  const [pagination, setPagination] = useState<Pagination>({
    page: 1,
    pageSize: 25,
    totalBlocks: 0,
    totalPages: 0,
  })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchBlocks = useCallback(async (page: number, pageSize: number) => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await fetch(`/api/blocks?page=${page}&pageSize=${pageSize}`)
      const data = await response.json()
      
      if (data.error) {
        setError(data.error)
      } else {
        setBlocks(data.blocks || [])
        setPagination(data.pagination || { page: 1, pageSize, totalBlocks: 0, totalPages: 0 })
      }
    } catch (err) {
      setError('Failed to fetch blocks')
      console.error(err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchBlocks(pagination.page, pagination.pageSize)
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= pagination.totalPages) {
      fetchBlocks(newPage, pagination.pageSize)
    }
  }

  const handlePageSizeChange = (newPageSize: number) => {
    fetchBlocks(1, newPageSize)
  }

  const handleRefresh = () => {
    fetchBlocks(pagination.page, pagination.pageSize)
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="sm:flex sm:items-center sm:justify-between mb-6">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-pyrax-500/10">
            <CubeIcon className="h-6 w-6 text-pyrax-500" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Blocks</h1>
            <p className="text-sm text-stone-400">
              {pagination.totalBlocks > 0 
                ? `${formatNumber(pagination.totalBlocks)} blocks found`
                : 'Browse all blocks on the chain'}
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

      {/* Error Message */}
      {error && (
        <div className="mb-6 rounded-lg bg-red-500/10 border border-red-500/20 p-4 text-red-400">
          {error}
        </div>
      )}

      {/* Blocks Table */}
      <div className="overflow-hidden rounded-xl bg-stone-900 border border-stone-800">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-800">
            <thead className="bg-stone-800/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Block
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Age
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Txns
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Miner
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Reward
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Size
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Difficulty
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Block Hash
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {loading ? (
                // Loading skeletons
                Array.from({ length: pagination.pageSize || 10 }).map((_, i) => (
                  <tr key={i}>
                    <td className="px-4 py-4">
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-10 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-28 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-36 bg-stone-800 rounded animate-pulse" />
                    </td>
                  </tr>
                ))
              ) : blocks.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-4 py-12 text-center text-stone-500">
                    No blocks found
                  </td>
                </tr>
              ) : (
                blocks.map((block) => (
                  <tr 
                    key={block.height} 
                    className="hover:bg-stone-800/50 transition-colors"
                  >
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/block/${block.height}`}
                        className="flex items-center gap-2 text-pyrax-500 hover:text-pyrax-400 font-mono font-medium"
                      >
                        <CubeIcon className="h-4 w-4" />
                        {formatNumber(block.height)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap text-sm text-stone-400">
                      {formatTimeAgo(block.timestamp)}
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="inline-flex items-center rounded-full bg-stone-800 px-2.5 py-0.5 text-xs font-medium text-stone-300">
                        {block.txCount}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/address/${block.miner}`}
                        className="text-sm text-pyrax-500 hover:text-pyrax-400 font-mono"
                      >
                        {formatAddress(block.miner)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm font-medium text-green-400">
                        {block.reward} PYRAX
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap text-sm text-stone-400">
                      {formatSize(block.size)}
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap text-sm text-stone-400 font-mono">
                      {formatNumber(block.difficulty)}
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/block/${block.hash}`}
                        className="text-sm text-stone-400 hover:text-white font-mono"
                        title={block.hash}
                      >
                        {formatHash(block.hash)}
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
                {Math.min(pagination.page * pagination.pageSize, pagination.totalBlocks)}
              </span>
              of{' '}
              <span className="font-medium text-white mx-1">
                {formatNumber(pagination.totalBlocks)}
              </span>
              blocks
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
