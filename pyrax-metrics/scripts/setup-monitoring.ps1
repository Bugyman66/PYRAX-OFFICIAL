# PYRAX Monitoring Stack Setup Script
# Downloads and configures Prometheus + Grafana for Windows (no Docker required)

$ErrorActionPreference = "Stop"
$MonitoringDir = "C:\monitoring"
$PrometheusVersion = "2.48.0"
$GrafanaVersion = "10.2.0"

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  PYRAX Monitoring Stack Setup" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Create monitoring directory
Write-Host "[1/6] Creating monitoring directory..." -ForegroundColor Yellow
if (-not (Test-Path $MonitoringDir)) {
    New-Item -ItemType Directory -Path $MonitoringDir -Force | Out-Null
}
Set-Location $MonitoringDir

# Download Prometheus
Write-Host "[2/6] Downloading Prometheus v$PrometheusVersion..." -ForegroundColor Yellow
$prometheusUrl = "https://github.com/prometheus/prometheus/releases/download/v$PrometheusVersion/prometheus-$PrometheusVersion.windows-amd64.zip"
$prometheusZip = "$MonitoringDir\prometheus.zip"

if (-not (Test-Path "$MonitoringDir\prometheus")) {
    Invoke-WebRequest -Uri $prometheusUrl -OutFile $prometheusZip -UseBasicParsing
    Expand-Archive -Path $prometheusZip -DestinationPath $MonitoringDir -Force
    Rename-Item -Path "$MonitoringDir\prometheus-$PrometheusVersion.windows-amd64" -NewName "prometheus"
    Remove-Item $prometheusZip
    Write-Host "    Prometheus installed!" -ForegroundColor Green
} else {
    Write-Host "    Prometheus already installed, skipping." -ForegroundColor Gray
}

# Download Grafana
Write-Host "[3/6] Downloading Grafana v$GrafanaVersion..." -ForegroundColor Yellow
$grafanaUrl = "https://dl.grafana.com/oss/release/grafana-$GrafanaVersion.windows-amd64.zip"
$grafanaZip = "$MonitoringDir\grafana.zip"

if (-not (Test-Path "$MonitoringDir\grafana")) {
    Invoke-WebRequest -Uri $grafanaUrl -OutFile $grafanaZip -UseBasicParsing
    Expand-Archive -Path $grafanaZip -DestinationPath $MonitoringDir -Force
    Rename-Item -Path "$MonitoringDir\grafana-v$GrafanaVersion" -NewName "grafana"
    Remove-Item $grafanaZip
    Write-Host "    Grafana installed!" -ForegroundColor Green
} else {
    Write-Host "    Grafana already installed, skipping." -ForegroundColor Gray
}

# Configure Prometheus
Write-Host "[4/6] Configuring Prometheus..." -ForegroundColor Yellow
$prometheusConfig = @"
global:
  scrape_interval: 5s
  evaluation_interval: 5s

scrape_configs:
  - job_name: 'pyrax-metrics'
    static_configs:
      - targets: ['localhost:9092']
    metrics_path: /metrics

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
"@

$prometheusConfig | Out-File -FilePath "$MonitoringDir\prometheus\prometheus.yml" -Encoding UTF8 -Force
Write-Host "    Prometheus configured to scrape localhost:9092" -ForegroundColor Green

# Configure Grafana datasource
Write-Host "[5/6] Configuring Grafana datasource..." -ForegroundColor Yellow
$grafanaProvisioningDir = "$MonitoringDir\grafana\conf\provisioning\datasources"
if (-not (Test-Path $grafanaProvisioningDir)) {
    New-Item -ItemType Directory -Path $grafanaProvisioningDir -Force | Out-Null
}

$datasourceConfig = @"
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://localhost:9090
    isDefault: true
    uid: prometheus
"@

$datasourceConfig | Out-File -FilePath "$grafanaProvisioningDir\prometheus.yml" -Encoding UTF8 -Force
Write-Host "    Grafana datasource configured!" -ForegroundColor Green

# Create start script
Write-Host "[6/6] Creating start script..." -ForegroundColor Yellow
$startScript = @"
# Start PYRAX Monitoring Stack
Write-Host "Starting PYRAX Monitoring Stack..." -ForegroundColor Cyan

# Start Prometheus in background
Write-Host "Starting Prometheus on http://localhost:9090" -ForegroundColor Yellow
Start-Process -FilePath "$MonitoringDir\prometheus\prometheus.exe" -WorkingDirectory "$MonitoringDir\prometheus" -WindowStyle Minimized

# Start Grafana in background
Write-Host "Starting Grafana on http://localhost:3000" -ForegroundColor Yellow
Start-Process -FilePath "$MonitoringDir\grafana\bin\grafana-server.exe" -WorkingDirectory "$MonitoringDir\grafana" -WindowStyle Minimized

Write-Host ""
Write-Host "=====================================" -ForegroundColor Green
Write-Host "  Monitoring Stack Started!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Prometheus: http://localhost:9090" -ForegroundColor White
Write-Host "  Grafana:    http://localhost:3000 (admin/admin)" -ForegroundColor White
Write-Host "  Metrics:    http://localhost:9092/metrics" -ForegroundColor White
Write-Host ""
Write-Host "Don't forget to start pyrax-metrics:" -ForegroundColor Yellow
Write-Host "  cargo run --bin pyrax-metrics -- --config pyrax-metrics/config/devnet.toml" -ForegroundColor Gray
"@

$startScript | Out-File -FilePath "$MonitoringDir\start-monitoring.ps1" -Encoding UTF8 -Force

# Create stop script
$stopScript = @"
# Stop PYRAX Monitoring Stack
Write-Host "Stopping PYRAX Monitoring Stack..." -ForegroundColor Yellow
Get-Process -Name "prometheus" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "grafana-server" -ErrorAction SilentlyContinue | Stop-Process -Force
Write-Host "Monitoring stack stopped." -ForegroundColor Green
"@

$stopScript | Out-File -FilePath "$MonitoringDir\stop-monitoring.ps1" -Encoding UTF8 -Force

Write-Host ""
Write-Host "=====================================" -ForegroundColor Green
Write-Host "  Setup Complete!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Installed to: $MonitoringDir" -ForegroundColor White
Write-Host ""
Write-Host "  To start monitoring:" -ForegroundColor Yellow
Write-Host "    C:\monitoring\start-monitoring.ps1" -ForegroundColor Gray
Write-Host ""
Write-Host "  To stop monitoring:" -ForegroundColor Yellow
Write-Host "    C:\monitoring\stop-monitoring.ps1" -ForegroundColor Gray
Write-Host ""
