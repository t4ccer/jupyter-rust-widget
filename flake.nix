{
  description = "jupyter-rust-widget";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable-small";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    pre-commit-hooks-nix = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    hercules-ci-effects = {
      url = "github:hercules-ci/hercules-ci-effects";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-parts.follows = "flake-parts";
    };
  };
  outputs = inputs @ {self, ...}:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} ({withSystem, ...}: {
      imports = [
        inputs.pre-commit-hooks-nix.flakeModule
        inputs.hercules-ci-effects.flakeModule
      ];

      # `nix flake show --impure` hack
      systems =
        if builtins.hasAttr "currentSystem" builtins
        then [builtins.currentSystem]
        else inputs.nixpkgs.lib.systems.flakeExposed;

      herculesCI.ciSystems = ["x86_64-linux"];

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        lib,
        system,
        ...
      }: let
        rustToolchain = pkgs.rust-bin.fromRustupToolchain {
          channel = "stable";
          components = ["rust-analyzer" "rust-src" "rustfmt" "rustc" "cargo"];
          targets = [
            "x86_64-unknown-linux-gnu"
            "x86_64-unknown-linux-musl"
            "wasm32-unknown-unknown"
          ];
        };

        pythonToolchain = "python313";

        hostPkgs = pkgs;
      in {
        _module.args.pkgs = import self.inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.rust-overlay
          ];
        };

        pre-commit.settings = {
          src = ./.;
          hooks = {
            alejandra.enable = true;
            rustfmt = {
              enable = true;
              args = ["--style-edition=2024"];
            };
            typos = {
              enable = true;
              settings.ignored-words = [
                "nimber"
                "numer" # `numerator` from `num-rational`
              ];
            };
          };
          tools = {
            rustfmt = lib.mkForce rustToolchain;
            clippy = lib.mkForce rustToolchain;
          };
        };

        devShells.default = pkgs.mkShell {
          shellHook = ''
            ${config.pre-commit.installationScript}
            if [ ! -d .venv ]; then
              python -m venv .venv
            fi
            source .venv/bin/activate
            export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.zlib}/lib:${pkgs.glib}/lib:$LD_LIBRARY_PATH"
          '';

          hardeningDisable = ["fortify"];

          nativeBuildInputs = [
            (pkgs.${pythonToolchain}.withPackages (ps: with ps; [pip jupyter anywidget]))
            # pkgs.${pythonToolchain}.pkgs.pip
            # pkgs.${pythonToolchain}.pkgs.jupyter
            pkgs.maturin
            pkgs.wasm-pack
            rustToolchain

            pkgs.stdenv.cc.cc.lib
            pkgs.zlib
            pkgs.glib

            pkgs.webpack-cli
          ];
        };
        formatter = pkgs.alejandra;
      };
    });
}
