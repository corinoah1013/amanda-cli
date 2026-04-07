#!/bin/bash
# Amanda OS CLI Tools - Installer Script
# Usage: curl -fsSL https://raw.githubusercontent.com/corinoah1013/amanda-cli/main/install.sh | bash

set -e

REPO="corinoah1013/amanda-cli"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TOOL="${TOOL:-amanda-watch}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64) PLATFORM="linux-x64" ;;
                aarch64|arm64) PLATFORM="linux-arm64" ;;
                *) echo "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
            esac
            ;;
        darwin)
            case "$ARCH" in
                x86_64) PLATFORM="macos-x64" ;;
                arm64) PLATFORM="macos-arm64" ;;
                *) echo "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
            esac
            ;;
        *)
            echo "${RED}Unsupported OS: $OS${NC}"
            exit 1
            ;;
    esac
}

# Get latest release version
get_latest_version() {
    curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
download() {
    VERSION=$(get_latest_version)
    if [ -z "$VERSION" ]; then
        echo "${RED}Failed to get latest version${NC}"
        exit 1
    fi
    
    echo "${GREEN}Installing $TOOL $VERSION for $PLATFORM...${NC}"
    
    URL="https://github.com/$REPO/releases/download/$VERSION/$TOOL-$PLATFORM.tar.gz"
    TMP_DIR=$(mktemp -d)
    
    echo "Downloading from $URL..."
    if ! curl -fsSL "$URL" -o "$TMP_DIR/$TOOL.tar.gz"; then
        echo "${RED}Download failed${NC}"
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    echo "Extracting..."
    tar xzf "$TMP_DIR/$TOOL.tar.gz" -C "$TMP_DIR"
    
    # Install
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_DIR/$TOOL" "$INSTALL_DIR/"
    else
        echo "${YELLOW}Installing to $INSTALL_DIR (requires sudo)...${NC}"
        sudo mv "$TMP_DIR/$TOOL" "$INSTALL_DIR/"
    fi
    
    chmod +x "$INSTALL_DIR/$TOOL"
    rm -rf "$TMP_DIR"
    
    echo "${GREEN}✓ $TOOL installed successfully to $INSTALL_DIR/$TOOL${NC}"
    echo ""
    echo "Run '$TOOL --version' to verify installation"
}

# Main
main() {
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║           Amanda OS CLI Tools Installer                      ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    detect_platform
    download
    
    echo ""
    echo "${GREEN}Installation complete!${NC}"
    echo ""
    echo "Quick start:"
    echo "  $TOOL --help           # Show help"
    echo "  $TOOL --top 10         # Show top 10 processes"
    echo "  $TOOL --system         # Include system resources"
}

main "$@"
