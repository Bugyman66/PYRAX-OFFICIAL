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

interface GeoLocation {
  country: string;
  countryCode: string;
  city: string;
  lat: number;
  lon: number;
}

interface NodeStats {
  totalNodes: number;
  byStream: Record<'A' | 'B' | 'C', number>;
  byCountry: Record<string, number>;
}

// RPC endpoint for getting connected peers
const DEVNET_RPC = process.env.DEVNET_RPC_URL || 'http://209.38.137.105:28545';

// Cache for IP geolocation to avoid repeated API calls
const geoCache = new Map<string, GeoLocation>();

// Get geolocation for an IP address using ip-api.com (free, no API key required)
async function getGeoLocation(ip: string): Promise<GeoLocation> {
  // Check cache first
  if (geoCache.has(ip)) {
    return geoCache.get(ip)!;
  }

  // Skip private/local IPs
  if (ip.startsWith('10.') || ip.startsWith('172.') || ip.startsWith('192.168.') || 
      ip.startsWith('127.') || ip === '0.0.0.0') {
    const defaultGeo: GeoLocation = {
      country: 'Local Network',
      countryCode: 'XX',
      city: 'Private',
      lat: 0,
      lon: 0,
    };
    geoCache.set(ip, defaultGeo);
    return defaultGeo;
  }

  try {
    // ip-api.com is free for non-commercial use, no API key needed
    const response = await fetch(`http://ip-api.com/json/${ip}?fields=status,country,countryCode,city,lat,lon`, {
      signal: AbortSignal.timeout(3000),
    });

    if (response.ok) {
      const data = await response.json();
      if (data.status === 'success') {
        const geo: GeoLocation = {
          country: data.country || 'Unknown',
          countryCode: data.countryCode || '',
          city: data.city || 'Unknown',
          lat: data.lat || 0,
          lon: data.lon || 0,
        };
        geoCache.set(ip, geo);
        return geo;
      }
    }
  } catch (error) {
    console.error(`Failed to get geolocation for ${ip}:`, error);
  }

  // Default fallback
  const fallback: GeoLocation = {
    country: 'Unknown',
    countryCode: '',
    city: 'Unknown',
    lat: 0,
    lon: 0,
  };
  geoCache.set(ip, fallback);
  return fallback;
}

// Extract IP from multiaddr or address string
function extractIP(address: string): string {
  // Handle multiaddr format: /ip4/1.2.3.4/tcp/30303
  const ipv4Match = address.match(/\/ip4\/([^/]+)/);
  if (ipv4Match) return ipv4Match[1];

  // Handle standard IP:port format
  const colonMatch = address.match(/^([^:]+):/);
  if (colonMatch) return colonMatch[1];

  // Handle just IP
  if (/^\d+\.\d+\.\d+\.\d+$/.test(address)) return address;

  return address;
}

export async function GET() {
  try {
    // Get network info from the node's RPC
    const response = await fetch(DEVNET_RPC, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'pyrax_getNetworkInfo',
        params: [],
        id: 1,
      }),
      signal: AbortSignal.timeout(10000),
    });

    if (!response.ok) {
      throw new Error(`RPC returned ${response.status}`);
    }

    const data = await response.json();
    
    if (data.error) {
      throw new Error(data.error.message || 'RPC error');
    }

    const networkInfo = data.result;
    const peers = networkInfo?.peers || [];

    // Process peers and get geolocation for each
    const nodes: ConnectedNode[] = await Promise.all(
      peers.map(async (peer: any, idx: number) => {
        const ip = extractIP(peer.address || peer.ip || '');
        const geo = await getGeoLocation(ip);
        
        // Determine stream based on port or protocol
        let stream: 'A' | 'B' | 'C' = 'A';
        const port = peer.port || 30303;
        if (peer.protocol?.includes('stratum') || port === 3333) stream = 'B';
        else if (peer.protocol?.includes('staking') || port === 28547) stream = 'C';

        return {
          id: peer.peer_id || `peer-${idx}`,
          peerId: peer.peer_id || '',
          ip,
          port,
          country: geo.country,
          countryCode: geo.countryCode,
          city: geo.city,
          lat: geo.lat,
          lon: geo.lon,
          stream,
          connectedAt: Date.now() - (peer.connected_secs || 0) * 1000,
          lastSeen: peer.last_seen || Date.now(),
          version: peer.version || '0.1.0',
          blockHeight: peer.block_height || 0,
        };
      })
    );

    // Calculate stats
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

    return NextResponse.json({ 
      nodes, 
      stats,
      localPeerId: networkInfo?.local_peer_id || '',
      listenAddresses: networkInfo?.listen_addresses || [],
    });
  } catch (error) {
    console.error('Failed to fetch nodes:', error);
    
    // Return empty data on error - no mocks
    return NextResponse.json({
      nodes: [],
      stats: {
        totalNodes: 0,
        byStream: { A: 0, B: 0, C: 0 },
        byCountry: {},
      },
      localPeerId: '',
      listenAddresses: [],
      error: String(error),
    });
  }
}
