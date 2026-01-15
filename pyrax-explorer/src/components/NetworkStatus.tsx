'use client'

import { useNetwork } from '@/context/NetworkContext'
import { ArrowPathIcon } from '@heroicons/react/20/solid'
import { ConnectionStatus, StreamType, STREAMS } from '@/lib/networks'

function classNames(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ')
}

function getStatusColors(status: ConnectionStatus) {
  switch (status) {
    case 'connected':
      return { ping: 'bg-green-400', dot: 'bg-green-500', text: 'text-green-500' }
    case 'degraded':
      return { ping: 'bg-yellow-400', dot: 'bg-yellow-500', text: 'text-yellow-500' }
    case 'disconnected':
    default:
      return { ping: 'bg-red-400', dot: 'bg-red-500', text: 'text-red-500' }
  }
}

function getStatusText(status: ConnectionStatus) {
  switch (status) {
    case 'connected': return 'Online'
    case 'degraded': return 'Slow'
    case 'disconnected': return 'Offline'
  }
}

export default function NetworkStatus() {
  const { networkState, checkConnection, isChecking, overallStatus } = useNetwork()

  const colors = getStatusColors(overallStatus)

  // Count connected streams
  const connectedStreams = Object.values(networkState.streams).filter(s => s.status === 'connected').length

  return (
    <div className="flex items-center gap-3">
      {/* Refresh button */}
      <button
        onClick={() => checkConnection()}
        disabled={isChecking}
        className="p-1 text-stone-400 hover:text-white transition-colors disabled:opacity-50"
        title="Refresh all streams"
      >
        <ArrowPathIcon className={classNames('size-4', isChecking && 'animate-spin')} />
      </button>

      {/* TriStream status indicators with pulsing dots */}
      <div className="hidden sm:flex items-center gap-1.5">
        {(['A', 'B', 'C'] as StreamType[]).map((streamId) => {
          const stream = networkState.streams[streamId]
          const streamInfo = STREAMS[streamId]
          const streamColors = getStatusColors(stream.status)
          const isOnline = stream.status === 'connected'
          
          return (
            <div
              key={streamId}
              className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-stone-800/50"
              title={`${streamInfo.name}: ${stream.status}${stream.latency ? ` (${stream.latency}ms)` : ''}`}
            >
              <span className="text-[10px] font-bold text-stone-500">{streamId}</span>
              <span className="relative flex h-2 w-2">
                {isOnline && (
                  <span className={classNames(
                    'animate-ping absolute inline-flex h-full w-full rounded-full opacity-75',
                    streamColors.ping
                  )} />
                )}
                <span className={classNames('relative inline-flex rounded-full h-2 w-2', streamColors.dot)} />
              </span>
            </div>
          )
        })}
      </div>

      {/* Overall status */}
      <div className="flex items-center gap-2">
        <span className="relative flex h-2.5 w-2.5">
          {overallStatus === 'connected' && (
            <span className={classNames('animate-ping absolute inline-flex h-full w-full rounded-full opacity-75', colors.ping)} />
          )}
          <span className={classNames('relative inline-flex rounded-full h-2.5 w-2.5', colors.dot)} />
        </span>

        <div className="hidden lg:flex flex-col items-end">
          <span className="text-sm font-medium text-white">
            {networkState.network.name}
          </span>
          <span className={classNames('text-xs', colors.text)}>
            {connectedStreams}/3 Streams
          </span>
        </div>

        <span className="lg:hidden text-sm text-stone-400">
          {networkState.network.name}
        </span>
      </div>
    </div>
  )
}
