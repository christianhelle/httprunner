#!/usr/bin/env bash
# Tests for make-app-bundle.sh. Finder runs a bare executable inside
# Terminal.app, so the GUI must ship as an .app bundle to open on its own.
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

printf '#!/bin/sh\n' > "$work/httprunner-gui"
chmod +x "$work/httprunner-gui"

"$script_dir/make-app-bundle.sh" "$work/httprunner-gui" "1.2.3" "$work/out"

app="$work/out/HTTP File Runner.app"
plist="$app/Contents/Info.plist"

[ -x "$app/Contents/MacOS/httprunner-gui" ] || fail "executable missing from bundle"
[ -f "$plist" ] || fail "Info.plist missing"
grep -q '<string>httprunner-gui</string>' "$plist" || fail "CFBundleExecutable not set"
grep -q '<string>1.2.3</string>' "$plist" || fail "version not set"
grep -q '<string>APPL</string>' "$plist" || fail "CFBundlePackageType not APPL"
if grep -q '__VERSION__' "$plist"; then fail "version placeholder left in Info.plist"; fi

if command -v plutil > /dev/null; then
    plutil -lint "$plist" > /dev/null || fail "Info.plist is not valid"
fi
if command -v iconutil > /dev/null; then
    [ -f "$app/Contents/Resources/httprunner.icns" ] || fail "icon missing"
fi

echo "PASS"
