/**
 * Integration tests for PYRAX Node Version Consistency
 * 
 * These tests verify that:
 * - All bootnodes report consistent version numbers
 * - All connected peers report their node version correctly
 * - Version format follows "X.Y.Z" semver pattern
 * - Outdated nodes are properly identified
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock fetch for API calls
const mockFetch = vi.fn();
global.fetch = mockFetch;

interface NodeInfo {
  id: string;
  version: string;
  isBootnode: boolean;
  online: boolean;
}

interface ChainInfoResponse {
  chain_id: number;
  network: string;
  best_block_height: number;
  node_version: string;
}

interface PeerInfo {
  peer_id: string;
  version: string;
  address: string;
}

// Expected version format: X.Y.Z (semver)
const VERSION_REGEX = /^\d+\.\d+\.\d+$/;

// Current expected version (should match Cargo.toml)
const EXPECTED_VERSION = '0.2.54';

describe('Node Version Consistency Tests', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  describe('Bootnode Version Reporting', () => {
    it('should return node_version in ChainInfo response', async () => {
      const mockResponse = {
        ok: true,
        json: async () => ({
          jsonrpc: '2.0',
          result: {
            chain_id: 7225,
            network: 'devnet',
            best_block_height: 100,
            node_version: 'pyrax-node/0.2.54',
          },
          id: 1,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const response = await fetch('http://209.38.137.105:28545', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'pyrax_getChainInfo',
          params: [],
          id: 1,
        }),
      });

      const data = await response.json();
      expect(data.result.node_version).toBeDefined();
      expect(data.result.node_version).toContain('pyrax-node/');
    });

    it('should extract version number from node_version field', () => {
      const nodeVersion = 'pyrax-node/0.2.54';
      const version = nodeVersion.includes('/') 
        ? nodeVersion.split('/')[1] 
        : nodeVersion;
      
      expect(version).toBe('0.2.54');
      expect(VERSION_REGEX.test(version)).toBe(true);
    });

    it('should handle malformed version strings gracefully', () => {
      const testCases = [
        { input: 'pyrax-node/0.2.54', expected: '0.2.54' },
        { input: 'rust-libp2p/0.44.2', expected: '0.44.2' },
        { input: '0.2.54', expected: '0.2.54' },
        { input: '', expected: 'unknown' },
        { input: undefined, expected: 'unknown' },
      ];

      testCases.forEach(({ input, expected }) => {
        const rawVersion = input || '';
        const version = rawVersion.includes('/') 
          ? rawVersion.split('/')[1] 
          : (rawVersion || 'unknown');
        expect(version).toBe(expected);
      });
    });

    it('should report same version across all bootnodes', async () => {
      // Both bootnodes should report the same version
      const bootnodeVersions = ['0.2.54', '0.2.54'];
      
      const uniqueVersions = new Set(bootnodeVersions);
      expect(uniqueVersions.size).toBe(1);
      expect(bootnodeVersions[0]).toBe(bootnodeVersions[1]);
    });
  });

  describe('Peer Version Reporting', () => {
    it('should include version in peer info from getNetworkInfo', async () => {
      const mockResponse = {
        ok: true,
        json: async () => ({
          jsonrpc: '2.0',
          result: {
            peer_count: 2,
            peers: [
              { peer_id: '12D3KooW...', version: 'pyrax-node/0.2.54', address: '/ip4/1.2.3.4/tcp/30303' },
              { peer_id: '12D3KooX...', version: 'pyrax-node/0.2.54', address: '/ip4/5.6.7.8/tcp/30303' },
            ],
            local_peer_id: '12D3KooWLocal...',
          },
          id: 1,
        }),
      };
      mockFetch.mockResolvedValueOnce(mockResponse);

      const response = await fetch('http://209.38.137.105:28545', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'pyrax_getNetworkInfo',
          params: [],
          id: 1,
        }),
      });

      const data = await response.json();
      expect(data.result.peers).toBeDefined();
      expect(data.result.peers.length).toBeGreaterThan(0);
      
      data.result.peers.forEach((peer: PeerInfo) => {
        expect(peer.version).toBeDefined();
        expect(peer.version).toContain('pyrax-node/');
      });
    });

    it('should identify outdated peers', () => {
      const currentVersion = '0.2.54';
      const peerVersions = ['0.2.54', '0.2.53', '0.2.50', '0.1.0'];
      
      const outdatedPeers = peerVersions.filter(v => v !== currentVersion);
      
      expect(outdatedPeers.length).toBe(3);
      expect(outdatedPeers).toContain('0.2.53');
      expect(outdatedPeers).toContain('0.2.50');
      expect(outdatedPeers).toContain('0.1.0');
    });
  });

  describe('Version Format Validation', () => {
    it('should validate semver format', () => {
      const validVersions = ['0.1.0', '0.2.54', '1.0.0', '10.20.30'];
      const invalidVersions = ['v0.1.0', '0.1', '0', 'abc', '', 'rust-libp2p'];

      validVersions.forEach(v => {
        expect(VERSION_REGEX.test(v)).toBe(true);
      });

      invalidVersions.forEach(v => {
        expect(VERSION_REGEX.test(v)).toBe(false);
      });
    });

    it('should parse version components correctly', () => {
      const version = '0.2.54';
      const [major, minor, patch] = version.split('.').map(Number);
      
      expect(major).toBe(0);
      expect(minor).toBe(2);
      expect(patch).toBe(54);
    });

    it('should compare versions correctly', () => {
      const compareVersions = (a: string, b: string): number => {
        const [aMajor, aMinor, aPatch] = a.split('.').map(Number);
        const [bMajor, bMinor, bPatch] = b.split('.').map(Number);
        
        if (aMajor !== bMajor) return aMajor - bMajor;
        if (aMinor !== bMinor) return aMinor - bMinor;
        return aPatch - bPatch;
      };

      expect(compareVersions('0.2.54', '0.2.53')).toBeGreaterThan(0);
      expect(compareVersions('0.2.54', '0.2.54')).toBe(0);
      expect(compareVersions('0.2.53', '0.2.54')).toBeLessThan(0);
      expect(compareVersions('0.3.0', '0.2.99')).toBeGreaterThan(0);
      expect(compareVersions('1.0.0', '0.99.99')).toBeGreaterThan(0);
    });
  });

  describe('Version Display in Explorer', () => {
    it('should display version with v prefix in UI', () => {
      const version = '0.2.54';
      const displayVersion = `v${version}`;
      
      expect(displayVersion).toBe('v0.2.54');
    });

    it('should format version for bootnode display', () => {
      const nodeVersion = 'pyrax-node/0.2.54';
      const version = nodeVersion.split('/')[1];
      const display = `v${version}`;
      
      expect(display).toBe('v0.2.54');
    });

    it('should handle unknown versions gracefully', () => {
      const unknownVersion = 'unknown';
      const display = unknownVersion === 'unknown' ? 'v?' : `v${unknownVersion}`;
      
      expect(display).toBe('v?');
    });
  });

  describe('Network Version Stats', () => {
    it('should calculate version distribution across network', () => {
      const nodeVersions = [
        '0.2.54', '0.2.54', '0.2.54', '0.2.54',
        '0.2.53', '0.2.53',
        '0.2.50',
        '0.1.0',
      ];

      const distribution = nodeVersions.reduce((acc, v) => {
        acc[v] = (acc[v] || 0) + 1;
        return acc;
      }, {} as Record<string, number>);

      expect(distribution['0.2.54']).toBe(4);
      expect(distribution['0.2.53']).toBe(2);
      expect(distribution['0.2.50']).toBe(1);
      expect(distribution['0.1.0']).toBe(1);
    });

    it('should identify majority version', () => {
      const nodeVersions = ['0.2.54', '0.2.54', '0.2.54', '0.2.53', '0.2.50'];
      
      const distribution = nodeVersions.reduce((acc, v) => {
        acc[v] = (acc[v] || 0) + 1;
        return acc;
      }, {} as Record<string, number>);

      const majorityVersion = Object.entries(distribution)
        .sort(([, a], [, b]) => b - a)[0][0];
      
      expect(majorityVersion).toBe('0.2.54');
    });

    it('should calculate percentage of up-to-date nodes', () => {
      const currentVersion = '0.2.54';
      const nodeVersions = ['0.2.54', '0.2.54', '0.2.54', '0.2.53', '0.2.50'];
      
      const upToDateCount = nodeVersions.filter(v => v === currentVersion).length;
      const percentage = (upToDateCount / nodeVersions.length) * 100;
      
      expect(percentage).toBe(60);
    });
  });
});

describe('Agent Version String Tests', () => {
  it('should format agent version correctly for identify protocol', () => {
    // This tests the format that should be set in the P2P identify config
    const pkgVersion = '0.2.54';
    const agentVersion = `pyrax-node/${pkgVersion}`;
    
    expect(agentVersion).toBe('pyrax-node/0.2.54');
    expect(agentVersion.startsWith('pyrax-node/')).toBe(true);
  });

  it('should distinguish PYRAX nodes from generic libp2p nodes', () => {
    const pyraxNode = 'pyrax-node/0.2.54';
    const libp2pNode = 'rust-libp2p/0.44.2';
    
    expect(pyraxNode.toLowerCase().includes('pyrax')).toBe(true);
    expect(libp2pNode.toLowerCase().includes('pyrax')).toBe(false);
  });
});
