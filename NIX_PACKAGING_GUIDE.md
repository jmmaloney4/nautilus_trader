# NautilusTrader Nix Packaging Guide

## Overview

This document describes the comprehensive Nix packaging solution for NautilusTrader, a high-performance algorithmic trading platform written in Python with Rust core components. The packaging solution addresses the complex hybrid architecture and provides multiple deployment options.

## Architecture Analysis

### Project Structure
- **Hybrid Language**: Core written in Rust (20+ crates), Python bindings via PyO3/Cython
- **Build System**: Poetry + custom `build.py` script + Cargo for Rust compilation
- **Precision Modes**: 64-bit (standard) and 128-bit (high precision) support
- **Modular Adapters**: Optional exchange-specific adapters with their dependencies
- **Complex Dependencies**: Mix of system libraries, Python packages, and Rust crates

### Key Challenges Addressed
1. **Multi-language Build**: Coordinating Rust and Python compilation
2. **Dependency Management**: Handling both Python and Rust dependency trees
3. **Build Variants**: Supporting different precision modes and adapter combinations
4. **Reproducibility**: Ensuring deterministic builds across environments
5. **Development Workflow**: Rich tooling for development, testing, and documentation

## Package Variants

The packaging provides multiple variants to support different use cases:

### Core Variants
- **`nautilus-trader-core`**: Minimal package with essential functionality
- **`nautilus-trader-std`**: Standard precision with common adapters
- **`nautilus-trader-full`**: Full package with all adapters and high precision
- **`nautilus-trader-dev`**: Development package with testing and documentation tools

### Adapter-Specific Variants
- **`nautilus-trader-betfair`**: Core + Betfair exchange adapter
- **`nautilus-trader-ib`**: Core + Interactive Brokers adapter

## File Structure

```
.
├── flake.nix                           # Main Nix flake with all variants
├── justfile                            # Development commands
├── examples/nix/
│   ├── README.md                       # Detailed documentation
│   ├── consumer-flake.nix              # Example consumption flake
│   ├── derivation.nix                  # Traditional derivation
│   └── .github-workflows-nix.yml      # CI/CD workflow
└── NIX_PACKAGING_GUIDE.md             # This guide
```

## Usage Examples

### As a Consumer Project

Create a `flake.nix` in your project:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nautilus-trader.url = "github:your-org/nautilus_trader";
  };

  outputs = { nixpkgs, nautilus-trader, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          (pkgs.python3.withPackages (ps: [
            nautilus-trader.packages.${system}.nautilus-trader-full
          ]))
        ];
      };
    };
}
```

### Direct Installation

```bash
# Install specific variant
nix profile install github:your-org/nautilus_trader#nautilus-trader-full

# Run in temporary shell
nix shell github:your-org/nautilus_trader#nautilus-trader-full

# Build locally
nix build .#nautilus-trader-core
```

### Development Workflow

```bash
# Enter development shell
just dev

# Build specific variants
just build-core
just build-full
just build-adapter betfair

# Run tests
just test
just test-rust
just test-python

# Format code
just fmt

# Generate documentation
just docs
```

## Technical Implementation

### Build Process

1. **Rust Compilation**: 
   - Uses Crane for efficient Rust builds
   - Builds all crates in workspace
   - Supports different feature flags for precision modes

2. **Python Integration**:
   - Custom derivation coordinating Rust artifacts with Python build
   - Proper handling of PyO3 bindings
   - Integration with Poetry dependencies

3. **Dependency Management**:
   - Smart caching of Rust dependencies
   - Separate Python and Rust dependency trees
   - Conditional dependencies based on adapters

### Key Design Decisions

1. **Separation of Concerns**: 
   - Rust crates built separately for caching efficiency
   - Python packaging as final assembly step

2. **Multiple Outputs**: 
   - Different packages for different use cases
   - Minimal dependencies for core functionality

3. **Development-First**: 
   - Rich development environment with all tools
   - Fast rebuild cycles during development

4. **CI/CD Integration**: 
   - Matrix builds for different variants
   - Caching strategies for efficient builds

## Configuration Options

### Precision Mode
```nix
# High precision (128-bit)
nautilus-trader-full  # Default high precision

