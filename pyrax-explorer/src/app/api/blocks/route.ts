import { NextResponse } from 'next/server'
import { getChainInfo, getBlockByNumber } from '@/lib/rpc'

export const dynamic = 'force-dynamic'
export const revalidate = 0

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const page = Math.max(1, parseInt(searchParams.get('page') || '1', 10))
  const pageSize = Math.min(100, Math.max(10, parseInt(searchParams.get('pageSize') || '25', 10)))

  try {
    const chainInfo = await getChainInfo()
    
    if (!chainInfo || chainInfo.best_block_height === 0) {
      return NextResponse.json({ 
        blocks: [], 
        pagination: { page: 1, pageSize, totalBlocks: 0, totalPages: 0 } 
      })
    }

    const totalBlocks = chainInfo.best_block_height + 1
    const totalPages = Math.ceil(totalBlocks / pageSize)
    
    // Calculate start and end heights for this page (descending order)
    const startHeight = chainInfo.best_block_height - ((page - 1) * pageSize)
    const endHeight = Math.max(0, startHeight - pageSize + 1)

    const blocks = []

    // Fetch blocks in parallel for speed
    const blockPromises = []
    for (let height = startHeight; height >= endHeight; height--) {
      blockPromises.push(getBlockByNumber(height, false))
    }

    const blockResults = await Promise.all(blockPromises)
    
    for (const block of blockResults) {
      if (block) {
        blocks.push({
          height: block.height,
          hash: block.hash,
          timestamp: block.timestamp,
          txCount: block.tx_count || 0,
          miner: block.miner || 'Unknown',
          reward: block.reward || 50,
          size: block.size || 0,
          difficulty: block.difficulty || 0,
          prevHash: block.prev_hash || '',
          nonce: block.nonce || 0,
        })
      }
    }

    return NextResponse.json({ 
      blocks,
      pagination: {
        page,
        pageSize,
        totalBlocks,
        totalPages,
      }
    })
  } catch (error) {
    console.error('Failed to fetch blocks:', error)
    return NextResponse.json({ 
      blocks: [], 
      pagination: { page: 1, pageSize, totalBlocks: 0, totalPages: 0 },
      error: 'Failed to fetch blocks' 
    })
  }
}
