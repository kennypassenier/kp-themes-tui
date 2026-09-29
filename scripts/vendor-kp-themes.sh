#!/usr/bin/env bash
# Moves the pin to another kp-themes release: downloads its tokens.tar,
# shows its sha256, unpacks it into vendor/kp-themes/ and rewrites vendor/PIN.
# Then regenerate (npm run generate) and read the diff before committing.
#
#   scripts/vendor-kp-themes.sh 8.1.0
set -euo pipefail
v="${1:?usage: scripts/vendor-kp-themes.sh <kp-themes version>}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
url="https://github.com/kennypassenier/kp-themes/releases/download/v$v/tokens.tar"
curl -fsSL "$url" -o "$tmp/tokens.tar"
curl -fsSL "${url%/tokens.tar}/SHA256SUMS" -o "$tmp/SHA256SUMS"
sha="$(sha256sum "$tmp/tokens.tar" | cut -d' ' -f1)"
sums="$(sha256sum "$tmp/SHA256SUMS" | cut -d' ' -f1)"
rm -rf "$root/vendor/kp-themes"
mkdir -p "$root/vendor/kp-themes"
tar -xf "$tmp/tokens.tar" -C "$root/vendor/kp-themes"
printf '# The kp-themes release this repository builds from. scripts/vendor-kp-themes.sh moves it;\n# gates/check-vendor.mjs holds vendor/kp-themes/ to it. sums is the sha256 of that\n# release'"'"'s SHA256SUMS, which scripts/fetch-fonts.sh checks the fonts against.\nversion %s\nsha256 %s\nsums %s\nurl %s\n' "$v" "$sha" "$sums" "$url" >"$root/vendor/PIN"
echo "vendor/kp-themes is now kp-themes $v (sha256 $sha)"
