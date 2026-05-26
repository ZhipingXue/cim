# Run All Services (Development Mode)
# Starts all CIM services in separate processes

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Starting CIM Services (Development)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$env:PROTOC = "D:\protoc-21.12-win64\bin\protoc.exe"

# Start Zenohd
Write-Host "`n[1/8] Starting Zenohd..." -ForegroundColor Yellow
Start-Process -FilePath "zenohd" -WindowStyle Normal

# Start Rust services
$rustDir = "$PSScriptRoot\..\rust"

Write-Host "`n[2/8] Starting E40 Process Job Service (port 50041)..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run","-p","e40_process_job","--release" -WorkingDirectory $rustDir -WindowStyle Normal

Write-Host "`n[3/8] Starting E94 Control Job Service (port 50042)..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run","-p","e94_control_job","--release" -WorkingDirectory $rustDir -WindowStyle Normal

Write-Host "`n[4/8] Starting E87 Carrier Manager (port 50043)..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run","-p","e87_carrier_manager","--release" -WorkingDirectory $rustDir -WindowStyle Normal

Write-Host "`n[5/8] Starting E90 Substrate Tracker (port 50044)..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run","-p","e90_substrate_tracker","--release" -WorkingDirectory $rustDir -WindowStyle Normal

Write-Host "`n[6/8] Starting E125 Metadata Manager (port 50045)..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run","-p","e125_metadata_manager","--release" -WorkingDirectory $rustDir -WindowStyle Normal

Write-Host "`n[7/8] Starting Data Collection Service (port 50046)..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run","-p","data_collection","--release" -WorkingDirectory $rustDir -WindowStyle Normal

# Start .NET control apps
$dotnetDir = "$PSScriptRoot\..\dotnet"

Write-Host "`n[8/8] Starting .NET Control Apps..." -ForegroundColor Yellow
Start-Process -FilePath "dotnet" -ArgumentList "run","--project","ChamberControl","--configuration","Release" -WorkingDirectory $dotnetDir -WindowStyle Normal
Start-Process -FilePath "dotnet" -ArgumentList "run","--project","RobotControl","--configuration","Release" -WorkingDirectory $dotnetDir -WindowStyle Normal
Start-Process -FilePath "dotnet" -ArgumentList "run","--project","SubstrateCacheControl","--configuration","Release" -WorkingDirectory $dotnetDir -WindowStyle Normal
Start-Process -FilePath "dotnet" -ArgumentList "run","--project","LoadPortControl","--configuration","Release" -WorkingDirectory $dotnetDir -WindowStyle Normal

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "All services started!" -ForegroundColor Green
Write-Host "Press Ctrl+C to stop all services" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan

# Keep script running
while ($true) {
    Start-Sleep -Seconds 1
}
