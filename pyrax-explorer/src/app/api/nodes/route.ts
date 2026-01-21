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
  latency: number; // latency in ms to bootnode
  isBootnode?: boolean; // true if this is a bootnode
  online?: boolean; // connection status
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
  averageLatency: number; // average latency across all nodes in ms
}

// RPC endpoints for all 3 streams
// Node runs via systemd on host - use host.docker.internal (configured via extra_hosts in docker-compose)
// In dev: use direct bootnode IP rpc.pyrax-devnet.org (209.38.137.105:28545)
// Currently: All streams use the same node (single node deployment)
const STREAM_ENDPOINTS = {
  A: process.env.STREAM_A_RPC || 'http://host.docker.internal:28545',
  B: process.env.STREAM_B_RPC || 'http://host.docker.internal:28545',
  C: process.env.STREAM_C_RPC || 'http://host.docker.internal:28545',
};

// Bootnode configurations - these are always shown regardless of P2P connections
const BOOTNODES = [
  {
    id: 'bootnode-nyc-1',
    ip: '209.38.137.105',
    port: 30303,
    rpcPort: 28545,
    country: 'United States',
    countryCode: 'US',
    city: 'New York',
    lat: 40.7128,
    lon: -74.0060,
    stream: 'A' as const,
    isBootnode: true,
  },
  {
    id: 'bootnode-sfo-2',
    ip: '137.184.118.228',
    port: 30303,
    rpcPort: 28545,
    country: 'United States',
    countryCode: 'US',
    city: 'San Francisco',
    lat: 37.7749,
    lon: -122.4194,
    stream: 'A' as const,
    isBootnode: true,
  },
];

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

  // Handle relay-connected peers (behind NAT, no direct IP)
  // Give them approximate random locations near bootnodes so they show on the map
  if (ip === 'relay-connected') {
    // Distribute relay peers across different regions for visual representation
    // Use a hash of the cache size to get consistent but varied positions
    const regionIndex = geoCache.size % 4;
    const regions = [
      { country: 'United States', countryCode: 'US', city: 'East Coast (Relay)', lat: 40.7128 + (Math.random() - 0.5) * 8, lon: -74.0060 + (Math.random() - 0.5) * 8 },
      { country: 'United States', countryCode: 'US', city: 'West Coast (Relay)', lat: 37.7749 + (Math.random() - 0.5) * 8, lon: -122.4194 + (Math.random() - 0.5) * 8 },
      { country: 'Europe', countryCode: 'EU', city: 'Europe (Relay)', lat: 51.5074 + (Math.random() - 0.5) * 10, lon: -0.1278 + (Math.random() - 0.5) * 10 },
      { country: 'Asia', countryCode: 'AS', city: 'Asia (Relay)', lat: 35.6762 + (Math.random() - 0.5) * 10, lon: 139.6503 + (Math.random() - 0.5) * 10 },
    ];
    const relayGeo: GeoLocation = regions[regionIndex];
    return relayGeo;
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
// For relay addresses, we need special handling since the first IP is the relay server, not the peer
function extractIP(address: string): string {
  // RELAY FIX: For relay addresses (/p2p-circuit/), the first IP is the relay server, not the peer
  // These peers don't have a direct IP we can geolocate - return empty to avoid filtering issues
  if (address.includes('/p2p-circuit/') || address.includes('/p2p-circuit')) {
    // For relay connections, try to find an IP after the circuit marker
    // Format: /ip4/RELAY_IP/tcp/PORT/p2p/RELAY_ID/p2p-circuit/p2p/PEER_ID
    // The peer's actual IP is not in the address - they're behind NAT
    // Return a marker so we don't filter them as bootnodes
    return 'relay-connected';
  }

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

// Measure latency to an endpoint
async function measureLatency(endpoint: string): Promise<number> {
  try {
    const start = performance.now();
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'pyrax_blockNumber',
        params: [],
        id: 1,
      }),
      signal: AbortSignal.timeout(5000),
    });
    if (response.ok) {
      await response.json();
      return Math.round(performance.now() - start);
    }
  } catch {
    // Latency measurement failed
  }
  return -1; // -1 indicates failed measurement
}

