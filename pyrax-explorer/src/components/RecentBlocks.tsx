'use client'

import { useState, useEffect } from 'react'
import Image from 'next/image'
import Link from 'next/link'
import {
  CubeIcon,
  ClockIcon,
  FireIcon,
} from '@heroicons/react/24/outline'

interface Block {
  height: number
  hash: string
  timestamp: number
  txCount: number
  miner: string
  reward: string
}

function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor(Date.now() / 1000 - timestamp)
  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
  return `${Math.floor(seconds / 86400)}d ago`
}

export default function RecentBlocks() {
  const [blocks, setBlocks] = useState<Block[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function fetchBlocks() {
      try {
        const response = await fetch('/api/blocks?limit=5')
        if (response.ok) {
          const data = await response.json()
          setBlocks(data.blocks || [])
        }
      } catch (error) {
        console.error('Failed to fetch blocks:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchBlocks()
    const interval = setInterval(fetchBlocks, 6000) // Refresh every 6 seconds (block time)
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="bg-stone-900 rounded-2xl border border-stone-800 overflow-hidden">
      <div className="px-6 py-4 border-b border-stone-800 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <FireIcon className="w-5 h-5 text-pyrax-500" />
          <h2 className="text-lg font-semibold text-white">Latest Blocks</h2>
        </div>
        <Link href="/blocks" className="text-pyrax-500 hover:text-pyrax-400 text-sm font-medium">
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
                    <div className="h-5 w-16 bg-stone-800 animate-pulse rounded" />
                    <div className="h-4 w-24 bg-stone-800 animate-pulse rounded mt-1" />
                  </div>
                </div>
                <div className="text-right">
                  <div className="h-4 w-12 bg-stone-800 animate-pulse rounded" />
                  <div className="h-3 w-20 bg-stone-800 animate-pulse rounded mt-1" />
                </div>
              </div>
            </div>
          ))
        ) : blocks.length === 0 ? (
          <div className="px-6 py-12 text-center text-stone-500">
            <CubeIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p>No blocks found</p>
            <p className="text-sm mt-1">Waiting for blocks...</p>
          </div>
        ) : (
          blocks.map((block) => (
            <div key={block.height} className="px-6 py-4 hover:bg-stone-800/50 transition-colors">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 rounded-xl pyrax-gradient flex items-center justify-center">
                    <CubeIcon className="w-6 h-6 text-white" />
                  </div>
                  <div>
                    <Link href={`/block/${block.height}`} className="text-pyrax-400 font-mono font-medium hover:text-pyrax-300">
                      #{block.height.toLocaleString()}
                    </Link>
                    <div className="flex items-center gap-2 text-sm text-stone-500">
                      <ClockIcon className="w-3 h-3" />
                      <span>{formatTimeAgo(block.timestamp)}</span>
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm text-stone-300">{block.txCount} txns</div>
                  <div className="text-xs text-stone-500 font-mono">
                    <Image src="/pyrax-coin.svg" alt="" width={12} height={12} className="inline w-3 h-3 mr-1" />
                    {block.reward} PYRAX
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
