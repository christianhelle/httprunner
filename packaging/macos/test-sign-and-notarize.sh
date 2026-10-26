#!/usr/bin/env bash
# Tests for sign-and-notarize.sh. Signing needs Apple credentials, so these
# cover the guard rails: releases still build when no credentials are
# configured, and a half-configured setup fails instead of shipping unsigned.
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

app="$work/HTTP File Runner.app"
mkdir -p "$app/Contents/MacOS"

unset MACOS_CERTIFICATE MACOS_CERTIFICATE_PASSWORD MACOS_SIGNING_IDENTITY \
    APPLE_ID APPLE_TEAM_ID APPLE_APP_PASSWORD

# No credentials: skip signing and succeed
output="$("$script_dir/sign-and-notarize.sh" "$app" 2>&1)" ||
    fail "should succeed when no signing credentials are configured"
echo "$output" | grep -qi "skipping" || fail "should report that signing was skipped"

# Certificate without the rest of the credentials: refuse
if MACOS_CERTIFICATE="abc" "$script_dir/sign-and-notarize.sh" "$app" > /dev/null 2>&1; then
    fail "should fail when signing credentials are incomplete"
fi

# Any other credential without the certificate: refuse
for var in MACOS_CERTIFICATE_PASSWORD MACOS_SIGNING_IDENTITY APPLE_ID APPLE_TEAM_ID APPLE_APP_PASSWORD; do
    if env "$var=value" "$script_dir/sign-and-notarize.sh" "$app" > /dev/null 2>&1; then
        fail "should fail when only $var is set"
    fi
done

# Missing bundle: refuse
if "$script_dir/sign-and-notarize.sh" "$work/missing.app" > /dev/null 2>&1; then
    fail "should fail when the app bundle does not exist"
fi

echo "PASS"
