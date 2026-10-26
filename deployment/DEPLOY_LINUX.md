# Linux Droplet Deployment Guide

## Automated 1-Click Deployment

From your local machine (Windows Command Prompt or PowerShell):

### Option A: Using the Batch Script
```cmd
deployment\deploy-to-droplet.bat
```

### Option B: Using PowerShell
```powershell
.\deployment\deploy-to-droplet.ps1
```

Both scripts automatically:
1. Connect to `root@209.97.164.167` using `-i %USERPROFILE%\.ssh\homekey` (or `$HOME\.ssh\homekey`).
2. Cross-compile an optimized musl-libc Linux binary on your PC (`cargo zigbuild --target x86_64-unknown-linux-musl --release` with AVX2/BMI2 target CPU).
3. Bundle fingerprinted static assets and Tailwind CSS directly from the Linux binary on your PC (`cargo run --example bundle_linux`).
4. Stop the remote `library_app` systemd service if running.
5. SCP upload only the pre-compiled ~7 MB binary, static assets, and configuration to `/opt/library_app` (no Rust toolchains, compilers, or build tools needed on the server).
6. Enable, restart, and verify the persistent 24/7 background `systemd` service (`library_app.service`).

---

## Verifying on the Droplet

SSH directly into the server:
```bash
ssh -i %USERPROFILE%\.ssh\homekey root@209.97.164.167
```

Check status and logs:
```bash
systemctl status library_app
journalctl -u library_app -f
```

The application is secured with Caddy reverse proxy and SSL, accessible at:
- **Production URL**: `https://lib.nciphered.com`
- **Direct App Port**: `http://209.97.164.167:3000`
