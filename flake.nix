{
  description = "Jurasic Sandbox Bevy — headless Rust dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Toolchain: edition 2024 needs Rust >= 1.85; bevy 0.19 needs a
        # reasonably recent stable. Override with the stable channel and the
        # extra components used by this repo (rustfmt, clippy).
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        # rust-analyzer from the SAME toolchain as the crate (avoids
        # version-mismatch false errors in Zed/LSP).
        rustAnalyzer = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-analyzer" ];
        };

        devShell = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            rustAnalyzer
            pkgs.cargo-binstall
            pkgs.cargo-edit
          ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # The crate has no system-level dependencies (terminal Bevy only), so
        # a plain buildRustPackage with the existing Cargo.lock is enough.
        package = pkgs.rustPlatform.buildRustPackage rec {
          pname = "jurasic-sandbox-bevy";
          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          buildAndTestSubdir = null;
          doCheck = false; # no test harness yet; enable if tests are added

          nativeBuildInputs = [ rustToolchain ];
        };
      in
      {
        packages.default = package;
        packages.${package.pname} = package;

        devShells.default = devShell;

        formatter = pkgs.nixpkgs-fmt;

        checks = {
          inherit package;
          fmt = pkgs.runCommand "fmt-check" { } ''
            ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${self}/flake.nix
            touch $out
          '';
          clippy = pkgs.runCommand "clippy-check" { } ''
            cp -r ${self} $out-src
            chmod -R u+w $out-src
            cd $out-src
            ${rustToolchain}/bin/cargo clippy --locked -- -D warnings
            touch $out
          '';
        };
      });
}
