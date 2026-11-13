{
  description = "Engrenage - A VintageStory server dashboard";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Rust toolchain derived from rust-toolchain.toml
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              bacon

              # Node toolchain
              nodejs_latest

              # DB toolchain
              sqlite
              sqlite-analyzer
              sqlitebrowser

              # Extra tools
              github-cli
              bruno
              git-cliff # Release-notes preview: `git cliff --latest --strip all`

              # Natively built chromium for the e2e suite on NixOS
              # (e2e/playwright.config.ts picks it up from PATH).
              chromium

              # Extra deps
              pkg-config
              clang
            ];
          };
        }
      );
    };
}
