# Build script for httprunner with version generation
param(
    [string]$BuildType = "Debug"
)

Write-Host "Generating version information..." -ForegroundColor Green
& powershell -ExecutionPolicy Bypass -File generate-version.ps1 -Version 1.0.0

Write-Host "Building httprunner..." -ForegroundColor Green
if ($BuildType -eq "Release") {
    zig build -Doptimize=ReleaseFast
} else {
    zig build
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build completed successfully!" -ForegroundColor Green
    Write-Host "Testing version display:" -ForegroundColor Yellow
    & .\zig-out\bin\httprunner.exe --version
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
