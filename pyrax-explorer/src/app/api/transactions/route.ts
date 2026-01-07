import { NextResponse } from 'next/server'
import { getChainInfo, getBlockByNumber, getTransaction } from '@/lib/rpc'

export const dynamic = 'force-dynamic'
export const revalidate = 0

interface TransactionData {
  hash: string
  from: string
  to: string
  value: number
  fee: number
  size: number
  timestamp: number
  blockNumber: number
  blockHash: string
  status: string
}

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const page = Math.max(1, parseInt(searchParams.get('page') || '1', 10))
  const pageSize = Math.min(100, Math.max(10, parseInt(searchParams.get('pageSize') || '25', 10)))

  try {
    const chainInfo = await getChainInfo()
    
    if (!chainInfo || chainInfo.best_block_height === 0) {
      return NextResponse.json({ 
        transactions: [],
        pagination: { page: 1, pageSize, totalTransactions: 0, totalPages: 0 }
      })
    }

    const allTransactions: TransactionData[] = []
    
    // We need to scan blocks to find transactions
    // For pagination, we'll scan enough blocks to get the requested page
    const maxBlocksToScan = Math.min(500, chainInfo.best_block_height + 1)
    const targetTxCount = page * pageSize + pageSize // Get enough for current page + buffer
    
    let scannedBlocks = 0
    let currentHeight = chainInfo.best_block_height

    while (scannedBlocks < maxBlocksToScan && currentHeight >= 0 && allTransactions.length < targetTxCount) {
      const block = await getBlockByNumber(currentHeight, true)
      
      if (block && block.transactions && block.transactions.length > 0) {
        // Fetch full transaction details in parallel
        const txPromises = block.transactions.map(txHash => getTransaction(txHash))
        const txResults = await Promise.all(txPromises)
        
        for (let i = 0; i < block.transactions.length; i++) {
          const txHash = block.transactions[i]
          const txInfo = txResults[i]
          
          // Calculate total input and output values
          let totalInput = 0
          let totalOutput = 0
          let fromAddress = 'Unknown'
          let toAddress = 'Unknown'
          
          if (txInfo) {
            totalInput = txInfo.inputs?.reduce((sum, inp) => sum + (inp.value || 0), 0) || 0
            totalOutput = txInfo.outputs?.reduce((sum, out) => sum + (out.value || 0), 0) || 0
            fromAddress = txInfo.inputs?.[0]?.prev_txid ? `tx:${txInfo.inputs[0].prev_txid.slice(0, 8)}` : 'Coinbase'
            toAddress = txInfo.outputs?.[0]?.address || 'Unknown'
          }
          
          allTransactions.push({
            hash: txHash,
            from: fromAddress,
            to: toAddress,
            value: txInfo ? totalOutput : 0,
            fee: txInfo?.fee || 0,
            size: txInfo?.size || 0,
            timestamp: block.timestamp,
            blockNumber: block.height,
            blockHash: block.hash,
            status: 'confirmed',
          })
        }
      }
      
      currentHeight--
      scannedBlocks++
    }

    // Calculate pagination
    const totalTransactions = allTransactions.length
    const totalPages = Math.ceil(totalTransactions / pageSize)
    
    // Slice for current page
    const startIndex = (page - 1) * pageSize
    const endIndex = startIndex + pageSize
    const paginatedTransactions = allTransactions.slice(startIndex, endIndex)

    return NextResponse.json({ 
      transactions: paginatedTransactions,
      pagination: {
        page,
        pageSize,
        totalTransactions,
        totalPages,
      }
    })
  } catch (error) {
    console.error('Failed to fetch transactions:', error)
    return NextResponse.json({ 
      transactions: [], 
      pagination: { page: 1, pageSize, totalTransactions: 0, totalPages: 0 },
      error: 'Failed to fetch transactions' 
    })
  }
}
