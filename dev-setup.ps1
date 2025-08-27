# Development Setup Script for HTTP File Runner
# This script helps set up the development environment

param(
    [switch]$Install,
    [switch]$Build,
    [switch]$Test,
    [switch]$Clean,
    [switch]$Format,
    [switch]$Help
)

function Show-Help {
    Write-Host "HTTP File Runner Development Setup" -ForegroundColor Cyan
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\dev-setup.ps1 [options]"
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -Install    Install/check Zig installation"
    Write-Host "  -Build      Build the project"
    Write-Host "  -Test       Run tests"
    Write-Host "  -Clean      Clean build artifacts"
    Write-Host "  -Format     Format code"
    Write-Host "  -Help       Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Green
    Write-Host "  .\dev-setup.ps1 -Install -Build -Test"
    Write-Host "  .\dev-setup.ps1 -Format"
    Write-Host "  .\dev-setup.ps1 -Clean -Build"
}

function Test-ZigInstallation {
    Write-Host "Checking Zig installation..." -ForegroundColor Yellow
    
    try {
        $zigVersion = zig version
        Write-Host "✅ Zig is installed: $zigVersion" -ForegroundColor Green
        
        # Check if version is compatible (0.15.1 or later)
        if ($zigVersion -match "(\d+)\.(\d+)\.(\d+)") {
            $major = [int]$Matches[1]
            $minor = [int]$Matches[2]
            
            if ($major -eq 0 -and $minor -lt 15) {
                Write-Warning "⚠️  Zig version $zigVersion detected. Version 0.15.1 is recommended."
            }
        }
        return $true
    }
    catch {
        Write-Host "❌ Zig is not installed or not in PATH" -ForegroundColor Red
        Write-Host "Please install Zig from: https://ziglang.org/download/" -ForegroundColor Yellow
        return $false
    }
}

function Install-Dependencies {
    Write-Host "Setting up development environment..." -ForegroundColor Yellow
    
    if (-not (Test-ZigInstallation)) {
        return $false
    }
    
    # Check Git
    try {
        git --version | Out-Null
        Write-Host "✅ Git is available" -ForegroundColor Green
    }
    catch {
        Write-Host "❌ Git is not installed" -ForegroundColor Red
        return $false
    }
    
    # Check if we're in a git repository
    if (Test-Path ".git") {
        Write-Host "✅ Git repository detected" -ForegroundColor Green
    }
    else {
        Write-Host "⚠️  Not in a git repository" -ForegroundColor Yellow
    }
    
    Write-Host "✅ Development environment is ready!" -ForegroundColor Green
    return $true
}

function Build-Project {
    Write-Host "Building project..." -ForegroundColor Yellow
    
    if (-not (Test-ZigInstallation)) {
        return $false
    }
    
    try {
        Write-Host "Running: zig build" -ForegroundColor Gray
        zig build
        
        if (Test-Path "zig-out/bin/httprunner.exe") {
            Write-Host "✅ Build successful! Binary created at zig-out/bin/httprunner.exe" -ForegroundColor Green
            
            # Show binary info
            $binary = Get-Item "zig-out/bin/httprunner.exe"
            Write-Host "Binary size: $([math]::Round($binary.Length / 1KB, 2)) KB" -ForegroundColor Gray
            
            return $true
        }
        else {
            Write-Host "❌ Build completed but binary not found" -ForegroundColor Red
            return $false
        }
    }
    catch {
        Write-Host "❌ Build failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Test-Project {
    Write-Host "Running tests..." -ForegroundColor Yellow
    
    if (-not (Test-ZigInstallation)) {
        return $false
    }
    
    try {
        Write-Host "Running: zig build test" -ForegroundColor Gray
        zig build test
        Write-Host "✅ All tests passed!" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "❌ Tests failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Clean-Project {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    
    $dirsToClean = @("zig-out", "zig-cache")
    
    foreach ($dir in $dirsToClean) {
        if (Test-Path $dir) {
            Remove-Item $dir -Recurse -Force
            Write-Host "✅ Removed $dir" -ForegroundColor Green
        }
        else {
            Write-Host "ℹ️  $dir does not exist" -ForegroundColor Gray
        }
    }
    
    Write-Host "✅ Clean completed!" -ForegroundColor Green
}

function Format-Code {
    Write-Host "Formatting code..." -ForegroundColor Yellow
    
    if (-not (Test-ZigInstallation)) {
        return $false
    }
    
    try {
        Write-Host "Running: zig fmt ." -ForegroundColor Gray
        zig fmt .
        Write-Host "✅ Code formatting completed!" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "❌ Code formatting failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Main execution
if ($Help -or (-not $Install -and -not $Build -and -not $Test -and -not $Clean -and -not $Format)) {
    Show-Help
    exit 0
}

$success = $true

if ($Install) {
    $success = $success -and (Install-Dependencies)
}

if ($Clean) {
    Clean-Project
}

if ($Format) {
    $success = $success -and (Format-Code)
}

if ($Build) {
    $success = $success -and (Build-Project)
}

if ($Test) {
    $success = $success -and (Test-Project)
}

if ($success) {
    Write-Host "`n🎉 All operations completed successfully!" -ForegroundColor Green
    Write-Host "You can now run: .\zig-out\bin\httprunner.exe --help" -ForegroundColor Cyan
}
else {
    Write-Host "`n❌ Some operations failed. Please check the output above." -ForegroundColor Red
    exit 1
}
