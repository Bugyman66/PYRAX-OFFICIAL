'use client'

import { useState, useEffect } from 'react'
import { ArrowTrendingUpIcon } from '@heroicons/react/24/outline'

interface NetworkStats {
  difficulty: string
  avgBlockTime: string
  price: string
  marketCap: string
  totalSupply: string
  circulatingSupply: string
}

export default function NetworkOverview() {
  const [stats, setStats] = useState<NetworkStats>({
    difficulty: '0',
    avgBlockTime: '6s',
    price: '$0.00',
    marketCap: '$0',
    totalSupply: '100,000,000,000',
    circulatingSupply: '0',
  })
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function fetchStats() {
      try {
        const response = await fetch('/api/stats')
        if (response.ok) {
          const data = await response.json()
          setStats({
            difficulty: (data.difficulty || 0).toLocaleString(),
            avgBlockTime: data.avgBlockTime || '6s',
            price: data.price || '$0.00',
            marketCap: data.marketCap || '$0',
            totalSupply: '100,000,000,000',
            circulatingSupply: data.circulatingSupply || '0',
          })
        }
      } catch (error) {
        console.error('Failed to fetch network stats:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchStats()
    const interval = setInterval(fetchStats, 10000) // Refresh every 10 seconds
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="bg-gradient-to-r from-pyrax-600/20 via-pyrax-500/10 to-rust-600/20 rounded-2xl border border-pyrax-500/30 p-6">
      <div className="flex items-center gap-3 mb-4">
        <ArrowTrendingUpIcon className="w-5 h-5 text-pyrax-500" />
        <h2 className="text-lg font-semibold text-white">Network Overview</h2>
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
        <StatItem 
          label="Difficulty" 
          value={stats.difficulty} 
          loading={loading} 
        />
        <StatItem 
          label="Avg Block Time" 
          value={stats.avgBlockTime} 
          loading={loading} 
        />
        <StatItem 
          label="PYRAX Price" 
          value={stats.price} 
          loading={loading} 
        />
        <StatItem 
          label="Market Cap" 
          value={stats.marketCap} 
          loading={loading} 
        />
      </div>
    </div>
  )
}

function StatItem({ 
  label, 
  value, 
  loading 
}: { 
  label: string
  value: string
  loading: boolean 
}) {
  return (
    <div>
      <p className="text-stone-400 text-sm">{label}</p>
      {loading ? (
        <div className="h-7 w-24 bg-stone-800/50 animate-pulse rounded mt-1" />
      ) : (
        <p className="text-white font-mono text-lg">{value}</p>
      )}
    </div>
  )
}
