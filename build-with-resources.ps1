# Optional script to compile Windows resources
# Requires Windows SDK and rc.exe to be available

param(
    [string]$Version = "1.0.0"
)

Write-Host "Attempting to compile Windows resource file..." -ForegroundColor Green

# Check if rc.exe is available
$rcPath = Get-Command rc.exe -ErrorAction SilentlyContinue

if ($null -eq $rcPath) {
    Write-Host "rc.exe not found. Windows SDK might not be installed or not in PATH." -ForegroundColor Yellow
    Write-Host "To add file version information, install Windows SDK or Visual Studio with C++ tools." -ForegroundColor Yellow
    exit 0
}

# Generate version and resource files first
Write-Host "Generating version information..." -ForegroundColor Green
& powershell -ExecutionPolicy Bypass -File generate-version.ps1 -Version $Version

# Compile the resource file
Write-Host "Compiling resource file..." -ForegroundColor Green
try {
    & rc.exe /fo src\httprunner.res src\httprunner.rc
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Resource file compiled successfully: src\httprunner.res" -ForegroundColor Green
        
        # Build with resource file
        Write-Host "Building with resource file..." -ForegroundColor Green
        $env:HTTPRUNNER_INCLUDE_RESOURCES = "1"
        zig build -Doptimize=ReleaseFast
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Build with resources completed successfully!" -ForegroundColor Green
            
            # Test the executable
            Write-Host "Testing version display:" -ForegroundColor Yellow
            & .\zig-out\bin\httprunner.exe --version
            
            Write-Host "`nChecking file properties..." -ForegroundColor Yellow
            if (Test-Path ".\zig-out\bin\httprunner.exe") {
                $fileInfo = Get-ItemProperty ".\zig-out\bin\httprunner.exe"
                Write-Host "File size: $($fileInfo.Length) bytes" -ForegroundColor Cyan
                Write-Host "Modified: $($fileInfo.LastWriteTime)" -ForegroundColor Cyan
            }
        } else {
            Write-Host "Build failed!" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "Resource compilation failed!" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "Error compiling resource file: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
