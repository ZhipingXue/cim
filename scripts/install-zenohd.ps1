# Install Zenohd Script
# Installs zenohd binary via cargo binstall

$ErrorActionPreference = "Stop"

Write-Host "Installing zenohd..." -ForegroundColor Cyan

try {
    cargo binstall zenohd --force
    Write-Host "zenohd installed successfully" -ForegroundColor Green
} catch {
    Write-Error "Failed to install zenohd. Make sure cargo-binstall is installed: cargo install cargo-binstall"
    exit 1
}
