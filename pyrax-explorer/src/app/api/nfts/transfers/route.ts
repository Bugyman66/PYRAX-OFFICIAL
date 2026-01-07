import { NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'
export const revalidate = 0

interface NFTTransfer {
  tx_hash: string
  block_number: number
  timestamp: number
  collection_address: string
  collection_name: string
  token_id: string
  token_type: string
  from: string
  to: string
  value_usd: number
  method: string
  image_url?: string
}

interface TransfersResponse {
  transfers: NFTTransfer[]
  pagination: {
    page: number
    per_page: number
    total: number
    total_pages: number
  }
}

type SortField = 'timestamp' | 'value_usd' | 'block_number'
type SortOrder = 'asc' | 'desc'

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const page = Math.max(1, parseInt(searchParams.get('page') || '1', 10))
  const pageSize = Math.min(100, Math.max(10, parseInt(searchParams.get('pageSize') || '25', 10)))
  const sortBy = (searchParams.get('sortBy') || 'timestamp') as SortField
  const sortOrder = (searchParams.get('sortOrder') || 'desc') as SortOrder
  const nftType = searchParams.get('type') || 'all' // all, ERC721, ERC1155, WASM721
  const minValue = parseFloat(searchParams.get('minValue') || '0')
  const collectionAddress = searchParams.get('collection') || ''
  const address = searchParams.get('address') || ''

  try {
    const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://localhost:28545'
    
    const rpcResponse = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'pyrax_getNFTTransfers',
        params: [{
          page,
          per_page: pageSize,
          sort_by: sortBy,
          sort_order: sortOrder,
          nft_type: nftType === 'all' ? null : nftType,
          min_value_usd: minValue,
          collection_address: collectionAddress || null,
          address: address || null,
        }],
      }),
    })

    const rpcResult = await rpcResponse.json()

    if (rpcResult.result) {
      const data: TransfersResponse = rpcResult.result
      return NextResponse.json({
        transfers: data.transfers.map(transfer => ({
          txHash: transfer.tx_hash,
          blockNumber: transfer.block_number,
          timestamp: transfer.timestamp,
          collectionAddress: transfer.collection_address,
          collectionName: transfer.collection_name,
          tokenId: transfer.token_id,
          tokenType: transfer.token_type,
          from: transfer.from,
          to: transfer.to,
          valueUsd: transfer.value_usd,
          method: transfer.method,
          imageUrl: transfer.image_url || null,
        })),
        pagination: {
          page: data.pagination.page,
          pageSize: data.pagination.per_page,
          totalTransfers: data.pagination.total,
          totalPages: data.pagination.total_pages,
        },
        filters: {
          sortBy,
          sortOrder,
          nftType,
          minValue,
          collectionAddress,
          address,
        }
      })
    }

    return NextResponse.json({
      transfers: [],
      pagination: {
        page: 1,
        pageSize,
        totalTransfers: 0,
        totalPages: 0,
      },
      filters: {
        sortBy,
        sortOrder,
        nftType,
        minValue,
        collectionAddress,
        address,
      }
    })
  } catch (error) {
    console.error('Failed to fetch NFT transfers:', error)
    return NextResponse.json({
      transfers: [],
      pagination: { page: 1, pageSize, totalTransfers: 0, totalPages: 0 },
      filters: { sortBy, sortOrder, nftType, minValue, collectionAddress, address },
      error: 'Failed to fetch NFT transfers'
    })
  }
}
