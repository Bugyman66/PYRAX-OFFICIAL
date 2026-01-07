// PYRAX Explorer Environment Configuration
// Automatically selects endpoints based on NEXT_PUBLIC_NETWORK

export type NetworkType = 'devnet' | 'testnet' | 'mainnet';

interface NetworkConfig {
  name: string;
  chainId: number;
  rpcUrl: string;
  wsUrl: string;
  apiUrl: string;
  explorerUrl: string;
  faucetUrl?: string;
}

const NETWORK_CONFIGS: Record<NetworkType, NetworkConfig> = {
  devnet: {
    name: 'Devnet',
    chainId: 28545,
    rpcUrl: process.env.NEXT_PUBLIC_RPC_URL || 'https://rpc.dev.pyrax.org',
    wsUrl: process.env.NEXT_PUBLIC_WS_URL || 'wss://rpc.dev.pyrax.org/ws',
    apiUrl: process.env.NEXT_PUBLIC_API_URL || 'https://api.dev.pyrax.org',
    explorerUrl: process.env.NEXT_PUBLIC_EXPLORER_URL || 'https://explorer.dev.pyrax.org',
    faucetUrl: 'https://faucet.dev.pyrax.org',
  },
  testnet: {
    name: 'Testnet',
    chainId: 18545,
    rpcUrl: process.env.NEXT_PUBLIC_RPC_URL || 'https://rpc.testnet.pyrax.org',
    wsUrl: process.env.NEXT_PUBLIC_WS_URL || 'wss://rpc.testnet.pyrax.org/ws',
    apiUrl: process.env.NEXT_PUBLIC_API_URL || 'https://api.testnet.pyrax.org',
    explorerUrl: process.env.NEXT_PUBLIC_EXPLORER_URL || 'https://explorer.testnet.pyrax.org',
    faucetUrl: 'https://faucet.testnet.pyrax.org',
  },
  mainnet: {
    name: 'Mainnet',
    chainId: 8545,
    rpcUrl: process.env.NEXT_PUBLIC_RPC_URL || 'https://rpc.pyrax.org',
    wsUrl: process.env.NEXT_PUBLIC_WS_URL || 'wss://rpc.pyrax.org/ws',
    apiUrl: process.env.NEXT_PUBLIC_API_URL || 'https://api.pyrax.org',
    explorerUrl: process.env.NEXT_PUBLIC_EXPLORER_URL || 'https://explorer.pyrax.org',
  },
};

// For local development, fall back to localhost
const LOCAL_CONFIG: NetworkConfig = {
  name: 'Local',
  chainId: 28545,
  rpcUrl: 'http://localhost:28545',
  wsUrl: 'ws://localhost:28546',
  apiUrl: 'http://localhost:8080',
  explorerUrl: 'http://localhost:3000',
  faucetUrl: 'http://localhost:8081',
};

export function getNetworkType(): NetworkType {
  const env = process.env.NEXT_PUBLIC_NETWORK as NetworkType;
  if (env && NETWORK_CONFIGS[env]) {
    return env;
  }
  return 'devnet'; // Default to devnet
}

export function getNetworkConfig(): NetworkConfig {
  const networkType = getNetworkType();
  
  // In development, check if we should use local config
  if (process.env.NODE_ENV === 'development' && !process.env.NEXT_PUBLIC_NETWORK) {
    return LOCAL_CONFIG;
  }
  
  return NETWORK_CONFIGS[networkType];
}

export function isMainnet(): boolean {
  return getNetworkType() === 'mainnet';
}

export function isTestnet(): boolean {
  return getNetworkType() === 'testnet';
}

export function isDevnet(): boolean {
  return getNetworkType() === 'devnet';
}

// Export all configs for network switcher
export function getAllNetworks(): NetworkConfig[] {
  return Object.values(NETWORK_CONFIGS);
}

export const config = getNetworkConfig();
