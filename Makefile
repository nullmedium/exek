# Makefile for exek
# A fast TUI application launcher with fuzzy matching

# Installation prefix - can be overridden by user
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
MANDIR = $(DATADIR)/man
APPLICATIONSDIR = $(DATADIR)/applications
COMPLETIONSDIR = $(DATADIR)/bash-completion/completions

# Binary name
BINARY = exek
TARGET = release

# Cargo build flags
# Try to find cargo in common locations
CARGO := $(shell which cargo 2>/dev/null || echo "$$HOME/.cargo/bin/cargo")
CARGO_FLAGS = --release

# Check if cargo exists
CARGO_EXISTS := $(shell $(CARGO) --version 2>/dev/null && echo yes || echo no)

# Installation program
INSTALL = install
INSTALL_PROGRAM = $(INSTALL) -m 755
INSTALL_DATA = $(INSTALL) -m 644

.PHONY: all build clean install install-only uninstall help

# Default target
all: build

# Help target
help:
	@echo "exek Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make              - Build the release binary"
	@echo "  make build        - Build the release binary"
	@echo "  make debug        - Build the debug binary"
	@echo "  make install      - Build (if needed) and install to PREFIX"
	@echo "  make install-only - Install without building (requires existing binary)"
	@echo "  make uninstall    - Remove installed files"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "Installation workflows:"
	@echo "  # If cargo is in PATH:"
	@echo "  sudo make install"
	@echo ""
	@echo "  # If cargo is not in sudo's PATH:"
	@echo "  make build              # Build as regular user"
	@echo "  sudo make install-only  # Install as root"
	@echo ""
	@echo "Configuration:"
	@echo "  PREFIX=/usr make install      - Install to /usr instead of /usr/local"
	@echo "  PREFIX=~/.local make install  - Install to user's .local directory"
	@echo ""
	@echo "Current settings:"
	@echo "  PREFIX=$(PREFIX)"
	@echo "  BINDIR=$(BINDIR)"

# Build release binary
build:
	@if [ "$(CARGO_EXISTS)" = "no" ]; then \
		echo "Error: cargo not found!"; \
		echo ""; \
		echo "Please ensure Rust is installed and cargo is in your PATH."; \
		echo "Visit https://rustup.rs/ to install Rust."; \
		echo ""; \
		echo "If cargo is installed but not in sudo's PATH, try:"; \
		echo "  1. Build first without sudo: make build"; \
		echo "  2. Then install: sudo make install"; \
		echo ""; \
		exit 1; \
	fi
	@echo "Building release binary..."
	$(CARGO) build $(CARGO_FLAGS)
	@echo "Build complete: target/$(TARGET)/$(BINARY)"

# Build debug binary
debug:
	@echo "Building debug binary..."
	$(CARGO) build
	@echo "Debug build complete: target/debug/$(BINARY)"

# Install the application
install:
	@# Check if binary exists, if not try to build
	@if [ ! -f "target/$(TARGET)/$(BINARY)" ]; then \
		echo "Binary not found, building first..."; \
		$(MAKE) build; \
	else \
		echo "Using existing binary at target/$(TARGET)/$(BINARY)"; \
	fi
	@$(MAKE) install-only

# Install without building (assumes binary exists)
install-only:
	@if [ ! -f "target/$(TARGET)/$(BINARY)" ]; then \
		echo "Error: Binary not found at target/$(TARGET)/$(BINARY)"; \
		echo "Please run 'make build' first"; \
		exit 1; \
	fi
	@echo "Installing to $(PREFIX)..."

	# Create directories
	@mkdir -p $(BINDIR)

	# Install binary
	$(INSTALL_PROGRAM) target/$(TARGET)/$(BINARY) $(BINDIR)/$(BINARY)

	# Install launcher script if it exists
	@if [ -f scripts/exek-launcher.sh ]; then \
		echo "Installing launcher script..."; \
		$(INSTALL_PROGRAM) scripts/exek-launcher.sh $(BINDIR)/exek-launcher; \
	fi

	# Install desktop file if running system-wide installation
	@if [ "$(PREFIX)" = "/usr" ] || [ "$(PREFIX)" = "/usr/local" ]; then \
		if [ -f config/exek.desktop ]; then \
			echo "Installing desktop file..."; \
			mkdir -p $(APPLICATIONSDIR); \
			$(INSTALL_DATA) config/exek.desktop $(APPLICATIONSDIR)/exek.desktop; \
		fi \
	fi

	@echo ""
	@echo "Installation complete!"
	@echo "Binary installed to: $(BINDIR)/$(BINARY)"
	@echo ""
	@echo "You can now run 'exek' from anywhere in your terminal."

# Uninstall the application
uninstall:
	@echo "Uninstalling from $(PREFIX)..."

	# Remove binary
	@rm -f $(BINDIR)/$(BINARY)
	@echo "Removed $(BINDIR)/$(BINARY)"

	# Remove launcher script
	@rm -f $(BINDIR)/exek-launcher
	@echo "Removed $(BINDIR)/exek-launcher"

	# Remove desktop file
	@rm -f $(APPLICATIONSDIR)/exek.desktop
	@echo "Removed $(APPLICATIONSDIR)/exek.desktop"

	# Remove user data (with confirmation)
	@echo ""
	@echo "Note: User configuration at ~/.config/exek/ was not removed."
	@echo "Run 'rm -rf ~/.config/exek' to remove it manually if desired."

	@echo ""
	@echo "Uninstall complete!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	@rm -f test_path_completion.sh
	@echo "Clean complete!"

# Test the installation
test: build
	@echo "Running tests..."
	$(CARGO) test
	$(CARGO) clippy

# Create distribution tarball
dist: clean
	@echo "Creating distribution tarball..."
	@mkdir -p dist
	@git archive --format=tar.gz --prefix=exek-$(shell git describe --tags --always)/ \
		-o dist/exek-$(shell git describe --tags --always).tar.gz HEAD
	@echo "Tarball created: dist/exek-$(shell git describe --tags --always).tar.gz"

# Check if running as root (for system-wide installation)
check-root:
	@if [ "$(PREFIX)" = "/usr" ] || [ "$(PREFIX)" = "/usr/local" ]; then \
		if [ "$$(id -u)" != "0" ]; then \
			echo "Error: System-wide installation requires root privileges."; \
			echo "Please run 'sudo make install' or use PREFIX=~/.local for user installation"; \
			exit 1; \
		fi \
	fi

# Install with root check
safe-install: check-root install
