{
  description = "NautilusTrader - High-performance algorithmic trading platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain specification matching rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.stable."1.87.0".default.override {
          extensions = [ "rust-src" "clippy" ];
        };

        # Crane lib for Rust builds
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common build inputs for all variants
        commonBuildInputs = with pkgs; [
          # Build tools
          pkg-config
          
          # C/C++ toolchain
          clang
          llvm
          
          # Python and Cython (headers included in main package)
          python3
          python3Packages.cython
          python3Packages.setuptools
          python3Packages.setuptools-rust
          python3Packages.poetry-core
          python3Packages.numpy
          maturin  # Better PyO3 integration
          
          # System libraries
          openssl
          zlib
          
          # For Redis integration (optional)
          libpq
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Python runtime dependencies
        pythonDeps = ps: with ps; [
          click
          fsspec
          msgspec
          numpy
          pandas
          pyarrow
          pytz
          tqdm
                 ] ++ pkgs.lib.optionals (!pkgs.stdenv.hostPlatform.isWindows) [
           uvloop
         ];

        # Adapter-specific dependencies
        adapterDeps = {
          betfair = ps: with ps; [ defusedxml ]; # Simplified dependencies
          ib = ps: with ps; [ defusedxml ]; # Interactive Brokers client
          docker = ps: with ps; [ docker ]; # Docker adapter
          dydx = ps: with ps; [ grpcio protobuf ]; # DyDx protocol deps
          polymarket = ps: []; # Placeholder for custom adapter packages
        };

        # Development dependencies
        devDeps = ps: with ps; [
          black mypy pytest pytest-asyncio pytest-cov
          pytest-mock coverage requests aiohttp
        ];

        # Source filtering for Rust builds
        rustSrc = craneLib.cleanCargoSource ./.;
        
        # Common cargo build arguments
        commonCargoArgs = {
          src = rustSrc;
          strictDeps = true;
          
          buildInputs = commonBuildInputs;
          
          # Environment variables for build
          CARGO_BUILD_INCREMENTAL = "false";
          RUST_BACKTRACE = "1";
          
          # PyO3 configuration for Python linking
          PYTHON_SYS_EXECUTABLE = "${pkgs.python3}/bin/python3";
          PYO3_PYTHON = "${pkgs.python3}/bin/python3";
          
          # Override PyO3 auto-detection with explicit paths
          PYO3_CROSS_PYTHON_VERSION = "${pkgs.python3.pythonVersion}";
          PYO3_CROSS_LIB_DIR = "${pkgs.python3}/lib";
          
          # OpenSSL configuration
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          
          # Python library configuration for PyO3
          PYTHON_INCLUDE_DIR = "${pkgs.python3}/include/python${pkgs.python3.pythonVersion}";
          PYTHON_LIB_DIR = "${pkgs.python3}/lib";
          
          # PKG_CONFIG_PATH for both OpenSSL and Python
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.python3}/lib/pkgconfig";
          
          # Direct PyO3 linking configuration
          # Additional library search paths for runtime
          LIBRARY_PATH = "${pkgs.python3}/lib:${pkgs.openssl.out}/lib";
          LD_LIBRARY_PATH = "${pkgs.python3}/lib:${pkgs.openssl.out}/lib";
          
          # Direct RUSTFLAGS to link Python library
          RUSTFLAGS = "-L ${pkgs.python3}/lib -l python${pkgs.python3.pythonVersion}";
        };

        # Cargo dependencies (for caching)
        cargoArtifacts = craneLib.buildDepsOnly (commonCargoArgs // {
          pname = "nautilus-trader-deps";
          # Add Python for PyO3 build scripts and pkg-config for OpenSSL
          nativeBuildInputs = [ pkgs.python3 pkgs.pkg-config ];
        });

        # Build Rust workspace with different precision modes
        buildRustWorkspace = { highPrecision ? true, features ? [] }: 
          let
            featureList = features ++ 
              (if highPrecision then ["high-precision"] else []) ++
              ["ffi" "python" "extension-module"];
                         featuresStr = pkgs.lib.concatStringsSep "," featureList;
          in
          craneLib.buildPackage (commonCargoArgs // {
            inherit cargoArtifacts;
            pname = "nautilus-trader-rust";
            
            cargoExtraArgs = "--features '${featuresStr}'";
            
            # Add Python for PyO3 linking and pkg-config for OpenSSL during build
            nativeBuildInputs = [ pkgs.python3 pkgs.pkg-config ];
            buildInputs = [ pkgs.python3 ];
            
            # Build profile
            CARGO_PROFILE = "release";
            BUILD_MODE = "release";
            HIGH_PRECISION = if highPrecision then "true" else "false";
            
            # Disable incremental builds for reproducibility
            CARGO_INCREMENTAL = "0";
            
            # Additional PyO3 environment variables
            PYTHONPATH = "${pkgs.python3}/${pkgs.python3.sitePackages}";
            
            # Override RUSTFLAGS to include Python linking for this specific build
            RUSTFLAGS = (commonCargoArgs.RUSTFLAGS or "") + " -L ${pkgs.python3}/lib -l python3.12";
            
            doCheck = false; # Skip tests in this phase
          });

        # Python package builder
        buildPythonPackage = { 
          pname
          , version ? "1.219.0"
          , rustWorkspace
          , adapters ? []
          , withDev ? false
          , highPrecision ? true 
        }: 
          pkgs.python3Packages.buildPythonPackage {
            inherit pname version;
            
            src = pkgs.lib.cleanSource ./.;
            
            format = "pyproject";
            
            nativeBuildInputs = [
              pkgs.python3Packages.poetry-core
              pkgs.python3Packages.cython
              pkgs.python3Packages.setuptools
              pkgs.python3Packages.numpy
              rustToolchain
              pkgs.pkg-config
              pkgs.clang
              pkgs.llvm
            ];
            
            buildInputs = [ 
              rustWorkspace 
              pkgs.python3
              pkgs.openssl
              pkgs.zlib
            ];
            
            propagatedBuildInputs = pythonDeps pkgs.python3Packages ++
              pkgs.lib.flatten (map (adapter: adapterDeps.${adapter} pkgs.python3Packages) adapters) ++
              pkgs.lib.optionals withDev (devDeps pkgs.python3Packages);
            
            # Set environment variables for the build
            CARGO_HOME = "$(mktemp -d cargo-home.XXX)";
            CARGO_TARGET_DIR = "$PWD/target";
            BUILD_MODE = "release";
            HIGH_PRECISION = if highPrecision then "true" else "false";
            # PYO3_ONLY - leave unset for false (empty string is falsy)
            # DRY_RUN - leave unset for false (empty string is falsy)  
            # COPY_TO_SOURCE - leave unset for false (empty string is falsy)
            PARALLEL_BUILD = "true";
            
            # PyO3 configuration
            PYTHON_SYS_EXECUTABLE = "${pkgs.python3}/bin/python3";
            PYO3_PYTHON = "${pkgs.python3}/bin/python3";
            PYO3_CROSS_PYTHON_VERSION = "${pkgs.python3.pythonVersion}";
            PYO3_CROSS_LIB_DIR = "${pkgs.python3}/lib";
            
            # Python library configuration
            PYTHON_INCLUDE_DIR = "${pkgs.python3}/include/python${pkgs.python3.pythonVersion}";
            PYTHON_LIB_DIR = "${pkgs.python3}/lib";
            
            # Custom build phase
            preBuild = ''
              # Set up cargo home and target directories
              export CARGO_HOME=$(mktemp -d cargo-home.XXX)
              export CARGO_TARGET_DIR="$PWD/target"
              
              # Copy Rust artifacts from pre-built workspace
              echo "Copying pre-built Rust artifacts..."
              mkdir -p target/release
              cp -r ${rustWorkspace}/lib/* target/release/ || true
              
              # Make copied files writable since Nix store files are read-only
              chmod -R u+w target/release/ || true
              
              # Patch build.py to skip Rust compilation since we have pre-built artifacts
              echo "Patching build.py to skip Rust compilation..."
              
              # Set environment variable to skip Rust build
              export SKIP_RUST_BUILD=1
              
              # Create a simple Python patch script
              python3 -c "
import re
import os

# Read the original build.py
with open('build.py', 'r') as f:
    content = f.read()

# Add the skip check at the beginning of _build_rust_libs function
pattern = r'(def _build_rust_libs\(\) -> None:\s*\n)(.*?print\(\"Compiling Rust libraries...\"\))'
replacement = r'\1    if os.environ.get(\"SKIP_RUST_BUILD\"):\n        print(\"Skipping Rust compilation - using pre-built artifacts\")\n        return\n\n\2'

new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)

# Write the patched version
with open('build.py', 'w') as f:
    f.write(new_content)

print('Successfully patched build.py')
"
              
              echo "Patching completed"
              
              # Patch pyproject.toml to relax build dependencies  
              echo "Relaxing build dependency requirements..."
              sed -i 's/setuptools>=80/setuptools/' pyproject.toml
              sed -i 's/cython==3.1.1/cython/' pyproject.toml
              
              # Run the custom build script
              echo "Running custom build script..."
              python build.py
            '';
            
            # Skip tests during build (run in check phase)
            doCheck = false;
            
            pythonImportsCheck = [ "nautilus_trader" ];
            
            meta = with pkgs.lib; {
              description = "High-performance algorithmic trading platform";
              homepage = "https://nautilustrader.io";
              license = licenses.lgpl3Plus;
              maintainers = [ maintainers.your-name-here ];
              platforms = platforms.unix;
            };
          };

      in
      {
        packages = rec {
          # Default package - high precision with core adapters
          default = nautilus-trader-full;
          
          # Core package - minimal dependencies
          nautilus-trader-core = buildPythonPackage {
            pname = "nautilus-trader-core";
            rustWorkspace = buildRustWorkspace { 
              highPrecision = true;
              features = [];
            };
          };
          
          # Standard precision variant
          nautilus-trader-std = buildPythonPackage {
            pname = "nautilus-trader-std";
            rustWorkspace = buildRustWorkspace { 
              highPrecision = false;
              features = [];
            };
            highPrecision = false;
          };
          
          # Full package with all adapters
          nautilus-trader-full = buildPythonPackage {
            pname = "nautilus-trader-full";
            rustWorkspace = buildRustWorkspace { 
              highPrecision = true;
              features = ["defi"];
            };
            adapters = [ "betfair" "ib" "dydx" "polymarket" ];
          };
          
          # Development package with all dependencies
          nautilus-trader-dev = buildPythonPackage {
            pname = "nautilus-trader-dev";
            rustWorkspace = buildRustWorkspace { 
              highPrecision = true;
              features = ["defi"];
            };
            adapters = [ "betfair" "ib" "dydx" "polymarket" ];
            withDev = true;
          };
          
          # Individual adapter packages
          nautilus-trader-betfair = buildPythonPackage {
            pname = "nautilus-trader-betfair";
            rustWorkspace = buildRustWorkspace { highPrecision = true; };
            adapters = [ "betfair" ];
          };
          
          nautilus-trader-ib = buildPythonPackage {
            pname = "nautilus-trader-ib";
            rustWorkspace = buildRustWorkspace { highPrecision = true; };
            adapters = [ "ib" ];
          };

          # Docker image
          docker-image = pkgs.dockerTools.buildLayeredImage {
            name = "nautilus-trader";
            tag = "latest";
            
            contents = [ 
              self.packages.${system}.nautilus-trader-full
              pkgs.python3
              pkgs.bash
              pkgs.coreutils
            ];
            
            config = {
              Env = [
                "PYTHONPATH=${self.packages.${system}.nautilus-trader-full}/${pkgs.python3.sitePackages}"
              ];
              Cmd = [ "${pkgs.python3}/bin/python" ];
              WorkingDir = "/workspace";
            };
          };
        };

        # Development shells
        devShells = {
          default = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ [
              rustToolchain
              pkgs.python3
              (pkgs.python3.withPackages (ps: pythonDeps ps ++ devDeps ps))
              
              # Additional development tools
              pkgs.uv
              pkgs.redis
              pkgs.docker-compose
              pkgs.just  # Modern make alternative
            ];
            
            shellHook = ''
              export RUST_SRC_PATH=${rustToolchain}/lib/rustlib/src/rust/library
              export CARGO_HOME=$PWD/.cargo
              export CARGO_TARGET_DIR=$PWD/target
              
              echo "NautilusTrader development environment"
              echo "Rust version: $(rustc --version)"
              echo "Python version: $(python --version)"
              echo ""
              echo "Available commands:"
              echo "  just build      - Build the project"
              echo "  just test       - Run tests"
              echo "  just fmt        - Format code"
              echo "  just check      - Run all checks"
            '';
          };

          # Rust-only development shell
          rust = pkgs.mkShell {
            buildInputs = [
              rustToolchain
              pkgs.pkg-config
              pkgs.openssl
            ];
          };

          # Python-only development shell  
          python = pkgs.mkShell {
            buildInputs = [
              (pkgs.python3.withPackages (ps: pythonDeps ps ++ devDeps ps))
              pkgs.uv
            ];
          };
        };

        # Apps for easy execution
        apps = {
          # Run nautilus trader CLI
          nautilus = flake-utils.lib.mkApp {
            drv = self.packages.${system}.nautilus-trader-full;
            exePath = "/bin/nautilus";
          };
          
          # Python with nautilus trader available
          python = flake-utils.lib.mkApp {
            drv = pkgs.python3.withPackages (ps: [ self.packages.${system}.nautilus-trader-full ]);
            exePath = "/bin/python";
          };
        };

        # Hydra jobs for CI
        hydraJobs = {
          inherit (self.packages.${system}) 
            nautilus-trader-core
            nautilus-trader-std  
            nautilus-trader-full
            nautilus-trader-dev;
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;
      });
} 