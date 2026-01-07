import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatBalance(value: string | number, decimals = 8): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  if (isNaN(num)) return '0';
  
  const units = num / Math.pow(10, decimals);
  
  if (units >= 1_000_000) {
    return `${(units / 1_000_000).toFixed(2)}M`;
  }
  if (units >= 1_000) {
    return `${(units / 1_000).toFixed(2)}K`;
  }
  return units.toFixed(decimals);
}

export function formatHashrate(hashrate: number): string {
  if (hashrate >= 1_000_000_000_000) {
    return `${(hashrate / 1_000_000_000_000).toFixed(2)} TH/s`;
  }
  if (hashrate >= 1_000_000_000) {
    return `${(hashrate / 1_000_000_000).toFixed(2)} GH/s`;
  }
  if (hashrate >= 1_000_000) {
    return `${(hashrate / 1_000_000).toFixed(2)} MH/s`;
  }
  if (hashrate >= 1_000) {
    return `${(hashrate / 1_000).toFixed(2)} KH/s`;
  }
  return `${hashrate.toFixed(2)} H/s`;
}

export function truncateHash(hash: string, chars = 8): string {
  if (!hash || hash.length <= chars * 2 + 2) return hash;
  return `${hash.slice(0, chars + 2)}...${hash.slice(-chars)}`;
}

export function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

export function formatTimeAgo(timestamp: number): string {
  const seconds = Math.floor(Date.now() / 1000 - timestamp);
  
  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}
