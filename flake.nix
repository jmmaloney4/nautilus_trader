{
  description = "Nautilus Trader - High-performance algorithmic trading platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    
    flake-utils.url = "github:numtide/flake-utils";
    
    # Rust toolchain management
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # Python dependency management with UV
    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    pyproject-nix,
    uv2nix,
    pyproject-build-systems,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (nixpkgs) lib;

        # Use Rust 1.87.0 stable as specified in rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.stable."1.87.0".default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        # Create crane lib with our custom toolchain
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Load UV workspace configuration
        workspace = uv2nix.lib.workspace.loadWorkspace { 
          workspaceRoot = ./.; 
        };

        # Python interpreter to use (matching Dockerfile)
        python = pkgs.python313;

        # Common native build inputs required by the project
        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          capnproto
          clang
          rustToolchain
        ];

        buildInputs = with pkgs; [
          openssl
          libcap
          # Add capnproto development libraries
          capnproto
          # Python library for PyO3 linking
          python
        ] ++ lib.optionals pkgs.stdenv.isDarwin [
          libiconv
          darwin.apple_sdk.frameworks.Security
        ];

        # Clean source (excluding build artifacts)
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        # Common cargo arguments for the build
        commonCargoArgs = {
          inherit src buildInputs nativeBuildInputs;
          strictDeps = true;
          
          # Set environment variables to match build.py
          RUSTUP_TOOLCHAIN = "stable";
          BUILD_MODE = "release";
          CC = "clang";
          CXX = "clang++";
          LDSHARED = "clang -shared";
          
          # Enable high precision mode (default in the project)
          HIGH_PRECISION = "true";
          
          # Required for linking with Python (matching build.py)
          PYO3_PYTHON = "${python}/bin/python";
          
          # Set PYTHON_LIB_DIR as build.py expects
          PYTHON_LIB_DIR = "${python}/lib";
          
          # Platform-specific RUSTFLAGS for Python linking (matching build.py logic)
          RUSTFLAGS = lib.optionalString pkgs.stdenv.isLinux 
            "-C link-arg=-L${python}/lib -C link-arg=-lpython${python.pythonVersion}";
          
          # Cargo target directory (matching build.py)
          CARGO_TARGET_DIR = "target";
          
          # Optimize for release builds
          CARGO_PROFILE_RELEASE_LTO = "thin";
          CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
        };

        # Build dependencies separately for faster incremental builds
        cargoArtifacts = craneLib.buildDepsOnly (commonCargoArgs // {
          pname = "nautilus-trader-deps";
          version = "0.49.0";
          # Don't run tests during dependency building
          doCheck = false;
        });

        # Build individual Rust libraries (following the build.py logic)
        rustLibraries = craneLib.buildPackage (commonCargoArgs // {
          inherit cargoArtifacts;
          pname = "nautilus-trader-rust";
          version = "0.49.0";
          
          # Build with high-precision mode (default and recommended)
          cargoExtraArgs = "--lib --features high-precision,ffi,python,extension-module";
          
          # Don't run tests during library build (we only need the compiled libraries)
          doCheck = false;
          
          # The build script expects these specific libraries
          postInstall = ''
            mkdir -p $out/lib
            # Copy the built static libraries that Python extensions will link against
            for lib in nautilus_backtest nautilus_common nautilus_core nautilus_model nautilus_persistence; do
              if [ -f target/release/lib''${lib}.a ]; then
                cp target/release/lib''${lib}.a $out/lib/
              fi
            done
            
            # Also copy any dynamic libraries for PyO3 extensions  
            find target/release -name "*.so" -type f -exec cp {} $out/lib/ \;
          '';
        });

        # Create overlay for Python packages
        pythonOverlay = workspace.mkPyprojectOverlay {
          sourcePreference = "wheel"; # Prefer binary wheels for faster builds
        };

        # Additional Python package overrides
        pyprojectOverrides = final: prev: {
          # Override nautilus_trader to use our Rust libraries
          nautilus-trader = prev.nautilus-trader.overrideAttrs (old: {
            nativeBuildInputs = (old.nativeBuildInputs or []) ++ nativeBuildInputs ++ [
              # Add Cython for building extensions
              final.cython
              final.setuptools
              final.numpy
            ];
            
            buildInputs = (old.buildInputs or []) ++ buildInputs;
            
            # Remove build script reference
            prePatch = ''
              sed -E -i '/^[[:space:]]*build[[:space:]]*=.*build\.py.*/d' pyproject.toml
              # Replace build.py with a no-op to prevent accidental execution
              echo 'print("noop build.py")' > build.py
            '';
            
            # Make Rust libraries available during build
            preBuild = ''
              # Environment for PyO3 setuptools-rust to find prebuilt libs
              export RUST_LIB_PATHS="${rustLibraries}/lib"
              export PYO3_ONLY=true

              # Use previously vendored Cargo registry for offline build.py
              export CARGO_HOME=${cargoArtifacts}
              export CARGO_NET_OFFLINE=true
            '';
            
            # The project uses a custom build script
            format = "pyproject";
            
            # Disable tests during build (they require special setup)
            doCheck = false;
            
            # Include Rust source in the source distribution as required
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                (lib.fileset.fromSource old.src)
                ./crates
                ./Cargo.toml
                ./Cargo.lock
                ./build.py
                ./rust-toolchain.toml
              ];
            };

            # After wheel installation, copy the pre-built Rust shared libraries next to the Python package
            postInstall = ''
              # Copy pre-built Rust shared libraries (*.so) next to the installed package
              target_site=$(python - <<'PY'
import sysconfig, pathlib, json, sys; print(pathlib.Path(sysconfig.get_paths()["purelib"]))
PY
              )
              cp -v ${rustLibraries}/lib/*.so "$target_site" 2>/dev/null || true
            '';
          });
        };

        # Construct Python package set
        pythonSet = (pkgs.callPackage pyproject-nix.build.packages {
          inherit python;
        }).overrideScope (lib.composeManyExtensions [
          pyproject-build-systems.overlays.default
          pythonOverlay  
          pyprojectOverrides
        ]);

      in {
        # Sub-packages broken down logically
        packages = {
          # Rust libraries only
          rust-libraries = rustLibraries;
          
          # Core Python package with Rust extensions
          nautilus-trader = pythonSet.nautilus-trader;
          
          # Full application with all dependencies
          default = pythonSet.mkVirtualEnv "nautilus-trader-env" workspace.deps.default;
          
          # Development environment with all optional dependencies
          dev-env = pythonSet.mkVirtualEnv "nautilus-trader-dev-env" workspace.deps.all;
          
          # Individual adapter packages (examples)
          adapters-betfair = pythonSet.mkVirtualEnv "nautilus-betfair-env" 
            (workspace.deps.default ++ workspace.deps.groups.betfair or []);
          adapters-dydx = pythonSet.mkVirtualEnv "nautilus-dydx-env"
            (workspace.deps.default ++ workspace.deps.groups.dydx or []);
          adapters-ib = pythonSet.mkVirtualEnv "nautilus-ib-env"
            (workspace.deps.default ++ workspace.deps.groups.ib or []);
        };

        # Development shells
        devShells = {
          # Pure Nix development environment
          default = craneLib.devShell {
            checks = self.checks.${system};
            packages = with pkgs; [
              # Core development tools
              uv
              python
              rustToolchain
              
              # Build tools
              cmake
              pkg-config
              capnproto
              clang
              
              # Development utilities  
              ruff
              mypy
              black
              pre-commit
              
              # Documentation
              mdbook
            ] ++ buildInputs;
            
            env = {
              # UV configuration
              UV_PYTHON = python.interpreter;
              UV_PYTHON_DOWNLOADS = "never";
              
              # Rust configuration
              RUSTUP_TOOLCHAIN = "stable";
              CC = "clang";
              
              # Python configuration
              PYO3_PYTHON = "${python}/bin/python";
              HIGH_PRECISION = "true";
              BUILD_MODE = "release";
            } // lib.optionalAttrs pkgs.stdenv.isLinux {
              LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
            };
            
            shellHook = ''
              echo "Nautilus Trader Development Environment"
              echo "Python: ${python}/bin/python"
              echo "Rust: $(rustc --version)"
              echo "UV: $(uv --version)"
              echo ""
              echo "Run 'uv sync' to set up Python dependencies"
              echo "Run 'cargo build --release' to build Rust components"
            '';
          };
          
          # Hybrid development environment using UV for Python deps
          impure = pkgs.mkShell {
            packages = with pkgs; [
              uv
              python
              rustToolchain
            ] ++ nativeBuildInputs ++ buildInputs;
            
            env = {
              UV_PYTHON = python.interpreter;
              UV_PYTHON_DOWNLOADS = "never";
              RUSTUP_TOOLCHAIN = "stable";
              CC = "clang";
              PYO3_PYTHON = "${python}/bin/python";
            } // lib.optionalAttrs pkgs.stdenv.isLinux {
              LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
            };
            
            shellHook = ''
              unset PYTHONPATH
              echo "Nautilus Trader Hybrid Development Environment"
              echo "Use 'uv sync' to manage Python dependencies with UV"
              echo "Use 'cargo build --release' to build Rust components"
            '';
          };
        };

        # Checks (tests and linting)
        checks = {
          # Rust checks
          rust-clippy = craneLib.cargoClippy (commonCargoArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          
          rust-fmt = craneLib.cargoFmt {
            inherit src;
          };
          
          rust-tests = craneLib.cargoNextest (commonCargoArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
          
          # Build checks
          rust-build = rustLibraries;
          python-build = self.packages.${system}.nautilus-trader;
        };

        # Formatter
        formatter = pkgs.writeShellApplication {
          name = "nautilus-formatter";
          runtimeInputs = with pkgs; [ rustToolchain ruff black ];
          text = ''
            echo "Formatting Rust code..."
            cargo fmt
            
            echo "Formatting Python code..."
            ruff format .
            black .
            
            echo "Linting..."
            ruff check .
            cargo clippy
          '';
        };

        # Container images (if desired)
        # This would mirror the Dockerfile structure
        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "nautilus-trader";
          tag = "latest";
          
          contents = [ self.packages.${system}.default ];
          
          config = {
            Env = [
              "PYTHONUNBUFFERED=1"
              "PYTHONDONTWRITEBYTECODE=1"
            ];
            WorkingDir = "/app";
            Entrypoint = [ "${self.packages.${system}.default}/bin/python" ];
          };
        };
      });
} 