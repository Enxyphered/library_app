@echo off
setlocal enabledelayedexpansion

set "DROPLET_USER=root"
set "DROPLET_IP=209.97.164.167"
set "SSH_KEY=%USERPROFILE%\.ssh\homekey"
set "LINES=50"

if not "%~1"=="" set "LINES=%~1"

if not exist "%SSH_KEY%" (
    echo [ERROR] SSH Key not found at %SSH_KEY%!
    exit /b 1
)

echo ==========================================================
echo  Athenaeum Library App - Live Droplet Logs
echo  Host: %DROPLET_USER%@%DROPLET_IP%
echo  Showing last %LINES% lines (Ctrl+C to exit)
echo ==========================================================
echo.

ssh -i "%SSH_KEY%" -t %DROPLET_USER%@%DROPLET_IP% "journalctl -u library_app -n %LINES% -f"
