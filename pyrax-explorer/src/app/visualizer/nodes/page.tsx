'use client'

import { useState, useEffect } from 'react'
import { useNetwork } from '@/context/NetworkContext'
import { Globe, Server, Activity, Users, ChevronLeft, ChevronRight, RefreshCw } from 'lucide-react'
import { STREAMS, StreamType } from '@/lib/networks'

interface ConnectedNode {
  id: string
  peerId: string
  ip: string
  port: number
  country: string
  countryCode: string
  city: string
  lat: number
  lon: number
  stream: StreamType
  connectedAt: number
  lastSeen: number
  version: string
  blockHeight: number
}

interface NodeStats {
  totalNodes: number
  byStream: Record<StreamType, number>
  byCountry: Record<string, number>
}

function classNames(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ')
}

function getStreamColor(stream: StreamType): string {
  switch (stream) {
    case 'A': return '#22c55e' // green
    case 'B': return '#f59e0b' // amber
    case 'C': return '#8b5cf6' // purple
  }
}

function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor((Date.now() - timestamp) / 1000)
  if (seconds < 60) return `${seconds}s ago`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  return `${Math.floor(hours / 24)}d ago`
}

export default function NodesVisualizerPage() {
  const { networkState, overallStatus } = useNetwork()
  const [nodes, setNodes] = useState<ConnectedNode[]>([])
  const [stats, setStats] = useState<NodeStats>({ totalNodes: 0, byStream: { A: 0, B: 0, C: 0 }, byCountry: {} })
  const [loading, setLoading] = useState(true)
  const [page, setPage] = useState(1)
  const [filterStream, setFilterStream] = useState<StreamType | 'all'>('all')
  const pageSize = 10

  useEffect(() => {
    fetchNodes()
    const interval = setInterval(fetchNodes, 30000)
    return () => clearInterval(interval)
  }, [])

  async function fetchNodes() {
    try {
      setLoading(true)
      const response = await fetch('/api/nodes')
      if (response.ok) {
        const data = await response.json()
        setNodes(data.nodes || [])
        setStats(data.stats || { totalNodes: 0, byStream: { A: 0, B: 0, C: 0 }, byCountry: {} })
      }
    } catch (error) {
      console.error('Failed to fetch nodes:', error)
    } finally {
      setLoading(false)
    }
  }

  const filteredNodes = filterStream === 'all' 
    ? nodes 
    : nodes.filter(n => n.stream === filterStream)
  
  const totalPages = Math.ceil(filteredNodes.length / pageSize)
  const paginatedNodes = filteredNodes.slice((page - 1) * pageSize, page * pageSize)

  // Group nodes by country for heatmap
  const countryHeatmap = Object.entries(stats.byCountry)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10)

  return (
    <div className="py-8 px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Globe className="w-8 h-8 text-pyrax-500" />
          <h1 className="text-2xl font-bold text-white">
            Node <span className="pyrax-gradient-text">Visualizer</span>
          </h1>
        </div>
        <p className="text-stone-400">
          Real-time view of connected nodes across the PYRAX TriStream network
        </p>
      </div>

      {/* Stream Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-stone-900 rounded-xl p-4 border border-stone-800">
          <div className="flex items-center justify-between mb-2">
            <span className="text-stone-400 text-sm">Total Nodes</span>
            <Users className="w-5 h-5 text-pyrax-500" />
          </div>
          <div className="text-3xl font-bold text-white">{stats.totalNodes}</div>
        </div>
        
        {(['A', 'B', 'C'] as StreamType[]).map(stream => {
          const streamInfo = STREAMS[stream]
          const count = stats.byStream[stream] || 0
          const streamStatus = networkState.streams[stream]
          const isOnline = streamStatus.status === 'connected'
          
          return (
            <div key={stream} className="bg-stone-900 rounded-xl p-4 border border-stone-800">
              <div className="flex items-center justify-between mb-2">
                <span className="text-stone-400 text-sm">Stream {stream}</span>
                <div className="flex items-center gap-2">
                  <span className="relative flex h-2.5 w-2.5">
                    {isOnline && (
                      <span 
                        className="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75"
                        style={{ backgroundColor: getStreamColor(stream) }}
                      />
                    )}
                    <span 
                      className="relative inline-flex rounded-full h-2.5 w-2.5"
                      style={{ backgroundColor: isOnline ? getStreamColor(stream) : '#ef4444' }}
                    />
                  </span>
                </div>
              </div>
              <div className="text-2xl font-bold text-white">{count}</div>
              <div className="text-xs text-stone-500 mt-1">{streamInfo.algorithm}</div>
            </div>
          )
        })}
      </div>

      {/* World Map Heatmap */}
      <div className="bg-stone-900 rounded-xl border border-stone-800 mb-8 overflow-hidden">
        <div className="p-4 border-b border-stone-800 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white flex items-center gap-2">
            <Activity className="w-5 h-5 text-pyrax-500" />
            Global Node Distribution
          </h2>
          <button 
            onClick={fetchNodes}
            disabled={loading}
            className="p-2 text-stone-400 hover:text-white transition-colors disabled:opacity-50"
          >
            <RefreshCw className={classNames('w-4 h-4', loading && 'animate-spin')} />
          </button>
        </div>
        
        {/* SVG World Map */}
        <div className="relative h-[400px] bg-stone-950 overflow-hidden">
          <svg viewBox="0 0 1000 500" className="w-full h-full" preserveAspectRatio="xMidYMid meet">
            {/* Simplified world map paths */}
            <g fill="#27272a" stroke="#3f3f46" strokeWidth="0.5">
              {/* North America */}
              <path d="M150,80 L280,80 L320,120 L300,180 L260,200 L200,180 L140,140 Z" />
              {/* South America */}
              <path d="M220,220 L280,200 L300,280 L280,380 L240,400 L200,350 L210,280 Z" />
              {/* Europe */}
              <path d="M420,80 L520,70 L540,120 L500,140 L440,130 L420,100 Z" />
              {/* Africa */}
              <path d="M440,160 L520,140 L560,200 L540,300 L480,340 L420,300 L420,200 Z" />
              {/* Asia */}
              <path d="M540,60 L780,50 L820,120 L800,180 L700,200 L600,180 L560,120 Z" />
              {/* Australia */}
              <path d="M760,280 L860,260 L880,320 L840,360 L780,340 L760,300 Z" />
            </g>
            
            {/* Node markers */}
            {nodes.map((node, idx) => {
              const x = ((node.lon + 180) / 360) * 1000
              const y = ((90 - node.lat) / 180) * 500
              return (
                <g key={node.id || idx}>
                  <circle 
                    cx={x} 
                    cy={y} 
                    r="8" 
                    fill={getStreamColor(node.stream)}
                    opacity="0.3"
                    className="animate-pulse"
                  />
                  <circle 
                    cx={x} 
                    cy={y} 
                    r="4" 
                    fill={getStreamColor(node.stream)}
                  />
                </g>
              )
            })}
          </svg>
          
          {/* Legend */}
          <div className="absolute bottom-4 left-4 bg-stone-900/90 rounded-lg p-3 border border-stone-700">
            <div className="text-xs text-stone-400 mb-2">Stream Types</div>
            <div className="flex gap-4">
              {(['A', 'B', 'C'] as StreamType[]).map(stream => (
                <div key={stream} className="flex items-center gap-1.5">
                  <span 
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: getStreamColor(stream) }}
                  />
                  <span className="text-xs text-stone-300">{STREAMS[stream].algorithm}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Top Countries */}
          <div className="absolute top-4 right-4 bg-stone-900/90 rounded-lg p-3 border border-stone-700 max-w-[200px]">
            <div className="text-xs text-stone-400 mb-2">Top Regions</div>
            {countryHeatmap.length > 0 ? (
              <div className="space-y-1">
                {countryHeatmap.slice(0, 5).map(([country, count]) => (
                  <div key={country} className="flex items-center justify-between text-xs">
                    <span className="text-stone-300">{country}</span>
                    <span className="text-pyrax-500 font-medium">{count}</span>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-xs text-stone-500">No data yet</div>
            )}
          </div>
        </div>
      </div>

      {/* Connected Nodes List */}
      <div className="bg-stone-900 rounded-xl border border-stone-800">
        <div className="p-4 border-b border-stone-800 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white flex items-center gap-2">
            <Server className="w-5 h-5 text-pyrax-500" />
            Connected Nodes
          </h2>
          
          {/* Stream Filter */}
          <div className="flex items-center gap-2">
            <span className="text-sm text-stone-400">Filter:</span>
            <select
              value={filterStream}
              onChange={(e) => { setFilterStream(e.target.value as StreamType | 'all'); setPage(1) }}
              className="bg-stone-800 border border-stone-700 rounded-lg px-3 py-1.5 text-sm text-white"
            >
              <option value="all">All Streams</option>
              <option value="A">Stream A (BLAKE3)</option>
              <option value="B">Stream B (KAWPOW)</option>
              <option value="C">Stream C (ZK)</option>
            </select>
          </div>
        </div>

        {/* Nodes Table */}
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-stone-800/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Peer ID</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Location</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Stream</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Block Height</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Version</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Last Seen</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {paginatedNodes.length > 0 ? paginatedNodes.map((node) => (
                <tr key={node.id} className="hover:bg-stone-800/30 transition-colors">
                  <td className="px-4 py-3">
                    <span className="font-mono text-sm text-stone-300">
                      {node.peerId.slice(0, 8)}...{node.peerId.slice(-6)}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <span className="text-lg">{node.countryCode ? getFlagEmoji(node.countryCode) : '🌐'}</span>
                      <div>
                        <div className="text-sm text-white">{node.city || 'Unknown'}</div>
                        <div className="text-xs text-stone-500">{node.country || 'Unknown'}</div>
                      </div>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <span 
                      className="px-2 py-1 rounded text-xs font-medium"
                      style={{ 
                        backgroundColor: `${getStreamColor(node.stream)}20`,
                        color: getStreamColor(node.stream)
                      }}
                    >
                      {node.stream} - {STREAMS[node.stream].algorithm}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <span className="text-sm text-stone-300 font-mono">
                      #{node.blockHeight.toLocaleString()}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <span className="text-sm text-stone-400">{node.version}</span>
                  </td>
                  <td className="px-4 py-3">
                    <span className="text-sm text-stone-400">{formatTimeAgo(node.lastSeen)}</span>
                  </td>
                </tr>
              )) : (
                <tr>
                  <td colSpan={6} className="px-4 py-12 text-center">
                    {loading ? (
                      <div className="flex items-center justify-center gap-2 text-stone-400">
                        <RefreshCw className="w-5 h-5 animate-spin" />
                        <span>Loading nodes...</span>
                      </div>
                    ) : (
                      <div className="text-stone-500">
                        No nodes connected yet. Be the first to run a PYRAX node!
                      </div>
                    )}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="p-4 border-t border-stone-800 flex items-center justify-between">
            <div className="text-sm text-stone-400">
              Showing {((page - 1) * pageSize) + 1} - {Math.min(page * pageSize, filteredNodes.length)} of {filteredNodes.length} nodes
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setPage(p => Math.max(1, p - 1))}
                disabled={page === 1}
                className="p-2 rounded-lg bg-stone-800 text-stone-300 hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ChevronLeft className="w-4 h-4" />
              </button>
              <span className="text-sm text-stone-300 px-3">
                Page {page} of {totalPages}
              </span>
              <button
                onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
                className="p-2 rounded-lg bg-stone-800 text-stone-300 hover:bg-stone-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

function getFlagEmoji(countryCode: string): string {
  const codePoints = countryCode
    .toUpperCase()
    .split('')
    .map(char => 127397 + char.charCodeAt(0))
  return String.fromCodePoint(...codePoints)
}
