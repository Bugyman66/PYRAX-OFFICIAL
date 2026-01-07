import { NextRequest, NextResponse } from 'next/server';
import { ethers } from 'ethers';

// In-memory rate limiting (use Redis in production)
const requestCache = new Map<string, number>();

const FAUCET_AMOUNT = 100; // PYRAX tokens per request
const COOLDOWN_MS = 60 * 60 * 1000; // 1 hour

// Network configuration
type NetworkConfig = {
  name: string;
  rpcUrl: string;
  chainId: number;
  explorerUrl: string;
};

const NETWORKS: Record<string, NetworkConfig> = {
  devnet: {
    name: 'PYRAX Devnet',
    rpcUrl: process.env.DEVNET_RPC_URL || 'http://localhost:8545',
    chainId: 13370,
    explorerUrl: 'https://explorer.dev.pyrax.org',
  },
  testnet: {
    name: 'PYRAX Testnet',
    rpcUrl: process.env.TESTNET_RPC_URL || 'https://rpc.testnet.pyrax.org',
    chainId: 13371,
    explorerUrl: 'https://explorer.testnet.pyrax.org',
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

// Get faucet wallet from environment
function getFaucetWallet(provider: ethers.Provider): ethers.Wallet | null {
  const privateKey = process.env.FAUCET_PRIVATE_KEY;
  if (!privateKey) {
    console.error('FAUCET_PRIVATE_KEY not configured');
    return null;
  }
  return new ethers.Wallet(privateKey, provider);
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

    // Check if valid Ethereum address format
    if (!ethers.isAddress(address)) {
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

    // Connect to PYRAX network
    const provider = new ethers.JsonRpcProvider(network.rpcUrl, {
      name: network.name,
      chainId: network.chainId,
    });

    // Get faucet wallet
    const faucetWallet = getFaucetWallet(provider);
    if (!faucetWallet) {
      return NextResponse.json(
        { error: 'Faucet wallet not configured' },
        { status: 500 }
      );
    }

    // Check faucet balance
    const faucetBalance = await provider.getBalance(faucetWallet.address);
    const amountWei = ethers.parseEther(FAUCET_AMOUNT.toString());
    
    if (faucetBalance < amountWei) {
      console.error(`Faucet balance too low: ${ethers.formatEther(faucetBalance)} PYRAX`);
      return NextResponse.json(
        { error: 'Faucet is temporarily out of funds. Please try again later.' },
        { status: 503 }
      );
    }

    // Send transaction
    const tx = await faucetWallet.sendTransaction({
      to: address,
      value: amountWei,
    });

    // Wait for transaction to be mined
    const receipt = await tx.wait();

    // Update rate limit cache
    requestCache.set(normalizedAddress, now);

    // Log the request
    console.log(`Faucet: Sent ${FAUCET_AMOUNT} PYRAX to ${address} - tx: ${tx.hash}`);

    return NextResponse.json({
      success: true,
      message: `Successfully sent ${FAUCET_AMOUNT} PYRAX to ${address.slice(0, 6)}...${address.slice(-4)}`,
      amount: FAUCET_AMOUNT,
      txHash: tx.hash,
      blockNumber: receipt?.blockNumber,
      address,
      network: network.name,
      explorerUrl: `${network.explorerUrl}/tx/${tx.hash}`,
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

  // Try to get faucet balance
  let balance = 'unknown';
  try {
    const provider = new ethers.JsonRpcProvider(network.rpcUrl, {
      name: network.name,
      chainId: network.chainId,
    });
    const privateKey = process.env.FAUCET_PRIVATE_KEY;
    if (privateKey) {
      const wallet = new ethers.Wallet(privateKey, provider);
      const balanceWei = await provider.getBalance(wallet.address);
      balance = `${ethers.formatEther(balanceWei)} PYRAX`;
    }
  } catch {
    // Ignore balance check errors
  }

  return NextResponse.json({
    name: network.name + ' Faucet',
    network: process.env.PYRAX_NETWORK || 'testnet',
    amount: FAUCET_AMOUNT,
    cooldown: '1 hour',
    status: 'online',
    balance,
  });
}
