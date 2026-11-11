{
  description = "Engrenage - A VintageStory server dashboard";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/snapshot/2026-03-11";
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
              # Rust toolchain
              rust-bin.stable.latest.default
              rust-analyzer
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

              # Extra deps
              pkg-config
              clang
            ];
          };
        }
      );
    };
}
