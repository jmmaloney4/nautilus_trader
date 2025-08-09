{
  description = "Dev shell for NautilusTrader (Python + Rust) with build/test dependencies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      python = pkgs.python312;
      pyPkgs = python.pkgs;
      pythonEnv = python.withPackages (ps: with ps; [
        pip
        setuptools
        wheel
        numpy
        cython
        pytest
        pytest-asyncio
        aiohttp
      ]);
    in {
      devShells.default = pkgs.mkShell {
        name = "nautilus-trader-devshell";
        packages = [
          pythonEnv
          pkgs.cargo
          pkgs.rustc
          pkgs.rustfmt
          pkgs.clang
          pkgs.cmake
          pkgs.pkg-config
          pkgs.openssl
          pkgs.git
        ];

        # Environment tweaks to align with build.py expectations
        shellHook = ''
          export RUSTUP_TOOLCHAIN=stable
          export BUILD_MODE=release
          export HIGH_PRECISION=true
          export PARALLEL_BUILD=true
          echo "Dev shell ready. Common commands:"
          echo "  python build.py"
          echo "  pytest -q tests/unit_tests/adapters/tastytrade"
        '';
      };
    }
  );
}


