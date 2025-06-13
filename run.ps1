# PowerShell script to run HTTP Runner with proper UTF-8 encoding for emoji support
param(
    [Parameter(Mandatory=$true)]
    [string]$HttpFile
)

# Set console to UTF-8 encoding for proper emoji display
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8

# Run the HTTP Runner
& ".\zig-out\bin\httprunner.exe" $HttpFile
