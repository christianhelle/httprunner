#!/usr/bin/env bash
# Signs an app bundle with a Developer ID certificate and notarizes it, so
# Gatekeeper lets users open the downloaded app from Finder.
#
# Usage: sign-and-notarize.sh <path to .app>
#
# Credentials come from the environment. When MACOS_CERTIFICATE is unset the
# bundle is left unsigned, so releases still build without Apple credentials.
#   MACOS_CERTIFICATE           base64-encoded Developer ID Application .p12
#   MACOS_CERTIFICATE_PASSWORD  password of the .p12
#   MACOS_SIGNING_IDENTITY      e.g. "Developer ID Application: Name (TEAMID)"
#   APPLE_ID                    Apple ID used for notarization
#   APPLE_TEAM_ID               Apple developer team ID
#   APPLE_APP_PASSWORD          app-specific password for APPLE_ID
set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <path to .app>" >&2
    exit 1
fi

app="$1"

if [ ! -d "$app" ]; then
    echo "App bundle not found: $app" >&2
    exit 1
fi

if [ -z "${MACOS_CERTIFICATE:-}" ]; then
    echo "MACOS_CERTIFICATE is not set, skipping signing and notarization"
    exit 0
fi

missing=0
for var in MACOS_CERTIFICATE_PASSWORD MACOS_SIGNING_IDENTITY APPLE_ID APPLE_TEAM_ID APPLE_APP_PASSWORD; do
    if [ -z "${!var:-}" ]; then
        echo "$var must be set to sign and notarize" >&2
        missing=1
    fi
done
[ "$missing" -eq 0 ] || exit 1

work="$(mktemp -d)"
keychain="$work/signing.keychain-db"
keychain_password="$(uuidgen)"

cleanup() {
    security delete-keychain "$keychain" > /dev/null 2>&1 || true
    rm -rf "$work"
}
trap cleanup EXIT

echo "$MACOS_CERTIFICATE" | base64 --decode > "$work/certificate.p12"

security create-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security import "$work/certificate.p12" -k "$keychain" \
    -P "$MACOS_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple: -s \
    -k "$keychain_password" "$keychain" > /dev/null
security list-keychains -d user -s "$keychain" $(security list-keychains -d user | tr -d '"')

codesign --force --options runtime --timestamp \
    --keychain "$keychain" --sign "$MACOS_SIGNING_IDENTITY" "$app"
codesign --verify --strict --verbose=2 "$app"

# notarytool takes a zip; the ticket is then stapled to the app itself so it
# stays valid inside the release tarball and works offline.
ditto -c -k --keepParent "$app" "$work/notarize.zip"
xcrun notarytool submit "$work/notarize.zip" \
    --apple-id "$APPLE_ID" \
    --team-id "$APPLE_TEAM_ID" \
    --password "$APPLE_APP_PASSWORD" \
    --wait
xcrun stapler staple "$app"
xcrun stapler validate "$app"
