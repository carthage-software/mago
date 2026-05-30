{
  description = "Mago - devshell using rustup and php 8.4 + composer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        isDarwin = pkgs.stdenv.isDarwin;
        toolchain = "1.96.0";
        php = pkgs.php84;
        composer = pkgs.php84Packages.composer;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustup
            pkgs.pkg-config
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
            if ! rustup toolchain list | grep -q "^${toolchain}-"; then
              rustup toolchain install ${toolchain} --no-self-update --profile default --component rust-analyzer >/dev/null
            fi

            rustup override set ${toolchain} >/dev/null
          '';
        };
      });
}
