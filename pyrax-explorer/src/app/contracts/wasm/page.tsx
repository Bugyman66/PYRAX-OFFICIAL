'use client'

import { useState, useEffect, useCallback } from 'react'
import Link from 'next/link'
import { 
  CommandLineIcon, 
  ChevronLeftIcon, 
  ChevronRightIcon,
  ChevronDoubleLeftIcon,
  ChevronDoubleRightIcon,
  ArrowPathIcon,
  CheckBadgeIcon,
  XCircleIcon,
} from '@heroicons/react/24/outline'

interface Contract {
  address: string
  creator: string
  creationTx: string
  creationBlock: number
  creationTimestamp: number
  bytecodeHash: string
  contractType: string
  isVerified: boolean
  name: string | null
  compilerVersion: string | null
  optimization: boolean
  license: string | null
  balance: number
  txCount: number
}

interface Pagination {
  page: number
  pageSize: number
  totalContracts: number
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

function formatAddress(address: string): string {
  if (!address || address.length < 16) return address || 'Unknown'
  return `${address.slice(0, 10)}...${address.slice(-6)}`
}

function formatNumber(num: number): string {
  return new Intl.NumberFormat().format(num)
}

function formatBalance(balance: number): string {
  if (balance === 0) return '0'
  if (balance < 0.0001) return '< 0.0001'
  return balance.toFixed(4)
}

export default function WASMContractsPage() {
  const [contracts, setContracts] = useState<Contract[]>([])
  const [pagination, setPagination] = useState<Pagination>({
    page: 1,
    pageSize: 25,
    totalContracts: 0,
    totalPages: 0,
  })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchContracts = useCallback(async (page: number, pageSize: number) => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await fetch(`/api/contracts?page=${page}&pageSize=${pageSize}&type=wasm`)
      const data = await response.json()
      
      if (data.error) {
        setError(data.error)
      } else {
        setContracts(data.contracts || [])
        setPagination(data.pagination || { page: 1, pageSize, totalContracts: 0, totalPages: 0 })
      }
    } catch (err) {
      setError('Failed to fetch contracts')
      console.error(err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchContracts(pagination.page, pagination.pageSize)
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= pagination.totalPages) {
      fetchContracts(newPage, pagination.pageSize)
    }
  }

  const handlePageSizeChange = (newPageSize: number) => {
    fetchContracts(1, newPageSize)
  }

  const handleRefresh = () => {
    fetchContracts(pagination.page, pagination.pageSize)
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="sm:flex sm:items-center sm:justify-between mb-6">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-purple-500/10">
            <CommandLineIcon className="h-6 w-6 text-purple-500" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Rust/WASM Contracts</h1>
            <p className="text-sm text-stone-400">
              {pagination.totalContracts > 0 
                ? `${formatNumber(pagination.totalContracts)} WASM contracts deployed`
                : 'Browse all Rust/WASM smart contracts on the chain'}
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
              className="rounded-lg bg-stone-800 border border-stone-700 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
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

      {/* Contracts Table */}
      <div className="overflow-hidden rounded-xl bg-stone-900 border border-stone-800">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-stone-800">
            <thead className="bg-stone-800/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Contract
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Name
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Creator
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Deployed
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Balance
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Calls
                </th>
                <th className="px-4 py-3 text-left text-xs font-semibold text-stone-400 uppercase tracking-wider">
                  Rust Version
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {loading ? (
                // Loading skeletons
                Array.from({ length: pagination.pageSize || 10 }).map((_, i) => (
                  <tr key={i}>
                    <td className="px-4 py-4">
                      <div className="h-5 w-32 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-24 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-28 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-16 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-12 bg-stone-800 rounded animate-pulse" />
                    </td>
                    <td className="px-4 py-4">
                      <div className="h-5 w-20 bg-stone-800 rounded animate-pulse" />
                    </td>
                  </tr>
                ))
              ) : contracts.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-4 py-12 text-center text-stone-500">
                    No Rust/WASM contracts found
                  </td>
                </tr>
              ) : (
                contracts.map((contract) => (
                  <tr 
                    key={contract.address} 
                    className="hover:bg-stone-800/50 transition-colors"
                  >
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/address/${contract.address}`}
                        className="flex items-center gap-2 text-purple-500 hover:text-purple-400 font-mono text-sm"
                        title={contract.address}
                      >
                        <CommandLineIcon className="h-4 w-4" />
                        {formatAddress(contract.address)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm text-white">
                        {contract.name || <span className="text-stone-500">-</span>}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      {contract.isVerified ? (
                        <span className="inline-flex items-center gap-1 rounded-full bg-green-500/10 px-2.5 py-0.5 text-xs font-medium text-green-400">
                          <CheckBadgeIcon className="h-3.5 w-3.5" />
                          Verified
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 rounded-full bg-stone-700/50 px-2.5 py-0.5 text-xs font-medium text-stone-400">
                          <XCircleIcon className="h-3.5 w-3.5" />
                          Unverified
                        </span>
                      )}
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <Link 
                        href={`/address/${contract.creator}`}
                        className="text-sm text-stone-300 hover:text-white font-mono"
                        title={contract.creator}
                      >
                        {formatAddress(contract.creator)}
                      </Link>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <div className="flex flex-col">
                        <span className="text-sm text-stone-400">
                          {formatTimeAgo(contract.creationTimestamp)}
                        </span>
                        <Link 
                          href={`/block/${contract.creationBlock}`}
                          className="text-xs text-purple-500 hover:text-purple-400"
                        >
                          Block #{formatNumber(contract.creationBlock)}
                        </Link>
                      </div>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm font-medium text-white">
                        {formatBalance(contract.balance)} <span className="text-stone-500">PYRAX</span>
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="inline-flex items-center rounded-full bg-stone-800 px-2.5 py-0.5 text-xs font-medium text-stone-300">
                        {formatNumber(contract.txCount)}
                      </span>
                    </td>
                    <td className="px-4 py-4 whitespace-nowrap">
                      <span className="text-sm text-stone-400 font-mono">
                        {contract.compilerVersion || '-'}
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
                {Math.min(pagination.page * pagination.pageSize, pagination.totalContracts)}
              </span>
              of{' '}
              <span className="font-medium text-white mx-1">
                {formatNumber(pagination.totalContracts)}
              </span>
              contracts
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
                          ? 'bg-purple-500 text-white'
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
