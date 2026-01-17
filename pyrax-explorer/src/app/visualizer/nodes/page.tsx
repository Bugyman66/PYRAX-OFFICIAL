'use client'

import { useState, useEffect, useMemo } from 'react'
import dynamic from 'next/dynamic'
import { useNetwork } from '@/context/NetworkContext'
import { Globe, Server, Activity, Users, ChevronLeft, ChevronRight, RefreshCw, AlertCircle, Zap, Wifi, WifiOff } from 'lucide-react'
import { STREAMS, StreamType } from '@/lib/networks'

const ComposableMap = dynamic(() => import('react-simple-maps').then(m => m.ComposableMap), { ssr: false })
const Geographies = dynamic(() => import('react-simple-maps').then(m => m.Geographies), { ssr: false })
const Geography = dynamic(() => import('react-simple-maps').then(m => m.Geography), { ssr: false })
const Marker = dynamic(() => import('react-simple-maps').then(m => m.Marker), { ssr: false })
const Line = dynamic(() => import('react-simple-maps').then(m => m.Line), { ssr: false })
const ZoomableGroup = dynamic(() => import('react-simple-maps').then(m => m.ZoomableGroup), { ssr: false })

const GEO_URL = 'https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json'

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
  latency: number
  isBootnode?: boolean
  online?: boolean
}

interface NodeStats {
  totalNodes: number
  byStream: Record<StreamType, number>
  byCountry: Record<string, number>
  averageLatency: number
}

interface Connection {
  from: string
  to: string
  fromCoords: [number, number]
  toCoords: [number, number]
}

function cn(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ')
}

function getStreamColor(stream: StreamType): string {
  switch (stream) {
    case 'A': return '#22c55e'
    case 'B': return '#f59e0b'
    case 'C': return '#8b5cf6'
  }
}

function formatTimeAgo(ts: number): string {
  const s = Math.floor((Date.now() - ts) / 1000)
  if (s < 0) return 'now'
  if (s < 60) return `${s}s ago`
  const m = Math.floor(s / 60)
  if (m < 60) return `${m}m ago`
  const h = Math.floor(m / 60)
  if (h < 24) return `${h}h ago`
  return `${Math.floor(h / 24)}d ago`
}

function getFlagEmoji(cc: string): string {
  if (!cc || cc.length !== 2) return '🌐'
  try {
    return String.fromCodePoint(...cc.toUpperCase().split('').map(c => 127397 + c.charCodeAt(0)))
  } catch { return '🌐' }
}

