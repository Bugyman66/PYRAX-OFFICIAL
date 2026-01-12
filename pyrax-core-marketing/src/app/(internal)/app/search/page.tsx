'use client';

import { useState, useEffect, useCallback } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import {
  Search,
  FileText,
  FolderOpen,
  MessageSquare,
  Filter,
  X,
  Loader2,
  ChevronLeft,
  ChevronRight,
  Calendar,
  User,
} from 'lucide-react';

interface SearchResult {
  type: 'proof' | 'folder' | 'comment';
  id: string;
  title: string;
  description?: string;
  matchedField?: string;
  proof?: {
    id: string;
    title: string;
    status: string;
  };
  folder?: {
    id: string;
    name: string;
  };
  createdBy?: {
    id: string;
    name: string | null;
    email: string;
  };
  createdAt: string;
}

interface SearchResponse {
  results: SearchResult[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

interface Department {
  id: string;
  name: string;
}

const STATUS_OPTIONS = [
  { value: '', label: 'All Statuses' },
  { value: 'Draft', label: 'Draft' },
  { value: 'InReview', label: 'In Review' },
  { value: 'Approved', label: 'Approved' },
  { value: 'Rejected', label: 'Rejected' },
  { value: 'Published', label: 'Published' },
];

export default function SearchPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  
  const [query, setQuery] = useState(searchParams.get('q') || '');
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [showFilters, setShowFilters] = useState(false);
  const [departments, setDepartments] = useState<Department[]>([]);
  
  // Filters
  const [departmentId, setDepartmentId] = useState(searchParams.get('departmentId') || '');
  const [status, setStatus] = useState(searchParams.get('status') || '');
  const [dateFrom, setDateFrom] = useState(searchParams.get('dateFrom') || '');
  const [dateTo, setDateTo] = useState(searchParams.get('dateTo') || '');
  const [page, setPage] = useState(parseInt(searchParams.get('page') || '1', 10));

  // Suggestions
  const [suggestions, setSuggestions] = useState<{ type: string; id: string; title: string }[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  useEffect(() => {
    fetchDepartments();
  }, []);

  useEffect(() => {
    const initialQuery = searchParams.get('q');
    if (initialQuery) {
      performSearch();
    }
  }, []);

  async function fetchDepartments() {
    try {
      const res = await fetch('/api/departments');
      if (res.ok) {
        const data = await res.json();
        setDepartments(data);
      }
    } catch (err) {
      console.error('Failed to fetch departments:', err);
    }
  }

  async function fetchSuggestions(searchQuery: string) {
    if (searchQuery.length < 2) {
      setSuggestions([]);
      return;
    }

    try {
      const res = await fetch(`/api/search?q=${encodeURIComponent(searchQuery)}&suggestions=true`);
      if (res.ok) {
        const data = await res.json();
        setSuggestions(data);
      }
    } catch (err) {
      console.error('Suggestion error:', err);
    }
  }

  const performSearch = useCallback(async (newPage = 1) => {
    setLoading(true);
    setShowSuggestions(false);

    const params = new URLSearchParams();
    if (query) params.set('q', query);
    if (departmentId) params.set('departmentId', departmentId);
    if (status) params.set('status', status);
    if (dateFrom) params.set('dateFrom', dateFrom);
    if (dateTo) params.set('dateTo', dateTo);
    params.set('page', newPage.toString());

    try {
      const res = await fetch(`/api/search?${params.toString()}`);
      if (res.ok) {
        const data = await res.json();
        setResults(data);
        setPage(newPage);

        // Update URL
        router.replace(`/app/search?${params.toString()}`, { scroll: false });
      }
    } catch (err) {
      console.error('Search error:', err);
    } finally {
      setLoading(false);
    }
  }, [query, departmentId, status, dateFrom, dateTo, router]);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    performSearch(1);
  }

  function handleQueryChange(value: string) {
    setQuery(value);
    fetchSuggestions(value);
    setShowSuggestions(true);
  }

  function handleSuggestionClick(suggestion: { type: string; id: string; title: string }) {
    setQuery(suggestion.title);
    setShowSuggestions(false);
    if (suggestion.type === 'proof') {
      router.push(`/app/proofs/${suggestion.id}`);
    } else if (suggestion.type === 'folder') {
      router.push(`/app/folders/${suggestion.id}`);
    }
  }

  function clearFilters() {
    setDepartmentId('');
    setStatus('');
    setDateFrom('');
    setDateTo('');
  }

  const hasActiveFilters = departmentId || status || dateFrom || dateTo;

  const getResultIcon = (type: string) => {
    switch (type) {
      case 'proof': return FileText;
      case 'folder': return FolderOpen;
      case 'comment': return MessageSquare;
      default: return FileText;
    }
  };

  const getResultLink = (result: SearchResult) => {
    switch (result.type) {
      case 'proof': return `/app/proofs/${result.id}`;
      case 'folder': return `/app/folders/${result.id}`;
      case 'comment': return result.proof ? `/app/proofs/${result.proof.id}` : '#';
      default: return '#';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Draft': return 'bg-stone-500';
      case 'InReview': return 'bg-blue-500';
      case 'Approved': return 'bg-green-500';
      case 'Rejected': return 'bg-red-500';
      case 'Published': return 'bg-purple-500';
      default: return 'bg-stone-500';
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-stone-50">Search</h1>
        <p className="text-stone-400 mt-1">Find proofs, folders, and comments</p>
      </div>

      {/* Search Form */}
      <form onSubmit={handleSubmit} className="relative">
        <div className="flex gap-3">
          <div className="flex-1 relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
            <input
              type="text"
              value={query}
              onChange={(e) => handleQueryChange(e.target.value)}
              onFocus={() => query.length >= 2 && setShowSuggestions(true)}
              onBlur={() => setTimeout(() => setShowSuggestions(false), 200)}
              placeholder="Search for proofs, folders, comments..."
              className="w-full pl-12 pr-4 py-3 bg-stone-900 border border-stone-700 rounded-xl text-stone-100 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 focus:border-transparent"
            />
            
            {/* Suggestions Dropdown */}
            {showSuggestions && suggestions.length > 0 && (
              <div className="absolute top-full left-0 right-0 mt-2 bg-stone-900 border border-stone-700 rounded-xl shadow-xl z-10 overflow-hidden">
                {suggestions.map((suggestion) => {
                  const Icon = getResultIcon(suggestion.type);
                  return (
                    <button
                      key={`${suggestion.type}-${suggestion.id}`}
                      type="button"
                      onClick={() => handleSuggestionClick(suggestion)}
                      className="w-full flex items-center gap-3 px-4 py-3 hover:bg-stone-800 text-left transition-colors"
                    >
                      <Icon className="w-4 h-4 text-stone-500" />
                      <span className="text-stone-200">{suggestion.title}</span>
                      <span className="text-xs text-stone-500 capitalize">{suggestion.type}</span>
                    </button>
                  );
                })}
              </div>
            )}
          </div>

          <button
            type="button"
            onClick={() => setShowFilters(!showFilters)}
            className={`px-4 py-3 rounded-xl border transition-colors flex items-center gap-2 ${
              showFilters || hasActiveFilters
                ? 'bg-pyrax-500/10 border-pyrax-500/30 text-pyrax-400'
                : 'bg-stone-900 border-stone-700 text-stone-400 hover:border-stone-600'
            }`}
          >
            <Filter className="w-5 h-5" />
            Filters
            {hasActiveFilters && (
              <span className="w-2 h-2 bg-pyrax-500 rounded-full" />
            )}
          </button>

          <button
            type="submit"
            disabled={loading}
            className="px-6 py-3 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white font-medium rounded-xl transition-colors"
          >
            {loading ? <Loader2 className="w-5 h-5 animate-spin" /> : 'Search'}
          </button>
        </div>

        {/* Filters Panel */}
        {showFilters && (
          <div className="mt-4 p-4 bg-stone-900 border border-stone-700 rounded-xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-medium text-stone-200">Filters</h3>
              {hasActiveFilters && (
                <button
                  type="button"
                  onClick={clearFilters}
                  className="text-sm text-pyrax-400 hover:text-pyrax-300"
                >
                  Clear all
                </button>
              )}
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div>
                <label className="block text-sm text-stone-400 mb-1">Department</label>
                <select
                  value={departmentId}
                  onChange={(e) => setDepartmentId(e.target.value)}
                  className="w-full px-3 py-2 bg-stone-800 border border-stone-600 rounded-lg text-stone-200 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                >
                  <option value="">All Departments</option>
                  {departments.map((dept) => (
                    <option key={dept.id} value={dept.id}>{dept.name}</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm text-stone-400 mb-1">Status</label>
                <select
                  value={status}
                  onChange={(e) => setStatus(e.target.value)}
                  className="w-full px-3 py-2 bg-stone-800 border border-stone-600 rounded-lg text-stone-200 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                >
                  {STATUS_OPTIONS.map((opt) => (
                    <option key={opt.value} value={opt.value}>{opt.label}</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm text-stone-400 mb-1">From Date</label>
                <input
                  type="date"
                  value={dateFrom}
                  onChange={(e) => setDateFrom(e.target.value)}
                  className="w-full px-3 py-2 bg-stone-800 border border-stone-600 rounded-lg text-stone-200 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                />
              </div>

              <div>
                <label className="block text-sm text-stone-400 mb-1">To Date</label>
                <input
                  type="date"
                  value={dateTo}
                  onChange={(e) => setDateTo(e.target.value)}
                  className="w-full px-3 py-2 bg-stone-800 border border-stone-600 rounded-lg text-stone-200 focus:outline-none focus:ring-2 focus:ring-pyrax-500"
                />
              </div>
            </div>
          </div>
        )}
      </form>

      {/* Results */}
      {results && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <p className="text-sm text-stone-400">
              {results.total} result{results.total !== 1 ? 's' : ''} found
              {query && <span> for &quot;{query}&quot;</span>}
            </p>
          </div>

          {results.results.length > 0 ? (
            <div className="space-y-3">
              {results.results.map((result) => {
                const Icon = getResultIcon(result.type);
                return (
                  <Link
                    key={`${result.type}-${result.id}`}
                    href={getResultLink(result)}
                    className="block bg-stone-900 border border-stone-800 rounded-xl p-4 hover:border-stone-700 transition-colors"
                  >
                    <div className="flex items-start gap-4">
                      <div className="w-10 h-10 bg-stone-800 rounded-lg flex items-center justify-center flex-shrink-0">
                        <Icon className="w-5 h-5 text-stone-400" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <h3 className="font-medium text-stone-100 truncate">{result.title}</h3>
                          <span className="text-xs text-stone-500 capitalize px-2 py-0.5 bg-stone-800 rounded">
                            {result.type}
                          </span>
                          {result.proof?.status && (
                            <span className={`text-xs text-white px-2 py-0.5 rounded ${getStatusColor(result.proof.status)}`}>
                              {result.proof.status}
                            </span>
                          )}
                        </div>
                        {result.description && (
                          <p className="text-sm text-stone-400 line-clamp-2">{result.description}</p>
                        )}
                        <div className="flex items-center gap-4 mt-2 text-xs text-stone-500">
                          {result.folder && (
                            <span className="flex items-center gap-1">
                              <FolderOpen className="w-3 h-3" />
                              {result.folder.name}
                            </span>
                          )}
                          {result.createdBy && (
                            <span className="flex items-center gap-1">
                              <User className="w-3 h-3" />
                              {result.createdBy.name || result.createdBy.email}
                            </span>
                          )}
                          <span className="flex items-center gap-1">
                            <Calendar className="w-3 h-3" />
                            {new Date(result.createdAt).toLocaleDateString()}
                          </span>
                        </div>
                      </div>
                    </div>
                  </Link>
                );
              })}
            </div>
          ) : (
            <div className="bg-stone-900 border border-stone-800 rounded-xl p-12 text-center">
              <Search className="w-12 h-12 text-stone-700 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-stone-300 mb-2">No results found</h3>
              <p className="text-stone-500">Try adjusting your search or filters</p>
            </div>
          )}

          {/* Pagination */}
          {results.totalPages > 1 && (
            <div className="flex items-center justify-center gap-2 pt-4">
              <button
                onClick={() => performSearch(page - 1)}
                disabled={page <= 1 || loading}
                className="p-2 rounded-lg bg-stone-800 text-stone-400 hover:text-stone-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ChevronLeft className="w-5 h-5" />
              </button>
              <span className="px-4 py-2 text-stone-300">
                Page {page} of {results.totalPages}
              </span>
              <button
                onClick={() => performSearch(page + 1)}
                disabled={page >= results.totalPages || loading}
                className="p-2 rounded-lg bg-stone-800 text-stone-400 hover:text-stone-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ChevronRight className="w-5 h-5" />
              </button>
            </div>
          )}
        </div>
      )}

      {/* Empty State - No search yet */}
      {!results && !loading && (
        <div className="bg-stone-900 border border-stone-800 rounded-xl p-12 text-center">
          <Search className="w-16 h-16 text-stone-700 mx-auto mb-4" />
          <h3 className="text-xl font-medium text-stone-300 mb-2">Search your workspace</h3>
          <p className="text-stone-500 max-w-md mx-auto">
            Search across proofs, folders, and comments. Use filters to narrow down your results.
          </p>
        </div>
      )}
    </div>
  );
}
