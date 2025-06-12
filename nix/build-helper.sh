#!/usr/bin/env bash
# Build helper script for Nautilus Trader using Nix
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Function to check if flake inputs are available
check_flake_inputs() {
    log "Checking flake inputs..."
    if ! nix flake show --json > /dev/null 2>&1; then
        error "Failed to evaluate flake. Please check your flake.nix syntax."
        exit 1
    fi
    success "Flake inputs OK"
}

# Function to build Rust components only
build_rust() {
    log "Building Rust libraries..."
    nix build .#rust-libraries -L
    if [ $? -eq 0 ]; then
        success "Rust libraries built successfully"
    else
        error "Rust build failed"
        exit 1
    fi
}

# Function to build Python package
build_python() {
    log "Building Python package..."
    nix build .#nautilus-trader -L
    if [ $? -eq 0 ]; then
        success "Python package built successfully"
    else
        error "Python build failed"
        exit 1
    fi
}

# Function to build the full application
build_full() {
    log "Building full application..."
    nix build .#default -L
    if [ $? -eq 0 ]; then
        success "Full application built successfully"
    else
        error "Full build failed"
        exit 1
    fi
}

# Function to run tests
run_tests() {
    log "Running tests..."
    nix flake check -L
    if [ $? -eq 0 ]; then
        success "All tests passed"
    else
        error "Some tests failed"
        exit 1
    fi
}

# Function to build development environment
build_dev() {
    log "Building development environment..."
    nix build .#dev-env -L
    if [ $? -eq 0 ]; then
        success "Development environment built successfully"
    else
        error "Development environment build failed"
        exit 1
    fi
}

# Function to enter development shell
dev_shell() {
    log "Entering development shell..."
    nix develop
}

# Function to enter impure development shell
dev_shell_impure() {
    log "Entering impure development shell..."
    nix develop .#impure
}

# Function to clean build artifacts
clean() {
    log "Cleaning build artifacts..."
    if [ -d "result" ]; then
        rm -rf result*
        success "Cleaned result symlinks"
    fi
    
    if [ -d "target" ]; then
        warn "Found Cargo target directory. Remove with: rm -rf target"
    fi
    
    if [ -d ".uv" ]; then
        warn "Found UV cache directory. Remove with: rm -rf .uv"
    fi
}

# Function to show available packages
show_packages() {
    log "Available packages:"
    nix flake show --json | jq -r '.packages."x86_64-linux" | keys[]' | while read pkg; do
        echo "  - $pkg"
    done
}

# Function to build a specific adapter
build_adapter() {
    local adapter=$1
    log "Building adapter: $adapter"
    nix build ".#adapters-$adapter" -L
    if [ $? -eq 0 ]; then
        success "Adapter $adapter built successfully"
    else
        error "Adapter $adapter build failed"
        exit 1
    fi
}

# Main script logic
case "${1:-help}" in
    "check")
        check_flake_inputs
        ;;
    "rust")
        build_rust
        ;;
    "python") 
        build_python
        ;;
    "full"|"all")
        build_full
        ;;
    "test")
        run_tests
        ;;
    "dev")
        build_dev
        ;;
    "shell")
        dev_shell
        ;;
    "shell-impure")
        dev_shell_impure
        ;;
    "clean")
        clean
        ;;
    "show")
        show_packages
        ;;
    "adapter")
        if [ -z "${2:-}" ]; then
            error "Please specify an adapter name (e.g., betfair, dydx, ib)"
            exit 1
        fi
        build_adapter "$2"
        ;;
    "help"|*)
        echo "Nautilus Trader Nix Build Helper"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  check         - Check flake inputs and syntax"
        echo "  rust          - Build Rust libraries only"
        echo "  python        - Build Python package only" 
        echo "  full|all      - Build full application"
        echo "  test          - Run all tests and checks"
        echo "  dev           - Build development environment"
        echo "  shell         - Enter pure development shell"
        echo "  shell-impure  - Enter impure development shell"
        echo "  clean         - Clean build artifacts"
        echo "  show          - Show available packages"
        echo "  adapter <name> - Build specific adapter (betfair, dydx, ib)"
        echo "  help          - Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0 full              # Build everything"
        echo "  $0 adapter betfair   # Build just the Betfair adapter"
        echo "  $0 shell             # Enter development environment"
        ;;
esac 