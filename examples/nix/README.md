# NautilusTrader Nix Packaging

This document describes the comprehensive Nix packaging solution for NautilusTrader, a high-performance algorithmic trading platform written in Python and Rust.

## Overview

The packaging approach addresses the complex requirements of NautilusTrader:

- **Hybrid Language Support**: Rust core with Python bindings via PyO3/Cython
- **Multiple Build Configurations**: High-precision vs standard precision modes
- **Modular Adapters**: Optional exchange-specific adapters with their dependencies  
- **Complex Build Process**: Custom build script coordinating Rust and Python compilation
- **Development Workflow**: Rich tooling for development, testing, and documentation

## Package Variants

### Core Variants

| Package | Description | Precision | Adapters | Dev Dependencies |
|---------|-------------|-----------|----------|------------------|
| `nautilus-trader-core` | Minimal package | High (128-bit) | None | No |
| `nautilus-trader-std` | Standard precision | Standard (64-bit) | None | No |
| `nautilus-trader-full` | Full package | High (128-bit) | All core adapters | No |
| `nautilus-trader-dev` | Development package | High (128-bit) | All adapters | Yes |

### Adapter-Specific Variants

- `nautilus-trader-betfair`: Betfair sports betting adapter
- `nautilus-trader-ib`: Interactive Brokers adapter
- `nautilus-trader-dydx`: dYdX DeFi adapter
- `nautilus-trader-polymarket`: Polymarket prediction market adapter

## Usage

### 1. Using the Flake (Recommended)

#### Quick Start

```bash
# Run Python with NautilusTrader available
nix run github:your-org/nautilus_trader#python -- -c "import nautilus_trader; print('Success!')"

# Install into your profile
nix profile install github:your-org/nautilus_trader#nautilus-trader-full

# Enter development environment
nix develop github:your-org/nautilus_trader
```

#### In Your Project's Flake

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nautilus-trader.url = "github:your-org/nautilus_trader";
  };

  outputs = { nixpkgs, nautilus-trader, ... }: {
    # Your project configuration
    devShells.default = pkgs.mkShell {
      buildInputs = [
        (pkgs.python3.withPackages (ps: [
          nautilus-trader.packages.${system}.nautilus-trader-full
          ps.jupyter
          ps.matplotlib
        ]))
      ];
    };
  };
}
```

### 2. Traditional Derivation

For use in nixpkgs overlays or traditional Nix expressions:

```nix
# In your overlay or package set
nautilus-trader = pkgs.callPackage ./examples/nix/derivation.nix {
  highPrecision = true;
  withBetfair = true;
  withInteractiveBrokers = true;
};

# In your Python environment
python3.withPackages (ps: [ nautilus-trader ])
```

### 3. Docker Usage

```bash
# Build Docker image
nix build .#docker-image

# Load into Docker
docker load < result

# Run container
docker run -it nautilus-trader:latest python
```

## Development Workflow

### Using Just (Recommended)

Install [just](https://just.systems/) and use the provided commands:

```bash
# Build all variants
just build-all

# Enter development shell
just dev

# Run checks
just check

# Show usage examples
just examples
```

### Direct Nix Commands

```bash
# Build specific variants
nix build .#nautilus-trader-core
nix build .#nautilus-trader-full

# Development shells
nix develop                    # Full development environment
nix develop .#rust            # Rust-only environment
nix develop .#python          # Python-only environment

# Run applications
nix run .#python -- -c "import nautilus_trader"
nix run .#nautilus -- --help
```

## Architecture

### Build Process

1. **Rust Compilation**: Cargo builds the Rust workspace with appropriate features
2. **Dependency Caching**: Crane caches Rust dependencies for faster rebuilds
3. **Python Integration**: Custom build script coordinates Rust artifacts with Python/Cython compilation
4. **Variant Generation**: Multiple packages with different configurations and dependencies

### Key Design Decisions

#### Two-Phase Build

The build is split into Rust and Python phases:

```nix
# Phase 1: Build Rust workspace
rustWorkspace = buildRustWorkspace { 
  highPrecision = true; 
  features = ["defi"]; 
};

