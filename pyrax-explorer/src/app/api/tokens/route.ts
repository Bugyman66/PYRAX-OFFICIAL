import { NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'
export const revalidate = 0

interface TokenInfo {
  address: string
  name: string
  symbol: string
  decimals: number
  total_supply: string
  holders: number
  transfers: number
  market_cap: number
  price: number
  change_24h: number
  volume_24h: number
  type: 'ERC20' | 'ERC721' | 'ERC1155'
  verified: boolean
  logo?: string
  website?: string
  created_at: number
  creator: string
}

interface TokensResponse {
  tokens: TokenInfo[]
  pagination: {
    page: number
    per_page: number
    total: number
    total_pages: number
  }
}

type SortField = 'market_cap' | 'holders' | 'transfers' | 'volume_24h' | 'change_24h'
type SortOrder = 'asc' | 'desc'

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const page = Math.max(1, parseInt(searchParams.get('page') || '1', 10))
  const pageSize = Math.min(100, Math.max(10, parseInt(searchParams.get('pageSize') || '25', 10)))
  const sortBy = (searchParams.get('sortBy') || 'market_cap') as SortField
  const sortOrder = (searchParams.get('sortOrder') || 'desc') as SortOrder
  const tokenType = searchParams.get('type') || 'all' // all, ERC20, ERC721, ERC1155
  const minHolders = parseInt(searchParams.get('minHolders') || '0', 10)
  const minValue = parseFloat(searchParams.get('minValue') || '0')
  const search = searchParams.get('search') || ''

  try {
    const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://localhost:28545'
    
    const rpcResponse = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'pyrax_getTokens',
        params: [{
          page,
          per_page: pageSize,
          sort_by: sortBy,
          sort_order: sortOrder,
          token_type: tokenType === 'all' ? null : tokenType,
          min_holders: minHolders,
          min_market_cap: minValue,
          search: search || null,
        }],
      }),
    })

    const rpcResult = await rpcResponse.json()

    if (rpcResult.result) {
      const data: TokensResponse = rpcResult.result
      return NextResponse.json({
        tokens: data.tokens.map(token => ({
          address: token.address,
          name: token.name,
          symbol: token.symbol,
          decimals: token.decimals,
          totalSupply: token.total_supply,
          holders: token.holders,
          transfers: token.transfers,
          marketCap: token.market_cap,
          price: token.price,
          change24h: token.change_24h,
          volume24h: token.volume_24h,
          type: token.type,
          verified: token.verified,
          logo: token.logo || null,
          website: token.website || null,
          createdAt: token.created_at,
          creator: token.creator,
        })),
        pagination: {
          page: data.pagination.page,
          pageSize: data.pagination.per_page,
          totalTokens: data.pagination.total,
          totalPages: data.pagination.total_pages,
        },
        filters: {
          sortBy,
          sortOrder,
          tokenType,
          minHolders,
          minValue,
          search,
        }
      })
    }

    // Return empty response if no data from node
    return NextResponse.json({
      tokens: [],
      pagination: {
        page: 1,
        pageSize,
        totalTokens: 0,
        totalPages: 0,
      },
      filters: {
        sortBy,
        sortOrder,
        tokenType,
        minHolders,
        minValue,
        search,
      }
    })
  } catch (error) {
    console.error('Failed to fetch tokens:', error)
    return NextResponse.json({
      tokens: [],
      pagination: { page: 1, pageSize, totalTokens: 0, totalPages: 0 },
      filters: { sortBy, sortOrder, tokenType, minHolders, minValue, search },
      error: 'Failed to fetch tokens'
    })
  }
}
