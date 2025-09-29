#!/bin/bash

# exek installer script
# Provides an interactive installation experience with prefix configuration

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Default values
DEFAULT_PREFIX="/usr/local"
PREFIX=""
BINARY_NAME="exek"

# Functions
print_header() {
    echo -e "${BLUE}${BOLD}"
    echo "================================"
    echo "    exek Installation Script    "
    echo "================================"
    echo -e "${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed!"
        echo "Please install Rust from https://rustup.rs/"
        exit 1
    fi
    print_success "Rust/Cargo found"
}

check_permissions() {
    local prefix=$1
    local bindir="${prefix}/bin"

    if [[ "$prefix" == "/usr" ]] || [[ "$prefix" == "/usr/local" ]]; then
        if [[ $EUID -ne 0 ]]; then
            print_warning "System-wide installation requires root privileges."
            echo ""
            echo "Options:"
            echo "  1. Run with sudo: ${BOLD}sudo ./install.sh${NC}"
            echo "  2. Install to user directory: ${BOLD}./install.sh --prefix=~/.local${NC}"
            echo ""
            read -p "Would you like to continue with sudo? (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                exec sudo "$0" "$@"
            else
                print_info "Installation cancelled. Consider using --prefix=~/.local for user installation."
                exit 0
            fi
        fi
    else
        # Check if we can write to the target directory
        if [[ -d "$bindir" ]] && [[ ! -w "$bindir" ]]; then
            print_error "Cannot write to $bindir"
            echo "Please check permissions or choose a different prefix."
            exit 1
        fi
    fi
}

select_prefix() {
    echo "Select installation location:"
    echo ""
    echo "  1) System-wide (/usr/local) - Recommended, requires sudo"
    echo "  2) System (/usr) - For package managers, requires sudo"
    echo "  3) User (~/.local) - No sudo required"
    echo "  4) Custom path"
    echo ""

    read -p "Choice (1-4) [1]: " choice

    case $choice in
        1|"")
            PREFIX="/usr/local"
            ;;
        2)
            PREFIX="/usr"
            ;;
        3)
            PREFIX="$HOME/.local"
            ;;
        4)
            read -p "Enter custom prefix path: " custom_prefix
            PREFIX="$custom_prefix"
            ;;
        *)
            print_error "Invalid choice"
            exit 1
            ;;
    esac

    # Expand ~ to home directory
    PREFIX="${PREFIX/#\~/$HOME}"

    print_info "Installation prefix: ${BOLD}$PREFIX${NC}"
}

build_binary() {
    print_info "Building release binary..."

    if cargo build --release; then
        print_success "Build completed successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

install_binary() {
    local bindir="${PREFIX}/bin"
    local binary_path="target/release/${BINARY_NAME}"

    print_info "Installing binary to ${bindir}..."

    # Create bin directory if it doesn't exist
    mkdir -p "$bindir"

    # Install the binary
    if install -m 755 "$binary_path" "${bindir}/${BINARY_NAME}"; then
        print_success "Binary installed: ${bindir}/${BINARY_NAME}"
    else
        print_error "Failed to install binary"
        exit 1
    fi
}

install_extras() {
    local bindir="${PREFIX}/bin"
    local datadir="${PREFIX}/share"
    local applicationsdir="${datadir}/applications"

    # Install launcher script if it exists
    if [[ -f "scripts/exek-launcher.sh" ]]; then
        print_info "Installing launcher script..."
        if install -m 755 "scripts/exek-launcher.sh" "${bindir}/exek-launcher"; then
            print_success "Launcher script installed"
        fi
    fi

    # Install desktop file for system-wide installations
    if [[ "$PREFIX" == "/usr" ]] || [[ "$PREFIX" == "/usr/local" ]]; then
        if [[ -f "config/exek.desktop" ]]; then
            print_info "Installing desktop file..."
            mkdir -p "$applicationsdir"
            if install -m 644 "config/exek.desktop" "${applicationsdir}/exek.desktop"; then
                print_success "Desktop file installed"
                # Update desktop database if available
                if command -v update-desktop-database &> /dev/null; then
                    update-desktop-database "$applicationsdir" 2>/dev/null || true
                fi
            fi
        fi
    fi
}

check_path() {
    local bindir="${PREFIX}/bin"

    if [[ ":$PATH:" != *":$bindir:"* ]]; then
        print_warning "The installation directory ${BOLD}$bindir${NC} is not in your PATH"
        echo ""
        echo "To use exek from anywhere, add this to your shell configuration:"
        echo ""

        if [[ -f "$HOME/.bashrc" ]]; then
            echo "  echo 'export PATH=\"$bindir:\$PATH\"' >> ~/.bashrc"
            echo "  source ~/.bashrc"
        elif [[ -f "$HOME/.zshrc" ]]; then
            echo "  echo 'export PATH=\"$bindir:\$PATH\"' >> ~/.zshrc"
            echo "  source ~/.zshrc"
        else
            echo "  export PATH=\"$bindir:\$PATH\""
        fi
        echo ""
    fi
}

print_completion() {
    local bindir="${PREFIX}/bin"

    echo ""
    echo -e "${GREEN}${BOLD}Installation Complete!${NC}"
    echo ""
    echo "exek has been installed to: ${BOLD}${bindir}/${BINARY_NAME}${NC}"
    echo ""
    echo "To get started:"
    echo "  • Run '${BOLD}exek${NC}' to launch the application"
    echo "  • Type to search for applications"
    echo "  • Use '/' or './' to browse executable files"
    echo ""
    echo "For more information:"
    echo "  • README: https://github.com/yourusername/exek"
    echo "  • Run '${BOLD}exek --help${NC}' for usage information"
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --prefix=PATH    Set installation prefix (default: /usr/local)"
            echo "  --help, -h       Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Interactive installation"
            echo "  $0 --prefix=/usr      # Install to /usr"
            echo "  $0 --prefix=~/.local  # Install to user directory"
            echo ""
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use '$0 --help' for usage information"
            exit 1
            ;;
    esac
done

# Main installation flow
print_header

# Check for Rust/Cargo
check_rust

# Select prefix if not provided
if [[ -z "$PREFIX" ]]; then
    select_prefix
else
    print_info "Using prefix from command line: ${BOLD}$PREFIX${NC}"
fi

# Check permissions
check_permissions "$PREFIX"

# Build the binary
build_binary

# Install binary and extras
install_binary
install_extras

# Check PATH
check_path

# Print completion message
print_completion
