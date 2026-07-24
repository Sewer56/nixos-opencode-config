{
  description = "OpenCode config utilities";
  # ── Inputs ──────────────────────────────────────────────────────────────
  # nixpkgs      – package set
  # rust-overlay – latest stable Rust toolchain (rustc, cargo, clippy, …)
  # llm-agents   – provides coderabbit-cli (auto-review tool)
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
    # Build entire Rust workspace *once* – all crates share the same
    # dependency tree so compiling 5× was pure waste.  Each per‑tool
    # derivation below just plucks its binary from this shared build.
    mkTools = pkgs: let
      workspaceDrv = pkgs.rustPlatform.buildRustPackage {
        pname = "opencode-tools";
        version = "0.1.0";

        src = ./tools;
        cargoLock.lockFile = ./tools/Cargo.lock;

        # Build every workspace member together → one compilation unit.
        # Default cargoInstallHook copies *all* built binaries to $out/bin.
        cargoBuildFlags = ["--workspace"];
      };

      # Derive a single‑binary package from the shared workspace build.
      mkTool = {
        pname,
        description,
        binary ? pname,
      }:
        pkgs.runCommand pname {
          meta = {
            inherit description;
            mainProgram = binary;
          };
        } ''
          mkdir -p $out/bin
          cp ${workspaceDrv}/bin/${binary} $out/bin/
        '';
    in rec {
      opencode-model-switcher = mkTool {
        pname = "opencode-model-switcher";
        description = "TUI/CLI for opencode # LOW/# MED/# HIGH model tier assignments";
      };

      opencode-sessions = mkTool {
        pname = "opencode-sessions";
        description = "Browse and export OpenCode conversations from local SQLite";
      };

      chunk-files-by-tokens = mkTool {
        pname = "chunk-files-by-tokens";
        description = "Chunk files by estimated token count";
      };

      token-count-after-expand = mkTool {
        pname = "token-count-after-expand";
        description = "Estimate prompt token counts after md-expand rendering";
      };

      # rust-llm-tidy lives in a git submodule with its own workspace.
      # Pure flakes cannot track submodule files, so it is built at runtime
      # via the cargo-backed wrapper in the Home-Manager module.
      default = opencode-model-switcher;
    };

    # ── Home‑Manager module ──────────────────────────────────────────────
    # Exported as homeManagerModules.default so the root NixOS flake can
    # import it directly.  Adds:
    #   • opencode & opencode-build wrapper scripts
    #   • CLI tools above
    #   • coderabbit-cli
    #   • MCP/runtime deps (node, yarn, docker, bun)
    #   • ~/.config/opencode → editable config symlink
    #   • ~/opencode           → convenience symlink to this repo
    homeModule = {
      pkgs,
      config,
      ...
    }: let
      system = pkgs.stdenv.hostPlatform.system;

      opencodeRepo = "${config.home.homeDirectory}/nixos/users/sewer/home-manager/programs/opencode";
      opencodeSource = "${opencodeRepo}/opencode-source";
      opencodeBin = "${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin/opencode";

      # Thin wrapper: default to CWD, forwards args. Runs with Exa search enabled.
      opencodeScript = pkgs.writeShellScriptBin "opencode" ''
        export OPENCODE_ENABLE_EXA=1
        if [ "$#" -eq 0 ]; then
          exec ${opencodeBin} .
        else
          exec ${opencodeBin} "$@"
        fi
      '';

      # Install runtime deps for local plugins. Plugins with runtime deps
      # (e.g. xdg-basedir) fail their dynamic import inside OpenCode silently
      # when node_modules is missing, so this must run on fresh checkouts.
      pluginDepsScript = pkgs.writeShellScriptBin "opencode-plugin-deps" ''
        set -uo pipefail
        failed=0
        for dir in ${opencodeRepo}/config/plugins/*/; do
          if [ ! -f "$dir/package.json" ]; then
            # Empty dir = uninitialized submodule (clone without --recurse-submodules).
            if [ -z "$(ls -A "$dir" 2>/dev/null)" ]; then
              echo "error: $dir is empty - uninitialized submodule? run: git submodule update --init" >&2
              failed=1
            fi
            continue
          fi
          # Only install when the plugin declares runtime dependencies.
          # jq parse failure (malformed package.json) must be surfaced, not skipped.
          if ! deps=$(${pkgs.jq}/bin/jq '.dependencies | length' "$dir/package.json" 2>&1); then
            echo "error: invalid package.json in $dir: $deps" >&2
            failed=1
            continue
          fi
          [ "$deps" -gt 0 ] 2>/dev/null || continue
          echo "installing plugin deps: $dir"
          # Frozen lockfile when committed (submodules) so installs never
          # rewrite bun.lock and dirty the tree; caveman has no lockfile.
          if [ -f "$dir/bun.lock" ] || [ -f "$dir/bun.lockb" ]; then
            (cd "$dir" && ${pkgs.bun}/bin/bun install --production --frozen-lockfile) || failed=1
          else
            (cd "$dir" && ${pkgs.bun}/bin/bun install --production) || failed=1
          fi
        done
        exit $failed
      '';

      # Rebuild the opencode‑source submodule (bun build).
      # I often iterate, so separate build via `opencode-build` command will do.
      opencodeBuildScript = pkgs.writeShellScriptBin "opencode-build" ''
        set -euo pipefail
        # Plugin deps failing (e.g. offline) shouldn't block the binary build.
        ${pluginDepsScript}/bin/opencode-plugin-deps || \
          echo "warning: plugin deps install failed; run opencode-plugin-deps manually"
        pushd ${opencodeSource}/packages/opencode > /dev/null
        bun install
        bun run build --single
        popd > /dev/null
        chmod -R +x ${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin
      '';

      # ── Cargo‑backed tool wrappers ──────────────────────────────────────
      # Delegate to `cargo run` at runtime instead of baking Nix‑built
      # binaries.  This avoids a full workspace‑wide Rust rebuild inside
      # `home‑manager switch` whenever a single .rs file changes.
      # Cargo handles incremental compilation; second run is near‑instant.
      mkCargoTool = {
        name,
        package ? name,
        dir ? "$HOME/opencode/tools",
      }:
        pkgs.writeShellScriptBin name ''
          set -euo pipefail
          cd "${dir}"
          exec cargo run --release --package ${package} -- "$@"
        '';
    in {
      home.packages = [
        opencodeScript
        opencodeBuildScript
        pluginDepsScript

        # CLI tools — cargo‑backed so editing tools/ costs zero Nix rebuild.
        (mkCargoTool {name = "opencode-model-switcher";})
        (mkCargoTool {name = "opencode-sessions";})
        (mkCargoTool {name = "chunk-files-by-tokens";})
        (mkCargoTool {name = "token-count-after-expand";})
        (mkCargoTool {
          name = "rust-llm-tidy";
          package = "rust-llm-tidy-cli";
          dir = "$HOME/opencode/tools/rust-llm-tidy/src";
        })

        llm-agents.packages.${system}.coderabbit-cli

        # Runtime deps for MCP servers / local hacking.
        pkgs.nodejs
        pkgs.yarn
        pkgs.docker
        pkgs.bun
      ];

      # Editable config → ~/.config/opencode.
      home.file.".config/opencode".source =
        config.lib.file.mkOutOfStoreSymlink "${opencodeRepo}/config";

      # Plugin runtime deps on every switch (bun install is a fast no-op when
      # node_modules is already in sync). Guarded so a missing repo checkout
      # doesn't break activation on a partially bootstrapped machine.
      home.activation.opencodePluginDeps = config.lib.dag.entryAfter ["writeBoundary"] ''
        if [ -d "${opencodeRepo}/config/plugins" ]; then
          run ${pluginDepsScript}/bin/opencode-plugin-deps || \
            echo "warning: opencode plugin deps install failed; run opencode-plugin-deps manually"
        fi
      '';

      # Repo shortcut → ~/opencode.
      home.file."opencode".source =
        config.lib.file.mkOutOfStoreSymlink opencodeRepo;
    };
  in {
    # ── Flake outputs ─────────────────────────────────────────────────────
    # nix build / nix run / nix develop all work from this repo directly.

    # nix build .#opencode-model-switcher   etc.
    packages = eachSystem (_system: pkgs: mkTools pkgs);

    # nix flake check
    checks = eachSystem (system: _pkgs: {
      opencode-model-switcher = self.packages.${system}.opencode-model-switcher;
      opencode-sessions = self.packages.${system}.opencode-sessions;
      chunk-files-by-tokens = self.packages.${system}.chunk-files-by-tokens;
      token-count-after-expand = self.packages.${system}.token-count-after-expand;
    });

    # nix run .#opencode-sessions -- tui
    apps = eachSystem (system: _pkgs: rec {
      opencode-model-switcher = {
        type = "app";
        program = "${self.packages.${system}.opencode-model-switcher}/bin/opencode-model-switcher";
        meta.description = "Open opencode model tier TUI/CLI";
      };

      opencode-sessions = {
        type = "app";
        program = "${self.packages.${system}.opencode-sessions}/bin/opencode-sessions";
        meta.description = "Browse and export OpenCode sessions";
      };

      chunk-files-by-tokens = {
        type = "app";
        program = "${self.packages.${system}.chunk-files-by-tokens}/bin/chunk-files-by-tokens";
        meta.description = "Chunk files by estimated token count";
      };

      token-count-after-expand = {
        type = "app";
        program = "${self.packages.${system}.token-count-after-expand}/bin/token-count-after-expand";
        meta.description = "Estimate prompt token counts after md-expand rendering";
      };

      default = opencode-model-switcher;
    });

    # nix develop  →  Rust toolchain + built CLI tools on PATH.
    devShells = eachSystem (system: pkgs: let
      tools = self.packages.${system};
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src"]; # needed for rust-analyzer type info
      };
    in {
      default = pkgs.mkShell {
        packages = [
          # Rust (rust‑overlay gives rustc/cargo/rustfmt/clippy;
          # standalone rust-analyzer is fresher than the bundled preview).
          rustToolchain
          pkgs.rust-analyzer
          pkgs.pkg-config
          pkgs.stdenv.cc
          (pkgs.python3.withPackages (pythonPackages: [
            pythonPackages.json5
            pythonPackages.pyyaml
          ]))

          # Built CLI tools - ready to run inside the shell.
          tools.opencode-model-switcher
          tools.opencode-sessions
          tools.chunk-files-by-tokens
          tools.token-count-after-expand
        ];
      };
    });

    # Consumed by the root NixOS flake as:
    #   inputs.opencode-config.homeManagerModules.default
    homeManagerModules.default = homeModule;
  };
}
