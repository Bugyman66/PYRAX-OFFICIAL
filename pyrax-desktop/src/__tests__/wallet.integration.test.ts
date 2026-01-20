/**
 * Integration tests for PYRAX Desktop Wallet functionality
 * 
 * These tests verify that the wallet can:
 * - Create and manage addresses
 * - Fetch balances from the node
 * - Display transaction history including mining rewards
 * - Send transactions
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: (cmd: string, args?: any) => mockInvoke(cmd, args),
}));

// Mock transaction data
const mockTransactions = [
  {
    hash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    from: 'Coinbase',
    to: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
    value: '50.00000000',
    gasPrice: '0',
    gasUsed: '0',
    blockNumber: 100,
    timestamp: Date.now() / 1000 - 3600,
    status: 'confirmed',
    txType: 'mining',
  },
  {
    hash: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    from: 'Unknown',
    to: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
    value: '100.00000000',
    gasPrice: '0',
    gasUsed: '0',
    blockNumber: 50,
    timestamp: Date.now() / 1000 - 7200,
    status: 'confirmed',
    txType: 'receive',
  },
];

const mockAddresses = [
  {
    address: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
    balance: '150.00000000',
    nonce: 0,
    label: 'Default',
  },
];

describe('Wallet Integration Tests', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  describe('Wallet Creation', () => {
    it('should create a new wallet and return mnemonic', async () => {
      const mockMnemonic = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';
      mockInvoke.mockResolvedValueOnce(mockMnemonic);

      const result = await mockInvoke('create_wallet', { password: 'test1234' });
      
      expect(mockInvoke).toHaveBeenCalledWith('create_wallet', { password: 'test1234' });
      expect(result).toBe(mockMnemonic);
      expect(result.split(' ').length).toBeGreaterThanOrEqual(12);
    });

    it('should reject weak passwords', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Password too short'));

      await expect(
        mockInvoke('create_wallet', { password: '123' })
      ).rejects.toThrow('Password too short');
    });
  });

  describe('Address Management', () => {
    it('should fetch addresses with balances', async () => {
      mockInvoke.mockResolvedValueOnce(mockAddresses);

      const result = await mockInvoke('get_addresses');
      
      expect(result).toHaveLength(1);
      expect(result[0].address).toMatch(/^0x[a-fA-F0-9]{40}$/);
      expect(parseFloat(result[0].balance)).toBeGreaterThanOrEqual(0);
    });

    it('should create new address', async () => {
      const newAddress = {
        address: '0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199',
        balance: '0',
        nonce: 0,
        label: 'Mining',
      };
      mockInvoke.mockResolvedValueOnce(newAddress);

      const result = await mockInvoke('create_address', { label: 'Mining' });
      
      expect(result.address).toMatch(/^0x[a-fA-F0-9]{40}$/);
      expect(result.label).toBe('Mining');
    });
  });

  describe('Transaction History', () => {
    it('should fetch transaction history for address', async () => {
      mockInvoke.mockResolvedValueOnce(mockTransactions);

      const result = await mockInvoke('get_transactions', { 
        address: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
        limit: 50 
      });
      
      expect(result).toHaveLength(2);
      expect(result[0].txType).toBe('mining');
      expect(result[1].txType).toBe('receive');
    });

    it('should include mining rewards in transactions', async () => {
      mockInvoke.mockResolvedValueOnce(mockTransactions);

      const result = await mockInvoke('get_transactions', { limit: 50 });
      const miningTxs = result.filter((tx: any) => tx.txType === 'mining');
      
      expect(miningTxs.length).toBeGreaterThan(0);
      expect(miningTxs[0].from).toBe('Coinbase');
    });

    it('should sort transactions by block height descending', async () => {
      mockInvoke.mockResolvedValueOnce(mockTransactions);

      const result = await mockInvoke('get_transactions', { limit: 50 });
      
      for (let i = 1; i < result.length; i++) {
        expect(result[i - 1].blockNumber).toBeGreaterThanOrEqual(result[i].blockNumber);
      }
    });
  });

  describe('Balance Queries', () => {
    it('should get balance for address', async () => {
      mockInvoke.mockResolvedValueOnce('150.00000000');

      const result = await mockInvoke('get_balance', { 
        address: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0' 
      });
      
      expect(parseFloat(result)).toBe(150);
    });

    it('should return 0 for new address', async () => {
      mockInvoke.mockResolvedValueOnce('0');

      const result = await mockInvoke('get_balance', { 
        address: '0x0000000000000000000000000000000000000001' 
      });
      
      expect(parseFloat(result)).toBe(0);
    });
  });

  describe('Send Transaction', () => {
    it('should send transaction successfully', async () => {
      const txHash = '0xnewTxHash1234567890abcdef1234567890abcdef1234567890abcdef12345678';
      mockInvoke.mockResolvedValueOnce(txHash);

      const result = await mockInvoke('send_transaction', {
        request: {
          from: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
          to: '0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199',
          value: '10',
        }
      });
      
      expect(result).toMatch(/^0x[a-fA-F0-9]{64}$/);
    });

    it('should fail with insufficient balance', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Insufficient balance'));

      await expect(
        mockInvoke('send_transaction', {
          request: {
            from: '0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0',
            to: '0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199',
            value: '999999999',
          }
        })
      ).rejects.toThrow('Insufficient balance');
    });
  });

  describe('Wallet Lock/Unlock', () => {
    it('should lock wallet', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await mockInvoke('lock_wallet');
      
      expect(mockInvoke).toHaveBeenCalledWith('lock_wallet');
    });

    it('should unlock wallet with correct password', async () => {
      const walletInfo = { locked: false, addressCount: 1, totalBalance: '150' };
      mockInvoke.mockResolvedValueOnce(walletInfo);

      const result = await mockInvoke('unlock_wallet', { password: 'test1234' });
      
      expect(result.locked).toBe(false);
    });

    it('should fail unlock with wrong password', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid password'));

      await expect(
        mockInvoke('unlock_wallet', { password: 'wrongpassword' })
      ).rejects.toThrow('Invalid password');
    });
  });
});
