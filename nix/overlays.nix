# Additional package overlays for Nautilus Trader specific requirements
final: prev: {
  
  # Custom UV package if we need a specific version
  uv = prev.uv.overrideAttrs (old: rec {
    version = "0.7.12";  # Match the version in uv-version file
    src = prev.fetchFromGitHub {
      owner = "astral-sh";
      repo = "uv";
      rev = version;
      hash = "sha256-PLACEHOLDER"; # You would need to update this with the actual hash
    };
  });

  # Enhanced Python package with additional build support
  python313 = prev.python313.override {
    packageOverrides = python-final: python-prev: {
      
      # Cython with specific version for compatibility
      cython = python-prev.cython.overridePythonAttrs (old: rec {
        version = "3.1.2";
        src = prev.fetchPypi {
          pname = "Cython";
          inherit version;
          hash = "sha256-PLACEHOLDER"; # Update with actual hash
        };
      });
      
      # Ensure numpy compatibility for the build process
      numpy = python-prev.numpy.overridePythonAttrs (old: {
        # Any specific numpy configuration if needed
        buildInputs = old.buildInputs ++ [ prev.blas prev.lapack ];
      });
      
      # msgspec package (core dependency)
      msgspec = python-prev.msgspec.overridePythonAttrs (old: {
        # Ensure rust is available for msgspec builds if needed
        nativeBuildInputs = old.nativeBuildInputs ++ [ prev.rustc prev.cargo ];
      });
      
    };
  };

  # Capnproto with proper development headers
  capnproto = prev.capnproto.overrideAttrs (old: {
    outputs = [ "out" "dev" "lib" ];
    postInstall = (old.postInstall or "") + ''
      # Ensure headers are properly linked for development
      ln -sf $dev/include $out/include
    '';
  });

  # Enhanced clang for the build process
  clang = prev.clang.overrideAttrs (old: {
    # Any specific clang configurations if needed
  });

} 