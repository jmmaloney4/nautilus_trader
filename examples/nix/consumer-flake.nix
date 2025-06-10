{
  description = "Example project using NautilusTrader as a dependency";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    
    # Reference to nautilus_trader flake
    # In practice, this would point to your published repository
    nautilus-trader = {
      url = "github:your-org/nautilus_trader";  # Replace with actual URL
      # Or for local development:
      # url = "path:../..";
    };
  };

  outputs = { self, nixpkgs, flake-utils, nautilus-trader }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        
        # Get the nautilus_trader package for this system
        nautilusPackage = nautilus-trader.packages.${system}.nautilus-trader-full;
        
        # Your project's Python dependencies
        pythonEnv = pkgs.python3.withPackages (ps: [
          # Include nautilus_trader
          nautilusPackage
          
          # Your additional dependencies
          ps.jupyter
          ps.matplotlib
          ps.scipy
          ps.sklearn-learn
        ]);
        
        # Your application
        myTradingApp = pkgs.writeScriptBin "my-trading-app" ''
          #!${pythonEnv}/bin/python
          
          import nautilus_trader
          print(f"Using NautilusTrader version: {nautilus_trader.__version__}")
          
          # Your trading logic here
          from nautilus_trader.backtest.config import BacktestConfig
          from nautilus_trader.trading.strategy import Strategy
          
          print("✓ NautilusTrader imported successfully!")
        '';

      in
      {
        packages = {
          default = myTradingApp;
          
          # Python environment with nautilus_trader available
          python-env = pythonEnv;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            pkgs.jupyter
          ];
          
          shellHook = ''
            echo "Development environment with NautilusTrader"
            echo "NautilusTrader version: $(python -c 'import nautilus_trader; print(nautilus_trader.__version__)')"
            echo ""
            echo "Available commands:"
            echo "  python         - Python with NautilusTrader"
            echo "  jupyter lab    - Start Jupyter Lab"
            echo "  my-trading-app - Run example application"
          '';
        };

        apps = {
          # Run your application
          default = flake-utils.lib.mkApp {
            drv = myTradingApp;
          };
          
          # Start Jupyter with NautilusTrader available
          jupyter = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "jupyter-launcher" ''
              #!${pkgs.bash}/bin/bash
              exec ${pythonEnv}/bin/jupyter lab "$@"
            '';
          };
        };
      });
} 