param(
    [string]$DropletIp = "209.97.164.167",
    [string]$DropletUser = "root",
    [string]$SshKey = "$HOME\.ssh\homekey",
    [string]$RemoteDir = "/opt/library_app"
)

$ErrorActionPreference = "Stop"
$ZigPath = "$env:LOCALAPPDATA\Programs\Python\Python314\Lib\site-packages\ziglang"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Athenaeum Library App - High-Performance Linux Deployer" -ForegroundColor Cyan
Write-Host " Target: $DropletUser@$DropletIp (Debian x86_64-v3)" -ForegroundColor DarkGray
Write-Host "==========================================================" -ForegroundColor Cyan

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location (Join-Path $ScriptDir "..")

if (-not (Test-Path $SshKey)) {
    Write-Error "SSH key not found at $SshKey"
    exit 1
}

Write-Host "`n[1/5] Compiling Linux binary on PC for Droplet CPU (AVX2/BMI2)..." -ForegroundColor Yellow
$env:PATH = "$ZigPath;$env:PATH"
$env:RUSTFLAGS = "-C target-cpu=x86-64-v3"
cargo zigbuild --target x86_64-unknown-linux-musl --release

Write-Host "`n[2/5] Bundling Tailwind CSS & static assets from Linux binary..." -ForegroundColor Yellow
cargo run --example bundle_linux

Write-Host "`n[3/5] Stopping remote service if active..." -ForegroundColor Yellow
ssh -i $SshKey -o StrictHostKeyChecking=accept-new "$DropletUser@$DropletIp" "if systemctl is-active --quiet library_app 2>/dev/null; then echo 'Stopping library_app service...'; systemctl stop library_app; fi; mkdir -p $RemoteDir/assets"

Write-Host "`n[4/5] Uploading compiled binary and assets (~7MB total)..." -ForegroundColor Yellow
scp -i $SshKey target\x86_64-unknown-linux-musl\release\library_app "$DropletUser@${DropletIp}:$RemoteDir/library_app"
scp -i $SshKey -r target\x86_64-unknown-linux-musl\release\assets\* "$DropletUser@${DropletIp}:$RemoteDir/assets/"
scp -i $SshKey (Join-Path $ScriptDir "library_app.service") "$DropletUser@${DropletIp}:$RemoteDir/library_app.service"
scp -i $SshKey (Join-Path $ScriptDir "Caddyfile") "$DropletUser@${DropletIp}:/etc/caddy/Caddyfile"

Write-Host "`n[5/5] Activating 24/7 background systemd service on Droplet..." -ForegroundColor Yellow
ssh -i $SshKey "$DropletUser@$DropletIp" "chmod +x $RemoteDir/library_app && cp $RemoteDir/library_app.service /etc/systemd/system/ && systemctl daemon-reload && systemctl enable library_app && systemctl restart library_app && (systemctl reload caddy || systemctl restart caddy) && sleep 1 && systemctl --no-pager status library_app"

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host " Deployment complete! Server live at:" -ForegroundColor Green
Write-Host "   https://lib.nciphered.com" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
