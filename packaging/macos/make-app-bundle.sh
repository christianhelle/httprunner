#!/usr/bin/env bash
# Wraps the httprunner-gui executable in a macOS .app bundle so Finder
# launches it as an application instead of running it inside Terminal.app.
#
# Usage: make-app-bundle.sh <httprunner-gui binary> <version> <output dir>
set -euo pipefail

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <httprunner-gui binary> <version> <output dir>" >&2
    exit 1
fi

binary="$1"
version="$2"
out_dir="$3"

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
app="$out_dir/HTTP File Runner.app"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

cp "$binary" "$app/Contents/MacOS/httprunner-gui"
chmod +x "$app/Contents/MacOS/httprunner-gui"

sed "s/__VERSION__/$version/g" "$script_dir/Info.plist" > "$app/Contents/Info.plist"

# iconutil only exists on macOS; the bundle still works without an icon.
if command -v iconutil > /dev/null; then
    iconutil -c icns "$repo_root/images/httprunner.iconset" \
        -o "$app/Contents/Resources/httprunner.icns"
fi
