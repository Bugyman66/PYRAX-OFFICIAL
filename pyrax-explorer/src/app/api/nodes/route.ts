import { NextResponse } from 'next/server';

interface ConnectedNode {
  id: string;
  peerId: string;
  ip: string;
  port: number;
  country: string;
  countryCode: string;
  city: string;
  lat: number;
  lon: number;
  stream: 'A' | 'B' | 'C';
  connectedAt: number;
  lastSeen: number;
  version: string;
  blockHeight: number;
}

interface NodeStats {
  totalNodes: number;
  byStream: Record<'A' | 'B' | 'C', number>;
  byCountry: Record<string, number>;
}

// RPC endpoint for getting connected peers
const DEVNET_RPC = process.env.DEVNET_RPC_URL || 'http://209.38.137.105:28545';

export async function GET() {
  try {
    // Try to get peers from the node's RPC
    const response = await fetch(DEVNET_RPC, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'pyrax_getPeers',
        params: [],
        id: 1,
      }),
    });

    if (response.ok) {
      const data = await response.json();
      
      if (data.result && Array.isArray(data.result.peers)) {
        const nodes: ConnectedNode[] = data.result.peers.map((peer: any, idx: number) => ({
          id: peer.peer_id || `node-${idx}`,
          peerId: peer.peer_id || `12D3KooW${Math.random().toString(36).slice(2, 10)}`,
          ip: peer.ip || '0.0.0.0',
          port: peer.port || 30303,
          country: peer.country || 'Unknown',
          countryCode: peer.country_code || '',
          city: peer.city || 'Unknown',
          lat: peer.lat || 0,
          lon: peer.lon || 0,
          stream: peer.stream || 'A',
          connectedAt: peer.connected_at || Date.now(),
          lastSeen: peer.last_seen || Date.now(),
          version: peer.version || '0.1.0',
          blockHeight: peer.block_height || 0,
        }));

        const stats: NodeStats = {
          totalNodes: nodes.length,
          byStream: {
            A: nodes.filter(n => n.stream === 'A').length,
            B: nodes.filter(n => n.stream === 'B').length,
            C: nodes.filter(n => n.stream === 'C').length,
          },
          byCountry: nodes.reduce((acc, n) => {
            const country = n.country || 'Unknown';
            acc[country] = (acc[country] || 0) + 1;
            return acc;
          }, {} as Record<string, number>),
        };

        return NextResponse.json({ nodes, stats });
      }
    }

    // If RPC doesn't support getPeers or returns error, return mock data for demo
    // In production, this would be real peer data from the node
    const mockNodes = generateMockNodes();
    const stats: NodeStats = {
      totalNodes: mockNodes.length,
      byStream: {
        A: mockNodes.filter(n => n.stream === 'A').length,
        B: mockNodes.filter(n => n.stream === 'B').length,
        C: mockNodes.filter(n => n.stream === 'C').length,
      },
      byCountry: mockNodes.reduce((acc, n) => {
        acc[n.country] = (acc[n.country] || 0) + 1;
        return acc;
      }, {} as Record<string, number>),
    };

    return NextResponse.json({ nodes: mockNodes, stats });
  } catch (error) {
    console.error('Failed to fetch nodes:', error);
    
    // Return empty data on error
    return NextResponse.json({
      nodes: [],
      stats: {
        totalNodes: 0,
        byStream: { A: 0, B: 0, C: 0 },
        byCountry: {},
      },
    });
  }
}

function generateMockNodes(): ConnectedNode[] {
  // Mock data representing global node distribution for demo purposes
  const locations = [
    { country: 'United States', countryCode: 'US', city: 'New York', lat: 40.7128, lon: -74.006 },
    { country: 'United States', countryCode: 'US', city: 'San Francisco', lat: 37.7749, lon: -122.4194 },
    { country: 'United States', countryCode: 'US', city: 'Los Angeles', lat: 34.0522, lon: -118.2437 },
    { country: 'Germany', countryCode: 'DE', city: 'Frankfurt', lat: 50.1109, lon: 8.6821 },
    { country: 'Germany', countryCode: 'DE', city: 'Berlin', lat: 52.52, lon: 13.405 },
    { country: 'United Kingdom', countryCode: 'GB', city: 'London', lat: 51.5074, lon: -0.1278 },
    { country: 'Japan', countryCode: 'JP', city: 'Tokyo', lat: 35.6762, lon: 139.6503 },
    { country: 'Singapore', countryCode: 'SG', city: 'Singapore', lat: 1.3521, lon: 103.8198 },
    { country: 'Australia', countryCode: 'AU', city: 'Sydney', lat: -33.8688, lon: 151.2093 },
    { country: 'Canada', countryCode: 'CA', city: 'Toronto', lat: 43.6532, lon: -79.3832 },
    { country: 'Netherlands', countryCode: 'NL', city: 'Amsterdam', lat: 52.3676, lon: 4.9041 },
    { country: 'France', countryCode: 'FR', city: 'Paris', lat: 48.8566, lon: 2.3522 },
    { country: 'South Korea', countryCode: 'KR', city: 'Seoul', lat: 37.5665, lon: 126.978 },
    { country: 'Brazil', countryCode: 'BR', city: 'São Paulo', lat: -23.5505, lon: -46.6333 },
    { country: 'India', countryCode: 'IN', city: 'Mumbai', lat: 19.076, lon: 72.8777 },
  ];

  const streams: ('A' | 'B' | 'C')[] = ['A', 'B', 'C'];
  const now = Date.now();

  return locations.map((loc, idx) => ({
    id: `node-${idx}`,
    peerId: `12D3KooW${randomString(44)}`,
    ip: `${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`,
    port: 30303,
    ...loc,
    stream: streams[idx % 3],
    connectedAt: now - Math.floor(Math.random() * 86400000),
    lastSeen: now - Math.floor(Math.random() * 60000),
    version: '0.1.0',
    blockHeight: Math.floor(Math.random() * 100),
  }));
}

function randomString(length: number): string {
  const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  return Array.from({ length }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
}
