# KeVoiceInput Windows Build Script
# PowerShell script for building Windows version
#
# SHERPA_LIB_PATH:
#   - Pass -SherpaPath, or set $env:SHERPA_LIB_PATH, or omit both:
#     after the first successful build, sherpa-rs stores DLLs under
#     %LOCALAPPDATA%\sherpa-rs\x86_64-pc-windows-msvc\... and this script
#     auto-fills SHERPA_LIB_PATH for the current session.
#   - -PersistUserEnv: also save SHERPA_LIB_PATH to your Windows user env
#     (visible in new terminals; current session already has $env: set).

param(
    [switch]$Clean = $false,
    [switch]$Dev = $false,
    [string]$SherpaPath = $env:SHERPA_LIB_PATH,
    [switch]$PersistUserEnv = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║         KeVoiceInput Windows Build Script                     ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# sherpa-rs download_binaries cache: dirs::cache_dir().join("sherpa-rs").join(target triple)
function Get-SherpaLibPathFromDownloadCache {
    $cacheRoot = Join-Path $env:LOCALAPPDATA "sherpa-rs"
    # dist.json currently ships only x86_64 Windows prebuilts
    $triple = "x86_64-pc-windows-msvc"
    $base = Join-Path $cacheRoot $triple
    if (-not (Test-Path $base)) {
        return $null
    }
    $dll = Get-ChildItem -Path $base -Recurse -Filter "sherpa-onnx-c-api.dll" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($dll) {
        return $dll.Directory.FullName
    }
    return $null
}

# Check prerequisites
function Test-Prerequisites {
    Write-Host "[1/7] Checking prerequisites..." -ForegroundColor Yellow

    # Check Rust
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Rust not found. Please install from https://rustup.rs/" -ForegroundColor Red
        exit 1
    }
    Write-Host "  ✓ Rust: $(rustc --version)" -ForegroundColor Green

    # Check Node.js/Bun
    if (Get-Command bun -ErrorAction SilentlyContinue) {
        Write-Host "  ✓ Bun: $(bun --version)" -ForegroundColor Green
        $script:PackageManager = "bun"
    } elseif (Get-Command npm -ErrorAction SilentlyContinue) {
        Write-Host "  ✓ npm: $(npm --version)" -ForegroundColor Green
        $script:PackageManager = "npm"
    } else {
        Write-Host "❌ Neither Bun nor npm found. Please install Node.js or Bun." -ForegroundColor Red
        exit 1
    }

    # Check Visual Studio Build Tools
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        Write-Host "  ✓ Visual Studio Build Tools found" -ForegroundColor Green
    } else {
        Write-Host "  ⚠  Visual Studio Build Tools not found (may be needed for some dependencies)" -ForegroundColor Yellow
    }

    Write-Host ""
}

# Check sherpa-onnx libraries
function Test-SherpaLibraries {
    Write-Host "[2/7] Checking sherpa-onnx libraries..." -ForegroundColor Yellow

    if ([string]::IsNullOrEmpty($SherpaPath)) {
        Write-Host "  ⚠  SHERPA_LIB_PATH not set" -ForegroundColor Yellow
        Write-Host "  Cargo may still download sherpa-onnx (download-binaries). After the first successful build," -ForegroundColor Yellow
        Write-Host "  re-run this script: it will auto-set SHERPA_LIB_PATH from:" -ForegroundColor Yellow
        Write-Host "    $([System.IO.Path]::Combine($env:LOCALAPPDATA, 'sherpa-rs', 'x86_64-pc-windows-msvc'))" -ForegroundColor Gray
        Write-Host "  Or pass -SherpaPath / set `$env:SHERPA_LIB_PATH to your sherpa-onnx\bin folder." -ForegroundColor Yellow
        Write-Host ""
        return
    }

    if (-not (Test-Path $SherpaPath)) {
        Write-Host "  ❌ Sherpa library path not found: $SherpaPath" -ForegroundColor Red
        exit 1
    }

    $requiredDlls = @(
        "sherpa-onnx-c-api.dll",
        "sherpa-onnx-cxx-api.dll",
        "onnxruntime.dll"
    )

    foreach ($dll in $requiredDlls) {
        $dllPath = Join-Path $SherpaPath $dll
        if (Test-Path $dllPath) {
            Write-Host "  ✓ Found: $dll" -ForegroundColor Green
        } else {
            Write-Host "  ⚠  Missing: $dll" -ForegroundColor Yellow
        }
    }

    Write-Host ""
}

# Clean build artifacts
function Clean-BuildArtifacts {
    if ($Clean) {
        Write-Host "[3/7] Cleaning build artifacts..." -ForegroundColor Yellow

        $targets = @(
            "$ProjectRoot\dist",
            "$ProjectRoot\src-tauri\target\release",
            "$ProjectRoot\node_modules\.vite"
        )

        foreach ($target in $targets) {
            if (Test-Path $target) {
                Write-Host "  Removing: $target" -ForegroundColor Gray
                Remove-Item -Path $target -Recurse -Force
            }
        }

        Write-Host "  ✓ Clean complete" -ForegroundColor Green
        Write-Host ""
    } else {
        Write-Host "[3/7] Skipping clean (use -Clean to clean build artifacts)" -ForegroundColor Gray
        Write-Host ""
    }
}

# Install dependencies
function Install-Dependencies {
    Write-Host "[4/7] Installing dependencies..." -ForegroundColor Yellow

    Set-Location $ProjectRoot

    if ($PackageManager -eq "bun") {
        & bun install
    } else {
        & npm install
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed to install dependencies" -ForegroundColor Red
        exit 1
    }

    Write-Host "  ✓ Dependencies installed" -ForegroundColor Green
    Write-Host ""
}

