{
  description = "Prometheus graph - a tool for visualizing PromQL queries";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

    mkPkgs = system:
      import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

    mkPackage = system: let
      pkgs = mkPkgs system;
      rustToolchain = pkgs.rust-bin.stable.latest.minimal;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      src = craneLib.cleanCargoSource ./.;

      commonArgs = {
        inherit src;
        pname = "promegraph";
        version = "0.1.0";

        buildInputs =
          pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.fontconfig
            pkgs.openssl
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.apple-sdk_15
          ];

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in
      craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;
        });
  in {
    packages = forAllSystems (system: {
      default = mkPackage system;
    });

    devShells = forAllSystems (system: let
      pkgs = mkPkgs system;
      rustToolchainDev = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src" "clippy"];
      };
    in {
      default = pkgs.mkShell {
        inputsFrom = [self.packages.${system}.default];
        buildInputs = [
          rustToolchainDev
          pkgs.rust-analyzer
          pkgs.cargo-watch
          pkgs.cargo-edit
        ];

        RUST_SRC_PATH = rustToolchainDev + "/lib/rustlib/src/rust/library";
      };
    });

    apps = forAllSystems (system: {
      default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/promegraph";
      };
    });
  };
}
