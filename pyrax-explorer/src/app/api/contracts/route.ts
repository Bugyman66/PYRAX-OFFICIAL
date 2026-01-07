import { NextResponse } from 'next/server'
import { getContracts } from '@/lib/rpc'

export const dynamic = 'force-dynamic'
export const revalidate = 0

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const page = Math.max(1, parseInt(searchParams.get('page') || '1', 10))
  const pageSize = Math.min(100, Math.max(10, parseInt(searchParams.get('pageSize') || '25', 10)))
  const contractType = (searchParams.get('type') || 'evm') as 'evm' | 'wasm'

  try {
    const response = await getContracts(contractType, page, pageSize)
    
    if (response && response.contracts) {
      return NextResponse.json({
        contracts: response.contracts.map(contract => ({
          address: contract.address,
          creator: contract.creator,
          creationTx: contract.creation_tx,
          creationBlock: contract.creation_block,
          creationTimestamp: contract.creation_timestamp,
          bytecodeHash: contract.bytecode_hash,
          contractType: contract.contract_type,
          isVerified: contract.is_verified,
          name: contract.name || null,
          compilerVersion: contract.compiler_version || null,
          optimization: contract.optimization || false,
          license: contract.license || null,
          balance: contract.balance,
          txCount: contract.tx_count,
        })),
        pagination: {
          page: response.pagination.page,
          pageSize: response.pagination.per_page,
          totalContracts: response.pagination.total,
          totalPages: response.pagination.total_pages,
        }
      })
    }

    // Return empty response if no data from node
    return NextResponse.json({
      contracts: [],
      pagination: {
        page: 1,
        pageSize,
        totalContracts: 0,
        totalPages: 0,
      }
    })
  } catch (error) {
    console.error('Failed to fetch contracts:', error)
    return NextResponse.json({
      contracts: [],
      pagination: { page: 1, pageSize, totalContracts: 0, totalPages: 0 },
      error: 'Failed to fetch contracts'
    })
  }
}
