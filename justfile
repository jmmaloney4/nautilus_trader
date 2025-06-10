# Justfile for NautilusTrader Nix Development
# Run `just --list` to see all available commands

# Default recipe - build the core package
default: build-core

# === Building ===

# Build core package (minimal dependencies)
build-core:
    nix build .#nautilus-trader-core

# Build standard precision variant
build-std:
    nix build .#nautilus-trader-std

# Build full package with all adapters
build-full:
    nix build .#nautilus-trader-full

# Build development package with dev dependencies
build-dev:
    nix build .#nautilus-trader-dev

# Build specific adapter variant
build-adapter adapter:
    nix build .#nautilus-trader-{{adapter}}

# Build all variants
build-all: build-core build-std build-full build-dev

# === Development ===

# Enter default development shell
dev:
    nix develop

# Enter Rust-only development shell
dev-rust:
    nix develop .#rust

# Enter Python-only development shell  
dev-python:
    nix develop .#python

# === Running ===

# Run Python with nautilus_trader available
python *args:
    nix run .#python -- {{args}}

# Run nautilus CLI (if available)
nautilus *args:
    nix run .#nautilus -- {{args}}

# === Docker ===

# Build Docker image
docker-build:
    nix build .#docker-image

# Load Docker image into Docker daemon
docker-load: docker-build
    docker load < result

# === Testing and Validation ===

# Run all checks
check: check-flake check-format

# Check flake validity
check-flake:
    nix flake check

# Check formatting
check-format:
    nix fmt --dry-run

# Format all nix files
format:
    nix fmt

# === Installation ===

# Install nautilus_trader into current profile
install variant="full":
    nix profile install .#nautilus-trader-{{variant}}

# Remove nautilus_trader from current profile
uninstall:
    nix profile remove nautilus-trader

# === Cleanup ===

# Clean build artifacts
clean:
    nix store gc
    rm -rf result*

# Deep clean including cached dependencies
clean-all:
    nix store gc
    nix-collect-garbage -d
    rm -rf result*

# === Information ===

# Show package info
info variant="full":
    nix show-derivation .#nautilus-trader-{{variant}}

# List all available packages
list:
    nix flake show

# Show dependencies
deps variant="full":
    nix-store --query --references $(nix build .#nautilus-trader-{{variant}} --no-link --print-out-paths)

# === Usage Examples ===

# Show usage examples
examples:
    @echo "=== Usage Examples ==="
    @echo ""
    @echo "# Build and run in one command:"
    @echo "nix run .#python -- -c 'import nautilus_trader; print(nautilus_trader.__version__)'"
    @echo ""
    @echo "# Use in another project's flake.nix:"
    @echo "inputs.nautilus-trader.url = \"github:your-repo/nautilus_trader\";"
    @echo "# Then in your overlay or package definition:"
    @echo "python3.withPackages (ps: [ inputs.nautilus-trader.packages.\${system}.nautilus-trader-full ])"
    @echo ""
    @echo "# Install specific variant:"
    @echo "just install core      # minimal"
    @echo "just install std       # standard precision"
    @echo "just install full      # all adapters"
    @echo "just install betfair   # betfair adapter only"

# === CI/CD Recipes ===

# CI build all variants (for automation)
ci-build:
    nix build .#nautilus-trader-core .#nautilus-trader-std .#nautilus-trader-full

# CI test (basic validation)
ci-test:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Testing core package import..."
    nix run .#python -- -c "import nautilus_trader; print('✓ Core import successful')"
    
    echo "Testing full package import..."
    nix build .#nautilus-trader-full
    echo "✓ Full package build successful"

# Generate CI matrix for GitHub Actions
ci-matrix:
    @echo "variants: [\"core\", \"std\", \"full\", \"dev\"]" 