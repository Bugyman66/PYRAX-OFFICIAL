'use client'

import { useState, useEffect } from 'react'
import {
  CubeIcon,
  ArrowsRightLeftIcon,
  CpuChipIcon,
  UsersIcon,
} from '@heroicons/react/24/outline'

interface Stats {
  blockHeight: number
  transactions: number
  hashrate: string
  peers: number
  difficulty: string
  avgBlockTime: string
  marketCap: string
  price: string
}

function formatHashrate(difficulty: number): string {
  const hashrate = difficulty / 10
  if (hashrate >= 1e12) return `${(hashrate / 1e12).toFixed(2)} TH/s`
  if (hashrate >= 1e9) return `${(hashrate / 1e9).toFixed(2)} GH/s`
  if (hashrate >= 1e6) return `${(hashrate / 1e6).toFixed(2)} MH/s`
  if (hashrate >= 1e3) return `${(hashrate / 1e3).toFixed(2)} KH/s`
  return `${hashrate.toFixed(0)} H/s`
}

export default function StatsGrid() {
  const [stats, setStats] = useState<Stats>({
    blockHeight: 0,
    transactions: 0,
    hashrate: '0 H/s',
    peers: 0,
    difficulty: '0',
    avgBlockTime: '6s',
    marketCap: '$0',
    price: '$0.00',
  })
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function fetchStats() {
      try {
        const response = await fetch('/api/stats')
        if (response.ok) {
          const data = await response.json()
          setStats({
            blockHeight: data.blockHeight || 0,
            transactions: data.transactions || 0,
            hashrate: formatHashrate(data.difficulty || 0),
            peers: data.peers || 0,
            difficulty: (data.difficulty || 0).toLocaleString(),
            avgBlockTime: data.avgBlockTime || '6s',
            marketCap: data.marketCap || '$0',
            price: data.price || '$0.00',
          })
        }
      } catch (error) {
        console.error('Failed to fetch stats:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchStats()
    const interval = setInterval(fetchStats, 5000) // Refresh every 5 seconds
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      <StatCard
        icon={<CubeIcon className="w-6 h-6" />}
        title="Block Height"
        value={stats.blockHeight.toLocaleString()}
        loading={loading}
      />
      <StatCard
        icon={<ArrowsRightLeftIcon className="w-6 h-6" />}
        title="Transactions"
        value={stats.transactions.toLocaleString()}
        loading={loading}
      />
      <StatCard
        icon={<CpuChipIcon className="w-6 h-6" />}
        title="Network Hashrate"
        value={stats.hashrate}
        loading={loading}
      />
      <StatCard
        icon={<UsersIcon className="w-6 h-6" />}
        title="Active Nodes"
        value={stats.peers.toString()}
        loading={loading}
      />
    </div>
  )
}

function StatCard({ 
  icon, 
  title, 
  value, 
  loading 
}: { 
  icon: React.ReactNode
  title: string
  value: string
  loading: boolean
}) {
  return (
    <div className="bg-stone-900 rounded-xl border border-stone-800 p-4 hover:border-pyrax-500/50 transition-colors">
      <div className="flex items-center gap-3">
        <div className="text-pyrax-500">{icon}</div>
        <div>
          <p className="text-stone-500 text-xs uppercase tracking-wider">{title}</p>
          {loading ? (
            <div className="h-7 w-20 bg-stone-800 animate-pulse rounded mt-1" />
          ) : (
            <p className="text-white font-semibold text-lg">{value}</p>
          )}
        </div>
      </div>
    </div>
  )
}
