# PYRAX Local Development Services Stop Script
# Stops all web services running on ports 3000-3004

Write-Host "=== Stopping PYRAX Development Services ===" -ForegroundColor Cyan

$ports = @(3000, 3001, 3002, 3003, 3004)
$stopped = 0

foreach ($port in $ports) {
    $connections = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
    if ($connections) {
        foreach ($conn in $connections) {
            $processId = $conn.OwningProcess
            $process = Get-Process -Id $processId -ErrorAction SilentlyContinue
            if ($process) {
                Write-Host "Stopping $($process.Name) (PID: $processId) on port $port..." -ForegroundColor Yellow
                Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
                $stopped++
            }
        }
    }
}

if ($stopped -eq 0) {
    Write-Host "No services were running." -ForegroundColor Gray
} else {
    Write-Host "`nStopped $stopped process(es)." -ForegroundColor Green
}

Write-Host "=== All Services Stopped ===" -ForegroundColor Cyan
