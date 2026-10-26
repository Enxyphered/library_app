@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0.."

echo ========================================================
echo  Athenaeum Library App - Production Release Build
echo ========================================================
echo.

echo [1/4] Configuring compiler flags for maximum host performance...
set "RUSTFLAGS=-C target-cpu=native"

echo.
echo [2/4] Compiling Rust release binary (opt-level=3, LTO=fat, panic=abort)...
cargo build --release
if errorlevel 1 (
    echo [ERROR] Cargo release compilation failed!
    exit /b %errorlevel%
)

echo.
echo [3/4] Bundling and fingerprinting static assets and Tailwind CSS...
topcoat asset bundle --release
if errorlevel 1 (
    echo [WARN] Topcoat asset bundle with --release failed or option not present, trying standard bundle...
    topcoat asset bundle
    if errorlevel 1 (
        echo [ERROR] Topcoat asset bundle failed!
        exit /b %errorlevel%
    )
)

echo.
echo [4/4] Verifying generated release artifacts...
if exist target\release\library_app.exe (
    echo [SUCCESS] Release executable generated successfully at:
    echo   target\release\library_app.exe
    for %%I in (target\release\library_app.exe) do (
        echo Binary size: %%~zI bytes
    )
) else (
    echo [ERROR] Expected binary target\release\library_app.exe was not found!
    exit /b 1
)

echo.
echo ========================================================
echo Production build complete!
echo To launch the high-performance server, run:
echo   target\release\library_app.exe
echo ========================================================
