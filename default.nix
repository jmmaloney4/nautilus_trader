{ pkgs ? import <nixpkgs> {} }:

pkgs.python3Packages.buildPythonPackage rec {
  pname = "nautilus_trader";
  version = "1.216.0";

  src = pkgs.lib.cleanSource ./nautilus_trader;

  # Build-time dependencies: Rust toolchain, Clang, uv, Cython, etc.
  buildInputs = with pkgs; [
    rustc
    cargo
    clang
    uv
    python3Packages.setuptools
    python3Packages.cython
    python3Packages.poetry-core
    python3Packages.numpy
  ];

  # Any Python runtime dependencies NautilusTrader propagates
  propagatedBuildInputs = with pkgs.python3Packages; [ msgspec pandas ];

  # Pre-build hook to run uv sync (which installs extra dependencies and compiles Rust)
  preBuild = ''
    uv sync --all-extras
  '';

  # Optionally disable tests if they require extra infrastructure
  doCheck = false;

  meta = with pkgs.lib; {
    description = "High-performance, hybrid Python/Rust algorithmic trading platform";
    homepage = "https://nautilustrader.io";
    license = licenses.lgpl3Plus;
    maintainers = [ "yourName" ];
  };
}

