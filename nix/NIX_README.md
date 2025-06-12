# Nautilus Trader Nix Flake

This flake provides a Nix-based build system for Nautilus Trader, a high-performance algorithmic trading platform written in Rust and Python. The flake replicates the build process from the existing Dockerfile but with the reproducibility and composability benefits of Nix.

## Overview

The Nautilus Trader project consists of:
- **Rust workspace**: 20+ crates providing core trading infrastructure
- **Python package**: High-level APIs and integrations built on the Rust core
- **System dependencies**: Requires clang, capnproto, and various native libraries

## Architecture

This flake breaks the build into logical sub-packages:

### Core Packages
- `rust-libraries` - Just the Rust static libraries
- `nautilus-trader` - Core Python package with Rust extensions
- `default` - Full application with all default dependencies

### Specialized Packages
- `dev-env` - Development environment with all optional dependencies
- `adapters-betfair` - Betfair-specific environment
- `adapters-dydx` - dYdX-specific environment  
- `adapters-ib` - Interactive Brokers environment

### Development Shells
- `default` - Pure Nix development environment
- `impure` - Hybrid environment using UV for Python dependencies

## Quick Start

### Prerequisites
- Nix with flakes enabled
- Sufficient disk space (the build is quite large)

### Building

```bash
# Build the full application
nix build

# Build just the Rust libraries
nix build .#rust-libraries

# Build the Python package only
nix build .#nautilus-trader

# Build development environment
nix build .#dev-env
```

### Development

```bash
# Enter the development shell (pure Nix)
nix develop

# Enter impure shell (uses UV for Python deps)
nix develop .#impure

# Run the build helper for guided builds
./nix/build-helper.sh help
```

### Using the Build Helper

The included build helper script provides convenient commands:

```bash
# Check flake syntax and inputs
./nix/build-helper.sh check

# Build everything
./nix/build-helper.sh full

# Build just Rust components
./nix/build-helper.sh rust

# Build specific adapter
./nix/build-helper.sh adapter betfair

# Enter development shell
./nix/build-helper.sh shell

# Run tests
./nix/build-helper.sh test

# Clean build artifacts
./nix/build-helper.sh clean
```

## Technical Details

### Rust Build Process

The flake uses [crane](https://github.com/ipetkov/crane) to build the Rust workspace:

1. **Dependency caching**: Cargo dependencies are built separately for faster incremental builds
2. **Static libraries**: Builds the core libraries needed by Python extensions
3. **Feature flags**: Matches the Dockerfile's feature configuration (`ffi,python,extension-module`)
4. **Toolchain**: Uses Rust 1.87.0 stable as specified in `rust-toolchain.toml`

### Python Build Process  

The flake uses [uv2nix](https://github.com/pyproject-nix/uv2nix) for Python dependency management:

1. **UV workspace**: Loads the UV lock file for reproducible Python builds
2. **Wheel preference**: Prefers binary wheels for faster builds when available
3. **Custom build script**: Integrates with the project's `build.py` script
4. **Rust integration**: Links against the pre-built Rust static libraries

### Key Environment Variables

The build process sets these environment variables to match the Dockerfile:

```bash
BUILD_MODE=release          # Release build configuration
RUSTUP_TOOLCHAIN=stable     # Use stable Rust toolchain  
CC=clang                    # Use clang as C compiler
HIGH_PRECISION=true         # Enable 128-bit precision mode
PYO3_PYTHON=/path/to/python # Python interpreter for PyO3
PARALLEL_BUILD=true         # Enable parallel compilation
```

### System Dependencies

The flake provides these native dependencies:

- **Build tools**: clang, cmake, pkg-config
- **Rust toolchain**: cargo, rustc, clippy, rustfmt
- **Protocol buffers**: capnproto and development headers
- **Libraries**: openssl, libcap
- **macOS specific**: libiconv, Security framework

## Development Workflows

### Pure Nix Development

For a completely reproducible environment:

```bash
nix develop
# All dependencies are provided by Nix
cargo build --release
python -m pytest
```

### Hybrid Development  

For faster iteration with UV managing Python dependencies:

```bash
nix develop .#impure
uv sync
cargo build --release  
uv run python -m pytest
```

### Cross-platform Development

The flake supports Linux and macOS:
- **Linux**: Uses clang by default, includes Linux-specific libraries
- **macOS**: Includes Darwin frameworks and handles code signing

## Continuous Integration

The flake includes comprehensive checks:

```bash
# Run all checks
nix flake check

# Individual checks  
nix build .#checks.x86_64-linux.rust-clippy
nix build .#checks.x86_64-linux.rust-fmt
nix build .#checks.x86_64-linux.rust-tests
```

## Customization

### Adding New Adapters

To add a new trading adapter:

1. Add dependency group to `pyproject.toml`
2. Add package to flake's `packages` section:

```nix
adapters-newexchange = pythonSet.mkVirtualEnv "nautilus-newexchange-env"
  (workspace.deps.default ++ workspace.deps.groups.newexchange or []);
```

### Overriding Dependencies

Use the `pyprojectOverrides` overlay in `flake.nix`:

```nix
pyprojectOverrides = final: prev: {
  some-package = prev.some-package.overrideAttrs (old: {
    # Custom build configuration
  });
};
```

### Custom Rust Features

Modify `commonCargoArgs` in the flake:

```nix
cargoExtraArgs = "--lib --release --features your-custom-features";
```

## Performance Notes

- **Build time**: Initial build can take 30-60 minutes depending on hardware
- **Cache usage**: Subsequent builds are much faster due to Nix caching
- **Memory usage**: Rust compilation requires significant RAM (8GB+ recommended)
- **Disk space**: Full build requires several GB of disk space

## Troubleshooting

### Common Issues

1. **Out of memory**: Reduce parallelism or increase swap space
2. **Missing dependencies**: Check that all system dependencies are available
3. **Python linking errors**: Ensure PyO3 environment variables are set correctly
4. **Rust compilation errors**: Verify Rust toolchain version matches project requirements

### Debug Commands

```bash
# Verbose build output
nix build -L

# Show dependency tree
nix show-derivation .#default

# Enter build environment for debugging
nix develop .#default --command bash
```

### Getting Help

- Check the [Nautilus Trader documentation](https://nautilustrader.io/docs)
- Review the [uv2nix documentation](https://pyproject-nix.github.io/uv2nix/)  
- Consult the [crane documentation](https://crane.dev/)

## Comparison with Docker Build

| Aspect | Docker | Nix |
|--------|--------|-----|
| **Reproducibility** | Good | Excellent |
| **Caching** | Layer-based | Content-addressed |
| **Incremental builds** | Limited | Excellent |
| **Debugging** | Difficult | Easy |
| **Composability** | Limited | Excellent |
| **Platform support** | Good | Good |

The Nix build provides better reproducibility and development experience while maintaining compatibility with the existing Docker-based workflow. 