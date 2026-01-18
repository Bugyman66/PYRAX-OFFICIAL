#!/bin/bash
#
# Inferno CLI Installer
# https://github.com/PYRAX-Chain/PYRAX-OFFICIAL
#
# Usage:
#   curl -fsSL https://get.pyrax-devnet.org/cli | bash
#   wget -qO- https://get.pyrax-devnet.org/cli | bash
#
# Direct from GitHub:
#   curl -fsSL https://raw.githubusercontent.com/PYRAX-Chain/PYRAX-OFFICIAL/devnet/inferno-cli/install.sh | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REPO="PYRAX-Chain/PYRAX-OFFICIAL"
BINARY_NAME="inferno"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.inferno"

# Print banner
print_banner() {
    echo ""
    echo -e "${RED}  ___        __                       ${NC}"
    echo -e "${RED} |_ _|_ __  / _| ___ _ __ _ __   ___  ${NC}"
    echo -e "${RED}  | || '_ \\| |_ / _ \\ '__| '_ \\ / _ \\ ${NC}"
    echo -e "${RED}  | || | | |  _|  __/ |  | | | | (_) |${NC}"
    echo -e "${RED} |___|_| |_|_|  \\___|_|  |_| |_|\\___/ ${NC}"
    echo ""
    echo -e "  ${CYAN}Inferno CLI Installer${NC}"
    echo ""
}

# Print message
info() {
    echo -e "  ${BLUE}→${NC} $1"
}

success() {
    echo -e "  ${GREEN}✓${NC} $1"
}

warn() {
    echo -e "  ${YELLOW}!${NC} $1"
}

error() {
    echo -e "  ${RED}✗${NC} $1"
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS="linux";;
        Darwin*)    OS="darwin";;
        MINGW*|MSYS*|CYGWIN*) OS="windows";;
        *)          error "Unsupported operating system: $(uname -s)";;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   ARCH="x86_64";;
        aarch64|arm64)  ARCH="aarch64";;
        armv7l)         ARCH="armv7";;
        *)              error "Unsupported architecture: $(uname -m)";;
    esac
}

# Detect if running in WSL
detect_wsl() {
    if grep -qEi "(microsoft|wsl)" /proc/version 2>/dev/null; then
        WSL=true
        info "Detected WSL environment"
    else
        WSL=false
    fi
}

# Get latest release version
get_latest_version() {
    info "Fetching latest version..."
    
    if command -v curl &> /dev/null; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget &> /dev/null; then
        VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
    
    if [ -z "$VERSION" ]; then
        error "Could not determine latest version"
    fi
    
    # Remove 'v' prefix if present for CLI releases
    VERSION_NUM="${VERSION#v}"
    
    success "Latest version: $VERSION"
}

# Build download URL
build_download_url() {
    # Handle different naming conventions
    if [ "$OS" = "darwin" ]; then
        FILENAME="inferno-cli-${VERSION_NUM}-${OS}-${ARCH}.tar.gz"
    else
        FILENAME="inferno-cli-${VERSION_NUM}-${OS}-${ARCH}.tar.gz"
    fi
    
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"
}

# Download binary
download_binary() {
    info "Downloading ${FILENAME}..."
    
    TEMP_DIR=$(mktemp -d)
    TEMP_FILE="${TEMP_DIR}/${FILENAME}"
    
    if command -v curl &> /dev/null; then
        curl -fsSL -o "$TEMP_FILE" "$DOWNLOAD_URL" || error "Download failed. URL: $DOWNLOAD_URL"
    else
        wget -qO "$TEMP_FILE" "$DOWNLOAD_URL" || error "Download failed. URL: $DOWNLOAD_URL"
    fi
    
    success "Downloaded successfully"
}