export default function NodesVisualizerPage() {
  const { networkState } = useNetwork()
  const [nodes, setNodes] = useState<ConnectedNode[]>([])
  const [connections, setConnections] = useState<Connection[]>([])
  const [stats, setStats] = useState<NodeStats>({ totalNodes: 0, byStream: { A: 0, B: 0, C: 0 }, byCountry: {}, averageLatency: 0 })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [page, setPage] = useState(1)
  const [filterStream, setFilterStream] = useState<StreamType | 'all'>('all')
  const [mapReady, setMapReady] = useState(false)
  const [animationPhase, setAnimationPhase] = useState(0)
  const pageSize = 10

  useEffect(() => {
    fetchNodes()
    const interval = setInterval(fetchNodes, 30000)
    return () => clearInterval(interval)
  }, [])

  useEffect(() => {
    const t = setTimeout(() => setMapReady(true), 100)
    return () => clearTimeout(t)
  }, [])

  // Animation loop for connection lines
  useEffect(() => {
    const animInterval = setInterval(() => {
      setAnimationPhase(p => (p + 1) % 100)
    }, 50)
    return () => clearInterval(animInterval)
  }, [])

  async function fetchNodes() {
    try {
      setLoading(true)
      setError(null)
      const res = await fetch('/api/nodes', { cache: 'no-store' })
      if (!res.ok) throw new Error(`API ${res.status}`)
      const data = await res.json()
      if (data.error) setError(data.error)
      setNodes(data.nodes || [])
      setConnections(data.connections || [])
      setStats(data.stats || { totalNodes: 0, byStream: { A: 0, B: 0, C: 0 }, byCountry: {}, averageLatency: 0 })
    } catch (e) {
      setError(String(e))
      setNodes([])
      setConnections([])
      setStats({ totalNodes: 0, byStream: { A: 0, B: 0, C: 0 }, byCountry: {}, averageLatency: 0 })
    } finally {
      setLoading(false)
    }
  }

  const filteredNodes = useMemo(() => filterStream === 'all' ? nodes : nodes.filter(n => n.stream === filterStream), [nodes, filterStream])
  const totalPages = Math.ceil(filteredNodes.length / pageSize)
  const paginatedNodes = useMemo(() => filteredNodes.slice((page - 1) * pageSize, page * pageSize), [filteredNodes, page])
  const mappableNodes = useMemo(() => {
    const filtered = filterStream === 'all' ? nodes : nodes.filter(n => n.stream === filterStream)
    return filtered.filter(n => n.lat !== 0 || n.lon !== 0)
  }, [nodes, filterStream])
  const countryHeatmap = useMemo(() => Object.entries(stats.byCountry).sort((a, b) => b[1] - a[1]).slice(0, 8), [stats.byCountry])

  return (
    <div className="py-8 px-4 sm:px-6 lg:px-8">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Globe className="w-8 h-8 text-pyrax-500" />
          <h1 className="text-2xl font-bold text-white">Node <span className="pyrax-gradient-text">Visualizer</span></h1>
        </div>
        <p className="text-stone-400">Real-time view of connected nodes across the PYRAX TriStream network</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-5 gap-4 mb-8">
        <div className="bg-stone-900 rounded-xl p-4 border border-stone-800">
          <div className="flex items-center justify-between mb-2">
            <span className="text-stone-400 text-sm">Total Nodes</span>
            <Users className="w-5 h-5 text-pyrax-500" />
          </div>
          <div className="text-3xl font-bold text-white">{stats.totalNodes}</div>
          <div className="text-xs text-stone-500 mt-1">Connected peers</div>
        </div>
        <div className="bg-stone-900 rounded-xl p-4 border border-stone-800">
          <div className="flex items-center justify-between mb-2">
            <span className="text-stone-400 text-sm">Avg Latency</span>
            <Zap className="w-5 h-5 text-yellow-500" />
          </div>
          <div className="text-3xl font-bold text-white">
            {stats.averageLatency > 0 ? `${stats.averageLatency}ms` : '--'}
          </div>
          <div className="text-xs text-stone-500 mt-1">Network response</div>
        </div>
        {(['A', 'B', 'C'] as StreamType[]).map(stream => {
          const info = STREAMS[stream]
          const count = stats.byStream[stream] || 0
          const status = networkState.streams[stream]
          const online = status.status === 'connected'
          return (
            <div key={stream} className="bg-stone-900 rounded-xl p-4 border border-stone-800">
              <div className="flex items-center justify-between mb-2">
                <span className="text-stone-400 text-sm">Stream {stream}</span>
                <span className="relative flex h-2.5 w-2.5">
                  {online && <span className="animate-ping absolute h-full w-full rounded-full opacity-75" style={{ backgroundColor: getStreamColor(stream) }} />}
                  <span className="relative rounded-full h-2.5 w-2.5" style={{ backgroundColor: online ? getStreamColor(stream) : '#ef4444' }} />
                </span>
              </div>
              <div className="text-2xl font-bold text-white">{count}</div>
              <div className="text-xs text-stone-500 mt-1">{info.algorithm}</div>
            </div>
          )
        })}
      </div>

      <div className="bg-stone-900 rounded-xl border border-stone-800 mb-8 overflow-hidden">
        <div className="p-4 border-b border-stone-800 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white flex items-center gap-2">
            <Activity className="w-5 h-5 text-pyrax-500" />Global Node Distribution
          </h2>
          <button onClick={fetchNodes} disabled={loading} className="p-2 text-stone-400 hover:text-white disabled:opacity-50">
            <RefreshCw className={cn('w-4 h-4', loading && 'animate-spin')} />
          </button>
        </div>
        <div className="relative h-[450px] bg-stone-950">
          {mapReady ? (
            <ComposableMap projection="geoMercator" projectionConfig={{ scale: 140, center: [0, 30] }} style={{ width: '100%', height: '100%' }}>
              <ZoomableGroup>
                <Geographies geography={GEO_URL}>
                  {({ geographies }) => geographies.map(geo => (
                    <Geography key={geo.rsmKey} geography={geo} fill="#27272a" stroke="#3f3f46" strokeWidth={0.5} style={{ default: { outline: 'none' }, hover: { fill: '#3f3f46', outline: 'none' }, pressed: { outline: 'none' } }} />
                  ))}
                </Geographies>
                {/* Animated connection lines between nodes */}
                {connections.map((conn, idx) => {
                  // Check if this is a bootnode-to-bootnode connection (thicker, red)
                  const isBootnodeLink = conn.from.includes('bootnode') && conn.to.includes('bootnode')
                  const opacity = Math.round((0.3 + (Math.sin((animationPhase + idx * 10) * 0.1) + 1) * 0.2) * 255).toString(16).padStart(2, '0')
                  const baseColor = isBootnodeLink ? '#ef4444' : '#22c55e'
                  const strokeColor = `${baseColor}${opacity}`
                  const strokeWidth = isBootnodeLink ? 2 : 1
                  return (
                    <Line
                      key={`${conn.from}-${conn.to}`}
                      from={conn.fromCoords}
                      to={conn.toCoords}
                      stroke={strokeColor}
                      strokeWidth={strokeWidth}
                      strokeLinecap="round"
                    />
                  )
                })}
                {mappableNodes.map(node => {
                  // Bootnodes get red color, others get stream color
                  const markerColor = node.isBootnode ? '#ef4444' : getStreamColor(node.stream)
                  const markerSize = node.isBootnode ? 5 : 3
                  return (
                    <Marker key={node.id} coordinates={[node.lon, node.lat]}>
                      <circle r={markerSize * 3} fill={markerColor} opacity={0.2} className="animate-ping" />
                      <circle r={markerSize * 2} fill={markerColor} opacity={0.4} />
                      <circle r={markerSize} fill={markerColor} />
                      <title>{node.isBootnode ? '🔴 BOOTNODE: ' : ''}{node.city}, {node.country} - Stream {node.stream}</title>
                    </Marker>
                  )
                })}
              </ZoomableGroup>
            </ComposableMap>
          ) : (
            <div className="flex items-center justify-center h-full"><RefreshCw className="w-8 h-8 animate-spin text-stone-600" /></div>
          )}
          <div className="absolute bottom-4 left-4 bg-stone-900/95 rounded-lg p-3 border border-stone-700">
            <div className="text-xs text-stone-400 mb-2">Node Types</div>
            <div className="flex flex-wrap gap-4">
              <div className="flex items-center gap-1.5">
                <span className="w-4 h-4 rounded-full bg-red-500" />
                <span className="text-xs text-stone-300 font-medium">Bootnode</span>
              </div>
              {(['A', 'B', 'C'] as StreamType[]).map(s => (
                <div key={s} className="flex items-center gap-1.5">
                  <span className="w-3 h-3 rounded-full" style={{ backgroundColor: getStreamColor(s) }} />
                  <span className="text-xs text-stone-300">{STREAMS[s].algorithm}</span>
                </div>
              ))}
            </div>
          </div>
          <div className="absolute top-4 right-4 bg-stone-900/95 rounded-lg p-3 border border-stone-700 min-w-[180px]">
            <div className="text-xs text-stone-400 mb-2">Top Regions</div>
            {countryHeatmap.length > 0 ? countryHeatmap.map(([c, n]) => (
              <div key={c} className="flex justify-between text-xs py-0.5">
                <span className="text-stone-300 truncate max-w-[120px]">{c}</span>
                <span className="text-pyrax-500 font-medium">{n}</span>
              </div>
            )) : <div className="text-xs text-stone-500">No nodes connected</div>}
          </div>
          {error && (
            <div className="absolute bottom-4 right-4 bg-red-900/90 rounded-lg p-2 border border-red-700 flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-red-400" /><span className="text-xs text-red-300">Connection issue</span>
            </div>
          )}
        </div>
      </div>

      <div className="bg-stone-900 rounded-xl border border-stone-800">
        <div className="p-4 border-b border-stone-800 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <h2 className="text-lg font-semibold text-white flex items-center gap-2">
            <Server className="w-5 h-5 text-pyrax-500" />Connected Nodes
            {nodes.length > 0 && <span className="text-sm font-normal text-stone-400">({nodes.length})</span>}
          </h2>
          <div className="flex items-center gap-2">
            <span className="text-sm text-stone-400">Filter:</span>
            <select value={filterStream} onChange={e => { setFilterStream(e.target.value as StreamType | 'all'); setPage(1) }} className="bg-stone-800 border border-stone-700 rounded-lg px-3 py-1.5 text-sm text-white">
              <option value="all">All Streams</option>
              <option value="A">Stream A (BLAKE3)</option>
              <option value="B">Stream B (KAWPOW)</option>
              <option value="C">Stream C (ZK)</option>
            </select>
          </div>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-stone-800/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Status</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Location</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Stream</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase">Version</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-stone-400 uppercase text-right">Latency</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800">
              {paginatedNodes.length > 0 ? paginatedNodes.map(node => {
                const isOnline = node.online !== false
                const isBootnode = node.isBootnode === true
                return (
                  <tr key={node.id} className={cn('hover:bg-stone-800/30', isBootnode && 'bg-red-900/10 border-l-2 border-red-500')}>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        {isOnline ? (
                          <Wifi className="w-4 h-4 text-green-400" />
                        ) : (
                          <WifiOff className="w-4 h-4 text-red-400" />
                        )}
                        <span className={isOnline ? 'text-green-400 text-sm' : 'text-red-400 text-sm'}>
                          {isOnline ? 'Online' : 'Offline'}
                        </span>
                        {isBootnode && (
                          <span className="text-xs bg-red-500/20 text-red-400 px-2 py-0.5 rounded-full font-medium">
                            BOOTNODE
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <span className="text-lg">{getFlagEmoji(node.countryCode)}</span>
                        <div>
                          <div className="text-sm text-white">{node.city || 'Unknown'}</div>
                          <div className="text-xs text-stone-500">{node.country || 'Unknown'}</div>
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <span className="px-2 py-1 rounded text-xs font-medium" style={{ backgroundColor: `${getStreamColor(node.stream)}20`, color: getStreamColor(node.stream) }}>
                        {STREAMS[node.stream].algorithm}
                      </span>
                    </td>
                    <td className="px-4 py-3">
                      <span className="text-sm font-mono text-pyrax-400">v{node.version}</span>
                    </td>
                    <td className="px-4 py-3 text-right">
                      <span className={`text-sm font-mono ${node.latency > 0 ? (node.latency < 50 ? 'text-green-400' : node.latency < 100 ? 'text-yellow-400' : node.latency < 200 ? 'text-orange-400' : 'text-red-400') : 'text-stone-500'}`}>
                        {isOnline && node.latency > 0 ? `${node.latency}ms` : '—'}
                      </span>
                    </td>
                  </tr>
                )
              }) : (
                <tr><td colSpan={5} className="px-4 py-12 text-center">
                  {loading ? <div className="flex items-center justify-center gap-2 text-stone-400"><RefreshCw className="w-5 h-5 animate-spin" /><span>Loading...</span></div> : <div className="text-stone-500">No nodes connected yet. Be the first to run a PYRAX node!</div>}
                </td></tr>
              )}
            </tbody>
          </table>
        </div>
        {totalPages > 1 && (
          <div className="p-4 border-t border-stone-800 flex items-center justify-between">
            <div className="text-sm text-stone-400">Showing {(page - 1) * pageSize + 1}-{Math.min(page * pageSize, filteredNodes.length)} of {filteredNodes.length}</div>
            <div className="flex items-center gap-2">
              <button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page === 1} className="p-2 rounded-lg bg-stone-800 text-stone-300 hover:bg-stone-700 disabled:opacity-50"><ChevronLeft className="w-4 h-4" /></button>
              <span className="text-sm text-stone-300 px-3">Page {page} of {totalPages}</span>
              <button onClick={() => setPage(p => Math.min(totalPages, p + 1))} disabled={page === totalPages} className="p-2 rounded-lg bg-stone-800 text-stone-300 hover:bg-stone-700 disabled:opacity-50"><ChevronRight className="w-4 h-4" /></button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
