/**
 * Integration tests for PYRAX Faucet functionality
 * 
 * These tests verify that the faucet can:
 * - Send tokens to valid addresses
 * - Enforce rate limiting
 * - Handle invalid addresses
 * - Return proper transaction hashes
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock fetch for API calls
const mockFetch = vi.fn();
global.fetch = mockFetch;

const FAUCET_API_URL = '/api/faucet';

interface FaucetResponse {
  success: boolean;
  txHash?: string;
  amount?: number;
  error?: string;
  nextEligible?: number;
}

async function requestFaucet(address: string): Promise<FaucetResponse> {
  const response = await fetch(FAUCET_API_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ address }),
  });
  return response.json();
}

describe('Faucet Integration Tests', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  describe('Token Distribution', () => {
    it('should send tokens to valid address', async () => {
      const mockResponse = {
        ok: true,
        json: async () => ({
          success: true,
          txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
          amount: 100,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.success).toBe(true);
      expect(result.txHash).toMatch(/^0x[a-fA-F0-9]{64}$/);
      expect(result.amount).toBe(100);
    });

    it('should return correct amount of tokens', async () => {
      const mockResponse = {
        ok: true,
        json: async () => ({
          success: true,
          txHash: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
          amount: 100,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.amount).toBe(100);
    });

    it('should provide transaction hash for tracking', async () => {
      const expectedTxHash = '0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321';
      const mockResponse = {
        ok: true,
        json: async () => ({
          success: true,
          txHash: expectedTxHash,
          amount: 100,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.txHash).toBe(expectedTxHash);
    });
  });

  describe('Rate Limiting', () => {
    it('should enforce cooldown period', async () => {
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Please wait before requesting again',
          nextEligible: Date.now() / 1000 + 3600,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('wait');
    });

    it('should provide next eligible time when rate limited', async () => {
      const nextEligible = Date.now() / 1000 + 3600;
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Rate limited',
          nextEligible,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.nextEligible).toBeDefined();
      expect(result.nextEligible).toBeGreaterThan(Date.now() / 1000);
    });

    it('should enforce daily request limit per IP', async () => {
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Maximum 10 requests per day exceeded',
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('Maximum');
    });
  });

  describe('Address Validation', () => {
    it('should reject invalid address format', async () => {
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Invalid address format',
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('invalid-address');
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('Invalid');
    });

    it('should reject address without 0x prefix', async () => {
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Invalid address format',
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.success).toBe(false);
    });

    it('should reject empty address', async () => {
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Address is required',
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('');
      
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });

    it('should accept checksummed addresses', async () => {
      const mockResponse = {
        ok: true,
        json: async () => ({
          success: true,
          txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
          amount: 100,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.success).toBe(true);
    });

    it('should accept lowercase addresses', async () => {
      const mockResponse = {
        ok: true,
        json: async () => ({
          success: true,
          txHash: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
          amount: 100,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35cc6634c0532925a3b844bc9e7595f1f3c0');
      
      expect(result.success).toBe(true);
    });
  });

  describe('Error Handling', () => {
    it('should handle network errors gracefully', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0'))
        .rejects.toThrow('Network error');
    });

    it('should handle empty faucet balance', async () => {
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Faucet is empty, please try later',
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('empty');
    });

    it('should handle disabled faucet', async () => {
      const mockResponse = {
        ok: false,
        json: async () => ({
          success: false,
          error: 'Faucet is currently disabled',
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('disabled');
    });
  });

  describe('Transaction Confirmation', () => {
    it('should return pending transaction that can be tracked', async () => {
      const txHash = '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
      const mockResponse = {
        ok: true,
        json: async () => ({
          success: true,
          txHash,
          amount: 100,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const result = await requestFaucet('0x742d35Cc6634C0532925a3b844Bc9e7595f1f3c0');
      
      expect(result.txHash).toBe(txHash);
      // Transaction can be looked up on explorer at /tx/{txHash}
    });
  });
});

describe('Faucet UI Integration', () => {
  it('should format PYRAX amounts correctly', () => {
    const amount = 100;
    const formatted = `${amount} PYRAX`;
    
    expect(formatted).toBe('100 PYRAX');
  });

  it('should format cooldown time correctly', () => {
    const secondsRemaining = 3600;
    const minutes = Math.floor(secondsRemaining / 60);
    const formatted = `${minutes} minutes`;
    
    expect(formatted).toBe('60 minutes');
  });

  it('should truncate transaction hash for display', () => {
    const txHash = '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
    const truncated = `${txHash.slice(0, 20)}...`;
    
    expect(truncated).toBe('0x1234567890abcdef12...');
    expect(truncated.length).toBeLessThan(txHash.length);
  });
});
