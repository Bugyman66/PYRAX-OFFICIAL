# PYRAX Build Script (Windows PowerShell)
# This script builds pyrax-node and verifies the build

param(
    [switch]$Clean,
    [switch]$Verify
)

$ErrorActionPreference = "Stop"

Write-Host "=== PYRAX Build Script ===" -ForegroundColor Cyan
Write-Host ""

# Check Rust installation
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Rust not installed. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}

# Navigate to pyrax-node directory
$nodeDir = Join-Path $PSScriptRoot "..\pyrax-node"
if (-not (Test-Path $nodeDir)) {
    Write-Host "ERROR: pyrax-node directory not found" -ForegroundColor Red
    exit 1
}

Push-Location $nodeDir

try {
    # Clean if requested
    if ($Clean) {
        Write-Host "Cleaning previous build..." -ForegroundColor Yellow
        cargo clean
    }

    # Build release
    Write-Host "Building pyrax-node (release)..." -ForegroundColor Yellow
    cargo build --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed" -ForegroundColor Red
        exit 1
    }

    Write-Host "Build successful!" -ForegroundColor Green

    # Get binary info
    $binary = "..\target\release\pyrax-node.exe"
    if (Test-Path $binary) {
        $hash = (Get-FileHash $binary -Algorithm SHA256).Hash
        $size = (Get-Item $binary).Length
        
        Write-Host ""
        Write-Host "=== Build Artifacts ===" -ForegroundColor Cyan
        Write-Host "Binary: pyrax-node.exe"
        Write-Host "Size: $size bytes"
        Write-Host "SHA256: $hash"
        
        if ($Verify) {
            Write-Host ""
            Write-Host "=== Verification ===" -ForegroundColor Cyan
            # Run basic verification
            & $binary --version
        }
    }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "Build complete!" -ForegroundColor Green
