'use client'

import { useState } from 'react'
import { Dialog, DialogBackdrop, DialogPanel, DialogTitle } from '@headlessui/react'
import { XMarkIcon, BeakerIcon, CheckCircleIcon, ExclamationCircleIcon } from '@heroicons/react/24/outline'
import Image from 'next/image'

interface FaucetModalProps {
  isOpen: boolean
  onClose: () => void
}

export default function FaucetModal({ isOpen, onClose }: FaucetModalProps) {
  const [address, setAddress] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<{ success: boolean; message: string; txHash?: string } | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!address.trim()) return

    setLoading(true)
    setResult(null)

    try {
      const response = await fetch('/api/faucet', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ address: address.trim() }),
      })

      const data = await response.json()

      if (response.ok) {
        setResult({
          success: true,
          message: `Successfully sent ${data.amount || 100} PYRAX!`,
          txHash: data.txHash,
        })
        setAddress('')
      } else {
        setResult({
          success: false,
          message: data.error || 'Failed to request tokens',
        })
      }
    } catch (error) {
      setResult({
        success: false,
        message: 'Network error. Please try again.',
      })
    } finally {
      setLoading(false)
    }
  }

  return (
    <Dialog open={isOpen} onClose={onClose} className="relative z-50">
      <DialogBackdrop className="fixed inset-0 bg-black/60 backdrop-blur-sm" />

      <div className="fixed inset-0 overflow-y-auto">
        <div className="flex min-h-full items-center justify-center p-4">
          <DialogPanel className="w-full max-w-md transform overflow-hidden rounded-2xl bg-stone-900 border border-stone-700 p-6 shadow-xl transition-all">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl pyrax-gradient flex items-center justify-center">
                  <BeakerIcon className="w-5 h-5 text-white" />
                </div>
                <div>
                  <DialogTitle className="text-lg font-semibold text-white">
                    PYRAX Faucet
                  </DialogTitle>
                  <p className="text-sm text-stone-400">Get testnet tokens</p>
                </div>
              </div>
              <button
                onClick={onClose}
                className="text-stone-400 hover:text-white transition-colors"
              >
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label htmlFor="address" className="block text-sm font-medium text-stone-300 mb-2">
                  Wallet Address
                </label>
                <input
                  type="text"
                  id="address"
                  value={address}
                  onChange={(e) => setAddress(e.target.value)}
                  placeholder="0x..."
                  className="w-full px-4 py-3 bg-stone-800 border border-stone-700 rounded-xl text-white placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 focus:border-transparent font-mono text-sm"
                  disabled={loading}
                />
              </div>

              <div className="bg-stone-800/50 rounded-xl p-4 space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-stone-400">Amount</span>
                  <span className="text-white font-medium flex items-center gap-1">
                    <Image src="/pyrax-coin.svg" alt="" width={16} height={16} className="w-4 h-4" />
                    100 PYRAX
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-stone-400">Cooldown</span>
                  <span className="text-white font-medium">1 hour</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-stone-400">Network</span>
                  <span className="text-pyrax-400 font-medium">Testnet</span>
                </div>
              </div>

              {result && (
                <div className={`rounded-xl p-4 ${result.success ? 'bg-green-500/10 border border-green-500/30' : 'bg-red-500/10 border border-red-500/30'}`}>
                  <div className="flex items-start gap-3">
                    {result.success ? (
                      <CheckCircleIcon className="w-5 h-5 text-green-400 shrink-0 mt-0.5" />
                    ) : (
                      <ExclamationCircleIcon className="w-5 h-5 text-red-400 shrink-0 mt-0.5" />
                    )}
                    <div>
                      <p className={`text-sm font-medium ${result.success ? 'text-green-400' : 'text-red-400'}`}>
                        {result.message}
                      </p>
                      {result.txHash && (
                        <a
                          href={`/tx/${result.txHash}`}
                          className="text-xs text-pyrax-400 hover:text-pyrax-300 font-mono mt-1 block"
                        >
                          {result.txHash.slice(0, 20)}...
                        </a>
                      )}
                    </div>
                  </div>
                </div>
              )}

              <button
                type="submit"
                disabled={loading || !address.trim()}
                className="w-full py-3 px-4 rounded-xl font-semibold text-white pyrax-gradient hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              >
                {loading ? (
                  <>
                    <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                    Requesting...
                  </>
                ) : (
                  <>
                    <BeakerIcon className="w-5 h-5" />
                    Request Tokens
                  </>
                )}
              </button>
            </form>

            <p className="text-xs text-stone-500 text-center mt-4">
              Testnet tokens have no real value. One request per hour.
            </p>
          </DialogPanel>
        </div>
      </div>
    </Dialog>
  )
}
