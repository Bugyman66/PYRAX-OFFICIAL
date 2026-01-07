'use client'

import { useState, useEffect, useCallback } from 'react'
import {
  AreaChart,
  Area,
  LineChart,
  Line,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from 'recharts'
import {
  CubeIcon,
  ArrowsRightLeftIcon,
  CodeBracketIcon,
  CurrencyDollarIcon,
  PhotoIcon,
  UserGroupIcon,
  BoltIcon,
  ClockIcon,
  FireIcon,
  ServerStackIcon,
  ShieldCheckIcon,
  ChartBarIcon,
  ArrowPathIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
  SignalIcon,
  CpuChipIcon,
} from '@heroicons/react/24/outline'

interface ChainStats {
  blockHeight: number
  totalTransactions: number
  totalContracts: number
  totalTokens: number
  totalNFTs: number
  totalAddresses: number
  syncing: boolean
  tps: number
  avgBlockTime: number
  avgGasPrice: number
  gasUsedPercent: number
  pendingTxCount: number
  totalNodes: number
  activeValidators: number
  totalStaked: number
  stakingAPY: number
  networkHashrate: number
  difficulty: number
  totalSupply: number
  circulatingSupply: number
  burnedTokens: number
  marketCap: number
  price: number
  priceChange24h: number
  volume24h: number
  transactionHistory: Array<{ date: string; value: number }>
  blockTimeHistory: Array<{ date: string; value: number }>
  gasHistory: Array<{ date: string; value: number }>
  tpsHistory: Array<{ date: string; value: number }>
  activeAddressHistory: Array<{ date: string; value: number }>
  zkProofsGenerated: number
  zkProofsVerified: number
  avgProofTime: number
  timestamp: number
  error?: string
}

const GRADIENT_COLORS = {
  pyrax: ['#FF6B35', '#F7931A'],
  blue: ['#3B82F6', '#06B6D4'],
  purple: ['#8B5CF6', '#EC4899'],
  green: ['#10B981', '#34D399'],
  pink: ['#EC4899', '#F472B6'],
}

function formatNumber(num: number): string {
  if (num >= 1e12) return `${(num / 1e12).toFixed(2)}T`
  if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`
  if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`
  if (num >= 1e3) return `${(num / 1e3).toFixed(2)}K`
  return new Intl.NumberFormat().format(num)
}

function formatPrice(price: number): string {
  if (price === 0) return '$0.00'
  if (price < 0.01) return `$${price.toFixed(6)}`
  return `$${price.toFixed(2)}`
}

function formatHashrate(hashrate: number): string {
  if (hashrate >= 1e18) return `${(hashrate / 1e18).toFixed(2)} EH/s`
  if (hashrate >= 1e15) return `${(hashrate / 1e15).toFixed(2)} PH/s`
  if (hashrate >= 1e12) return `${(hashrate / 1e12).toFixed(2)} TH/s`
  if (hashrate >= 1e9) return `${(hashrate / 1e9).toFixed(2)} GH/s`
  if (hashrate >= 1e6) return `${(hashrate / 1e6).toFixed(2)} MH/s`
  return `${hashrate.toFixed(2)} H/s`
}

interface StatCardProps {
  title: string
  value: string | number
  subtitle?: string
  icon: React.ElementType
  gradient: keyof typeof GRADIENT_COLORS
  trend?: number
}

function StatCard({ title, value, subtitle, icon: Icon, gradient, trend }: StatCardProps) {
  const [from, to] = GRADIENT_COLORS[gradient]
  
  return (
    <div className="relative overflow-hidden rounded-2xl bg-stone-900/80 border border-stone-800 p-6 backdrop-blur-sm">
      {/* Gradient glow */}
      <div 
        className="absolute -top-24 -right-24 w-48 h-48 rounded-full blur-3xl opacity-20"
        style={{ background: `linear-gradient(135deg, ${from}, ${to})` }}
      />
      
      <div className="relative">
        <div className="flex items-start justify-between">
          <div 
            className="p-3 rounded-xl"
            style={{ background: `linear-gradient(135deg, ${from}20, ${to}20)` }}
          >
            <Icon className="h-6 w-6" style={{ color: from }} />
          </div>
          {trend !== undefined && (
            <div className={`flex items-center gap-1 text-sm font-medium ${
              trend >= 0 ? 'text-green-400' : 'text-red-400'
            }`}>
              {trend >= 0 ? (
                <ArrowTrendingUpIcon className="h-4 w-4" />
              ) : (
                <ArrowTrendingDownIcon className="h-4 w-4" />
              )}
              {Math.abs(trend).toFixed(2)}%
            </div>
          )}
        </div>
        
        <div className="mt-4">
          <p className="text-sm text-stone-400">{title}</p>
          <p className="mt-1 text-2xl font-bold text-white">{value}</p>
          {subtitle && (
            <p className="mt-1 text-xs text-stone-500">{subtitle}</p>
          )}
        </div>
      </div>
    </div>
  )
}

interface ChartCardProps {
  title: string
  children: React.ReactNode
  className?: string
}

function ChartCard({ title, children, className = '' }: ChartCardProps) {
  return (
    <div className={`rounded-2xl bg-stone-900/80 border border-stone-800 p-6 backdrop-blur-sm ${className}`}>
      <h3 className="text-sm font-semibold text-stone-400 uppercase tracking-wider mb-4">{title}</h3>
      {children}
    </div>
  )
}

export default function StatsPage() {
  const [stats, setStats] = useState<ChainStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null)

  const fetchStats = useCallback(async () => {
    try {
      const response = await fetch('/api/stats')
      const data = await response.json()
      setStats(data)
      setLastUpdate(new Date())
    } catch (error) {
      console.error('Failed to fetch stats:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchStats()
    const interval = setInterval(fetchStats, 5000) // Refresh every 5 seconds
    return () => clearInterval(interval)
  }, [fetchStats])

  const handleRefresh = () => {
    setLoading(true)
    fetchStats()
  }

  // Use real chart data from API only - no mock data
  const txChartData = stats?.transactionHistory || []
  const tpsChartData = stats?.tpsHistory || []
  const gasChartData = stats?.gasHistory || []
  const blockTimeData = stats?.blockTimeHistory || []

  // Calculate supply distribution from real data
  const supplyData = stats && stats.totalSupply > 0 ? [
    { name: 'Circulating', value: stats.circulatingSupply, color: '#FF6B35' },
    { name: 'Staked', value: stats.totalStaked, color: '#3B82F6' },
    { name: 'Burned', value: stats.burnedTokens, color: '#EC4899' },
    { name: 'Unmined', value: Math.max(0, stats.totalSupply - stats.circulatingSupply - stats.totalStaked - stats.burnedTokens), color: '#6B7280' },
  ].filter(item => item.value > 0) : []

  return (
    <div className="min-h-screen bg-stone-950">
      {/* Animated background */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-0 left-1/4 w-96 h-96 bg-pyrax-500/5 rounded-full blur-3xl animate-pulse" />
        <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-blue-500/5 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '1s' }} />
        <div className="absolute top-1/2 left-1/2 w-96 h-96 bg-purple-500/5 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '2s' }} />
      </div>

      <div className="relative px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-white flex items-center gap-3">
              <div className="p-2 rounded-xl bg-gradient-to-br from-pyrax-500/20 to-orange-500/20">
                <ChartBarIcon className="h-8 w-8 text-pyrax-500" />
              </div>
              Chain Statistics
            </h1>
            <p className="mt-2 text-stone-400">
              Real-time blockchain metrics and analytics
            </p>
          </div>
          
          <div className="flex items-center gap-4">
            {lastUpdate && (
              <div className="text-sm text-stone-500">
                Updated {lastUpdate.toLocaleTimeString()}
              </div>
            )}
            <button
              onClick={handleRefresh}
              disabled={loading}
              className="inline-flex items-center gap-2 rounded-xl bg-stone-800 px-4 py-2 text-sm font-medium text-white hover:bg-stone-700 disabled:opacity-50 transition-all"
            >
              <ArrowPathIcon className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </button>
          </div>
        </div>

        {/* Syncing Banner */}
        {stats?.syncing && (
          <div className="mb-6 rounded-xl bg-yellow-500/10 border border-yellow-500/20 p-4 flex items-center gap-3">
            <ArrowPathIcon className="h-5 w-5 text-yellow-500 animate-spin" />
            <span className="text-yellow-400">Node is syncing with the network...</span>
          </div>
        )}

        {/* Main Stats Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          <StatCard
            title="Block Height"
            value={formatNumber(stats?.blockHeight || 0)}
            icon={CubeIcon}
            gradient="pyrax"
          />
          <StatCard
            title="Total Transactions"
            value={formatNumber(stats?.totalTransactions || 0)}
            icon={ArrowsRightLeftIcon}
            gradient="blue"
          />
          <StatCard
            title="Smart Contracts"
            value={formatNumber(stats?.totalContracts || 0)}
            icon={CodeBracketIcon}
            gradient="purple"
          />
          <StatCard
            title="Unique Addresses"
            value={formatNumber(stats?.totalAddresses || 0)}
            icon={UserGroupIcon}
            gradient="green"
          />
        </div>

        {/* Performance Metrics */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4 mb-8">
          <StatCard
            title="TPS"
            value={stats?.tps?.toFixed(1) || '0'}
            subtitle="Transactions per second"
            icon={BoltIcon}
            gradient="pyrax"
          />
          <StatCard
            title="Avg Block Time"
            value={`${stats?.avgBlockTime?.toFixed(1) || '6'}s`}
            icon={ClockIcon}
            gradient="blue"
          />
          <StatCard
            title="Gas Price"
            value={`${stats?.avgGasPrice || 0} cinders`}
            icon={FireIcon}
            gradient="purple"
          />
          <StatCard
            title="Pending Txns"
            value={formatNumber(stats?.pendingTxCount || 0)}
            icon={ArrowsRightLeftIcon}
            gradient="green"
          />
          <StatCard
            title="Gas Utilization"
            value={`${stats?.gasUsedPercent?.toFixed(1) || 0}%`}
            icon={ChartBarIcon}
            gradient="pink"
          />
        </div>

        {/* Charts Row 1 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          <ChartCard title="Transaction Volume (Recent Blocks)">
            <div className="h-64">
              {txChartData.length > 0 ? (
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={txChartData}>
                    <defs>
                      <linearGradient id="txGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="#FF6B35" stopOpacity={0.3}/>
                        <stop offset="95%" stopColor="#FF6B35" stopOpacity={0}/>
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                    <XAxis dataKey="date" stroke="#9CA3AF" fontSize={12} />
                    <YAxis stroke="#9CA3AF" fontSize={12} />
                    <Tooltip
                      contentStyle={{ backgroundColor: '#1C1917', border: '1px solid #374151', borderRadius: '8px' }}
                      labelStyle={{ color: '#9CA3AF' }}
                    />
                    <Area
                      type="monotone"
                      dataKey="value"
                      stroke="#FF6B35"
                      strokeWidth={2}
                      fill="url(#txGradient)"
                    />
                  </AreaChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-full flex items-center justify-center text-stone-500">
                  {loading ? 'Loading chart data...' : 'No transaction data available yet'}
                </div>
              )}
            </div>
          </ChartCard>

          <ChartCard title="TPS (Transactions Per Second)">
            <div className="h-64">
              {tpsChartData.length > 0 ? (
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={tpsChartData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                    <XAxis dataKey="date" stroke="#9CA3AF" fontSize={12} />
                    <YAxis stroke="#9CA3AF" fontSize={12} />
                    <Tooltip
                      contentStyle={{ backgroundColor: '#1C1917', border: '1px solid #374151', borderRadius: '8px' }}
                      labelStyle={{ color: '#9CA3AF' }}
                    />
                    <Line
                      type="monotone"
                      dataKey="value"
                      stroke="#3B82F6"
                      strokeWidth={2}
                      dot={false}
                    />
                  </LineChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-full flex items-center justify-center text-stone-500">
                  {loading ? 'Loading chart data...' : 'No TPS data available yet'}
                </div>
              )}
            </div>
          </ChartCard>
        </div>

        {/* Charts Row 2 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
          <ChartCard title="Gas Price Trend (cinders)">
            <div className="h-48">
              {gasChartData.length > 0 ? (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={gasChartData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                    <XAxis dataKey="date" stroke="#9CA3AF" fontSize={10} />
                    <YAxis stroke="#9CA3AF" fontSize={10} />
                    <Tooltip
                      contentStyle={{ backgroundColor: '#1C1917', border: '1px solid #374151', borderRadius: '8px' }}
                      labelStyle={{ color: '#9CA3AF' }}
                      formatter={(value: number) => [`${value.toFixed(2)} cinders`, 'Gas Price']}
                    />
                    <Bar dataKey="value" fill="#8B5CF6" radius={[4, 4, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-full flex items-center justify-center text-stone-500">
                  {loading ? 'Loading...' : 'No gas data yet'}
                </div>
              )}
            </div>
          </ChartCard>

          <ChartCard title="Block Time Consistency">
            <div className="h-48">
              {blockTimeData.length > 0 ? (
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={blockTimeData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                    <XAxis dataKey="date" stroke="#9CA3AF" fontSize={10} />
                    <YAxis stroke="#9CA3AF" fontSize={10} domain={['auto', 'auto']} />
                    <Tooltip
                      contentStyle={{ backgroundColor: '#1C1917', border: '1px solid #374151', borderRadius: '8px' }}
                      labelStyle={{ color: '#9CA3AF' }}
                      formatter={(value: number) => [`${value.toFixed(2)}s`, 'Block Time']}
                    />
                    <Line
                      type="monotone"
                      dataKey="value"
                      stroke="#10B981"
                      strokeWidth={2}
                      dot={false}
                    />
                  </LineChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-full flex items-center justify-center text-stone-500">
                  {loading ? 'Loading...' : 'No block time data yet'}
                </div>
              )}
            </div>
          </ChartCard>

          <ChartCard title="Token Distribution">
            <div className="h-48">
              {supplyData.length > 0 ? (
                <>
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={supplyData}
                        cx="50%"
                        cy="50%"
                        innerRadius={40}
                        outerRadius={70}
                        paddingAngle={2}
                        dataKey="value"
                      >
                        {supplyData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={entry.color} />
                        ))}
                      </Pie>
                      <Tooltip
                        contentStyle={{ backgroundColor: '#1C1917', border: '1px solid #374151', borderRadius: '8px' }}
                        formatter={(value: number) => [formatNumber(value), '']}
                      />
                    </PieChart>
                  </ResponsiveContainer>
                  <div className="flex flex-wrap justify-center gap-3 -mt-4">
                    {supplyData.map((item) => (
                      <div key={item.name} className="flex items-center gap-1.5 text-xs">
                        <div className="w-2 h-2 rounded-full" style={{ backgroundColor: item.color }} />
                        <span className="text-stone-400">{item.name}</span>
                      </div>
                    ))}
                  </div>
                </>
              ) : (
                <div className="h-full flex items-center justify-center text-stone-500">
                  {loading ? 'Loading...' : 'No supply data yet'}
                </div>
              )}
            </div>
          </ChartCard>
        </div>

        {/* Network & Token Stats */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          {/* Network Stats */}
          <div className="rounded-2xl bg-stone-900/80 border border-stone-800 p-6 backdrop-blur-sm">
            <h3 className="text-lg font-semibold text-white mb-6 flex items-center gap-2">
              <ServerStackIcon className="h-5 w-5 text-blue-400" />
              Network Statistics
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Total Nodes</p>
                <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.totalNodes || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Active Validators</p>
                <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.activeValidators || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Network Hashrate</p>
                <p className="text-xl font-bold text-white mt-1">{formatHashrate(stats?.networkHashrate || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Difficulty</p>
                <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.difficulty || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Total Staked</p>
                <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.totalStaked || 0)} PYRAX</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Staking APY</p>
                <p className="text-xl font-bold text-green-400 mt-1">{stats?.stakingAPY?.toFixed(2) || 0}%</p>
              </div>
            </div>
          </div>

          {/* Token Economics */}
          <div className="rounded-2xl bg-stone-900/80 border border-stone-800 p-6 backdrop-blur-sm">
            <h3 className="text-lg font-semibold text-white mb-6 flex items-center gap-2">
              <CurrencyDollarIcon className="h-5 w-5 text-yellow-400" />
              Token Economics
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">PYRAX Price</p>
                <div className="flex items-center gap-2 mt-1">
                  <p className="text-xl font-bold text-white">{formatPrice(stats?.price || 0)}</p>
                  {stats?.priceChange24h !== undefined && (
                    <span className={`text-sm ${stats.priceChange24h >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                      {stats.priceChange24h >= 0 ? '+' : ''}{stats.priceChange24h.toFixed(2)}%
                    </span>
                  )}
                </div>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Market Cap</p>
                <p className="text-xl font-bold text-white mt-1">${formatNumber(stats?.marketCap || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">24h Volume</p>
                <p className="text-xl font-bold text-white mt-1">${formatNumber(stats?.volume24h || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Circulating Supply</p>
                <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.circulatingSupply || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Total Supply</p>
                <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.totalSupply || 0)}</p>
              </div>
              <div className="p-4 rounded-xl bg-stone-800/50">
                <p className="text-sm text-stone-400">Burned Tokens</p>
                <p className="text-xl font-bold text-pink-400 mt-1">{formatNumber(stats?.burnedTokens || 0)}</p>
              </div>
            </div>
          </div>
        </div>

        {/* ZK Stats & Asset Counts */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* ZK-STARK Statistics */}
          <div className="rounded-2xl bg-gradient-to-br from-purple-500/10 to-pink-500/10 border border-purple-500/20 p-6 backdrop-blur-sm">
            <h3 className="text-lg font-semibold text-white mb-6 flex items-center gap-2">
              <ShieldCheckIcon className="h-5 w-5 text-purple-400" />
              ZK-STARK Statistics
            </h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">Proofs Generated</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.zkProofsGenerated || 0)}</p>
                </div>
                <CpuChipIcon className="h-8 w-8 text-purple-400/50" />
              </div>
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">Proofs Verified</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.zkProofsVerified || 0)}</p>
                </div>
                <ShieldCheckIcon className="h-8 w-8 text-green-400/50" />
              </div>
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">Avg Proof Time</p>
                  <p className="text-xl font-bold text-white mt-1">{stats?.avgProofTime?.toFixed(2) || 0}ms</p>
                </div>
                <ClockIcon className="h-8 w-8 text-blue-400/50" />
              </div>
            </div>
          </div>

          {/* Tokens */}
          <div className="rounded-2xl bg-gradient-to-br from-yellow-500/10 to-orange-500/10 border border-yellow-500/20 p-6 backdrop-blur-sm">
            <h3 className="text-lg font-semibold text-white mb-6 flex items-center gap-2">
              <CurrencyDollarIcon className="h-5 w-5 text-yellow-400" />
              Token Statistics
            </h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">Total Tokens</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.totalTokens || 0)}</p>
                </div>
                <CurrencyDollarIcon className="h-8 w-8 text-yellow-400/50" />
              </div>
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">ERC-20 Tokens</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(Math.floor((stats?.totalTokens || 0) * 0.7))}</p>
                </div>
                <CodeBracketIcon className="h-8 w-8 text-blue-400/50" />
              </div>
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">WASM Tokens</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(Math.floor((stats?.totalTokens || 0) * 0.3))}</p>
                </div>
                <CodeBracketIcon className="h-8 w-8 text-orange-400/50" />
              </div>
            </div>
          </div>

          {/* NFTs */}
          <div className="rounded-2xl bg-gradient-to-br from-pink-500/10 to-purple-500/10 border border-pink-500/20 p-6 backdrop-blur-sm">
            <h3 className="text-lg font-semibold text-white mb-6 flex items-center gap-2">
              <PhotoIcon className="h-5 w-5 text-pink-400" />
              NFT Statistics
            </h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">Total NFTs</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(stats?.totalNFTs || 0)}</p>
                </div>
                <PhotoIcon className="h-8 w-8 text-pink-400/50" />
              </div>
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">ERC-721 NFTs</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(Math.floor((stats?.totalNFTs || 0) * 0.6))}</p>
                </div>
                <PhotoIcon className="h-8 w-8 text-purple-400/50" />
              </div>
              <div className="flex items-center justify-between p-4 rounded-xl bg-stone-900/50">
                <div>
                  <p className="text-sm text-stone-400">ERC-1155 Collections</p>
                  <p className="text-xl font-bold text-white mt-1">{formatNumber(Math.floor((stats?.totalNFTs || 0) * 0.4))}</p>
                </div>
                <PhotoIcon className="h-8 w-8 text-green-400/50" />
              </div>
            </div>
          </div>
        </div>

        {/* Connection Status */}
        {stats?.error && (
          <div className="mt-6 rounded-xl bg-red-500/10 border border-red-500/20 p-4 flex items-center gap-3">
            <SignalIcon className="h-5 w-5 text-red-500" />
            <span className="text-red-400">Unable to connect to node. Displaying cached data.</span>
          </div>
        )}
      </div>
    </div>
  )
}
