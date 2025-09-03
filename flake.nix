{
  description = "Mago - devshell using rustup (stable 1.89.0 + nightly) and php 8.4 + composer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        isDarwin = pkgs.stdenv.isDarwin;
        php = pkgs.php84;
        composer = pkgs.php84Packages.composer;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustup
            pkgs.rust-analyzer
            pkgs.pkg-config
            pkgs.openssl
            pkgs.just
            pkgs.wasm-pack
            php
            composer
          ] ++ pkgs.lib.optionals isDarwin [
            pkgs.libiconv
          ];

          NIX_LDFLAGS = pkgs.lib.optionalString isDarwin ''
            -framework Security -framework SystemConfiguration
          '';

          OPENSSL_NO_VENDOR = 1;
          RUSTFLAGS = "-C debuginfo=1";
          CARGO_TERM_COLOR = "always";
          CARGO_INCREMENTAL = "1";

          shellHook = ''
            export PATH="$HOME/.cargo/bin:$PATH"
            if ! command -v rustc >/dev/null 2>&1; then
              rustup toolchain install 1.89.0 --profile minimal
              rustup toolchain install nightly --profile minimal
              rustup default 1.89.0
            fi
            echo "[mago] rustc:     $(rustc --version)"
            echo "[mago] nightly:   $(rustup run nightly rustc --version)"
            echo "[mago] php:       $(php -v | head -n1)"
            echo "[mago] composer:  $(composer --version)"
            echo "[mago] Run: just build | just test | just lint | just fix | just build-wasm"
          '';
        };
      });
}
