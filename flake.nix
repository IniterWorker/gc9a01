{
  description = "GC9A01 SPI display driver (gc9a01-rs) — reproducible Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSystem = nixpkgs.lib.genAttrs systems;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      version = cargoToml.package.version;

      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

      # Toolchain mirrors CI (.github/workflows/rust.yml): stable with
      # clippy + rustfmt and the thumbv7em-none-eabihf stdlib for the
      # no_std cross check.
      toolchainFor =
        pkgs:
        pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "clippy"
            "rust-src"
            "rustfmt"
            "rust-analyzer"
          ];
          targets = [ "thumbv7em-none-eabihf" ];
        };

      packageFor =
        system: features:
        let
          pkgs = pkgsFor system;
          rustToolchain = toolchainFor pkgs;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
        in
        rustPlatform.buildRustPackage {
          pname = "gc9a01-rs";
          inherit version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          buildFeatures = features;
          # Tests + doctests run in the check phase; git hooks must not be
          # installed by the cargo-husky dev-dependency inside the sandbox.
          CARGO_HUSKY_DONT_INSTALL_HOOKS = "true";
          meta = {
            description = "SPI 4-wire driver for GC9A01 display controller";
            homepage = "https://github.com/IniterWorker/gc9a01";
            license = with pkgs.lib.licenses; [
              mit
              asl20
            ];
          };
        };
    in
    {
      packages = forEachSystem (system: {
        default = packageFor system [ ];
        async = packageFor system [ "async" ];
      });

      checks = forEachSystem (system: {
        default = self.packages.${system}.default;
        async = self.packages.${system}.async;
      });

      devShells = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            name = "gc9a01-dev";
            packages = [
              (toolchainFor pkgs)
              # Task runner and README generator used by the justfile and
              # the cargo-husky pre-commit hook (`just check-readme`).
              pkgs.just
              pkgs.cargo-readme
              pkgs.git
              # Formatter for this flake (`nix fmt`).
              pkgs.nixfmt
            ];
            shellHook = ''
              echo "gc9a01 dev shell — $(rustc --version) · thumbv7em-none-eabihf stdlib installed"
              echo "gates:   cargo test --features async · cargo clippy --all-targets --all-features -- -D warnings"
              echo "         cargo check --target thumbv7em-none-eabihf --all-features · just check-readme"
            '';
          };
        }
      );

      formatter = forEachSystem (system: (pkgsFor system).nixfmt);
    };
}
