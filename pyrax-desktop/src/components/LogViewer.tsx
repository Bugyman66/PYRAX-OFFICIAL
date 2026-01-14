import { useEffect, useRef, useState } from 'react';
import { 
  Terminal, 
  Trash2, 
  Filter, 
  Pause, 
  Play,
  ChevronDown,
  Copy,
  Check
} from 'lucide-react';
import { useLogStore, LogEntry } from '../stores/logStore';

const levelColors: Record<LogEntry['level'], string> = {
  info: 'text-blue-400',
  warn: 'text-yellow-400',
  error: 'text-red-400',
  debug: 'text-gray-400',
};

const categoryColors: Record<LogEntry['category'], string> = {
  node: 'bg-purple-900/50 text-purple-300',
  block: 'bg-green-900/50 text-green-300',
  p2p: 'bg-blue-900/50 text-blue-300',
  rpc: 'bg-cyan-900/50 text-cyan-300',
  mining: 'bg-yellow-900/50 text-yellow-300',
  staking: 'bg-pink-900/50 text-pink-300',
  system: 'bg-gray-700/50 text-gray-300',
};

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const time = date.toLocaleTimeString('en-US', { 
    hour12: false, 
    hour: '2-digit', 
    minute: '2-digit', 
    second: '2-digit',
  });
  const ms = String(date.getMilliseconds()).padStart(3, '0');
  return `${time}.${ms}`;
}

export default function LogViewer() {
  const { logs, filters, clearLogs, setFilters } = useLogStore();
  const [paused, setPaused] = useState(false);
  const [showFilters, setShowFilters] = useState(false);
  const [copied, setCopied] = useState(false);
  const logContainerRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (autoScroll && !paused && logContainerRef.current) {
      logContainerRef.current.scrollTop = 0;
    }
  }, [logs, autoScroll, paused]);

  const filteredLogs = logs.filter((log) => 
    filters.level.includes(log.level) && 
    filters.category.includes(log.category)
  );

  const displayLogs = paused ? [] : filteredLogs;

  const handleCopyLogs = async () => {
    const text = filteredLogs
      .map((log) => `[${formatTime(log.timestamp)}] [${log.level.toUpperCase()}] [${log.category}] ${log.message}`)
      .join('\n');
    
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Failed to copy logs:', e);
    }
  };

  const toggleLevel = (level: string) => {
    const newLevels = filters.level.includes(level)
      ? filters.level.filter((l) => l !== level)
      : [...filters.level, level];
    setFilters({ level: newLevels });
  };

  const toggleCategory = (category: string) => {
    const newCategories = filters.category.includes(category)
      ? filters.category.filter((c) => c !== category)
      : [...filters.category, category];
    setFilters({ category: newCategories });
  };

  return (
    <div className="bg-gray-900 rounded-xl border border-gray-700 overflow-hidden flex flex-col h-[400px]">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center gap-2">
          <Terminal size={16} className="text-green-400" />
          <span className="font-medium text-sm">Real-time Logs</span>
          <span className="text-xs text-gray-500">({filteredLogs.length} entries)</span>
        </div>
        
        <div className="flex items-center gap-1">
          <button
            onClick={() => setPaused(!paused)}
            className={`p-1.5 rounded hover:bg-gray-700 transition-colors ${paused ? 'text-yellow-400' : 'text-gray-400'}`}
            title={paused ? 'Resume' : 'Pause'}
          >
            {paused ? <Play size={14} /> : <Pause size={14} />}
          </button>
          
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`p-1.5 rounded hover:bg-gray-700 transition-colors ${showFilters ? 'text-purple-400' : 'text-gray-400'}`}
            title="Filters"
          >
            <Filter size={14} />
          </button>
          
          <button
            onClick={handleCopyLogs}
            className="p-1.5 rounded hover:bg-gray-700 transition-colors text-gray-400"
            title="Copy logs"
          >
            {copied ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
          </button>
          
          <button
            onClick={clearLogs}
            className="p-1.5 rounded hover:bg-gray-700 transition-colors text-gray-400 hover:text-red-400"
            title="Clear logs"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className="px-4 py-2 bg-gray-800/50 border-b border-gray-700 flex flex-wrap gap-4 text-xs">
          <div className="flex items-center gap-2">
            <span className="text-gray-500">Level:</span>
            {(['info', 'warn', 'error', 'debug'] as const).map((level) => (
              <button
                key={level}
                onClick={() => toggleLevel(level)}
                className={`px-2 py-0.5 rounded ${
                  filters.level.includes(level) 
                    ? levelColors[level] + ' bg-gray-700' 
                    : 'text-gray-600'
                }`}
              >
                {level}
              </button>
            ))}
          </div>
          
          <div className="flex items-center gap-2">
            <span className="text-gray-500">Category:</span>
            {(['node', 'block', 'p2p', 'rpc', 'mining', 'staking'] as const).map((cat) => (
              <button
                key={cat}
                onClick={() => toggleCategory(cat)}
                className={`px-2 py-0.5 rounded ${
                  filters.category.includes(cat) 
                    ? categoryColors[cat]
                    : 'text-gray-600 bg-gray-800'
                }`}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Log Content */}
      <div 
        ref={logContainerRef}
        className="flex-1 overflow-y-auto font-mono text-xs p-2 space-y-0.5"
      >
        {paused ? (
          <div className="flex items-center justify-center h-full text-yellow-400">
            <Pause size={20} className="mr-2" />
            Logging paused
          </div>
        ) : displayLogs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            Waiting for logs...
          </div>
        ) : (
          displayLogs.map((log) => (
            <div 
              key={log.id} 
              className="flex items-start gap-2 py-0.5 hover:bg-gray-800/50 rounded px-1"
            >
              <span className="text-gray-600 flex-shrink-0">
                {formatTime(log.timestamp)}
              </span>
              <span className={`flex-shrink-0 uppercase text-[10px] font-bold w-10 ${levelColors[log.level]}`}>
                {log.level}
              </span>
              <span className={`flex-shrink-0 px-1.5 py-0 rounded text-[10px] ${categoryColors[log.category]}`}>
                {log.category}
              </span>
              <span className="text-gray-300 break-all">
                {log.message}
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
