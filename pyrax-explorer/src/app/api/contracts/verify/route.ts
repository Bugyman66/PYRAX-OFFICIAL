import { NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'

// Supported Solidity compiler versions
const COMPILER_VERSIONS = [
  '0.8.24', '0.8.23', '0.8.22', '0.8.21', '0.8.20',
  '0.8.19', '0.8.18', '0.8.17', '0.8.16', '0.8.15',
  '0.8.14', '0.8.13', '0.8.12', '0.8.11', '0.8.10',
  '0.8.9', '0.8.8', '0.8.7', '0.8.6', '0.8.5',
  '0.8.4', '0.8.3', '0.8.2', '0.8.1', '0.8.0',
  '0.7.6', '0.7.5', '0.7.4', '0.7.3', '0.7.2', '0.7.1', '0.7.0',
  '0.6.12', '0.6.11', '0.6.10', '0.6.9', '0.6.8', '0.6.7', '0.6.6',
  '0.6.5', '0.6.4', '0.6.3', '0.6.2', '0.6.1', '0.6.0',
]

// License types
const LICENSE_TYPES = [
  { value: 'none', label: 'No License (None)' },
  { value: 'unlicense', label: 'The Unlicense' },
  { value: 'mit', label: 'MIT License' },
  { value: 'gpl-2.0', label: 'GNU GPLv2' },
  { value: 'gpl-3.0', label: 'GNU GPLv3' },
  { value: 'lgpl-2.1', label: 'GNU LGPLv2.1' },
  { value: 'lgpl-3.0', label: 'GNU LGPLv3' },
  { value: 'bsd-2-clause', label: 'BSD 2-Clause' },
  { value: 'bsd-3-clause', label: 'BSD 3-Clause' },
  { value: 'mpl-2.0', label: 'Mozilla Public License 2.0' },
  { value: 'osl-3.0', label: 'Open Software License 3.0' },
  { value: 'apache-2.0', label: 'Apache 2.0' },
  { value: 'agpl-3.0', label: 'GNU AGPLv3' },
  { value: 'busl-1.1', label: 'Business Source License 1.1' },
]

// EVM versions
const EVM_VERSIONS = [
  { value: 'default', label: 'Default (compiler default)' },
  { value: 'cancun', label: 'Cancun' },
  { value: 'shanghai', label: 'Shanghai' },
  { value: 'paris', label: 'Paris' },
  { value: 'london', label: 'London' },
  { value: 'berlin', label: 'Berlin' },
  { value: 'istanbul', label: 'Istanbul' },
  { value: 'petersburg', label: 'Petersburg' },
  { value: 'constantinople', label: 'Constantinople' },
  { value: 'byzantium', label: 'Byzantium' },
]

interface VerificationRequest {
  contractAddress: string
  contractName: string
  compilerVersion: string
  optimization: boolean
  optimizationRuns: number
  evmVersion: string
  sourceCode: string
  constructorArguments: string
  license: string
  libraries: Array<{ name: string; address: string }>
}

function validateAddress(address: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(address)
}

function validateSourceCode(code: string): { valid: boolean; error?: string } {
  if (!code || code.trim().length === 0) {
    return { valid: false, error: 'Source code is required' }
  }
  
  if (code.length < 50) {
    return { valid: false, error: 'Source code appears to be too short' }
  }
  
  // Check for basic Solidity structure
  if (!code.includes('pragma solidity') && !code.includes('// SPDX-License-Identifier')) {
    return { valid: false, error: 'Source code must include pragma solidity directive' }
  }
  
  // Check for contract definition
  if (!code.includes('contract ') && !code.includes('interface ') && !code.includes('library ')) {
    return { valid: false, error: 'Source code must define a contract, interface, or library' }
  }
  
  return { valid: true }
}

function validateConstructorArgs(args: string): { valid: boolean; error?: string } {
  if (!args || args.trim().length === 0) {
    return { valid: true } // Constructor args are optional
  }
  
  // Should be hex encoded without 0x prefix or with 0x prefix
  const cleanArgs = args.startsWith('0x') ? args.slice(2) : args
  if (!/^[a-fA-F0-9]*$/.test(cleanArgs)) {
    return { valid: false, error: 'Constructor arguments must be ABI-encoded hex string' }
  }
  
  return { valid: true }
}

export async function GET() {
  // Return available options for the verification form
  return NextResponse.json({
    compilerVersions: COMPILER_VERSIONS,
    licenseTypes: LICENSE_TYPES,
    evmVersions: EVM_VERSIONS,
  })
}

export async function POST(request: Request) {
  try {
    const body: VerificationRequest = await request.json()
    
    // Validate required fields
    const errors: Record<string, string> = {}
    
    // Contract address validation
    if (!body.contractAddress) {
      errors.contractAddress = 'Contract address is required'
    } else if (!validateAddress(body.contractAddress)) {
      errors.contractAddress = 'Invalid contract address format'
    }
    
    // Contract name validation
    if (!body.contractName || body.contractName.trim().length === 0) {
      errors.contractName = 'Contract name is required'
    } else if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(body.contractName)) {
      errors.contractName = 'Invalid contract name format'
    }
    
    // Compiler version validation
    if (!body.compilerVersion) {
      errors.compilerVersion = 'Compiler version is required'
    } else if (!COMPILER_VERSIONS.includes(body.compilerVersion)) {
      errors.compilerVersion = 'Invalid compiler version'
    }
    
    // Source code validation
    const sourceValidation = validateSourceCode(body.sourceCode)
    if (!sourceValidation.valid) {
      errors.sourceCode = sourceValidation.error!
    }
    
    // Constructor arguments validation
    const argsValidation = validateConstructorArgs(body.constructorArguments)
    if (!argsValidation.valid) {
      errors.constructorArguments = argsValidation.error!
    }
    
    // Optimization runs validation
    if (body.optimization && (body.optimizationRuns < 1 || body.optimizationRuns > 10000000)) {
      errors.optimizationRuns = 'Optimization runs must be between 1 and 10,000,000'
    }
    
    // Library addresses validation
    if (body.libraries && body.libraries.length > 0) {
      for (let i = 0; i < body.libraries.length; i++) {
        const lib = body.libraries[i]
        if (!lib.name || lib.name.trim().length === 0) {
          errors[`library_${i}_name`] = `Library ${i + 1} name is required`
        }
        if (!lib.address || !validateAddress(lib.address)) {
          errors[`library_${i}_address`] = `Library ${i + 1} has invalid address`
        }
      }
    }
    
    // If validation errors, return them
    if (Object.keys(errors).length > 0) {
      return NextResponse.json({ 
        success: false, 
        errors,
        message: 'Validation failed' 
      }, { status: 400 })
    }
    
    // Submit verification request to node
    const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://localhost:28545'
    
    const rpcResponse = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'pyrax_verifyContract',
        params: [{
          address: body.contractAddress,
          name: body.contractName,
          compiler_version: body.compilerVersion,
          optimization: body.optimization,
          optimization_runs: body.optimizationRuns,
          evm_version: body.evmVersion,
          source_code: body.sourceCode,
          constructor_arguments: body.constructorArguments || '',
          license: body.license,
          libraries: body.libraries || [],
        }],
      }),
    })
    
    const rpcResult = await rpcResponse.json()
    
    if (rpcResult.error) {
      return NextResponse.json({
        success: false,
        message: rpcResult.error.message || 'Verification failed',
        code: rpcResult.error.code,
      }, { status: 400 })
    }
    
    // Return success with verification result
    return NextResponse.json({
      success: true,
      message: 'Contract verified successfully',
      data: rpcResult.result,
    })
    
  } catch (error) {
    console.error('Verification error:', error)
    return NextResponse.json({
      success: false,
      message: 'Failed to process verification request',
    }, { status: 500 })
  }
}
