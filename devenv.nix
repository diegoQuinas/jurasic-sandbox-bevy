{ pkgs, ... }:

{
  name = "jurasic-sandbox-bevy";

  # Bevy 0.19 + edition 2024 need a recent stable (rust-overlay, not nixpkgs).
  languages.rust = {
    enable = true;
    channel = "nightly";
    version = "latest";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "rust-src"
    ];
  };

  packages = [
    pkgs.cargo-binstall
    pkgs.cargo-edit
    pkgs.watchexec
  ];

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };

  enterShell = ''
    echo "jurasic-sandbox-bevy — rust $(rustc --version)"
  '';
}