// Mesh connection from RPC
interface MeshConnection {
  peer_a: string;
  peer_b: string;
  topic: string;
  connection_type: string; // "mesh", "gossip", "direct"
}

// Fetch peers from a specific stream endpoint
async function fetchStreamPeers(endpoint: string, stream: 'A' | 'B' | 'C'): Promise<{ 
  peers: any[]; 
  localPeerId: string; 
  listenAddresses: string[]; 
  latency: number;
  meshConnections: MeshConnection[];
}> {
  try {
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'pyrax_getNetworkInfo',
        params: [],
        id: 1,
      }),
      signal: AbortSignal.timeout(5000),
    });

    if (!response.ok) {
      return { peers: [], localPeerId: '', listenAddresses: [], latency: -1, meshConnections: [] };
    }

    const data = await response.json();
    if (data.error) {
      return { peers: [], localPeerId: '', listenAddresses: [], latency: -1, meshConnections: [] };
    }

    const networkInfo = data.result;
    // Tag each peer with the stream it came from
    const peers = (networkInfo?.peers || []).map((p: any) => ({ ...p, _stream: stream }));
    
    // CAMELCASE FIX: Support both camelCase (new) and snake_case (legacy) field names
    // This ensures compatibility during the transition period
    const meshConnections: MeshConnection[] = networkInfo?.meshConnections || networkInfo?.mesh_connections || [];
    
    // Measure latency to this endpoint
    const latency = await measureLatency(endpoint);
    
    return {
      peers,
      localPeerId: networkInfo?.localPeerId || networkInfo?.local_peer_id || '',
      listenAddresses: networkInfo?.listenAddresses || networkInfo?.listen_addresses || [],
      latency,
      meshConnections,
    };
  } catch {
    return { peers: [], localPeerId: '', listenAddresses: [], latency: -1, meshConnections: [] };
  }
}

// Check if bootnode RPC is online
async function checkBootnodeStatus(rpcUrl: string): Promise<{ online: boolean; latency: number; blockHeight: number; version: string }> {
  try {
    const start = performance.now();
    const response = await fetch(rpcUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'pyrax_getChainInfo',
        params: [],
        id: 1,
      }),
      signal: AbortSignal.timeout(5000),
    });
    
    if (response.ok) {
      const data = await response.json();
      const latency = Math.round(performance.now() - start);
      // CAMELCASE FIX: Support both camelCase (new) and snake_case (legacy) field names
      const fullVersion = data.result?.nodeVersion || data.result?.node_version || '';
      const version = fullVersion.includes('/') 
        ? fullVersion.split('/')[1] 
        : (fullVersion || '0.1.0');
      return {
        online: true,
        latency,
        blockHeight: data.result?.bestBlockHeight || data.result?.best_block_height || data.result?.block_height || 0,
        version,
      };
    }
  } catch {
    // Bootnode offline
  }
  return { online: false, latency: -1, blockHeight: 0, version: 'offline' };
}

