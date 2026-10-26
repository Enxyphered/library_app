@echo off
setlocal enabledelayedexpansion

set "DROPLET_USER=root"
set "DROPLET_IP=209.97.164.167"
set "SSH_KEY=%USERPROFILE%\.ssh\homekey"
set "REMOTE_DIR=/opt/library_app"
set "ZIG_PATH=%LOCALAPPDATA%\Programs\Python\Python314\Lib\site-packages\ziglang"

echo ==========================================================
echo  Athenaeum Library App - High-Performance Linux Deployer
echo  Target: %DROPLET_USER%@%DROPLET_IP% (Debian x86_64-v3)
echo ==========================================================
echo.

cd /d "%~dp0.."
set "SCRIPT_DIR=%~dp0"

if not exist "%SSH_KEY%" (
    echo [ERROR] SSH Key not found at: %SSH_KEY%
    exit /b 1
)

echo [1/5] Compiling Linux binary on PC targeting Droplet CPU (AVX2/BMI2)...
set "PATH=%ZIG_PATH%;%PATH%"
set "RUSTFLAGS=-C target-cpu=x86-64-v3"
cargo zigbuild --target x86_64-unknown-linux-musl --release
if errorlevel 1 (
    echo [ERROR] Cross-compilation failed!
    exit /b %errorlevel%
)

echo.
echo [2/5] Bundling Tailwind CSS & static assets from Linux binary...
cargo run --example bundle_linux
if errorlevel 1 (
    echo [ERROR] Asset bundle failed!
    exit /b %errorlevel%
)

echo.
echo [3/5] Stopping active service on Droplet...
ssh -i "%SSH_KEY%" -o StrictHostKeyChecking=accept-new %DROPLET_USER%@%DROPLET_IP% "if systemctl is-active --quiet library_app 2>/dev/null; then echo 'Stopping library_app service...'; systemctl stop library_app; fi; mkdir -p %REMOTE_DIR%/assets"

echo.
echo [4/5] Uploading Linux binary and assets to Droplet (~7MB total)...
scp -i "%SSH_KEY%" target\x86_64-unknown-linux-musl\release\library_app %DROPLET_USER%@%DROPLET_IP%:%REMOTE_DIR%/library_app
scp -i "%SSH_KEY%" -r target\x86_64-unknown-linux-musl\release\assets\* %DROPLET_USER%@%DROPLET_IP%:%REMOTE_DIR%/assets/
scp -i "%SSH_KEY%" "%SCRIPT_DIR%library_app.service" %DROPLET_USER%@%DROPLET_IP%:%REMOTE_DIR%/library_app.service
scp -i "%SSH_KEY%" "%SCRIPT_DIR%Caddyfile" %DROPLET_USER%@%DROPLET_IP%:/etc/caddy/Caddyfile

echo.
echo [5/5] Activating 24/7 background systemd service on Droplet...
ssh -i "%SSH_KEY%" %DROPLET_USER%@%DROPLET_IP% "chmod +x %REMOTE_DIR%/library_app && cp %REMOTE_DIR%/library_app.service /etc/systemd/system/ && systemctl daemon-reload && systemctl enable library_app && systemctl restart library_app && (systemctl reload caddy || systemctl restart caddy) && sleep 1 && systemctl --no-pager status library_app"

echo.
echo ==========================================================
echo Deployment complete! Server live at:
echo   https://lib.nciphered.com
echo ==========================================================