# Phase 2: Build Python package using Rust artifacts
buildPythonPackage {
  rustWorkspace = rustWorkspace;
  # ... Python-specific configuration
}
```

This separation allows:
- Caching of expensive Rust compilation
- Multiple Python variants sharing the same Rust build
- Clear dependency tracking

#### Feature Flags

Rust features control functionality:
- `high-precision`: Enable 128-bit precision arithmetic
- `ffi`: Foreign function interface for Python bindings
- `python`: Python-specific code paths
- `extension-module`: PyO3 extension module build
- `defi`: DeFi-specific adapters

#### Precision Modes

Two precision modes are supported:
- **High-precision** (128-bit): Default on Linux/macOS, more accurate calculations
- **Standard-precision** (64-bit): Required on Windows, faster but less precise

### Dependencies

#### Build-time Dependencies
- Rust toolchain (1.87.0+)
- Clang/LLVM (for compilation)
- Python 3.11-3.13
- Cython 3.1.1
- Various system libraries (OpenSSL, zlib, etc.)

#### Runtime Dependencies
- Core Python packages: numpy, pandas, pyarrow, etc.
- Adapter-specific packages: varies by adapter
- Optional: Redis for caching/messaging

## Configuration Options

### Environment Variables

The build process respects several environment variables:

- `BUILD_MODE`: `release` or `debug`
- `HIGH_PRECISION`: `true` or `false`
- `CARGO_TARGET_DIR`: Rust build output directory
- `PYTHON_SYS_EXECUTABLE`: Python interpreter path

### Adapter Selection

Adapters can be enabled individually:

```nix
buildPythonPackage {
  adapters = [ "betfair" "ib" "dydx" ];
  # Only specified adapters and their dependencies included
}
```

## Troubleshooting

### Common Issues

#### Missing Rust Dependencies
```bash
# Ensure Rust toolchain is available
nix develop .#rust
rustc --version  # Should show 1.87.0+
```

#### Python Import Errors
```bash
# Check Python path
nix develop
python -c "import sys; print(sys.path)"

# Verify package installation
python -c "import nautilus_trader; print(nautilus_trader.__file__)"
```

#### Build Failures
```bash
# Clean build cache
nix-collect-garbage -d
rm -rf result*

# Rebuild with verbose output
nix build .#nautilus-trader-core --verbose
```

### Platform-Specific Notes

#### Linux
- High-precision mode supported
- All adapters available
- Recommended platform for development

#### macOS
- High-precision mode supported
- May require additional frameworks for some adapters
- ARM64 and x86_64 supported

#### Windows
- Only standard-precision mode supported
- Some adapters may not be available
- Consider using WSL2 with Linux builds

## Contributing

### Adding New Adapters

1. Add Rust crate dependencies to `Cargo.toml`
2. Add Python dependencies to `pyproject.toml`
3. Update flake.nix adapter configuration:

```nix
adapterDeps = {
  # ... existing adapters ...
  my-new-adapter = ps: with ps; [ 
    new-dependency-1
    new-dependency-2
  ];
};
```

4. Add package variant:

```nix
nautilus-trader-my-adapter = buildPythonPackage {
  pname = "nautilus-trader-my-adapter";
  rustWorkspace = buildRustWorkspace { highPrecision = true; };
  adapters = [ "my-new-adapter" ];
};
```

### Testing Changes

```bash
# Test all variants build
just ci-build

# Test basic functionality
just ci-test

# Check flake validity
nix flake check
```

## Performance Considerations

### Build Performance
- Rust dependencies are cached via Crane
- Multiple Python variants share Rust builds
- Use `nix develop` for incremental development

### Runtime Performance
- High-precision mode has slight performance overhead
- Choose appropriate adapter set for your use case
- Consider using `nautilus-trader-core` for minimal overhead

### Memory Usage
- Full package with all adapters: ~500MB installed
- Core package: ~200MB installed
- Development package with dev deps: ~800MB installed

## Maintenance

### Updating Dependencies

1. Update `rust-toolchain.toml` for Rust version
2. Update `pyproject.toml` for Python dependencies
3. Update flake inputs for nixpkgs/other dependencies
4. Test all variants after updates

### Version Management

Versions are managed in several places:
- `version.json`: Main version file
- `pyproject.toml`: Python package version
- `Cargo.toml`: Rust workspace version
- `flake.nix`: Default version in package builders

Keep these synchronized when releasing new versions. 