# Standard precision (64-bit)  
nautilus-trader-std
```

### Adapter Selection
```nix
# Individual adapters
nautilus-trader-betfair
nautilus-trader-ib

# Custom combination
mkNautilusTrader {
  adapters = [ "betfair" "databento" ];
  highPrecision = true;
}
```

### Development Features
```nix
# Development package with testing tools
nautilus-trader-dev

# Or custom development environment
mkNautilusTrader {
  withDev = true;
  withDocs = true;
}
```

## Testing and Quality Assurance

### Build Testing
- Matrix builds for all variants
- Cross-platform testing (Linux, macOS)
- Dependency conflict detection

### Runtime Testing
- Python import tests
- Basic functionality tests
- Performance benchmarks

### Security
- Reproducible builds
- Pinned dependencies
- Supply chain verification

## Performance Considerations

### Build Performance
- **Incremental Builds**: Crane enables efficient incremental Rust builds
- **Parallel Compilation**: Full utilization of available CPU cores
- **Caching Strategy**: Smart separation of Rust and Python builds
- **Dependency Caching**: Pre-built dependencies for faster builds

### Runtime Performance
- **Native Compilation**: All code compiled for target architecture
- **Link-Time Optimization**: Enabled for release builds
- **Memory Efficiency**: Minimal dependency footprint
- **Startup Time**: Optimized for fast application startup

## Deployment Scenarios

### Local Development
```bash
nix develop  # Full development environment
```

### Production Deployment
```bash
nix build .#nautilus-trader-full
./result/bin/nautilus --help
```

### Container Deployment
```bash
nix build .#docker-image
docker load < result
```

### CI/CD Pipeline
```yaml
- uses: nixbuild/nix-quick-install-action@v28
- run: nix build .#nautilus-trader-full
```

## Maintenance and Updates

### Dependency Updates
```bash
# Update flake inputs
nix flake update

# Update specific input
nix flake lock --update-input nixpkgs
```

### Version Management
- Version controlled via `version.json`
- Automatic version propagation to all packages
- Semantic versioning support

### Security Updates
- Regular dependency audits
- CVE monitoring
- Automated security patches

## Troubleshooting

### Common Issues

1. **Build Failures**:
   ```bash
   # Check build logs
   nix build .#nautilus-trader-core --print-build-logs
   
   # Clean build cache
   nix store gc
   ```

2. **Import Errors**:
   ```bash
   # Verify Python path
   nix develop -c python -c "import nautilus_trader; print(nautilus_trader.__file__)"
   ```

3. **Missing Dependencies**:
   ```bash
   # Check derivation
   nix show-derivation .#nautilus-trader-full
   ```

### Debugging Tools
- Build log analysis
- Dependency graph visualization
- Runtime environment inspection

## Future Enhancements

### Planned Improvements
1. **WebAssembly Support**: Enable browser deployment
2. **Cross-Compilation**: Support for ARM architectures
3. **Binary Caching**: Hydra-based binary cache
4. **NixOS Module**: System-wide service configuration

### Contribution Guidelines
1. Test changes across all variants
2. Update documentation for new features
3. Maintain backward compatibility
4. Follow Nix packaging best practices

## Conclusion

This Nix packaging solution provides a robust, scalable, and maintainable way to build and deploy NautilusTrader. It addresses the complex requirements of the hybrid Python/Rust architecture while providing flexibility for different deployment scenarios.

The modular design allows users to choose the appropriate variant for their needs, while the comprehensive development environment supports efficient development workflows. The CI/CD integration ensures reliability and consistency across different environments.

For questions or issues, please refer to the examples in `examples/nix/` or consult the detailed README in that directory. 