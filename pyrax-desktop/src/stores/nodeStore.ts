import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';

export interface NodeStatus {
  running: boolean;
  connected: boolean;
  syncing: boolean;
  syncProgress: number;
  peerCount: number;
  blockHeight: number;
  blockHash: string;
  network: string;
  version: string;
}

export interface ChainInfo {
  chainId: number;
  bestBlockHash: string;
  bestBlockHeight: number;
  genesisHash: string;
  difficulty: string;
  totalDifficulty: string;
  peerCount: number;
}

export interface PeerInfo {
  id: string;
  address: string;
  clientVersion: string;
  bestHeight: number;
  latencyMs: number;
  direction: string;
}

interface NodeStore {
  status: NodeStatus | null;
  chainInfo: ChainInfo | null;
  peers: PeerInfo[];
  loading: boolean;
  error: string | null;
  
  fetchStatus: () => Promise<void>;
  fetchChainInfo: () => Promise<void>;
  fetchPeers: () => Promise<void>;
  startNode: () => Promise<void>;
  stopNode: () => Promise<void>;
}

export const useNodeStore = create<NodeStore>((set, get) => ({
  status: null,
  chainInfo: null,
  peers: [],
  loading: false,
  error: null,

  fetchStatus: async () => {
    try {
      const status = await invoke<NodeStatus>('get_node_status');
      set({ status, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  fetchChainInfo: async () => {
    try {
      const chainInfo = await invoke<ChainInfo>('get_chain_info');
      set({ chainInfo, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  fetchPeers: async () => {
    try {
      const peers = await invoke<PeerInfo[]>('get_peers');
      set({ peers, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  startNode: async () => {
    set({ loading: true, error: null });
    try {
      const status = await invoke<NodeStatus>('start_node');
      if (status) {
        set({ status, loading: false, error: null });
      } else {
        set({ loading: false, error: 'No status returned' });
      }
    } catch (e) {
      console.error('Failed to start node:', e);
      set({ error: String(e), loading: false });
      // Still fetch status to update UI
      try {
        const status = await invoke<NodeStatus>('get_node_status');
        if (status) {
          set({ status });
        }
      } catch (e2) {
        console.error('Failed to fetch status after start error:', e2);
      }
    }
  },

  stopNode: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('stop_node');
      await get().fetchStatus();
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
}));
