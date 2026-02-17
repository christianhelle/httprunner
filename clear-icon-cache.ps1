# Script to clear Windows icon cache
# This helps when Windows doesn't show updated icons for executables

Write-Host "Clearing Windows Icon Cache..." -ForegroundColor Yellow
Write-Host ""

# Stop Explorer
Write-Host "Stopping Windows Explorer..." -ForegroundColor Cyan
Stop-Process -Name explorer -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Find and delete icon cache files
Write-Host "Deleting icon cache files..." -ForegroundColor Cyan
$iconCachePaths = @(
    "$env:LOCALAPPDATA\IconCache.db",
    "$env:LOCALAPPDATA\Microsoft\Windows\Explorer\iconcache_*.db",
    "$env:LOCALAPPDATA\Microsoft\Windows\Explorer\thumbcache_*.db"
)

foreach ($pattern in $iconCachePaths) {
    $files = Get-Item $pattern -ErrorAction SilentlyContinue
    foreach ($file in $files) {
        try {
            Remove-Item $file.FullName -Force -ErrorAction Stop
            Write-Host "  Deleted: $($file.Name)" -ForegroundColor Green
        } catch {
            Write-Host "  Could not delete: $($file.Name) - $($_.Exception.Message)" -ForegroundColor Red
        }
    }
}

# Restart Explorer
Write-Host ""
Write-Host "Restarting Windows Explorer..." -ForegroundColor Cyan
Start-Process explorer.exe

Write-Host ""
Write-Host "Icon cache cleared! The GUI executable icon should now display correctly." -ForegroundColor Green
Write-Host "If the icon still doesn't appear, try:" -ForegroundColor Yellow
Write-Host "  1. Right-click the executable > Properties > Change icon > OK" -ForegroundColor Yellow
Write-Host "  2. Reboot your computer" -ForegroundColor Yellow
