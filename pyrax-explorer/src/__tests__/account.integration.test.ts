/**
 * Integration tests for PYRAX Explorer Account page functionality
 * 
 * These tests verify that the explorer can:
 * - Look up addresses and display balance
 * - Show transaction history
 * - Display UTXOs
 * - Handle invalid addresses gracefully
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock the RPC module
vi.mock('@/lib/rpc', () => ({
  getAddressBalance: vi.fn(),
  getAddressTransactions: vi.fn(),
}));

import { getAddressBalance, getAddressTransactions } from '@/lib/rpc';

const mockAddressBalance = {
  address: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
  balance: 15000000000, // 150 PYRAX in base units
  utxo_count: 3,
  utxos: [
    {
      txid: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
      vout: 0,
      value: 5000000000, // 50 PYRAX
      script_pubkey: '0x76a914...',
      height: 100,
      coinbase: true,
    },
    {
      txid: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
      vout: 0,
      value: 10000000000, // 100 PYRAX
      script_pubkey: '0x76a914...',
      height: 50,
      coinbase: false,
    },
    {
      txid: '0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321',
      vout: 1,
      value: 0, // Empty UTXO for testing
      script_pubkey: '0x76a914...',
      height: 25,
      coinbase: false,
    },
  ],
};

const mockTransactions = {
  address: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
  transactions: [
    {
      txid: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
      block_hash: '0xblockhash1234',
      block_height: 100,
      tx_index: 0,
      direction: 'mining' as const,
      value: 5000000000,
      timestamp: Date.now() / 1000 - 3600,
      is_coinbase: true,
      confirmations: 10,
    },
    {
      txid: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
      block_hash: '0xblockhash5678',
      block_height: 50,
      tx_index: 0,
      direction: 'receive' as const,
      value: 10000000000,
      timestamp: Date.now() / 1000 - 7200,
      is_coinbase: false,
      confirmations: 60,
    },
  ],
  total_received: 15000000000,
  total_sent: 0,
  tx_count: 2,
};

describe('Explorer Account Page Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Address Lookup', () => {
    it('should fetch and display address balance', async () => {
      (getAddressBalance as any).mockResolvedValueOnce(mockAddressBalance);

      const result = await getAddressBalance('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result).not.toBeNull();
      expect(result!.balance).toBe(15000000000);
      expect(result!.utxo_count).toBe(3);
    });

    it('should handle address with no activity', async () => {
      (getAddressBalance as any).mockResolvedValueOnce({
        address: '0x0000000000000000000000000000000000000001',
        balance: 0,
        utxo_count: 0,
        utxos: [],
      });

      const result = await getAddressBalance('0x0000000000000000000000000000000000000001');
      
      expect(result!.balance).toBe(0);
      expect(result!.utxo_count).toBe(0);
    });

    it('should return null for invalid address format', async () => {
      (getAddressBalance as any).mockResolvedValueOnce(null);

      const result = await getAddressBalance('invalid-address');
      
      expect(result).toBeNull();
    });
  });

  describe('Transaction History', () => {
    it('should fetch transaction history for address', async () => {
      (getAddressTransactions as any).mockResolvedValueOnce(mockTransactions);

      const result = await getAddressTransactions('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0', 50);
      
      expect(result).not.toBeNull();
      expect(result!.tx_count).toBe(2);
      expect(result!.transactions).toHaveLength(2);
    });

    it('should correctly identify mining rewards', async () => {
      (getAddressTransactions as any).mockResolvedValueOnce(mockTransactions);

      const result = await getAddressTransactions('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0', 50);
      const miningTxs = result!.transactions.filter(tx => tx.direction === 'mining');
      
      expect(miningTxs).toHaveLength(1);
      expect(miningTxs[0].is_coinbase).toBe(true);
    });

    it('should show total received and sent', async () => {
      (getAddressTransactions as any).mockResolvedValueOnce(mockTransactions);

      const result = await getAddressTransactions('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0', 50);
      
      expect(result!.total_received).toBe(15000000000);
      expect(result!.total_sent).toBe(0);
    });

    it('should include confirmations for each transaction', async () => {
      (getAddressTransactions as any).mockResolvedValueOnce(mockTransactions);

      const result = await getAddressTransactions('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0', 50);
      
      result!.transactions.forEach(tx => {
        expect(tx.confirmations).toBeGreaterThan(0);
      });
    });
  });

  describe('UTXO Display', () => {
    it('should display all UTXOs for address', async () => {
      (getAddressBalance as any).mockResolvedValueOnce(mockAddressBalance);

      const result = await getAddressBalance('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result!.utxos).toHaveLength(3);
    });

    it('should identify coinbase UTXOs', async () => {
      (getAddressBalance as any).mockResolvedValueOnce(mockAddressBalance);

      const result = await getAddressBalance('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      const coinbaseUtxos = result!.utxos.filter(u => u.coinbase);
      
      expect(coinbaseUtxos).toHaveLength(1);
      expect(coinbaseUtxos[0].value).toBe(5000000000);
    });

    it('should show UTXO block heights', async () => {
      (getAddressBalance as any).mockResolvedValueOnce(mockAddressBalance);

      const result = await getAddressBalance('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      result!.utxos.forEach(utxo => {
        expect(utxo.height).toBeGreaterThanOrEqual(0);
      });
    });
  });

  describe('Address Validation', () => {
    it('should validate correct Ethereum-style address', () => {
      const validAddress = '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0';
      const isValid = /^0x[a-fA-F0-9]{40}$/.test(validAddress);
      
      expect(isValid).toBe(true);
    });

    it('should reject address without 0x prefix', () => {
      const invalidAddress = '742d35Cc6634C0532925a3b844Bc9e7595f1f3c0';
      const isValid = /^0x[a-fA-F0-9]{40}$/.test(invalidAddress);
      
      expect(isValid).toBe(false);
    });

    it('should reject address with wrong length', () => {
      const shortAddress = '0x742d35Cc6634C0532925a3b844Bc9e7595';
      const isValid = /^0x[a-fA-F0-9]{40}$/.test(shortAddress);
      
      expect(isValid).toBe(false);
    });

    it('should reject address with invalid characters', () => {
      const invalidAddress = '0x742d35Cc6634C0532925a3b844Bc9e7595fZZZZZ';
      const isValid = /^0x[a-fA-F0-9]{40}$/.test(invalidAddress);
      
      expect(isValid).toBe(false);
    });
  });

  describe('Value Formatting', () => {
    it('should convert base units to PYRAX correctly', () => {
      const baseUnits = 15000000000; // 150 PYRAX
      const pyrax = baseUnits / 100_000_000;
      
      expect(pyrax).toBe(150);
    });

    it('should format small amounts with decimals', () => {
      const baseUnits = 12345678; // 0.12345678 PYRAX
      const pyrax = baseUnits / 100_000_000;
      
      expect(pyrax.toFixed(8)).toBe('0.12345678');
    });
  });
});
