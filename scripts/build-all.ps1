# Build All Script
# Builds both Rust workspace and .NET solution

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building CIM App Group" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Set PROTOC environment variable
$env:PROTOC = "D:\protoc-21.12-win64\bin\protoc.exe"

# Build Rust workspace
Write-Host "`n[1/2] Building Rust workspace..." -ForegroundColor Yellow
cd $PSScriptRoot\..\rust
$cargoResult = cargo build --workspace --release 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "Rust build failed"
    exit 1
}
Write-Host "Rust workspace built successfully" -ForegroundColor Green

# Build .NET solution
Write-Host "`n[2/2] Building .NET solution..." -ForegroundColor Yellow
cd $PSScriptRoot\..\dotnet
$dotnetResult = dotnet build --configuration Release 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error ".NET build failed"
    exit 1
}
Write-Host ".NET solution built successfully" -ForegroundColor Green

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