# Extract binary
extract_binary() {
    info "Extracting..."
    
    cd "$TEMP_DIR"
    
    if [[ "$FILENAME" == *.tar.gz ]]; then
        tar -xzf "$TEMP_FILE" || error "Extraction failed"
    elif [[ "$FILENAME" == *.zip ]]; then
        unzip -q "$TEMP_FILE" || error "Extraction failed"
    fi
    
    if [ ! -f "$BINARY_NAME" ]; then
        # Try to find the binary
        BINARY_NAME=$(find . -name "inferno" -type f | head -n 1)
        if [ -z "$BINARY_NAME" ]; then
            error "Binary not found in archive"
        fi
    fi
    
    success "Extracted successfully"
}

# Install binary
install_binary() {
    info "Installing to ${INSTALL_DIR}..."
    
    # Check if we need sudo
    if [ -w "$INSTALL_DIR" ]; then
        cp "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    else
        warn "Requesting sudo access to install to ${INSTALL_DIR}"
        sudo cp "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
        sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    fi
    
    success "Installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Create config directory
create_config_dir() {
    if [ ! -d "$CONFIG_DIR" ]; then
        info "Creating config directory at ${CONFIG_DIR}..."
        mkdir -p "$CONFIG_DIR"
        success "Config directory created"
    fi
}

# Install shell completions
install_completions() {
    info "Installing shell completions..."
    
    SHELL_NAME=$(basename "$SHELL")
    
    case "$SHELL_NAME" in
        bash)
            COMP_DIR="${HOME}/.bash_completion.d"
            mkdir -p "$COMP_DIR"
            "${INSTALL_DIR}/inferno" completions bash > "${COMP_DIR}/inferno.bash" 2>/dev/null || true
            ;;
        zsh)
            COMP_DIR="${HOME}/.zsh/completions"
            mkdir -p "$COMP_DIR"
            "${INSTALL_DIR}/inferno" completions zsh > "${COMP_DIR}/_inferno" 2>/dev/null || true
            ;;
        fish)
            COMP_DIR="${HOME}/.config/fish/completions"
            mkdir -p "$COMP_DIR"
            "${INSTALL_DIR}/inferno" completions fish > "${COMP_DIR}/inferno.fish" 2>/dev/null || true
            ;;
    esac
    
    success "Shell completions installed for ${SHELL_NAME}"
}

# Cleanup
cleanup() {
    if [ -n "$TEMP_DIR" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}

# Verify installation
verify_installation() {
    info "Verifying installation..."
    
    if command -v inferno &> /dev/null; then
        VERSION_OUTPUT=$(inferno --version 2>/dev/null || echo "unknown")
        success "Installation verified: ${VERSION_OUTPUT}"
    else
        warn "Binary installed but not in PATH. Add ${INSTALL_DIR} to your PATH."
    fi
}

# Print next steps
print_next_steps() {
    echo ""
    echo -e "  ${GREEN}Installation complete!${NC}"
    echo ""
    echo -e "  ${BLUE}📖 Getting Started:${NC}"
    echo ""
    echo -e "    ${CYAN}1.${NC} Initialize your first node:"
    echo -e "       ${GREEN}inferno init${NC}"
    echo ""
    echo -e "    ${CYAN}2.${NC} Start your node:"
    echo -e "       ${GREEN}inferno start${NC}"
    echo ""
    echo -e "    ${CYAN}3.${NC} View logs:"
    echo -e "       ${GREEN}inferno logs --follow${NC}"
    echo ""
    echo -e "    ${CYAN}4.${NC} Open dashboard:"
    echo -e "       ${GREEN}inferno dashboard --open${NC}"
    echo ""
    echo -e "  ${BLUE}📚 Documentation:${NC}"
    echo -e "     https://github.com/${REPO}"
    echo ""
    echo -e "  ${BLUE}💬 Support:${NC}"
    echo -e "     https://discord.gg/pyrax"
    echo ""
}

# Main installation function
main() {
    print_banner
    
    detect_os
    detect_arch
    detect_wsl
    
    info "Detected: ${OS} ${ARCH}"
    
    get_latest_version
    build_download_url
    download_binary
    extract_binary
    install_binary
    create_config_dir
    install_completions
    cleanup
    verify_installation
    print_next_steps
}

# Run with cleanup on exit
trap cleanup EXIT
main "$@"
