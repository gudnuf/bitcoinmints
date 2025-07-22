{
  description = "Bitcoin Mints Retyr - Rust development environment with auto-formatting for Rust and Nix files";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        # Auto-formatter script that watches for file changes
        auto-formatter = pkgs.writeShellScriptBin "auto-format" ''
          echo "🚀 Starting auto-formatter..."
          echo "Watching for changes in src/ directory (Rust files) and root directory (Nix files)..."
          echo "Press Ctrl+C to stop"
          
          # Start Rust formatter
          ${pkgs.watchexec}/bin/watchexec \
            --watch src \
            --exts rs \
            --on-busy-update restart \
            --shell none \
            -- ${rustToolchain}/bin/rustfmt --edition 2021 --emit files \
            $(find src -name "*.rs" -type f) &
          
          # Start Nix formatter
          ${pkgs.watchexec}/bin/watchexec \
            --watch . \
            --exts nix \
            --ignore-paths .git \
            --ignore-paths target \
            --ignore-paths result \
            --on-busy-update restart \
            --shell none \
            -- ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt \
            $(find . -name "*.nix" -type f -not -path "./.git/*" -not -path "./target/*" -not -path "./result/*") &
          
          # Wait for both background processes
          wait
        '';

        # Format all files once
        format-all = pkgs.writeShellScriptBin "format-all" ''
          echo "🎨 Formatting all Rust and Nix files..."
          
          # Format Rust files
          echo "  📦 Formatting Rust files..."
          find src -name "*.rs" -type f -exec ${rustToolchain}/bin/rustfmt --edition 2021 --emit files {} \;
          
          # Format Nix files
          echo "  ❄️  Formatting Nix files..."
          find . -name "*.nix" -type f -not -path "./.git/*" -not -path "./target/*" -not -path "./result/*" -exec ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt {} \;
          
          echo "✅ All files formatted!"
        '';

        # Development shell script that starts auto-formatting
        dev-with-formatting = pkgs.writeShellScriptBin "dev-with-formatting" ''
          echo "🔧 Bitcoin Mints Retyr Development Environment"
          echo "================================================"
          echo ""
          echo "Available commands:"
          echo "  auto-format     - Start file watcher for auto-formatting (Rust + Nix)"
          echo "  format-all      - Format all Rust and Nix files once"
          echo "  cargo build     - Build the project"
          echo "  cargo test      - Run tests"
          echo "  cargo run       - Run the application"
          echo ""
          echo "Starting auto-formatter in background..."
          
          # Format all files first
          ${format-all}/bin/format-all
          
          # Start auto-formatter in background
          ${auto-formatter}/bin/auto-format &
          AUTO_FORMATTER_PID=$!
          
          echo "Auto-formatter started (PID: $AUTO_FORMATTER_PID)"
          echo "Your Rust and Nix files will be automatically formatted on save!"
          echo ""
          echo "Press Ctrl+C to stop auto-formatter and exit dev shell"
          
          # Set up trap to kill auto-formatter when shell exits
          trap "echo 'Stopping auto-formatter...'; kill $AUTO_FORMATTER_PID 2>/dev/null; exit" INT TERM
          
          # Start a new shell session
          ${pkgs.zsh}/bin/zsh
        '';

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Development tools
            cargo-watch
            watchexec

            # Nix formatting
            nixpkgs-fmt

            # Database tools (for your SQLite database)
            sqlite

            # Network tools
            curl

            # Build dependencies for cdk-signatory and protobuf
            pkg-config
            openssl
            openssl.dev
            cmake
            gcc
            gnumake
            perl
            python3
            protobuf
            zlib
            libz

            # Our custom scripts
            auto-formatter
            format-all
            dev-with-formatting

            # Additional useful tools
            libiconv
          ] ++ lib.optionals stdenv.isDarwin [
            # macOS specific dependencies
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          shellHook = ''
            echo "🔧 Bitcoin Mints Retyr Development Environment"
            echo "================================================"
            echo ""
            echo "🦀 Rust toolchain: $(rustc --version)"
            echo "📦 Cargo: $(cargo --version)"
            echo "❄️  Nix formatter: $(nixpkgs-fmt --version)"
            echo "🔧 Protobuf compiler: $(protoc --version)"
            echo ""
            echo "Available commands:"
            echo "  dev-with-formatting - Start dev environment with auto-formatting (Rust + Nix)"
            echo "  auto-format         - Start file watcher for auto-formatting (Rust + Nix)"
            echo "  format-all          - Format all Rust and Nix files once"
            echo ""
            echo "💡 Quick start: run 'dev-with-formatting' to begin!"
            echo ""
          '';

          # Environment variables
          RUST_BACKTRACE = "1";
          RUST_LOG = "debug";
          # Ensure protoc is found by build scripts
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";
        };

        # Additional outputs for convenience
        packages = {
          inherit auto-formatter format-all dev-with-formatting;
          default = dev-with-formatting;
        };

        # Formatter for the flake itself
        formatter = pkgs.nixpkgs-fmt;
      });
}
