import { NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'

// Supported Rust compiler versions
const RUST_VERSIONS = [
  '1.75.0', '1.74.1', '1.74.0', '1.73.0', '1.72.1', '1.72.0',
  '1.71.1', '1.71.0', '1.70.0', '1.69.0', '1.68.2', '1.68.0',
  '1.67.1', '1.67.0', '1.66.1', '1.66.0', '1.65.0', '1.64.0',
  '1.63.0', '1.62.1', '1.62.0', '1.61.0', '1.60.0', '1.59.0',
]

// WASM optimization levels
const OPTIMIZATION_LEVELS = [
  { value: 'none', label: 'None (Debug)' },
  { value: 'size', label: 'Optimize for Size (s)' },
  { value: 'size-aggressive', label: 'Aggressive Size (z)' },
  { value: 'speed', label: 'Optimize for Speed (3)' },
  { value: 'release', label: 'Release Default (2)' },
]

// License types
const LICENSE_TYPES = [
  { value: 'none', label: 'No License (None)' },
  { value: 'unlicense', label: 'The Unlicense' },
  { value: 'mit', label: 'MIT License' },
  { value: 'apache-2.0', label: 'Apache 2.0' },
  { value: 'gpl-3.0', label: 'GNU GPLv3' },
  { value: 'lgpl-3.0', label: 'GNU LGPLv3' },
  { value: 'bsd-2-clause', label: 'BSD 2-Clause' },
  { value: 'bsd-3-clause', label: 'BSD 3-Clause' },
  { value: 'mpl-2.0', label: 'Mozilla Public License 2.0' },
  { value: 'agpl-3.0', label: 'GNU AGPLv3' },
]

// WASM target environments
const WASM_TARGETS = [
  { value: 'wasm32-unknown-unknown', label: 'wasm32-unknown-unknown (Standard)' },
  { value: 'wasm32-wasi', label: 'wasm32-wasi (WASI)' },
]

// Cargo features commonly used
const COMMON_FEATURES = [
  'std', 'alloc', 'default', 'serde', 'borsh', 'json', 'panic-handler',
]

interface VerificationRequest {
  contractAddress: string
  contractName: string
  crateVersion: string
  rustVersion: string
  optimizationLevel: string
  wasmTarget: string
  sourceCode: string
  cargoToml: string
  initArguments: string
  license: string
  features: string[]
  dependencies: Array<{ name: string; version: string }>
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
  
  // Check for basic Rust WASM contract structure
  if (!code.includes('fn ') && !code.includes('pub fn ')) {
    return { valid: false, error: 'Source code must contain function definitions' }
  }
  
  // Check for common WASM contract patterns
  if (!code.includes('#[') && !code.includes('use ')) {
    return { valid: false, error: 'Source code should include Rust attributes or use statements' }
  }
  
  return { valid: true }
}

function validateCargoToml(toml: string): { valid: boolean; error?: string } {
  if (!toml || toml.trim().length === 0) {
    return { valid: false, error: 'Cargo.toml is required' }
  }
  
  if (!toml.includes('[package]')) {
    return { valid: false, error: 'Cargo.toml must include [package] section' }
  }
  
  if (!toml.includes('name =') && !toml.includes('name=')) {
    return { valid: false, error: 'Cargo.toml must include package name' }
  }
  
  return { valid: true }
}

function validateInitArgs(args: string): { valid: boolean; error?: string } {
  if (!args || args.trim().length === 0) {
    return { valid: true } // Init args are optional
  }
  
  // Should be valid JSON or hex encoded
  try {
    JSON.parse(args)
    return { valid: true }
  } catch {
    // Check if hex encoded
    const cleanArgs = args.startsWith('0x') ? args.slice(2) : args
    if (!/^[a-fA-F0-9]*$/.test(cleanArgs)) {
      return { valid: false, error: 'Init arguments must be valid JSON or hex-encoded bytes' }
    }
    return { valid: true }
  }
}

export async function GET() {
  // Return available options for the verification form
  return NextResponse.json({
    rustVersions: RUST_VERSIONS,
    optimizationLevels: OPTIMIZATION_LEVELS,
    licenseTypes: LICENSE_TYPES,
    wasmTargets: WASM_TARGETS,
    commonFeatures: COMMON_FEATURES,
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
      errors.contractName = 'Contract/crate name is required'
    } else if (!/^[a-zA-Z_][a-zA-Z0-9_-]*$/.test(body.contractName)) {
      errors.contractName = 'Invalid crate name format (use snake_case or kebab-case)'
    }
    
    // Rust version validation
    if (!body.rustVersion) {
      errors.rustVersion = 'Rust version is required'
    } else if (!RUST_VERSIONS.includes(body.rustVersion)) {
      errors.rustVersion = 'Invalid Rust version'
    }
    
    // Source code validation
    const sourceValidation = validateSourceCode(body.sourceCode)
    if (!sourceValidation.valid) {
      errors.sourceCode = sourceValidation.error!
    }
    
    // Cargo.toml validation
    const cargoValidation = validateCargoToml(body.cargoToml)
    if (!cargoValidation.valid) {
      errors.cargoToml = cargoValidation.error!
    }
    
    // Init arguments validation
    const argsValidation = validateInitArgs(body.initArguments)
    if (!argsValidation.valid) {
      errors.initArguments = argsValidation.error!
    }
    
    // Dependency validation
    if (body.dependencies && body.dependencies.length > 0) {
      for (let i = 0; i < body.dependencies.length; i++) {
        const dep = body.dependencies[i]
        if (!dep.name || dep.name.trim().length === 0) {
          errors[`dependency_${i}_name`] = `Dependency ${i + 1} name is required`
        }
        if (!dep.version || !/^\d+\.\d+(\.\d+)?$/.test(dep.version)) {
          errors[`dependency_${i}_version`] = `Dependency ${i + 1} has invalid version format`
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
        method: 'pyrax_verifyWasmContract',
        params: [{
          address: body.contractAddress,
          name: body.contractName,
          crate_version: body.crateVersion,
          rust_version: body.rustVersion,
          optimization_level: body.optimizationLevel,
          wasm_target: body.wasmTarget,
          source_code: body.sourceCode,
          cargo_toml: body.cargoToml,
          init_arguments: body.initArguments || '',
          license: body.license,
          features: body.features || [],
          dependencies: body.dependencies || [],
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
      message: 'WASM contract verified successfully',
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
