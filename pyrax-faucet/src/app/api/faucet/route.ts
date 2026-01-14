import { NextRequest, NextResponse } from 'next/server';

// In-memory rate limiting (use Redis in production)
const requestCache = new Map<string, number>();

const FAUCET_AMOUNT = 100; // PYRAX tokens per request

// Simple address validation (20 bytes hex string)
function isValidAddress(address: string): boolean {
  if (!address) return false;
  const cleaned = address.toLowerCase().replace(/^0x/, '');
  return /^[0-9a-f]{40}$/.test(cleaned);
}
const COOLDOWN_MS = 60 * 60 * 1000; // 1 hour

// Network configuration
type NetworkConfig = {
  name: string;
  rpcUrl: string;
  chainId: number;
  explorerUrl: string;
  faucetAddress: string;
};

// PYRAX Chain IDs: Mainnet=79729, Testnet=797291, Devnet=797292
const NETWORKS: Record<string, NetworkConfig> = {
  devnet: {
    name: 'PYRAX Devnet',
    rpcUrl: process.env.DEVNET_RPC_URL || 'http://node:8545',
    chainId: 797292,
    explorerUrl: 'https://explorer.pyrax-devnet.org',
    faucetAddress: '0x1111111111111111111111111111111111111111', // Genesis faucet pool
  },
  testnet: {
    name: 'PYRAX Testnet',
    rpcUrl: process.env.TESTNET_RPC_URL || 'https://rpc.testnet.pyrax.org',
    chainId: 797291,
    explorerUrl: 'https://explorer.testnet.pyrax.org',
    faucetAddress: '0x1111111111111111111111111111111111111111', // Genesis faucet pool
  },
};

// Get current network from environment
function getNetwork(): NetworkConfig | null {
  const networkEnv = process.env.PYRAX_NETWORK || 'testnet';
  
  // Faucet is NOT available on mainnet
  if (networkEnv === 'mainnet') {
    return null;
  }
  
  return NETWORKS[networkEnv] || NETWORKS.testnet;
}

// PYRAX RPC helper for native transactions
async function pyraxRpcCall(rpcUrl: string, method: string, params: unknown[]): Promise<unknown> {
  const response = await fetch(rpcUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
      id: Date.now(),
    }),
  });
  const data = await response.json();
  if (data.error) {
    throw new Error(data.error.message || 'RPC error');
  }
  return data.result;
}

// Get faucet balance from PYRAX RPC
async function getFaucetBalance(rpcUrl: string, faucetAddress: string): Promise<bigint> {
  try {
    const result = await pyraxRpcCall(rpcUrl, 'pyrax_getBalance', [faucetAddress]) as { balance: number };
    return BigInt(result.balance || 0);
  } catch {
    return BigInt(0);
  }
}

export async function POST(request: NextRequest) {
  try {
    // Check if faucet is available on this network
    const network = getNetwork();
    if (!network) {
      return NextResponse.json(
        { error: 'Faucet is not available on mainnet' },
        { status: 403 }
      );
    }

    const { address } = await request.json();

    // Validate address
    if (!address) {
      return NextResponse.json(
        { error: 'Wallet address is required' },
        { status: 400 }
      );
    }

    // Check if valid PYRAX address format (20 bytes hex)
    if (!isValidAddress(address)) {
      return NextResponse.json(
        { error: 'Invalid wallet address format' },
        { status: 400 }
      );
    }

    // Normalize address
    const normalizedAddress = address.toLowerCase();

    // Check rate limit
    const lastRequest = requestCache.get(normalizedAddress);
    const now = Date.now();
    
    if (lastRequest && now - lastRequest < COOLDOWN_MS) {
      const remainingMs = COOLDOWN_MS - (now - lastRequest);
      const remainingMinutes = Math.ceil(remainingMs / (60 * 1000));
      return NextResponse.json(
        { error: `Rate limited. Please try again in ${remainingMinutes} minute(s).` },
        { status: 429 }
      );
    }

    // Check faucet balance using PYRAX RPC
    const faucetBalance = await getFaucetBalance(network.rpcUrl, network.faucetAddress);
    const amountUnits = BigInt(FAUCET_AMOUNT) * BigInt(100_000_000); // 8 decimals
    
    if (faucetBalance < amountUnits) {
      console.error(`Faucet balance too low: ${faucetBalance} units`);
      return NextResponse.json(
        { error: 'Faucet is temporarily out of funds. Please try again later.' },
        { status: 503 }
      );
    }

    // Send transaction using PYRAX native RPC (from genesis faucet pool)
    // Uses pyrax_createTestTransaction which spends from faucet address to recipient
    const result = await pyraxRpcCall(network.rpcUrl, 'pyrax_createTestTransaction', [
      network.faucetAddress,  // from: faucet pool address
      normalizedAddress,      // to: recipient address
      Number(amountUnits),    // amount in smallest units
    ]) as { accepted: boolean; hash?: string; error?: string };

    if (!result.accepted) {
      console.error(`Faucet transaction failed: ${result.error}`);
      return NextResponse.json(
        { error: result.error || 'Transaction rejected' },
        { status: 500 }
      );
    }

    // Update rate limit cache
    requestCache.set(normalizedAddress, now);

    // Log the request
    console.log(`Faucet: Sent ${FAUCET_AMOUNT} PYRAX to ${address} - tx: ${result.hash}`);

    return NextResponse.json({
      success: true,
      message: `Successfully sent ${FAUCET_AMOUNT} PYRAX to ${address.slice(0, 6)}...${address.slice(-4)}`,
      amount: FAUCET_AMOUNT,
      txHash: result.hash,
      address,
      network: network.name,
      explorerUrl: `${network.explorerUrl}/tx/${result.hash}`,
    });

  } catch (error) {
    console.error('Faucet error:', error);
    
    // Handle specific errors
    if (error instanceof Error) {
      if (error.message.includes('insufficient funds')) {
        return NextResponse.json(
          { error: 'Faucet is temporarily out of funds' },
          { status: 503 }
        );
      }
      if (error.message.includes('network') || error.message.includes('connect')) {
        return NextResponse.json(
          { error: 'Unable to connect to PYRAX network' },
          { status: 503 }
        );
      }
    }
    
    return NextResponse.json(
      { error: 'Failed to send tokens. Please try again.' },
      { status: 500 }
    );
  }
}

export async function GET() {
  const network = getNetwork();
  
  if (!network) {
    return NextResponse.json({
      name: 'PYRAX Faucet',
      status: 'disabled',
      message: 'Faucet is not available on mainnet',
    });
  }

  // Try to get faucet balance using PYRAX RPC
  let balance = 'unknown';
  let balanceUnits = BigInt(0);
  try {
    balanceUnits = await getFaucetBalance(network.rpcUrl, network.faucetAddress);
    // Convert from smallest units (8 decimals) to PYRAX
    const pyraxBalance = Number(balanceUnits) / 100_000_000;
    balance = `${pyraxBalance.toLocaleString()} PYRAX`;
  } catch {
    // Ignore balance check errors
  }

  return NextResponse.json({
    name: network.name + ' Faucet',
    network: process.env.PYRAX_NETWORK || 'devnet',
    chainId: network.chainId,
    faucetAddress: network.faucetAddress,
    amount: FAUCET_AMOUNT,
    cooldown: '1 hour',
    status: 'online',
    balance,
  });
}