export async function GET() {
  try {
    // Fetch peers from all 3 streams in parallel
    const [streamA, streamC] = await Promise.all([
      fetchStreamPeers(STREAM_ENDPOINTS.A, 'A'),
      fetchStreamPeers(STREAM_ENDPOINTS.C, 'C'),
    ]);

    // Combine all peers, using _stream tag or detecting from endpoint
    const allPeers = [...streamA.peers, ...streamC.peers];
    
    // Get bootnode IPs to filter them out from peer list (avoid duplicates)
    const bootnodeIPs = new Set(BOOTNODES.map(bn => bn.ip));
    
    // Deduplicate peers by peer_id and filter out bootnode IPs
    const seenPeerIds = new Set<string>();
    const uniquePeers = allPeers.filter(peer => {
      const id = peer.peer_id || peer.id;
      if (seenPeerIds.has(id)) return false;
      seenPeerIds.add(id);
      
      // Filter out peers that are actually bootnodes (by IP)
      const peerIP = extractIP(peer.address || peer.ip || '');
      if (bootnodeIPs.has(peerIP)) return false;
      
      return true;
    });

    // Process peers and get geolocation for each
    const peerNodes: ConnectedNode[] = await Promise.all(
      uniquePeers.map(async (peer: any, idx: number) => {
        const ip = extractIP(peer.address || peer.ip || '');
        const geo = await getGeoLocation(ip);
        
        // Use the stream tag we added, or determine from port
        let stream: 'A' | 'B' | 'C' = peer._stream || 'A';
        const port = peer.port || 30303;
        if (!peer._stream) {
          if (peer.protocol?.includes('stratum') || port === 3333) stream = 'B';
          else if (peer.protocol?.includes('staking') || port === 28547) stream = 'C';
        }

        // Get latency based on which stream this peer belongs to
        const peerLatency = stream === 'A' ? streamA.latency : stream === 'C' ? streamC.latency : -1;
        // Simulate per-peer latency variance (±20% of base latency)
        const variance = peerLatency > 0 ? Math.round(peerLatency * (0.8 + Math.random() * 0.4)) : -1;
        
        // VERSION FIX: Extract version number from agent string (e.g., "pyrax-node/0.2.54" -> "0.2.54")
        const rawVersion = peer.version || '';
        const version = rawVersion.includes('/') 
          ? rawVersion.split('/')[1] 
          : (rawVersion || 'unknown');
        
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
          // CAMELCASE FIX: Support both camelCase (new) and snake_case (legacy) field names
          connectedAt: Date.now() - (peer.connectedSecs || peer.connected_secs || 0) * 1000,
          lastSeen: peer.lastSeen || peer.last_seen || Date.now(),
          version,
          blockHeight: peer.blockHeight || peer.block_height || 0,
          latency: variance,
          isBootnode: false,
          online: true,
        };
      })
    );

    // Add bootnodes to the list (check their status)
    const bootnodeNodes: ConnectedNode[] = await Promise.all(
      BOOTNODES.map(async (bn) => {
        const rpcUrl = `http://${bn.ip}:${bn.rpcPort}`;
        const status = await checkBootnodeStatus(rpcUrl);
        
        return {
          id: bn.id,
          peerId: bn.id,
          ip: bn.ip,
          port: bn.port,
          country: bn.country,
          countryCode: bn.countryCode,
          city: bn.city,
          lat: bn.lat,
          lon: bn.lon,
          stream: bn.stream,
          connectedAt: Date.now() - 86400000, // Bootnodes are always "connected"
          lastSeen: Date.now(),
          version: status.version,
          blockHeight: status.blockHeight,
          latency: status.latency,
          isBootnode: true,
          online: status.online,
        };
      })
    );

    // Combine bootnodes first, then peer nodes (bootnodes at top)
    const nodes = [...bootnodeNodes, ...peerNodes];

    // Calculate average latency
    const validLatencies = nodes.filter(n => n.latency > 0).map(n => n.latency);
    const averageLatency = validLatencies.length > 0 
      ? Math.round(validLatencies.reduce((a, b) => a + b, 0) / validLatencies.length)
      : 0;

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
      averageLatency,
    };

    // Generate connections between nodes (bootnodes connect to all peers via relay or direct)
    const connections: Array<{ from: string; to: string; fromCoords: [number, number]; toCoords: [number, number]; isRelay?: boolean; isMesh?: boolean; connectionType?: string }> = [];
    
    // Each online peer is connected to at least one bootnode (via relay or direct)
    const onlineBootnodes = bootnodeNodes.filter(bn => bn.online);
    const onlinePeers = peerNodes.filter(p => p.online && (p.lat !== 0 || p.lon !== 0));
    
    // Connect bootnodes to each other (direct connections)
    for (let i = 0; i < onlineBootnodes.length; i++) {
      for (let j = i + 1; j < onlineBootnodes.length; j++) {
        const bn1 = onlineBootnodes[i];
        const bn2 = onlineBootnodes[j];
        connections.push({
          from: bn1.id,
          to: bn2.id,
          fromCoords: [bn1.lon, bn1.lat],
          toCoords: [bn2.lon, bn2.lat],
          isRelay: false, // Bootnode-to-bootnode is always direct
        });
      }
    }
    
    // Connect peers to their nearest bootnode (may be relay or direct)
    // Most home users are behind NAT and connect via relay
    for (const peer of onlinePeers) {
      if (onlineBootnodes.length > 0) {
        // Check if peer address indicates relay connection
        const peerData = uniquePeers.find((p: any) => (p.peer_id || p.id) === peer.id);
        const isRelayConnection = peerData?.address?.includes('/p2p-circuit/') ?? true; // Assume relay if unknown
        
        // Find nearest bootnode by simple distance
        let nearestBn = onlineBootnodes[0];
        let minDist = Math.abs(peer.lat - nearestBn.lat) + Math.abs(peer.lon - nearestBn.lon);
        for (const bn of onlineBootnodes) {
          const dist = Math.abs(peer.lat - bn.lat) + Math.abs(peer.lon - bn.lon);
          if (dist < minDist) {
            minDist = dist;
            nearestBn = bn;
          }
        }
        connections.push({
          from: nearestBn.id,
          to: peer.id,
          fromCoords: [nearestBn.lon, nearestBn.lat],
          toCoords: [peer.lon, peer.lat],
          isRelay: isRelayConnection,
        });
      }
    }
    
    // USER-TO-USER CONNECTIONS inferred from shared mesh topics
    // Bootnodes report mesh_connections as bootnode->user, so we infer user-to-user
    // by finding users that share the same mesh topic on the same bootnode
    const allMeshConnections = [...streamA.meshConnections, ...streamC.meshConnections];
    const peerIdToNode = new Map<string, ConnectedNode>();
    
    // Build lookup map of peer IDs to nodes (for coordinate lookup)
    for (const node of [...bootnodeNodes, ...peerNodes]) {
      peerIdToNode.set(node.peerId, node);
    }
    
    // Group user peers by topic to infer user-to-user connections
    // If users A and B are both in mesh for topic "blocks", they can communicate
    const topicToUsers = new Map<string, string[]>();
    
    for (const meshConn of allMeshConnections) {
      // peer_a is bootnode, peer_b is user in mesh for this topic
      const userPeer = peerIdToNode.get(meshConn.peer_b);
      if (userPeer && !userPeer.isBootnode && meshConn.connection_type === 'mesh') {
        const topic = meshConn.topic;
        if (!topicToUsers.has(topic)) {
          topicToUsers.set(topic, []);
        }
        const users = topicToUsers.get(topic)!;
        if (!users.includes(meshConn.peer_b)) {
          users.push(meshConn.peer_b);
        }
      }
    }
    
    // Track which user-to-user connections we've already added (avoid duplicates)
    const seenUserConnections = new Set<string>();
    
    // Create connections between all users sharing the same topic
    for (const [topic, userPeerIds] of topicToUsers) {
      // Connect users pairwise (limit to avoid O(n²) explosion with many users)
      const maxPairs = 20; // Limit visual clutter
      let pairCount = 0;
      
      for (let i = 0; i < userPeerIds.length && pairCount < maxPairs; i++) {
        for (let j = i + 1; j < userPeerIds.length && pairCount < maxPairs; j++) {
          const peerA = peerIdToNode.get(userPeerIds[i]);
          const peerB = peerIdToNode.get(userPeerIds[j]);
          
          if (!peerA || !peerB) continue;
          // Skip if no valid coordinates
          if ((peerA.lat === 0 && peerA.lon === 0) || (peerB.lat === 0 && peerB.lon === 0)) continue;
          
          // Create unique key for this connection (order-independent)
          const connKey = [userPeerIds[i], userPeerIds[j]].sort().join('-');
          if (seenUserConnections.has(connKey)) continue;
          seenUserConnections.add(connKey);
          
          connections.push({
            from: peerA.id,
            to: peerB.id,
            fromCoords: [peerA.lon, peerA.lat],
            toCoords: [peerB.lon, peerB.lat],
            isRelay: false,
            isMesh: true,
            connectionType: 'mesh',
          });
          pairCount++;
        }
      }
    }

    return NextResponse.json({ 
      nodes, 
      stats,
      connections,
      localPeerId: streamA.localPeerId || streamC.localPeerId || '',
      listenAddresses: [...streamA.listenAddresses, ...streamC.listenAddresses],
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
        averageLatency: 0,
      },
      connections: [],
      localPeerId: '',
      listenAddresses: [],
      error: String(error),
    });
  }
}
