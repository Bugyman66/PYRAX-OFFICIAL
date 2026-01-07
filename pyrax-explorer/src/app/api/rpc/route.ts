import { NextRequest, NextResponse } from 'next/server';

const RPC_ENDPOINTS: Record<string, string> = {
  mainnet: 'http://127.0.0.1:8545',
  testnet: 'http://127.0.0.1:18545',
  devnet: 'http://127.0.0.1:28545',
};

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const network = request.headers.get('x-network') || 'devnet';
    const streamId = request.headers.get('x-stream') || 'A';
    
    // Determine endpoint based on network and stream
    let endpoint = RPC_ENDPOINTS[network] || RPC_ENDPOINTS.devnet;
    
    // Adjust port for different streams (A=x545, B=x546, C=x547)
    if (streamId === 'B') {
      endpoint = endpoint.replace(/545$/, '546');
    } else if (streamId === 'C') {
      endpoint = endpoint.replace(/545$/, '547');
    }

    const startTime = Date.now();
    
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });

    const latency = Date.now() - startTime;

    if (!response.ok) {
      return NextResponse.json(
        { error: `Node returned ${response.status}` },
        { status: response.status }
      );
    }

    const data = await response.json();
    
    // Add latency info to response
    return NextResponse.json({
      ...data,
      _latency: latency,
    });
  } catch (error) {
    console.error('RPC proxy error:', error);
    return NextResponse.json(
      { error: 'Failed to connect to node', details: String(error) },
      { status: 503 }
    );
  }
}
