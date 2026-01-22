'use client'

import { useState, useEffect, useRef, useCallback } from 'react'
import { useRouter } from 'next/navigation'
import { MagnifyingGlassIcon, XMarkIcon } from '@heroicons/react/20/solid'
import { CubeIcon, ArrowsRightLeftIcon, UserCircleIcon, CodeBracketIcon, CurrencyDollarIcon, CheckCircleIcon, XCircleIcon } from '@heroicons/react/24/outline'

interface SearchResult {
  type: 'block' | 'transaction' | 'address' | 'contract' | 'token'
  value: string
  label: string
  sublabel?: string
  exists?: boolean
  data?: Record<string, unknown>
}

interface ApiSearchResponse {
  results: SearchResult[]
  query: string
  timestamp?: number
  error?: string
}

export default function SearchAutocomplete() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [isOpen, setIsOpen] = useState(false)
  const [loading, setLoading] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const [searchError, setSearchError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const abortControllerRef = useRef<AbortController | null>(null)
  const router = useRouter()

  // Real-time API search with debouncing
  const performSearch = useCallback(async (searchQuery: string) => {
    const trimmed = searchQuery.trim()
    if (!trimmed) {
      setResults([])
      setIsOpen(false)
      setSearchError(null)
      return
    }

    // Cancel any pending request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
    }
    abortControllerRef.current = new AbortController()

    setLoading(true)
    setSearchError(null)

    try {
      const response = await fetch(`/api/search?q=${encodeURIComponent(trimmed)}`, {
        signal: abortControllerRef.current.signal,
        cache: 'no-store',
      })

      if (!response.ok) {
        throw new Error('Search failed')
      }

      const data: ApiSearchResponse = await response.json()
      
      if (data.error) {
        setSearchError(data.error)
        setResults([])
      } else {
        setResults(data.results)
        setIsOpen(data.results.length > 0)
      }
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        // Request was cancelled, ignore
        return
      }
      console.error('Search error:', error)
      setSearchError('Search failed. Please try again.')
      setResults([])
    } finally {
      setLoading(false)
      setSelectedIndex(-1)
    }
  }, [])

  // Debounced search effect
  useEffect(() => {
    if (!query.trim()) {
      setResults([])
      setIsOpen(false)
      setSearchError(null)
      return
    }

    setLoading(true)
    const timer = setTimeout(() => {
      performSearch(query)
    }, 200) // 200ms debounce for API calls

    return () => {
      clearTimeout(timer)
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
      }
    }
  }, [query, performSearch])

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
      case 'token':
        return <CurrencyDollarIcon className="h-4 w-4" />
      default:
        return <MagnifyingGlassIcon className="h-4 w-4" />
    }
  }

  const getExistsIndicator = (exists?: boolean) => {
    if (exists === undefined) return null
    if (exists) {
      return <CheckCircleIcon className="h-3.5 w-3.5 text-green-400" title="Found on chain" />
    }
    return <XCircleIcon className="h-3.5 w-3.5 text-stone-500" title="Not found" />
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
      {(isOpen || loading || searchError) && (
        <div className="absolute top-full left-0 right-0 mt-2 rounded-lg bg-stone-900 border border-stone-700 shadow-xl overflow-hidden z-50">
          {loading ? (
            <div className="px-4 py-3 text-sm text-stone-500 flex items-center gap-2">
              <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              Searching blockchain...
            </div>
          ) : searchError ? (
            <div className="px-4 py-3 text-sm text-red-400">{searchError}</div>
          ) : results.length === 0 ? (
            <div className="px-4 py-3 text-sm text-stone-500">No results found for &quot;{query}&quot;</div>
          ) : (
            <ul className="py-1 max-h-80 overflow-y-auto">
              {results.map((result, index) => (
                <li key={`${result.type}-${result.value}-${index}`}>
                  <button
                    onClick={() => handleSelect(result)}
                    className={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors ${
                      selectedIndex === index
                        ? 'bg-pyrax-500/20 text-pyrax-400'
                        : 'text-stone-300 hover:bg-stone-800 hover:text-white'
                    } ${result.exists === false ? 'opacity-60' : ''}`}
                  >
                    <span className={`shrink-0 ${selectedIndex === index ? 'text-pyrax-400' : 'text-stone-500'}`}>
                      {getIcon(result.type)}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="text-sm font-medium truncate font-mono flex items-center gap-2">
                        {result.label}
                        {getExistsIndicator(result.exists)}
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
                      result.type === 'token' ? 'bg-pink-500/10 text-pink-400' :
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
