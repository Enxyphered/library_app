@echo off
setlocal enabledelayedexpansion

set "DROPLET_USER=root"
set "DROPLET_IP=209.97.164.167"
set "SSH_KEY=%USERPROFILE%\.ssh\homekey"

set "THREADS=2"
set "CONNECTIONS=100"
set "DURATION=10s"
set "TARGET_URL=http://127.0.0.1:3000/login"

if not "%~1"=="" set "TARGET_URL=%~1"
if not "%~2"=="" set "CONNECTIONS=%~2"
if not "%~3"=="" set "DURATION=%~3"

if not exist "%SSH_KEY%" (
    echo [ERROR] SSH Key not found at %SSH_KEY%!
    exit /b 1
)

echo ==========================================================
echo  Athenaeum Library App - Requests/Sec Benchmark (wrk)
echo  Droplet Host: %DROPLET_USER%@%DROPLET_IP%
echo  Endpoint:     %TARGET_URL%
echo  Load:         %CONNECTIONS% concurrent connections for %DURATION%
echo ==========================================================
echo.

echo Running high-concurrency benchmark directly on the Droplet...
echo (Tests raw server throughput without home internet latency limits)
echo.

ssh -i "%SSH_KEY%" %DROPLET_USER%@%DROPLET_IP% "wrk -t%THREADS% -c%CONNECTIONS% -d%DURATION% --latency %TARGET_URL%"

echo.
echo ==========================================================
echo Benchmark completed!
echo ==========================================================
