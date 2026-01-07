'use client'

import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react'
import { 
  Network, 
  NetworkState, 
  ConnectionStatus, 
  StreamType,
  StreamStatus,
  NETWORKS, 
  getDefaultNetwork,
  getOverallStatus,
} from '@/lib/networks'

interface NetworkContextType {
  networkState: NetworkState
  setNetwork: (network: Network) => void
  checkConnection: () => Promise<void>
  isChecking: boolean
  overallStatus: ConnectionStatus
}

const NetworkContext = createContext<NetworkContextType | undefined>(undefined)

const LATENCY_THRESHOLD_DEGRADED = 500 // ms
const CHECK_INTERVAL = 10000 // 10 seconds

function createInitialStreamStatus(stream: StreamType): StreamStatus {
  return {
    stream,
    status: 'disconnected',
    latency: null,
    blockHeight: null,
    lastChecked: null,
  }
}

function createInitialNetworkState(network: Network): NetworkState {
  return {
    network,
    streams: {
      A: createInitialStreamStatus('A'),
      B: createInitialStreamStatus('B'),
      C: createInitialStreamStatus('C'),
    },
    evmStatus: 'disconnected',
    evmLatency: null,
  }
}

async function checkStreamConnection(
  networkId: string,
  stream: StreamType
): Promise<StreamStatus> {
  const startTime = performance.now()
  
  try {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 5000)

    // Use our API proxy to avoid CORS issues
    const response = await fetch('/api/rpc', {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/json',
        'x-network': networkId,
        'x-stream': stream,
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'pyrax_getChainInfo',
        params: [],
        id: 1,
      }),
      signal: controller.signal,
    })

    clearTimeout(timeoutId)
    const endTime = performance.now()
    const latency = Math.round(endTime - startTime)

    if (!response.ok) throw new Error(`HTTP ${response.status}`)

    const data = await response.json()
    if (data.error) throw new Error(data.error.message || 'RPC Error')

    // pyrax_getChainInfo returns { best_block_height, network, chain_id, ... }
    const blockHeight = data.result?.best_block_height ?? 0
    const status: ConnectionStatus = latency > LATENCY_THRESHOLD_DEGRADED ? 'degraded' : 'connected'

    return {
      stream,
      status,
      latency: data._latency || latency,
      blockHeight,
      lastChecked: Date.now(),
    }
  } catch {
    const endTime = performance.now()
    const latency = Math.round(endTime - startTime)

    return {
      stream,
      status: 'disconnected',
      latency: latency > 5000 ? null : latency,
      blockHeight: null,
      lastChecked: Date.now(),
    }
  }
}

export function NetworkProvider({ children }: { children: React.ReactNode }) {
  const [networkState, setNetworkState] = useState<NetworkState>(() => 
    createInitialNetworkState(getDefaultNetwork())
  )
  const [isChecking, setIsChecking] = useState(false)
  const intervalRef = useRef<NodeJS.Timeout | null>(null)

  const checkConnection = useCallback(async () => {
    if (isChecking) return
    setIsChecking(true)

    try {
      const network = networkState.network

      // Check all three streams in parallel using network ID
      const [streamA, streamB, streamC] = await Promise.all([
        checkStreamConnection(network.id, 'A'),
        checkStreamConnection(network.id, 'B'),
        checkStreamConnection(network.id, 'C'),
      ])

      // Check EVM sidechain (use stream A endpoint for now)
      const evmResult = await checkStreamConnection(network.id, 'A')

      setNetworkState((prev) => ({
        ...prev,
        streams: {
          A: streamA,
          B: streamB,
          C: streamC,
        },
        evmStatus: evmResult.status,
        evmLatency: evmResult.latency,
      }))
    } finally {
      setIsChecking(false)
    }
  }, [networkState.network, isChecking])

  const setNetwork = useCallback((network: Network) => {
    setNetworkState(createInitialNetworkState(network))
  }, [])

  // Initial check and set up interval
  useEffect(() => {
    checkConnection()

    intervalRef.current = setInterval(() => {
      checkConnection()
    }, CHECK_INTERVAL)

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current)
      }
    }
  }, [networkState.network.id])

  const overallStatus = getOverallStatus(networkState)

  return (
    <NetworkContext.Provider
      value={{
        networkState,
        setNetwork,
        checkConnection,
        isChecking,
        overallStatus,
      }}
    >
      {children}
    </NetworkContext.Provider>
  )
}

export function useNetwork() {
  const context = useContext(NetworkContext)
  if (context === undefined) {
    throw new Error('useNetwork must be used within a NetworkProvider')
  }
  return context
}
