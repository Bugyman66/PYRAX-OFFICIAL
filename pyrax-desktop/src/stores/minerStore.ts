import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';

export interface MinerStatus {
  running: boolean;
  hashrate: number;
  acceptedShares: number;
  rejectedShares: number;
  blocksFound: number;
  temperature?: number;
  fanSpeed?: number;
  powerUsage?: number;
  algorithm: string;
  pool?: string;
}

interface MinerStore {
  status: MinerStatus | null;
  loading: boolean;
  error: string | null;

  fetchStatus: () => Promise<void>;
  startMiner: (address: string, threads?: number) => Promise<void>;
  stopMiner: () => Promise<void>;
  getHashrate: () => Promise<number>;
}

export const useMinerStore = create<MinerStore>((set, get) => ({
  status: null,
  loading: false,
  error: null,

  fetchStatus: async () => {
    try {
      const status = await invoke<MinerStatus>('get_miner_status');
      set({ status, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  startMiner: async (address: string, threads?: number) => {
    set({ loading: true, error: null });
    try {
      await invoke('start_miner', { address, threads: threads || 0 });
      await get().fetchStatus();
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  stopMiner: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('stop_miner');
      await get().fetchStatus();
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  getHashrate: async () => {
    try {
      const hashrate = await invoke<number>('get_hashrate');
      return hashrate;
    } catch (e) {
      return 0;
    }
  },
}));
