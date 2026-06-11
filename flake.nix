{
  description = "OpenCode config utilities";
  # ── Inputs ──────────────────────────────────────────────────────────────
  # nixpkgs        – package set
  # rust-overlay   – latest stable Rust toolchain (rustc, cargo, clippy, …)
  # llm-agents     – provides coderabbit-cli (auto-review tool)
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    llm-agents.url = "github:numtide/llm-agents.nix";
  };

  # ── Outputs ─────────────────────────────────────────────────────────────
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    llm-agents,
    ...
  }: let
    # ── Helpers ───────────────────────────────────────────────────────────
    systems = ["x86_64-linux"];

    # Nixpkgs instantiated *with* the Rust overlay so every system gets the
    # same toolchain (buildRustPackage + devShell).
    mkPkgs = system:
      import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

    # Map a function over each system with overlay‑patched pkgs.
    eachSystem = fn:
      nixpkgs.lib.genAttrs systems (system: fn system (mkPkgs system));

    # ── Tool derivations (shared by packages / apps / devShells) ──────────
    # Go:   opencode-model-tiers  — TUI/CLI for model tier assignments
    #        opencode-work-mode    — thin "work" shortcut over model-tiers
    # Rust: opencode-sessions     — browse/export OpenCode SQLite sessions
    mkTools = pkgs: rec {
      opencode-model-tiers = pkgs.buildGoModule {
        pname = "opencode-model-tiers";
        version = "0.1.0";

        src = ./tools/model-tiers;
        vendorHash = "sha256-i7iMej6SH9OCeO5SXt/DcfDBH+VVFbw+HD0S6XwXABY=";

        env.CGO_ENABLED = "0";
        ldflags = ["-s" "-w"];

        checkPhase = ''
          runHook preCheck
          go test ./...
          runHook postCheck
        '';

        meta = {
          description = "TUI/CLI for opencode # LOW/# MED/# HIGH model tier assignments";
          mainProgram = "opencode-model-tiers";
        };
      };

      opencode-work-mode = pkgs.writeShellApplication {
        name = "opencode-work-mode";
        runtimeInputs = [opencode-model-tiers];
        text = ''
          exec opencode-model-tiers work "$@"
        '';
        meta = {
          description = "Shortcut that applies opencode work-mode model tiers";
          mainProgram = "opencode-work-mode";
        };
      };

      opencode-sessions = pkgs.rustPlatform.buildRustPackage {
        pname = "opencode-sessions";
        version = "0.1.0";

        src = ./tools/opencode-sessions;
        cargoLock.lockFile = ./tools/opencode-sessions/Cargo.lock;

        meta = {
          description = "Browse and export OpenCode conversations from local SQLite";
          mainProgram = "opencode-sessions";
        };
      };

      default = opencode-model-tiers;
    };

    # ── Home‑Manager module ──────────────────────────────────────────────
    # Exported as homeManagerModules.default so the root NixOS flake can
    # import it directly.  Adds:
    #   • opencode & opencode-build wrapper scripts
    #   • the three CLI tools above
    #   • coderabbit-cli
    #   • MCP runtime deps (node, yarn, docker, ts, bun, go)
    #   • ~/.config/opencode → editable config symlink
    #   • ~/opencode           → convenience symlink to this repo
    homeModule = {
      pkgs,
      config,
      ...
    }: let
      system = pkgs.stdenv.hostPlatform.system;
      tools = self.packages.${system};

      opencodeRepo = "${config.home.homeDirectory}/nixos/users/sewer/home-manager/programs/opencode";
      opencodeSource = "${opencodeRepo}/opencode-source";
      opencodeBin = "${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin/opencode";

      # Thin wrapper: default to CWD, forwards args.
      opencodeScript = pkgs.writeShellScriptBin "opencode" ''
        export OPENCODE_ENABLE_EXA=1
        if [ "$#" -eq 0 ]; then
          exec ${opencodeBin} .
        else
          exec ${opencodeBin} "$@"
        fi
      '';

      # Rebuild the opencode‑source submodule (bun build).
      opencodeBuildScript = pkgs.writeShellScriptBin "opencode-build" ''
        set -euo pipefail
        pushd ${opencodeSource}/packages/opencode > /dev/null
        bun install
        bun run build --single
        popd > /dev/null
        chmod -R +x ${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin
      '';
    in {
      home.packages = [
        opencodeScript
        opencodeBuildScript

        # Built CLI tools — land on PATH after activation.
        tools.opencode-model-tiers
        tools.opencode-work-mode
        tools.opencode-sessions

        llm-agents.packages.${system}.coderabbit-cli

        # Runtime deps for MCP servers / local hacking.
        pkgs.nodejs
        pkgs.yarn
        pkgs.docker
        pkgs.typescript
        pkgs.bun
        pkgs.go
      ];

      # Editable config → ~/.config/opencode.
      home.file.".config/opencode".source =
        config.lib.file.mkOutOfStoreSymlink "${opencodeRepo}/config";

      # Repo shortcut → ~/opencode.
      home.file."opencode".source =
        config.lib.file.mkOutOfStoreSymlink opencodeRepo;
    };
  in {
    # ── Flake outputs ─────────────────────────────────────────────────────
    # nix build / nix run / nix develop all work from this repo directly.

    # nix build .#opencode-model-tiers   etc.
    packages = eachSystem (_system: pkgs: mkTools pkgs);

    # nix flake check  (builds + runs Go tests)
    checks = eachSystem (system: _pkgs: {
      opencode-model-tiers = self.packages.${system}.opencode-model-tiers;
      opencode-sessions = self.packages.${system}.opencode-sessions;
    });

    # nix run .#opencode-sessions -- tui
    apps = eachSystem (system: _pkgs: rec {
      opencode-model-tiers = {
        type = "app";
        program = "${self.packages.${system}.opencode-model-tiers}/bin/opencode-model-tiers";
        meta.description = "Open opencode model tier TUI/CLI";
      };

      opencode-work-mode = {
        type = "app";
        program = "${self.packages.${system}.opencode-work-mode}/bin/opencode-work-mode";
        meta.description = "Apply opencode work-mode model tiers";
      };

      opencode-sessions = {
        type = "app";
        program = "${self.packages.${system}.opencode-sessions}/bin/opencode-sessions";
        meta.description = "Browse and export OpenCode sessions";
      };

      default = opencode-model-tiers;
    });

    # nix develop  →  Go + Rust toolchains + built CLI tools on PATH.
    devShells = eachSystem (system: pkgs: let
      tools = self.packages.${system};
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src"];   # needed for rust-analyzer type info
      };
    in {
      default = pkgs.mkShell {
        packages = [
          # Go.
          pkgs.go
          pkgs.gopls
          pkgs.gotools

          # Rust (rust‑overlay gives rustc/cargo/rustfmt/clippy;
          # standalone rust-analyzer is fresher than the bundled preview).
          rustToolchain
          pkgs.rust-analyzer
          pkgs.pkg-config
          pkgs.stdenv.cc

          # Built CLI tools — ready to run inside the shell.
          tools.opencode-model-tiers
          tools.opencode-work-mode
          tools.opencode-sessions
        ];

        CGO_ENABLED = "0";
      };
    });

    # Consumed by the root NixOS flake as:
    #   inputs.opencode-config.homeManagerModules.default
    homeManagerModules.default = homeModule;
  };
}