# Build frontend
function Build-Frontend {
    Write-Host "[5/7] Building frontend..." -ForegroundColor Yellow

    Set-Location $ProjectRoot

    if ($PackageManager -eq "bun") {
        & bun run build
    } else {
        & npm run build
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Frontend build failed" -ForegroundColor Red
        exit 1
    }

    Write-Host "  ✓ Frontend build complete" -ForegroundColor Green
    Write-Host ""
}

# Build Tauri application
function Build-TauriApp {
    Write-Host "[6/7] Building Tauri application..." -ForegroundColor Yellow

    Set-Location $ProjectRoot

    # Set environment variable if provided
    if (-not [string]::IsNullOrEmpty($SherpaPath)) {
        $env:SHERPA_LIB_PATH = $SherpaPath
        Write-Host "  Using SHERPA_LIB_PATH: $SherpaPath" -ForegroundColor Gray
    }

    if ($Dev) {
        Write-Host "  Starting development mode..." -ForegroundColor Gray
        if ($PackageManager -eq "bun") {
            & bun run tauri dev
        } else {
            & npm run tauri dev
        }
    } else {
        if ($PackageManager -eq "bun") {
            & bun run tauri build
        } else {
            & npm run tauri build
        }
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Tauri build failed" -ForegroundColor Red
        exit 1
    }

    Write-Host "  ✓ Tauri build complete" -ForegroundColor Green
    Write-Host ""
}

# Copy DLLs to output
function Copy-Dependencies {
    if ($Dev) {
        Write-Host "[7/7] Skipping DLL copy in dev mode" -ForegroundColor Gray
        return
    }

    Write-Host "[7/7] Copying DLL dependencies..." -ForegroundColor Yellow

    if ([string]::IsNullOrEmpty($SherpaPath) -or -not (Test-Path $SherpaPath)) {
        Write-Host "  ⚠  Skipping (SHERPA_LIB_PATH not set or invalid)" -ForegroundColor Yellow
        Write-Host ""
        return
    }

    $targetExe = "$ProjectRoot\src-tauri\target\release\kevoiceinput.exe"
    if (-not (Test-Path $targetExe)) {
        Write-Host "  ❌ Executable not found: $targetExe" -ForegroundColor Red
        Write-Host ""
        return
    }

    $targetDir = Split-Path -Parent $targetExe

    $dlls = Get-ChildItem -Path $SherpaPath -Filter "*.dll"
    $copiedCount = 0

    foreach ($dll in $dlls) {
        $destPath = Join-Path $targetDir $dll.Name
        if (-not (Test-Path $destPath)) {
            Copy-Item $dll.FullName -Destination $destPath -Force
            Write-Host "  ✓ Copied: $($dll.Name)" -ForegroundColor Green
            $copiedCount++
        }
    }

    if ($copiedCount -eq 0) {
        Write-Host "  All DLLs already present" -ForegroundColor Gray
    }

    Write-Host ""
}

# Show build summary
function Show-BuildSummary {
    Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║                    Build Complete!                            ║" -ForegroundColor Cyan
    Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""

    if (-not $Dev) {
        Write-Host "Output locations:" -ForegroundColor Yellow
        Write-Host "  Executable: src-tauri\target\release\kevoiceinput.exe" -ForegroundColor Green
        Write-Host "  MSI Installer: src-tauri\target\release\bundle\msi\" -ForegroundColor Green
        Write-Host ""

        $msiDir = "$ProjectRoot\src-tauri\target\release\bundle\msi"
        if (Test-Path $msiDir) {
            $msiFiles = Get-ChildItem -Path $msiDir -Filter "*.msi"
            if ($msiFiles.Count -gt 0) {
                foreach ($msi in $msiFiles) {
                    $sizeMB = [math]::Round($msi.Length / 1MB, 2)
                    Write-Host "  📦 $($msi.Name) ($sizeMB MB)" -ForegroundColor Cyan
                }
            }
        }

        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Yellow
        Write-Host "  1. Test the executable or install the MSI" -ForegroundColor Gray
        Write-Host "  2. Run comprehensive tests (see docs/WINDOWS_PORT.md)" -ForegroundColor Gray
        Write-Host "  3. Sign the executable (if releasing)" -ForegroundColor Gray
    }

    Write-Host ""
}

# Main execution
try {
    if ([string]::IsNullOrEmpty($SherpaPath)) {
        $cached = Get-SherpaLibPathFromDownloadCache
        if ($cached) {
            $SherpaPath = $cached
            $env:SHERPA_LIB_PATH = $cached
            Write-Host "[Auto] SHERPA_LIB_PATH (from sherpa-rs download cache): $cached" -ForegroundColor Green
            if ($PersistUserEnv) {
                [System.Environment]::SetEnvironmentVariable("SHERPA_LIB_PATH", $cached, "User")
                Write-Host "[Auto] Persisted SHERPA_LIB_PATH to user environment (new terminals)." -ForegroundColor Green
            }
            Write-Host ""
        }
    } elseif ($PersistUserEnv -and -not [string]::IsNullOrEmpty($SherpaPath)) {
        [System.Environment]::SetEnvironmentVariable("SHERPA_LIB_PATH", $SherpaPath, "User")
        Write-Host "[Auto] Persisted SHERPA_LIB_PATH to user environment: $SherpaPath" -ForegroundColor Green
        Write-Host ""
    }

    Test-Prerequisites
    Test-SherpaLibraries
    Clean-BuildArtifacts
    Install-Dependencies
    Build-Frontend
    Build-TauriApp
    Copy-Dependencies
    Show-BuildSummary
} catch {
    Write-Host ""
    Write-Host "❌ Build failed with error:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host ""
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}
