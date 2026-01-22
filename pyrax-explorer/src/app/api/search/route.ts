import { NextResponse } from 'next/server'
import { 
  getChainInfo, 
  getBlockByNumber, 
  getBlockByHash, 
  getTransaction, 
  getAddressBalance,
  getContract
} from '@/lib/rpc'

export const dynamic = 'force-dynamic'
export const revalidate = 0

interface SearchResult {
  type: 'block' | 'transaction' | 'address' | 'contract' | 'token'
  value: string
  label: string
  sublabel?: string
  exists: boolean
  data?: Record<string, unknown>
}

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const query = searchParams.get('q')?.trim() || ''
  
  if (!query) {
    return NextResponse.json({ results: [], query: '' })
  }

  const results: SearchResult[] = []

  try {
    // 1. Block number search (pure digits)
    if (/^\d+$/.test(query)) {
      const blockNumber = parseInt(query, 10)
      const block = await getBlockByNumber(blockNumber)
      
      if (block) {
        results.push({
          type: 'block',
          value: query,
          label: `Block #${blockNumber.toLocaleString()}`,
          sublabel: `${block.tx_count} transactions • ${new Date(block.timestamp * 1000).toLocaleString()}`,
          exists: true,
          data: {
            height: block.height,
            hash: block.hash,
            txCount: block.tx_count,
            timestamp: block.timestamp,
            miner: block.miner,
          }
        })
      } else {
        // Check if it's a future block
        const chainInfo = await getChainInfo()
        if (chainInfo && blockNumber > chainInfo.best_block_height) {
          results.push({
            type: 'block',
            value: query,
            label: `Block #${blockNumber.toLocaleString()}`,
            sublabel: `Not yet mined (current height: ${chainInfo.best_block_height.toLocaleString()})`,
            exists: false,
          })
        }
      }
    }

    // 2. Block hash search (0x + 64 hex chars - could be block hash)
    if (/^0x[a-fA-F0-9]{64}$/.test(query)) {
      // Try as block hash first
      const block = await getBlockByHash(query)
      if (block) {
        results.push({
          type: 'block',
          value: block.height.toString(),
          label: `Block #${block.height.toLocaleString()}`,
          sublabel: `Hash: ${query.slice(0, 10)}...${query.slice(-8)}`,
          exists: true,
          data: {
            height: block.height,
            hash: block.hash,
            txCount: block.tx_count,
          }
        })
      }

      // Try as transaction hash
      const tx = await getTransaction(query)
      if (tx) {
        const totalValue = tx.outputs?.reduce((sum, out) => sum + (out.value || 0), 0) || 0
        results.push({
          type: 'transaction',
          value: query,
          label: `${query.slice(0, 10)}...${query.slice(-8)}`,
          sublabel: `${(totalValue / 1e8).toFixed(8)} PYRAX • Block #${tx.block_height?.toLocaleString() || 'pending'}`,
          exists: true,
          data: {
            txid: tx.txid,
            blockHeight: tx.block_height,
            value: totalValue,
            fee: tx.fee,
          }
        })
      } else {
        // Transaction not found - show as pending search
        results.push({
          type: 'transaction',
          value: query,
          label: `${query.slice(0, 10)}...${query.slice(-8)}`,
          sublabel: 'Transaction not found',
          exists: false,
        })
      }
    }

    // 3. Address search (0x + 40 hex chars or other address formats)
    if (/^0x[a-fA-F0-9]{40}$/.test(query) || /^[a-zA-Z0-9]{26,35}$/.test(query)) {
      // Try as address
      const addressInfo = await getAddressBalance(query)
      if (addressInfo) {
        results.push({
          type: 'address',
          value: query,
          label: `${query.slice(0, 10)}...${query.slice(-8)}`,
          sublabel: `Balance: ${(addressInfo.balance / 1e8).toFixed(8)} PYRAX • ${addressInfo.utxo_count} UTXOs`,
          exists: true,
          data: {
            address: addressInfo.address,
            balance: addressInfo.balance,
            utxoCount: addressInfo.utxo_count,
          }
        })
      } else {
        // Address might exist but have no balance
        results.push({
          type: 'address',
          value: query,
          label: `${query.slice(0, 10)}...${query.slice(-8)}`,
          sublabel: 'New or empty address',
          exists: true, // Addresses always "exist"
        })
      }

      // Also try as contract
      const contract = await getContract(query)
      if (contract) {
        results.push({
          type: 'contract',
          value: query,
          label: contract.name || `${query.slice(0, 10)}...${query.slice(-8)}`,
          sublabel: `${contract.contract_type.toUpperCase()} Contract • ${contract.is_verified ? 'Verified' : 'Unverified'}`,
          exists: true,
          data: {
            address: contract.address,
            type: contract.contract_type,
            verified: contract.is_verified,
            name: contract.name,
          }
        })
      }
    }

    // 4. Partial hex search (starts with 0x but incomplete)
    if (/^0x[a-fA-F0-9]+$/.test(query) && query.length > 4 && query.length < 66) {
      // Provide suggestions for completing the search
      if (query.length < 42) {
        results.push({
          type: 'address',
          value: query,
          label: `Search addresses starting with "${query.slice(0, 16)}..."`,
          sublabel: 'Continue typing for address (42 chars total)',
          exists: false,
        })
      }
      if (query.length < 66) {
        results.push({
          type: 'transaction',
          value: query,
          label: `Search transactions starting with "${query.slice(0, 16)}..."`,
          sublabel: 'Continue typing for tx hash (66 chars total)',
          exists: false,
        })
      }
    }

    // 5. If no results and query is alphanumeric, suggest it could be a token name
    if (results.length === 0 && /^[a-zA-Z][a-zA-Z0-9]*$/.test(query) && query.length >= 2) {
      results.push({
        type: 'token',
        value: query,
        label: `Search for "${query}"`,
        sublabel: 'Search token names and symbols',
        exists: false,
      })
    }

    return NextResponse.json({ 
      results, 
      query,
      timestamp: Date.now(),
    })
  } catch (error) {
    console.error('Search API error:', error)
    return NextResponse.json({ 
      results: [], 
      query, 
      error: 'Search failed' 
    }, { status: 500 })
  }
}
