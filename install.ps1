# Copyright (C) 2025 Verseles
# SPDX-License-Identifier: AGPL-3.0
#
# Install script for 'run' CLI on Windows
# Usage: irm https://raw.githubusercontent.com/verseles/run/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "verseles/run"
$BinaryName = "run.exe"
$InstallDir = "$env:USERPROFILE\.local\bin"

function Write-Info { param($Message) Write-Host "🔍 $Message" -ForegroundColor Cyan }
function Write-Success { param($Message) Write-Host "✓ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠ $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }

function Get-Architecture {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        "X64" { return "x86_64" }
        "Arm64" { return "aarch64" }
        default { throw "Unsupported architecture: $arch" }
    }
}

function Get-LatestVersion {
    Write-Info "Fetching latest version..."
    $releaseUrl = "https://api.github.com/repos/$Repo/releases/latest"
    $release = Invoke-RestMethod -Uri $releaseUrl -Headers @{ "User-Agent" = "run-installer" }
    return $release.tag_name
}

function Install-Run {
    Write-Host ""
    Write-Host "  🚀 run - Universal Task Runner Installer" -ForegroundColor Magenta
    Write-Host "  ===========================================" -ForegroundColor Magenta
    Write-Host ""

    # Detect architecture
    $arch = Get-Architecture
    Write-Info "Detected architecture: windows-$arch"

    # Get latest version
    $version = Get-LatestVersion
    Write-Info "Latest version: $version"

    # Build asset name and URL
    $assetName = "run-windows-$arch.exe"
    $downloadUrl = "https://github.com/$Repo/releases/download/$version/$assetName"
    $checksumUrl = "$downloadUrl.sha256"

    # Create temp directory
    $tempDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "run-install-$(Get-Random)")
    $tempBinary = Join-Path $tempDir $BinaryName
    $tempChecksum = Join-Path $tempDir "$assetName.sha256"

    try {
        # Download binary
        Write-Info "Downloading $assetName..."
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempBinary -UseBasicParsing

        # Download and verify checksum
        Write-Info "Verifying checksum..."
        try {
            Invoke-WebRequest -Uri $checksumUrl -OutFile $tempChecksum -UseBasicParsing
            $expectedHash = (Get-Content $tempChecksum).Split(" ")[0].ToUpper()
            $actualHash = (Get-FileHash -Path $tempBinary -Algorithm SHA256).Hash.ToUpper()

            if ($expectedHash -eq $actualHash) {
                Write-Success "Checksum verified"
            } else {
                Write-Warning "Checksum mismatch (continuing anyway)"
            }
        } catch {
            Write-Warning "Could not verify checksum (continuing anyway)"
        }

        # Create install directory
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
            Write-Info "Created directory: $InstallDir"
        }

        # Install binary
        $installPath = Join-Path $InstallDir $BinaryName
        Copy-Item -Path $tempBinary -Destination $installPath -Force
        Write-Success "Installed to $installPath"

        # Check if directory is in PATH
        $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($userPath -notlike "*$InstallDir*") {
            Write-Warning "$InstallDir is not in your PATH"
            Write-Host ""
            Write-Host "  To add it permanently, run:" -ForegroundColor Yellow
            Write-Host ""
            Write-Host "    [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallDir', 'User')" -ForegroundColor White
            Write-Host ""
            Write-Host "  Or add it to the current session:" -ForegroundColor Yellow
            Write-Host ""
            Write-Host "    `$env:PATH += ';$InstallDir'" -ForegroundColor White
            Write-Host ""

            # Offer to add to PATH automatically
            $addToPath = Read-Host "  Add to PATH automatically? (y/N)"
            if ($addToPath -eq "y" -or $addToPath -eq "Y") {
                $newPath = "$userPath;$InstallDir"
                [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
                $env:PATH += ";$InstallDir"
                Write-Success "Added to PATH (restart terminal for changes to take effect)"
            }
        }

        Write-Host ""
        Write-Success "Installation complete!"
        Write-Host ""
        Write-Host "  Run 'run --help' to get started" -ForegroundColor Cyan
        Write-Host ""

    } finally {
        # Cleanup
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Run installer
Install-Run
