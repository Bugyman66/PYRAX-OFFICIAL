import { create } from 'zustand';

export interface LogEntry {
  id: string;
  timestamp: number;
  level: 'info' | 'warn' | 'error' | 'debug';
  category: 'node' | 'block' | 'p2p' | 'rpc' | 'mining' | 'staking' | 'system';
  message: string;
}

interface LogStore {
  logs: LogEntry[];
  maxLogs: number;
  filters: {
    level: string[];
    category: string[];
  };
  addLog: (level: LogEntry['level'], category: LogEntry['category'], message: string) => void;
  clearLogs: () => void;
  setFilters: (filters: Partial<LogStore['filters']>) => void;
}

export const useLogStore = create<LogStore>((set) => ({
  logs: [],
  maxLogs: 500,
  filters: {
    level: ['info', 'warn', 'error', 'debug'],
    category: ['node', 'block', 'p2p', 'rpc', 'mining', 'staking', 'system'],
  },
  
  addLog: (level, category, message) => {
    const entry: LogEntry = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      timestamp: Date.now(),
      level,
      category,
      message,
    };
    
    set((state) => {
      const newLogs = [entry, ...state.logs].slice(0, state.maxLogs);
      return { logs: newLogs };
    });
  },
  
  clearLogs: () => {
    set({ logs: [] });
  },
  
  setFilters: (filters) => {
    set((state) => ({
      filters: { ...state.filters, ...filters },
    }));
  },
}));

// Helper to add logs from outside React
export const log = {
  info: (category: LogEntry['category'], message: string) => 
    useLogStore.getState().addLog('info', category, message),
  warn: (category: LogEntry['category'], message: string) => 
    useLogStore.getState().addLog('warn', category, message),
  error: (category: LogEntry['category'], message: string) => 
    useLogStore.getState().addLog('error', category, message),
  debug: (category: LogEntry['category'], message: string) => 
    useLogStore.getState().addLog('debug', category, message),
};
