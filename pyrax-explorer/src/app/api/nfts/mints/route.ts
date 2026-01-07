import { NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'
export const revalidate = 0

interface NFTMint {
  tx_hash: string
  block_number: number
  timestamp: number
  collection_address: string
  collection_name: string
  token_id: string
  token_type: string
  creator: string
  owner: string
  mint_price: number
  mint_price_usd: number
  holders: number
  image_url?: string
  name?: string
}

interface MintsResponse {
  mints: NFTMint[]
  pagination: {
    page: number
    per_page: number
    total: number
    total_pages: number
  }
}

type SortField = 'timestamp' | 'mint_price_usd' | 'holders' | 'block_number'
type SortOrder = 'asc' | 'desc'

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const page = Math.max(1, parseInt(searchParams.get('page') || '1', 10))
  const pageSize = Math.min(100, Math.max(10, parseInt(searchParams.get('pageSize') || '25', 10)))
  const sortBy = (searchParams.get('sortBy') || 'timestamp') as SortField
  const sortOrder = (searchParams.get('sortOrder') || 'desc') as SortOrder
  const nftType = searchParams.get('type') || 'all'
  const minValue = parseFloat(searchParams.get('minValue') || '0')
  const minHolders = parseInt(searchParams.get('minHolders') || '0', 10)
  const days = parseInt(searchParams.get('days') || '5', 10)
  const collectionAddress = searchParams.get('collection') || ''

  try {
    const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://localhost:28545'
    
    // Calculate timestamp for X days ago
    const sinceTimestamp = Math.floor(Date.now() / 1000) - (days * 24 * 60 * 60)
    
    const rpcResponse = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'pyrax_getNFTMints',
        params: [{
          page,
          per_page: pageSize,
          sort_by: sortBy,
          sort_order: sortOrder,
          nft_type: nftType === 'all' ? null : nftType,
          min_value_usd: minValue,
          min_holders: minHolders,
          since_timestamp: sinceTimestamp,
          collection_address: collectionAddress || null,
        }],
      }),
    })

    const rpcResult = await rpcResponse.json()

    if (rpcResult.result) {
      const data: MintsResponse = rpcResult.result
      return NextResponse.json({
        mints: data.mints.map(mint => ({
          txHash: mint.tx_hash,
          blockNumber: mint.block_number,
          timestamp: mint.timestamp,
          collectionAddress: mint.collection_address,
          collectionName: mint.collection_name,
          tokenId: mint.token_id,
          tokenType: mint.token_type,
          creator: mint.creator,
          owner: mint.owner,
          mintPrice: mint.mint_price,
          mintPriceUsd: mint.mint_price_usd,
          holders: mint.holders,
          imageUrl: mint.image_url || null,
          name: mint.name || null,
        })),
        pagination: {
          page: data.pagination.page,
          pageSize: data.pagination.per_page,
          totalMints: data.pagination.total,
          totalPages: data.pagination.total_pages,
        },
        filters: {
          sortBy,
          sortOrder,
          nftType,
          minValue,
          minHolders,
          days,
          collectionAddress,
        }
      })
    }

    return NextResponse.json({
      mints: [],
      pagination: {
        page: 1,
        pageSize,
        totalMints: 0,
        totalPages: 0,
      },
      filters: {
        sortBy,
        sortOrder,
        nftType,
        minValue,
        minHolders,
        days,
        collectionAddress,
      }
    })
  } catch (error) {
    console.error('Failed to fetch NFT mints:', error)
    return NextResponse.json({
      mints: [],
      pagination: { page: 1, pageSize, totalMints: 0, totalPages: 0 },
      filters: { sortBy, sortOrder, nftType, minValue, minHolders, days, collectionAddress },
      error: 'Failed to fetch NFT mints'
    })
  }
}
