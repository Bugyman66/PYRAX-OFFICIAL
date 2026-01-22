import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from './toastStore';

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
  // Extended P2P stats for realtime connection monitoring
  inboundPeers?: number;
  outboundPeers?: number;
  targetPeers?: number;
  maxPeers?: number;
  dialAttempts?: number;
  dialSuccesses?: number;
  dialFailures?: number;
  averageRttMs?: number;
  networkState?: string;
  natStatus?: string;
  meshPeers?: number;
  gossipPeers?: number;
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
  ip: string;
  port: number;
  protocol: string;
  direction: string;
  connectedSecs: number;
  version: string;
  blockHeight: number;
}

export interface MeshConnection {
  peerA: string;
  peerB: string;
  topic: string;
  connectionType: string;
}

export interface RelayCircuit {
  srcPeer: string;
  dstPeer: string;
  establishedAt: number;
}

export interface BootnodeInfo {
  id: string;
  url: string;
  latencyMs: number;
  city: string;
  region: string;
  country: string;
  countryCode: string;
  stream: 'A' | 'B' | 'C';
  online: boolean;
}

interface NodeStore {
  status: NodeStatus | null;
  chainInfo: ChainInfo | null;
  peers: PeerInfo[];
  bootnodes: BootnodeInfo[];
  loading: boolean;
  error: string | null;
  reconnectAttempts: number;
  maxReconnectAttempts: number;
  isReconnecting: boolean;
  // Flag to track user-requested stop - prevents auto-reconnect after manual stop
  userRequestedStop: boolean;
  
  fetchStatus: () => Promise<void>;
  fetchChainInfo: () => Promise<void>;
  fetchPeers: () => Promise<void>;
  fetchBootnodes: () => Promise<void>;
  startNode: () => Promise<void>;
  stopNode: () => Promise<void>;
  attemptReconnect: () => Promise<void>;
}

export const useNodeStore = create<NodeStore>((set, get) => ({
  status: null,
  chainInfo: null,
  peers: [],
  bootnodes: [],
  loading: false,
  error: null,
  reconnectAttempts: 0,
  maxReconnectAttempts: 5,
  isReconnecting: false,
  userRequestedStop: false,

  fetchStatus: async () => {
    try {
      const status = await invoke<NodeStatus>('get_node_status');
      const prevStatus = get().status;
      set({ status, error: null });
      
      // Auto-reconnect if connection was lost (but NOT if user manually stopped)
      const { userRequestedStop } = get();
      if (prevStatus?.connected && !status.connected && status.running && !userRequestedStop) {
        console.log('[NodeStore] Connection lost (network issue), attempting reconnect...');
        get().attemptReconnect();
      } else if (userRequestedStop && !status.running) {
        // User stopped the node, reset the flag for next start
        console.log('[NodeStore] User-requested stop completed, skipping auto-reconnect');
      }
      
      // Reset reconnect attempts on successful connection
      if (status.connected) {
        set({ reconnectAttempts: 0, isReconnecting: false });
      }
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

  fetchBootnodes: async () => {
    try {
      const bootnodes = await invoke<BootnodeInfo[]>('get_bootnode_info');
      set({ bootnodes, error: null });
    } catch (e) {
      console.error('Failed to fetch bootnodes:', e);
      // Don't set error for bootnode fetch failure
    }
  },

  attemptReconnect: async () => {
    const { reconnectAttempts, maxReconnectAttempts, isReconnecting, status, userRequestedStop } = get();
    
    // Don't reconnect if user manually stopped the node
    if (userRequestedStop) {
      console.log('[NodeStore] Skipping reconnect - user requested stop');
      return;
    }
    
    if (isReconnecting || reconnectAttempts >= maxReconnectAttempts) {
      if (reconnectAttempts >= maxReconnectAttempts) {
        toast.error('Max reconnect attempts reached. Please restart the node manually.');
      }
      return;
    }
    
    set({ isReconnecting: true, reconnectAttempts: reconnectAttempts + 1 });
    toast.info(`Reconnecting... (attempt ${reconnectAttempts + 1}/${maxReconnectAttempts})`);
    
    try {
      // Stop and restart the node
      await invoke('stop_node');
      await new Promise(resolve => setTimeout(resolve, 2000)); // Wait 2 seconds
      await invoke<NodeStatus>('start_node');
      
      // Check if connected after restart
      const newStatus = await invoke<NodeStatus>('get_node_status');
      if (newStatus.connected) {
        set({ status: newStatus, isReconnecting: false, reconnectAttempts: 0 });
        toast.success('Reconnected successfully!');
      } else {
        set({ isReconnecting: false });
        // Try again after delay
        setTimeout(() => get().attemptReconnect(), 5000);
      }
    } catch (e) {
      console.error('Reconnect failed:', e);
      set({ isReconnecting: false });
      // Try again after delay
      setTimeout(() => get().attemptReconnect(), 5000);
    }
  },

  startNode: async () => {
    // Clear userRequestedStop flag - user wants the node running
    set({ loading: true, error: null, reconnectAttempts: 0, userRequestedStop: false });
    try {
      const status = await invoke<NodeStatus>('start_node');
      if (status) {
        set({ status, loading: false, error: null });
        toast.success(`Connected to ${status.network} network`);
        // Fetch bootnodes after connecting
        get().fetchBootnodes();
      } else {
        const err = 'No status returned from node';
        set({ loading: false, error: err });
        toast.error(err);
      }
    } catch (e) {
      console.error('Failed to start node:', e);
      const errorMsg = String(e);
      set({ error: errorMsg, loading: false });
      toast.error(`Failed to start node: ${errorMsg}`);
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
    // Set userRequestedStop flag - prevents auto-reconnect after manual stop
    set({ loading: true, error: null, isReconnecting: false, reconnectAttempts: 0, userRequestedStop: true });
    console.log('[NodeStore] User requested stop - auto-reconnect disabled');
    try {
      await invoke('stop_node');
      await get().fetchStatus();
      set({ loading: false, bootnodes: [] });
      toast.info('Node stopped');
    } catch (e) {
      const errorMsg = String(e);
      set({ error: errorMsg, loading: false });
      toast.error(`Failed to stop node: ${errorMsg}`);
    }
  },
}));
