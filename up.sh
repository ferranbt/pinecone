#!/usr/bin/env bash
# Install the `pinecone` CLI from a GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/ferranbt/pinecone/main/up.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/ferranbt/pinecone/main/up.sh | bash -s -- --version v0.2.4
#
set -euo pipefail

REPO="ferranbt/pinecone"
PINECONE_DIR="${PINECONE_DIR:-$HOME/.pinecone}"
BIN_DIR="$PINECONE_DIR/bin"
VERSION=""

say()  { printf '\033[1;32m%s\033[0m %s\n' "pinecone:" "$*"; }
warn() { printf '\033[1;33m%s\033[0m %s\n' "pinecone:" "$*" >&2; }
die()  { printf '\033[1;31m%s\033[0m %s\n' "pinecone:" "$*" >&2; exit 1; }

usage() {
  cat <<EOF
Install the pinecone CLI.

Usage: up.sh [options]

Options:
  -v, --version <tag>   Install a specific release tag (e.g. v0.2.4). Default: latest.
  -h, --help            Show this help.

Environment:
  PINECONE_DIR          Install prefix (default: \$HOME/.pinecone).
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    -v|--version) VERSION="${2:-}"; shift 2 ;;
    -h|--help)    usage; exit 0 ;;
    *)            die "unknown option: $1 (see --help)" ;;
  esac
done

# A downloader that works with either curl or wget.
download() { # <url> <output>
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1" -o "$2"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$2" "$1"
  else
    die "need curl or wget to download."
  fi
}
fetch() { # <url> -> stdout
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1"
  else
    wget -qO- "$1"
  fi
}

# Map the platform to one of the targets we publish binaries for.
os="$(uname -s)"
arch="$(uname -m)"
case "$os/$arch" in
  Linux/x86_64)        target="x86_64-unknown-linux-gnu" ;;
  Darwin/arm64)        target="aarch64-apple-darwin" ;;
  Darwin/aarch64)      target="aarch64-apple-darwin" ;;
  *) die "no prebuilt binary for $os/$arch (published: Linux x86_64, macOS arm64). Build from source: cargo install --git https://github.com/$REPO pinecone" ;;
esac

# Resolve the version: the given tag, or the latest release.
if [ -z "$VERSION" ]; then
  say "resolving latest release…"
  VERSION="$(fetch "https://api.github.com/repos/$REPO/releases/latest" \
    | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
  [ -n "$VERSION" ] || die "could not determine the latest release."
fi
case "$VERSION" in v*) ;; *) VERSION="v$VERSION" ;; esac  # tags are v-prefixed

archive="pinecone-${VERSION}-${target}.tar.gz"
url="https://github.com/$REPO/releases/download/${VERSION}/${archive}"

say "installing pinecone $VERSION ($target)…"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
download "$url" "$tmp/$archive" || die "download failed: $url"
tar -xzf "$tmp/$archive" -C "$tmp"
[ -f "$tmp/pinecone" ] || die "archive did not contain a pinecone binary."

mkdir -p "$BIN_DIR"
mv "$tmp/pinecone" "$BIN_DIR/pinecone"
chmod +x "$BIN_DIR/pinecone"
say "installed to $BIN_DIR/pinecone"

# Add the bin dir to PATH via the shell profile, if it isn't already reachable.
if ! command -v pinecone >/dev/null 2>&1 || [ "$(command -v pinecone)" != "$BIN_DIR/pinecone" ]; then
  case "${SHELL:-}" in
    */zsh)  profile="$HOME/.zshrc" ;;
    */bash) profile="$HOME/.bashrc" ;;
    *)      profile="$HOME/.profile" ;;
  esac
  line="export PATH=\"$BIN_DIR:\$PATH\""
  if [ -f "$profile" ] && grep -qF "$BIN_DIR" "$profile"; then
    :
  else
    printf '\n# pinecone\n%s\n' "$line" >> "$profile"
    say "added $BIN_DIR to PATH in $profile"
  fi
  warn "restart your shell or run:  export PATH=\"$BIN_DIR:\$PATH\""
fi

say "done — try:  pinecone --help"
