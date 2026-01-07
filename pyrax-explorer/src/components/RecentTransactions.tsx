'use client'

import { useState, useEffect } from 'react'
import Image from 'next/image'
import Link from 'next/link'
import {
  ArrowsRightLeftIcon,
  ClockIcon,
  HashtagIcon,
} from '@heroicons/react/24/outline'

interface Transaction {
  hash: string
  from: string
  to: string
  value: string
  timestamp: number
  blockNumber?: number
}

function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor(Date.now() / 1000 - timestamp)
  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
  return `${Math.floor(seconds / 86400)}d ago`
}

function truncateHash(hash: string): string {
  if (hash.length <= 16) return hash
  return `${hash.slice(0, 8)}...${hash.slice(-6)}`
}

export default function RecentTransactions() {
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function fetchTransactions() {
      try {
        const response = await fetch('/api/transactions?limit=5')
        if (response.ok) {
          const data = await response.json()
          setTransactions(data.transactions || [])
        }
      } catch (error) {
        console.error('Failed to fetch transactions:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchTransactions()
    const interval = setInterval(fetchTransactions, 5000) // Refresh every 5 seconds
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="bg-stone-900 rounded-2xl border border-stone-800 overflow-hidden">
      <div className="px-6 py-4 border-b border-stone-800 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <ArrowsRightLeftIcon className="w-5 h-5 text-pyrax-500" />
          <h2 className="text-lg font-semibold text-white">Latest Transactions</h2>
        </div>
        <Link href="/transactions" className="text-pyrax-500 hover:text-pyrax-400 text-sm font-medium">
          View all →
        </Link>
      </div>
      <div className="divide-y divide-stone-800">
        {loading ? (
          // Loading skeletons
          Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 rounded-xl bg-stone-800 animate-pulse" />
                  <div>
                    <div className="h-5 w-24 bg-stone-800 animate-pulse rounded" />
                    <div className="h-4 w-20 bg-stone-800 animate-pulse rounded mt-1" />
                  </div>
                </div>
                <div className="text-right">
                  <div className="h-4 w-16 bg-stone-800 animate-pulse rounded" />
                  <div className="h-3 w-28 bg-stone-800 animate-pulse rounded mt-1" />
                </div>
              </div>
            </div>
          ))
        ) : transactions.length === 0 ? (
          <div className="px-6 py-12 text-center text-stone-500">
            <ArrowsRightLeftIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p>No transactions found</p>
            <p className="text-sm mt-1">Waiting for transactions...</p>
          </div>
        ) : (
          transactions.map((tx, i) => (
            <div key={tx.hash || i} className="px-6 py-4 hover:bg-stone-800/50 transition-colors">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 rounded-xl bg-stone-800 flex items-center justify-center">
                    <HashtagIcon className="w-6 h-6 text-pyrax-500" />
                  </div>
                  <div>
                    <Link href={`/tx/${tx.hash}`} className="text-pyrax-400 font-mono text-sm hover:text-pyrax-300">
                      {truncateHash(tx.hash)}
                    </Link>
                    <div className="flex items-center gap-2 text-sm text-stone-500">
                      <ClockIcon className="w-3 h-3" />
                      <span>{formatTimeAgo(tx.timestamp)}</span>
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm text-stone-300">
                    <Image src="/pyrax-coin.svg" alt="" width={14} height={14} className="inline w-3.5 h-3.5 mr-1" />
                    {tx.value} PYRAX
                  </div>
                  <div className="text-xs text-stone-500 font-mono">
                    {truncateHash(tx.from)} → {truncateHash(tx.to)}
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
