# PYRAX Local Development Services Startup Script
# This script starts all web services on ports 3000-3004
# Services run in background and persist after closing this window

Write-Host "=== Starting PYRAX Development Services ===" -ForegroundColor Cyan

# Kill any existing processes on these ports
$ports = @(3000, 3001, 3002, 3003, 3004)
foreach ($port in $ports) {
    $process = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -ErrorAction SilentlyContinue
    if ($process) {
        Write-Host "Stopping process on port $port..." -ForegroundColor Yellow
        Stop-Process -Id $process -Force -ErrorAction SilentlyContinue
    }
}

Start-Sleep -Seconds 2

$baseDir = $PSScriptRoot

# Start pyrax-website on port 3000
Write-Host "`n[1/5] Starting pyrax-website on http://localhost:3000" -ForegroundColor Green
Start-Process -FilePath "cmd" -ArgumentList "/c cd /d `"$baseDir\pyrax-website`" && npm run dev -- -p 3000" -WindowStyle Minimized

# Start pyrax-explorer on port 3001
Write-Host "[2/5] Starting pyrax-explorer on http://localhost:3001" -ForegroundColor Green
Start-Process -FilePath "cmd" -ArgumentList "/c cd /d `"$baseDir\pyrax-explorer`" && npm run dev -- -p 3001" -WindowStyle Minimized

# Start pyrax-docs on port 3002
Write-Host "[3/5] Starting pyrax-docs on http://localhost:3002" -ForegroundColor Green
Start-Process -FilePath "cmd" -ArgumentList "/c cd /d `"$baseDir\pyrax-docs`" && npm run start -- --port 3002" -WindowStyle Minimized

# Start pyrax-faucet on port 3003
Write-Host "[4/5] Starting pyrax-faucet on http://localhost:3003" -ForegroundColor Green
Start-Process -FilePath "cmd" -ArgumentList "/c cd /d `"$baseDir\pyrax-faucet`" && npm run dev -- -p 3003" -WindowStyle Minimized

# Start pyrax-core-marketing on port 3004
Write-Host "[5/5] Starting pyrax-marketing on http://localhost:3004" -ForegroundColor Green
Start-Process -FilePath "cmd" -ArgumentList "/c cd /d `"$baseDir\pyrax-core-marketing`" && npm run dev -- -p 3004" -WindowStyle Minimized

Write-Host "`n=== All Services Started ===" -ForegroundColor Cyan
Write-Host @"

Services running (minimized windows):
  - Website:    http://localhost:3000
  - Explorer:   http://localhost:3001  
  - Docs:       http://localhost:3002
  - Faucet:     http://localhost:3003
  - Marketing:  http://localhost:3004

To stop all services, run: .\stop-all-services.ps1
Services will persist after closing Windsurf!

"@ -ForegroundColor White
