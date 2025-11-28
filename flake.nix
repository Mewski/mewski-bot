{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        { system, ... }:
        let
          overlays = [ rust-overlay.overlays.default ];
          pkgs' = import inputs.nixpkgs { inherit system overlays; };
          rustToolchain = pkgs'.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };
        in
        {
          devShells.default = pkgs'.mkShell {
            nativeBuildInputs = with pkgs'; [
              rustToolchain
              rust-analyzer
              pkg-config
              openssl
            ];

            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
    };
}
