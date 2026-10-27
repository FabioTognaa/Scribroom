#!/bin/sh
# Install scrib from the latest GitHub release.
#
#   curl -fsSL https://github.com/FabioTognaa/Scribroom/releases/latest/download/install.sh | sh
#
# SCRIB_INSTALL_BASE_URL overrides the download location (mirrors and tests).
set -eu

repo="FabioTognaa/Scribroom"
binary="scrib"
asset="scrib-aarch64-apple-darwin.tar.gz"
base_url="${SCRIB_INSTALL_BASE_URL:-https://github.com/$repo/releases/latest/download}"

fail() {
    printf 'scrib install: %s\n' "$*" >&2
    exit 1
}

[ "$(uname -s)" = "Darwin" ] || fail "prebuilt binaries are macOS only; build from source: https://github.com/$repo"
[ "$(uname -m)" = "arm64" ] || fail "prebuilt binaries need Apple Silicon; build from source: https://github.com/$repo"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl -fsSL --retry 3 -o "$tmp/$asset" "$base_url/$asset" ||
    fail "cannot download $base_url/$asset"
curl -fsSL --retry 3 -o "$tmp/$asset.sha256" "$base_url/$asset.sha256" ||
    fail "cannot download $base_url/$asset.sha256"
(cd "$tmp" && shasum -a 256 -c "$asset.sha256") ||
    fail "checksum verification failed; nothing was installed"

tar -xzf "$tmp/$asset" -C "$tmp"
[ -f "$tmp/$binary" ] || fail "$asset does not contain $binary"

if command -v brew >/dev/null 2>&1; then
    destination="$(brew --prefix)/bin"
else
    destination="/usr/local/bin"
fi

if [ -w "$destination" ]; then
    install -m 755 "$tmp/$binary" "$destination/$binary"
elif command -v sudo >/dev/null 2>&1; then
    sudo install -m 755 "$tmp/$binary" "$destination/$binary" ||
        fail "cannot write to $destination"
else
    fail "$destination is not writable and sudo is unavailable"
fi

printf 'installed %s\n' "$destination/$binary"
"$destination/$binary" --version
printf 'next: %s start\n' "$binary"
