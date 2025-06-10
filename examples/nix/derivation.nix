# Traditional Nix derivation for NautilusTrader
# This can be used in nixpkgs overlays or imported directly
{
  lib,
  buildPythonPackage,
  fetchFromGitHub,
  rustPlatform,
  pkg-config,
  clang,
  llvm,
  openssl,
  zlib,
  darwin,
  
  # Python dependencies
  cython,
  setuptools,
  poetry-core,
  numpy,
  click,
  fsspec,
  msgspec,
  pandas,
  pyarrow,
  pytz,
  tqdm,
  uvloop,
  
  # Optional adapter dependencies
  withBetfair ? false,
  betfair-parser,
  withInteractiveBrokers ? false,
  defusedxml,
  nautilus-ibapi,
  withDydx ? false,
  v4-proto,
  grpcio,
  protobuf,
  bech32,
  ecdsa,
  bip-utils,
  pycryptodome,
  
  # Build options
  highPrecision ? true,
  enableAllAdapters ? false
}:

let
  pname = "nautilus_trader";
  version = "1.219.0";
  
  # Rust dependencies
  rustDeps = rustPlatform.buildRustPackage {
    pname = "${pname}-rust";
    inherit version;
    
    src = fetchFromGitHub {
      owner = "nautechsystems";
      repo = "nautilus_trader";
      rev = "v${version}";
      sha256 = lib.fakeSha256;  # Replace with actual hash
    };
    
    cargoLock = {
      lockFile = ./Cargo.lock;
      outputHashes = {
        # Add any git dependencies here
      };
    };
    
    nativeBuildInputs = [
      pkg-config
      clang
      llvm
    ];
    
    buildInputs = [
      openssl
      zlib
    ] ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];
    
    buildFeatures = [ "ffi" "python" "extension-module" ] 
      ++ lib.optional highPrecision "high-precision"
      ++ lib.optional enableAllAdapters "defi";
    
    # Build only libraries, not binaries
    cargoBuildFlags = [ "--lib" ];
    
    doCheck = false;
    
    meta = {
      description = "Rust core for NautilusTrader";
      license = lib.licenses.lgpl3Plus;
    };
  };

in buildPythonPackage {
  inherit pname version;
  
  src = fetchFromGitHub {
    owner = "nautechsystems";
    repo = "nautilus_trader"; 
    rev = "v${version}";
    sha256 = lib.fakeSha256;  # Replace with actual hash
  };
  
  format = "pyproject";
  
  nativeBuildInputs = [
    pkg-config
    clang
    llvm
    rustPlatform.cargoSetupHook
    rustPlatform.rust.cargo
    rustPlatform.rust.rustc
    cython
    setuptools
    poetry-core
    numpy
  ];
  
  buildInputs = [
    openssl
    zlib
    rustDeps
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];
  
  propagatedBuildInputs = [
    click
    fsspec
    msgspec
    numpy
    pandas
    pyarrow
    pytz
    tqdm
  ] ++ lib.optionals (!stdenv.hostPlatform.isWindows) [
    uvloop
  ] ++ lib.optionals withBetfair [
    betfair-parser
  ] ++ lib.optionals withInteractiveBrokers [
    defusedxml
    nautilus-ibapi
  ] ++ lib.optionals withDydx [
    v4-proto
    grpcio
    protobuf
    bech32
    ecdsa
    bip-utils
    pycryptodome
  ];
  
  preBuild = ''
    export CARGO_HOME=$(mktemp -d cargo-home.XXX)
    export CARGO_TARGET_DIR="$PWD/target"
    
    # Copy pre-built Rust artifacts
    mkdir -p target/release
    cp -r ${rustDeps}/lib/* target/release/
    
    # Set build environment variables
    export BUILD_MODE=release
    export HIGH_PRECISION=${if highPrecision then "true" else "false"}
    export RUST_LIB_PATHS="${rustDeps}/lib"
    export PYO3_ONLY=false
    export DRY_RUN=false
    export COPY_TO_SOURCE=false
    
    # Run the custom build script
    python build.py
  '';
  
  # Skip tests during build - they require additional setup
  doCheck = false;
  
  # Basic import test
  pythonImportsCheck = [ "nautilus_trader" ];
  
  meta = with lib; {
    description = "High-performance algorithmic trading platform";
    longDescription = ''
      NautilusTrader is an open-source, high-performance, production-grade 
      algorithmic trading platform, providing quantitative traders with the 
      ability to backtest portfolios of automated trading strategies on 
      historical data with an event-driven engine, and also deploy those 
      same strategies live, with no code changes.
    '';
    homepage = "https://nautilustrader.io";
    changelog = "https://github.com/nautechsystems/nautilus_trader/blob/master/RELEASES.md";
    license = licenses.lgpl3Plus;
    maintainers = with maintainers; [ /* your-name-here */ ];
    platforms = platforms.unix;
    # Rust requirement
    broken = stdenv.hostPlatform.isWindows && highPrecision;
  };
  
  passthru = {
    inherit rustDeps highPrecision;
    
    # Provide variants with different adapter sets
    withAdapters = adapterList: 
      buildPythonPackage.override {
        withBetfair = builtins.elem "betfair" adapterList;
        withInteractiveBrokers = builtins.elem "ib" adapterList;
        withDydx = builtins.elem "dydx" adapterList;
      };
      
    # Standard precision variant
    standardPrecision = buildPythonPackage.override {
      highPrecision = false;
    };
  };
} 