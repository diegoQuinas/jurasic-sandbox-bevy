{ pkgs, ... }:

{
  name = "jurasic-sandbox-bevy";

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
    ];
  };

  packages = with pkgs; [
    bacon
    cargo-seek
    cargo-nextest
    cargo-generate
  ];

  scripts.watcher = {
    exec = ''
      watchexec -c -e rs \
      "cargo clippy && cargo test && cargo run"
      '';
    packages = [ pkgs.watchexec ];
  };

  enterShell = ''
    echo "Rust"
  '';

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };
}
