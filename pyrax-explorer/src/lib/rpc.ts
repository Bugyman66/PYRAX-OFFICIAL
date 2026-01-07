// PYRAX Node RPC Client for Explorer
// Connects to pyrax-node JSON-RPC API

const RPC_ENDPOINTS = {
  mainnet: 'http://localhost:8545',
  testnet: 'http://localhost:18545',
  devnet: 'http://localhost:28545',
};

// Default to devnet for development
const DEFAULT_NETWORK = 'devnet';

interface RpcRequest {
  jsonrpc: string;
  method: string;
  params: unknown[];
  id: number;
}

interface RpcResponse<T> {
  jsonrpc: string;
  result?: T;
  error?: { code: number; message: string };
  id: number;
}

let requestId = 1;

export async function rpcCall<T>(method: string, params: unknown[] = [], network: string = DEFAULT_NETWORK): Promise<T | null> {
  const endpoint = RPC_ENDPOINTS[network as keyof typeof RPC_ENDPOINTS] || RPC_ENDPOINTS.devnet;
  
  const request: RpcRequest = {
    jsonrpc: '2.0',
    method,
    params,
    id: requestId++,
  };

  try {
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
      cache: 'no-store',
    });

    if (!response.ok) {
      console.error(`RPC request failed: ${response.status}`);
      return null;
    }

    const data: RpcResponse<T> = await response.json();
    
    if (data.error) {
      console.error(`RPC error: ${data.error.message}`);
      return null;
    }

    return data.result ?? null;
  } catch (error) {
    console.error('RPC call failed:', error);
    return null;
  }
}

// Chain Info Response from node
export interface ChainInfo {
  chain_id: number;
  network: string;
  best_block_height: number;
  best_block_hash: string;
  genesis_hash: string;
  difficulty: number;
  utxo_count: number;
  syncing: boolean;
}

// Block Response from node
export interface BlockInfo {
  height: number;
  hash: string;
  prev_hash: string;
  timestamp: number;
  difficulty: number;
  nonce: number;
  tx_count: number;
  size: number;
  miner: string;
  reward: number;
  transactions?: string[];
}

// Transaction Response from node
export interface TxInfo {
  txid: string;
  block_hash?: string;
  block_height?: number;
  timestamp?: number;
  inputs: Array<{
    prev_txid: string;
    prev_vout: number;
    value: number;
  }>;
  outputs: Array<{
    address: string;
    value: number;
    script: string;
  }>;
  fee: number;
  size: number;
}

// API Functions
export async function getChainInfo(network?: string): Promise<ChainInfo | null> {
  return rpcCall<ChainInfo>('pyrax_getChainInfo', [], network);
}

export async function getBlockByNumber(height: number, fullTxs: boolean = false, network?: string): Promise<BlockInfo | null> {
  return rpcCall<BlockInfo>('pyrax_getBlockByNumber', [height, fullTxs], network);
}

export async function getBlockByHash(hash: string, fullTxs: boolean = false, network?: string): Promise<BlockInfo | null> {
  return rpcCall<BlockInfo>('pyrax_getBlockByHash', [hash, fullTxs], network);
}

export async function getTransaction(txid: string, network?: string): Promise<TxInfo | null> {
  return rpcCall<TxInfo>('pyrax_getTransaction', [txid], network);
}

// Contract Response from node
export interface ContractInfo {
  address: string;
  creator: string;
  creation_tx: string;
  creation_block: number;
  creation_timestamp: number;
  bytecode_hash: string;
  contract_type: 'evm' | 'wasm';
  is_verified: boolean;
  name?: string;
  compiler_version?: string;
  optimization?: boolean;
  license?: string;
  balance: number;
  tx_count: number;
}

export interface ContractsResponse {
  contracts: ContractInfo[];
  pagination: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

export async function getContracts(
  contractType: 'evm' | 'wasm' = 'evm',
  page: number = 1,
  perPage: number = 25,
  network?: string
): Promise<ContractsResponse | null> {
  return rpcCall<ContractsResponse>('pyrax_getContracts', [contractType, page, perPage], network);
}

export async function getContract(address: string, network?: string): Promise<ContractInfo | null> {
  return rpcCall<ContractInfo>('pyrax_getContract', [address], network);
}

export async function getRecentBlocksFromNode(count: number = 10, network?: string): Promise<BlockInfo[]> {
  const chainInfo = await getChainInfo(network);
  if (!chainInfo || chainInfo.best_block_height === 0) {
    return [];
  }

  const blocks: BlockInfo[] = [];
  const startHeight = chainInfo.best_block_height;
  const endHeight = Math.max(0, startHeight - count + 1);

  for (let height = startHeight; height >= endHeight; height--) {
    const block = await getBlockByNumber(height, false, network);
    if (block) {
      blocks.push(block);
    }
  }

  return blocks;
}
