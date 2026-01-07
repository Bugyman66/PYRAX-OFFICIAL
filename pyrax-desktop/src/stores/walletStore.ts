import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';

export interface WalletInfo {
  locked: boolean;
  addressCount: number;
  totalBalance: string;
}

export interface AddressInfo {
  address: string;
  balance: string;
  nonce: number;
  label?: string;
}

export interface TransactionInfo {
  hash: string;
  from: string;
  to?: string;
  value: string;
  gasPrice: string;
  gasUsed: string;
  blockNumber?: number;
  timestamp?: number;
  status: string;
  txType: string;
}

interface WalletStore {
  info: WalletInfo | null;
  addresses: AddressInfo[];
  transactions: TransactionInfo[];
  selectedAddress: string | null;
  loading: boolean;
  error: string | null;

  createWallet: (password: string) => Promise<string>;
  importMnemonic: (mnemonic: string, password: string) => Promise<void>;
  unlockWallet: (password: string) => Promise<void>;
  lockWallet: () => Promise<void>;
  fetchAddresses: () => Promise<void>;
  fetchBalance: (address: string) => Promise<string>;
  fetchTransactions: (address?: string) => Promise<void>;
  sendTransaction: (to: string, value: string, from?: string) => Promise<string>;
  createAddress: (label?: string) => Promise<void>;
  setSelectedAddress: (address: string) => void;
}

export const useWalletStore = create<WalletStore>((set, get) => ({
  info: null,
  addresses: [],
  transactions: [],
  selectedAddress: null,
  loading: false,
  error: null,

  createWallet: async (password: string) => {
    set({ loading: true, error: null });
    try {
      const mnemonic = await invoke<string>('create_wallet', { password });
      // Set wallet info as unlocked after creation
      set({ 
        info: { locked: false, addressCount: 1, totalBalance: '0' },
        loading: false,
        error: null
      });
      // Fetch addresses after wallet creation
      await get().fetchAddresses();
      return mnemonic;
    } catch (e) {
      console.error('Failed to create wallet:', e);
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  importMnemonic: async (mnemonic: string, password: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('import_mnemonic', { mnemonic, password });
      await get().fetchAddresses();
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  unlockWallet: async (password: string) => {
    set({ loading: true, error: null });
    try {
      const info = await invoke<WalletInfo>('unlock_wallet', { password });
      set({ info, loading: false });
      await get().fetchAddresses();
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  lockWallet: async () => {
    try {
      await invoke('lock_wallet');
      set({ info: { locked: true, addressCount: 0, totalBalance: '0' }, addresses: [], transactions: [] });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  fetchAddresses: async () => {
    try {
      const addresses = await invoke<AddressInfo[]>('get_addresses');
      set({ addresses, error: null });
      if (addresses.length > 0 && !get().selectedAddress) {
        set({ selectedAddress: addresses[0].address });
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },

  fetchBalance: async (address: string) => {
    try {
      const balance = await invoke<string>('get_balance', { address });
      return balance;
    } catch (e) {
      set({ error: String(e) });
      return '0';
    }
  },

  fetchTransactions: async (address?: string) => {
    try {
      const transactions = await invoke<TransactionInfo[]>('get_transactions', { 
        address: address || get().selectedAddress,
        limit: 50 
      });
      set({ transactions, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  sendTransaction: async (to: string, value: string, from?: string) => {
    set({ loading: true, error: null });
    try {
      const hash = await invoke<string>('send_transaction', {
        request: {
          from: from || get().selectedAddress,
          to,
          value,
        }
      });
      set({ loading: false });
      await get().fetchTransactions();
      return hash;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  createAddress: async (label?: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('create_address', { label });
      await get().fetchAddresses();
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  setSelectedAddress: (address: string) => {
    set({ selectedAddress: address });
  },
}));
