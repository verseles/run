#!/usr/bin/env bash
# Copyright (C) 2025 Verseles
# SPDX-License-Identifier: AGPL-3.0

set -euo pipefail

REPO="verseles/run"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="run"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Icons
INFO="🔍"
PACKAGE="📦"
SUCCESS="✓"
WARNING="⚠"
ERROR="❌"

print_info() {
    echo -e "${BLUE}${INFO} $1${NC}"
}

print_success() {
    echo -e "${GREEN}${SUCCESS} $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}${WARNING} $1${NC}"
}

print_error() {
    echo -e "${RED}${ERROR} $1${NC}"
}

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="macos"
            ;;
        mingw*|msys*|cygwin*)
            OS="windows"
            ;;
        *)
            print_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    print_info "Detected platform: ${PLATFORM}"
}

# Get the latest release version
get_latest_version() {
    print_info "Fetching latest version..."

    LATEST_VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name"' | \
        sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

    if [ -z "$LATEST_VERSION" ]; then
        print_error "Failed to fetch latest version"
        exit 1
    fi

    print_info "Latest version: ${LATEST_VERSION}"
}

# Download and verify binary
download_binary() {
    local asset_name="run-${PLATFORM}"
    if [ "$OS" = "windows" ]; then
        asset_name="${asset_name}.exe"
    fi

    local download_url="https://github.com/${REPO}/releases/download/${LATEST_VERSION}/${asset_name}"
    local checksum_url="${download_url}.sha256"

    print_info "Downloading ${asset_name}..."

    local tmp_dir=$(mktemp -d)
    local tmp_binary="${tmp_dir}/${BINARY_NAME}"
    local tmp_checksum="${tmp_dir}/${asset_name}.sha256"

    # Download binary
    if ! curl -sL "$download_url" -o "$tmp_binary"; then
        print_error "Failed to download binary"
        rm -rf "$tmp_dir"
        exit 1
    fi

    # Download and verify checksum
    print_info "Verifying checksum..."
    if curl -sL "$checksum_url" -o "$tmp_checksum" 2>/dev/null; then
        cd "$tmp_dir"
        if command -v sha256sum &> /dev/null; then
            if sha256sum -c "$tmp_checksum" --status 2>/dev/null; then
                print_success "Checksum verified"
            else
                print_warning "Checksum verification failed (continuing anyway)"
            fi
        elif command -v shasum &> /dev/null; then
            if shasum -a 256 -c "$tmp_checksum" --status 2>/dev/null; then
                print_success "Checksum verified"
            else
                print_warning "Checksum verification failed (continuing anyway)"
            fi
        else
            print_warning "No checksum tool found, skipping verification"
        fi
        cd - > /dev/null
    else
        print_warning "Could not download checksum file, skipping verification"
    fi

    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"

    # Install binary
    print_info "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
    mv "$tmp_binary" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Cleanup
    rm -rf "$tmp_dir"
}

# Check if directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        print_warning "${INSTALL_DIR} is not in your PATH"
        echo ""
        echo "Add the following line to your shell configuration file:"
        echo ""

        local shell_name=$(basename "$SHELL")
        case "$shell_name" in
            bash)
                echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
                echo "  source ~/.bashrc"
                ;;
            zsh)
                echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
                echo "  source ~/.zshrc"
                ;;
            fish)
                echo "  fish_add_path ~/.local/bin"
                ;;
            *)
                echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
                ;;
        esac
        echo ""
    fi
}

# Main installation flow
main() {
    echo ""
    echo "  🚀 run - Universal Task Runner Installer"
    echo "  ==========================================="
    echo ""

    detect_platform
    get_latest_version
    download_binary
    check_path

    print_success "Installation complete!"
    echo ""
    echo "  Run 'run --help' to get started"
    echo ""
}

main "$@"
