import { NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'
export const revalidate = 0

interface TokenTransfer {
  tx_hash: string
  block_number: number
  timestamp: number
  token_address: string
  token_name: string
  token_symbol: string
  token_type: string
  from: string
  to: string
  amount: string
  value_usd: number
  method: string
}

interface TransfersResponse {
  transfers: TokenTransfer[]
  pagination: {
    page: number
    per_page: number
    total: number
    total_pages: number
  }
}

type SortField = 'timestamp' | 'value_usd' | 'amount' | 'block_number'
type SortOrder = 'asc' | 'desc'

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const page = Math.max(1, parseInt(searchParams.get('page') || '1', 10))
  const pageSize = Math.min(100, Math.max(10, parseInt(searchParams.get('pageSize') || '25', 10)))
  const sortBy = (searchParams.get('sortBy') || 'timestamp') as SortField
  const sortOrder = (searchParams.get('sortOrder') || 'desc') as SortOrder
  const tokenType = searchParams.get('type') || 'all'
  const minValue = parseFloat(searchParams.get('minValue') || '0')
  const tokenAddress = searchParams.get('token') || ''
  const address = searchParams.get('address') || '' // filter by from/to address

  try {
    const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://localhost:28545'
    
    const rpcResponse = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'pyrax_getTokenTransfers',
        params: [{
          page,
          per_page: pageSize,
          sort_by: sortBy,
          sort_order: sortOrder,
          token_type: tokenType === 'all' ? null : tokenType,
          min_value_usd: minValue,
          token_address: tokenAddress || null,
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
          tokenAddress: transfer.token_address,
          tokenName: transfer.token_name,
          tokenSymbol: transfer.token_symbol,
          tokenType: transfer.token_type,
          from: transfer.from,
          to: transfer.to,
          amount: transfer.amount,
          valueUsd: transfer.value_usd,
          method: transfer.method,
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
          tokenType,
          minValue,
          tokenAddress,
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
        tokenType,
        minValue,
        tokenAddress,
        address,
      }
    })
  } catch (error) {
    console.error('Failed to fetch token transfers:', error)
    return NextResponse.json({
      transfers: [],
      pagination: { page: 1, pageSize, totalTransfers: 0, totalPages: 0 },
      filters: { sortBy, sortOrder, tokenType, minValue, tokenAddress, address },
      error: 'Failed to fetch token transfers'
    })
  }
}
