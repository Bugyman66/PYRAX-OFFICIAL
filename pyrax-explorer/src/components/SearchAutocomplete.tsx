'use client'

import { useState, useEffect, useRef, useCallback } from 'react'
import { useRouter } from 'next/navigation'
import { MagnifyingGlassIcon, XMarkIcon } from '@heroicons/react/20/solid'
import { CubeIcon, ArrowsRightLeftIcon, UserCircleIcon, CodeBracketIcon } from '@heroicons/react/24/outline'

interface SearchResult {
  type: 'block' | 'transaction' | 'address' | 'contract' | 'token'
  value: string
  label: string
  sublabel?: string
}

export default function SearchAutocomplete() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [isOpen, setIsOpen] = useState(false)
  const [loading, setLoading] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const inputRef = useRef<HTMLInputElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const router = useRouter()

  // Detect search type based on query
  const detectSearchType = useCallback((q: string): SearchResult[] => {
    const trimmed = q.trim()
    if (!trimmed) return []

    const results: SearchResult[] = []

    // Block number (pure digits)
    if (/^\d+$/.test(trimmed)) {
      results.push({
        type: 'block',
        value: trimmed,
        label: `Block #${trimmed}`,
        sublabel: 'View block details',
      })
    }

    // Transaction hash (0x + 64 hex chars)
    if (/^0x[a-fA-F0-9]{64}$/.test(trimmed)) {
      results.push({
        type: 'transaction',
        value: trimmed,
        label: `${trimmed.slice(0, 10)}...${trimmed.slice(-8)}`,
        sublabel: 'Transaction',
      })
    }

    // Address (0x + 40 hex chars)
    if (/^0x[a-fA-F0-9]{40}$/.test(trimmed)) {
      results.push({
        type: 'address',
        value: trimmed,
        label: `${trimmed.slice(0, 10)}...${trimmed.slice(-8)}`,
        sublabel: 'Address',
      })
      results.push({
        type: 'contract',
        value: trimmed,
        label: `${trimmed.slice(0, 10)}...${trimmed.slice(-8)}`,
        sublabel: 'View as Contract',
      })
    }

    // Partial hash (starts with 0x but not complete)
    if (/^0x[a-fA-F0-9]+$/.test(trimmed) && trimmed.length > 2 && trimmed.length < 66) {
      results.push({
        type: 'transaction',
        value: trimmed,
        label: `Search for "${trimmed.slice(0, 16)}..."`,
        sublabel: 'Partial hash search',
      })
    }

    return results
  }, [])

  // Debounced search
  useEffect(() => {
    if (!query.trim()) {
      setResults([])
      setIsOpen(false)
      return
    }

    setLoading(true)
    const timer = setTimeout(() => {
      const detected = detectSearchType(query)
      setResults(detected)
      setIsOpen(detected.length > 0)
      setLoading(false)
      setSelectedIndex(-1)
    }, 150)

    return () => clearTimeout(timer)
  }, [query, detectSearchType])

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  // Navigate to result
  const handleSelect = (result: SearchResult) => {
    setIsOpen(false)
    setQuery('')
    
    switch (result.type) {
      case 'block':
        router.push(`/block/${result.value}`)
        break
      case 'transaction':
        router.push(`/tx/${result.value}`)
        break
      case 'address':
        router.push(`/address/${result.value}`)
        break
      case 'contract':
        router.push(`/address/${result.value}`)
        break
      case 'token':
        router.push(`/token/${result.value}`)
        break
    }
  }

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isOpen) return

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setSelectedIndex(prev => Math.min(prev + 1, results.length - 1))
        break
      case 'ArrowUp':
        e.preventDefault()
        setSelectedIndex(prev => Math.max(prev - 1, -1))
        break
      case 'Enter':
        e.preventDefault()
        if (selectedIndex >= 0 && results[selectedIndex]) {
          handleSelect(results[selectedIndex])
        } else if (results.length > 0) {
          handleSelect(results[0])
        }
        break
      case 'Escape':
        setIsOpen(false)
        inputRef.current?.blur()
        break
    }
  }

  const getIcon = (type: string) => {
    switch (type) {
      case 'block':
        return <CubeIcon className="h-4 w-4" />
      case 'transaction':
        return <ArrowsRightLeftIcon className="h-4 w-4" />
      case 'address':
        return <UserCircleIcon className="h-4 w-4" />
      case 'contract':
        return <CodeBracketIcon className="h-4 w-4" />
      default:
        return <MagnifyingGlassIcon className="h-4 w-4" />
    }
  }

  return (
    <div ref={containerRef} className="relative flex-1 flex items-center">
      <div className="relative w-full">
        <MagnifyingGlassIcon
          className="pointer-events-none absolute left-0 top-1/2 -translate-y-1/2 h-5 w-5 text-stone-500"
        />
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onFocus={() => results.length > 0 && setIsOpen(true)}
          onKeyDown={handleKeyDown}
          placeholder="Search by address, txn hash, block..."
          className="block w-full bg-transparent pl-8 pr-8 py-2 text-base text-white outline-none placeholder:text-stone-500 sm:text-sm"
        />
        {query && (
          <button
            onClick={() => { setQuery(''); setIsOpen(false) }}
            className="absolute right-0 top-1/2 -translate-y-1/2 p-1 text-stone-500 hover:text-white"
          >
            <XMarkIcon className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* Autocomplete dropdown */}
      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-2 rounded-lg bg-stone-900 border border-stone-700 shadow-xl overflow-hidden z-50">
          {loading ? (
            <div className="px-4 py-3 text-sm text-stone-500">Searching...</div>
          ) : results.length === 0 ? (
            <div className="px-4 py-3 text-sm text-stone-500">No results found</div>
          ) : (
            <ul className="py-1">
              {results.map((result, index) => (
                <li key={`${result.type}-${result.value}-${index}`}>
                  <button
                    onClick={() => handleSelect(result)}
                    className={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors ${
                      selectedIndex === index
                        ? 'bg-pyrax-500/20 text-pyrax-400'
                        : 'text-stone-300 hover:bg-stone-800 hover:text-white'
                    }`}
                  >
                    <span className={`shrink-0 ${selectedIndex === index ? 'text-pyrax-400' : 'text-stone-500'}`}>
                      {getIcon(result.type)}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="text-sm font-medium truncate font-mono">
                        {result.label}
                      </div>
                      {result.sublabel && (
                        <div className="text-xs text-stone-500">{result.sublabel}</div>
                      )}
                    </div>
                    <span className={`shrink-0 text-xs px-2 py-0.5 rounded-full ${
                      result.type === 'block' ? 'bg-blue-500/10 text-blue-400' :
                      result.type === 'transaction' ? 'bg-purple-500/10 text-purple-400' :
                      result.type === 'address' ? 'bg-green-500/10 text-green-400' :
                      result.type === 'contract' ? 'bg-yellow-500/10 text-yellow-400' :
                      'bg-stone-700 text-stone-400'
                    }`}>
                      {result.type}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
          <div className="px-4 py-2 border-t border-stone-800 text-xs text-stone-500">
            Press <kbd className="px-1.5 py-0.5 rounded bg-stone-800 text-stone-400 font-mono">↵</kbd> to search, 
            <kbd className="px-1.5 py-0.5 rounded bg-stone-800 text-stone-400 font-mono ml-1">↑↓</kbd> to navigate
          </div>
        </div>
      )}
    </div>
  )
}
