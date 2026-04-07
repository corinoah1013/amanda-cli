#!/bin/bash
# Build Debian package for amanda-watch

set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
PKG_DIR="target/debian-pkg"

mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/local/bin"
mkdir -p "$PKG_DIR/usr/share/doc/amanda-watch"

# Copy binary
cp target/release/amanda-watch "$PKG_DIR/usr/local/bin/"

# Control file
cat > "$PKG_DIR/DEBIAN/control" << EOF
Package: amanda-watch
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Maintainer: corinoah1013 <corinoah1013@github.com>
Description: Process and resource monitoring for Amanda OS
 Amanda-watch is a system monitoring tool designed for scripting,
 automation, and pipelines. It provides structured output (JSON, CSV),
 configurable alerts, and immutable audit snapshots.
Homepage: https://github.com/corinoah1013/amanda-cli
EOF

# Copyright
cat > "$PKG_DIR/usr/share/doc/amanda-watch/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: amanda-watch
Source: https://github.com/corinoah1013/amanda-cli

Files: *
Copyright: 2026 corinoah1013
License: MIT
EOF

# Build package
dpkg-deb --build "$PKG_DIR" "target/amanda-watch_${VERSION}_${ARCH}.deb"

rm -rf "$PKG_DIR"
echo "✓ Created: target/amanda-watch_${VERSION}_${ARCH}.deb"
