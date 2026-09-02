{
  description = "OpenCode config utilities";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    llm-agents.url = "github:numtide/llm-agents.nix";

    # Flakes cannot see submodule files, so consume the rust-llm-tidy
    # working repo directly (nixos-secrets pattern). .githooks/pre-commit
    # keeps the locked rev in sync with the submodule pointer.
    rust-llm-tidy = {
      url = "git+file:///home/sewer/nixos/users/sewer/home-manager/programs/opencode/tools/rust-llm-tidy";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    llm-agents,
    rust-llm-tidy,
    ...
  }: let
    systems = ["x86_64-linux"];

    # nixpkgs + rust overlay: one toolchain for builds and shells.
    mkPkgs = system:
      import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

    eachSystem = fn:
      nixpkgs.lib.genAttrs systems (system: fn system (mkPkgs system));

    # ── Tool packages (packages / apps / devShells) ───────────────────────
    # tools/ compiles once; per-tool packages pluck their binary.
    # rust-llm-tidy is a separate workspace built from the pinned input.
    mkTools = pkgs: llmTidySrc: let
      workspaceDrv = pkgs.rustPlatform.buildRustPackage {
        pname = "opencode-tools";
        version = "0.1.0";
        src = ./tools;
        cargoLock.lockFile = ./tools/Cargo.lock;
        # --workspace: one compilation unit; install copies every bin.
        cargoBuildFlags = ["--workspace"];
      };

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

      rustLlmTidy = pkgs.rustPlatform.buildRustPackage {
        pname = "rust-llm-tidy";
        # Version comes from the locked source, so it cannot go stale here.
        version = (pkgs.lib.importTOML "${llmTidySrc}/src/cli/Cargo.toml").package.version;
        src = "${llmTidySrc}/src";
        cargoLock.lockFile = "${llmTidySrc}/src/Cargo.lock";
        # Only the CLI member ships a binary; the rest are libraries.
        cargoBuildFlags = ["--package" "rust-llm-tidy-cli"];
        # Upstream's corpus test needs the nested submodule checkout;
        # its CI covers that, this package build does not.
        doCheck = false;
        meta = {
          description = "Reorder and lint Rust source and doc comments";
          mainProgram = "rust-llm-tidy";
        };
      };
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

      opencode-yolo-mode = mkTool {
        pname = "opencode-yolo-mode";
        description = "Toggle external_directory '*' between ask (regular) and allow (yolo) across agent frontmatter and global config";
      };

      rust-llm-tidy = rustLlmTidy;

      default = opencode-model-switcher;
    };

    # ── Home-Manager module ────────────────────────────────────────────────
    # Consumed by the root NixOS flake.
    homeModule = {
      pkgs,
      config,
      ...
    }: let
      system = pkgs.stdenv.hostPlatform.system;

      opencodeRepo = "${config.home.homeDirectory}/nixos/users/sewer/home-manager/programs/opencode";
      opencodeSource = "${opencodeRepo}/opencode-source";
      opencodeBin = "${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin/opencode";

      # Defaults to CWD, forwards args, enables Exa search.
      opencodeScript = pkgs.writeShellScriptBin "opencode" ''
        export OPENCODE_ENABLE_EXA=1
        if [ "$#" -eq 0 ]; then
          exec ${opencodeBin} .
        else
          exec ${opencodeBin} "$@"
        fi
      '';

      # Local plugins import node_modules at runtime; missing deps fail
      # silently inside OpenCode.
      pluginDepsScript = pkgs.writeShellScriptBin "opencode-plugin-deps" ''
        set -uo pipefail
        failed=0
        for dir in ${opencodeRepo}/config/plugins/*/; do
          if [ ! -f "$dir/package.json" ]; then
            # Empty dir = uninitialized submodule.
            if [ -z "$(ls -A "$dir" 2>/dev/null)" ]; then
              echo "error: $dir is empty - uninitialized submodule? run: git submodule update --init" >&2
              failed=1
            fi
            continue
          fi
          # Only install when deps are declared; jq failures must surface.
          if ! deps=$(${pkgs.jq}/bin/jq '.dependencies | length' "$dir/package.json" 2>&1); then
            echo "error: invalid package.json in $dir: $deps" >&2
            failed=1
            continue
          fi
          [ "$deps" -gt 0 ] 2>/dev/null || continue
          echo "installing plugin deps: $dir"
          # Frozen when a lockfile exists so installs never dirty the tree.
          if [ -f "$dir/bun.lock" ] || [ -f "$dir/bun.lockb" ]; then
            (cd "$dir" && ${pkgs.bun}/bin/bun install --production --frozen-lockfile) || failed=1
          else
            (cd "$dir" && ${pkgs.bun}/bin/bun install --production) || failed=1
          fi
        done
        exit $failed
      '';

      # Rebuild opencode-source (bun build); separate command for iteration.
      opencodeBuildScript = pkgs.writeShellScriptBin "opencode-build" ''
        set -euo pipefail
        # Plugin deps failing (e.g. offline) must not block the binary build.
        ${pluginDepsScript}/bin/opencode-plugin-deps || \
          echo "warning: plugin deps install failed; run opencode-plugin-deps manually"
        pushd ${opencodeSource}/packages/opencode > /dev/null
        bun install
        bun run build --single
        popd > /dev/null
        chmod -R +x ${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin
      '';

      # ── Cargo wrappers for tools/ members ────────────────────────────────
      # Editing tools/*.rs costs zero Nix rebuild. rust-llm-tidy is not
      # wrapped: it would inherit the caller's rustup toolchain and
      # recompile the shared target dir; the pinned Nix build installs
      # instead.
      mkCargoTool = {
        name,
        package ? name,
        dir ? "$HOME/opencode/tools",
      }:
        pkgs.writeShellScriptBin name ''
          set -euo pipefail
          # Physical path: a symlinked dir is a second workspace root and
          # forces full rebuilds when invocation styles switch.
          cd "${dir}"
          cargo build --release --package ${package}
          exec cargo run --release --package ${package} -- "$@"
        '';
    in {
      home.packages = [
        opencodeScript
        opencodeBuildScript
        pluginDepsScript

        (mkCargoTool {name = "opencode-model-switcher";})
        (mkCargoTool {name = "opencode-sessions";})
        (mkCargoTool {name = "chunk-files-by-tokens";})
        (mkCargoTool {name = "token-count-after-expand";})
        (mkCargoTool {name = "opencode-yolo-mode";})

        # Pinned build: one toolchain, no per-caller recompilation.
        self.packages.${system}.rust-llm-tidy

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

      # Plugin deps on every switch (no-op when in sync); guarded so a
      # missing checkout doesn't break activation.
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

    # nix build .#opencode-model-switcher   etc.
    packages = eachSystem (_system: pkgs: mkTools pkgs rust-llm-tidy);

    # nix flake check
    checks = eachSystem (system: _pkgs: {
      opencode-model-switcher = self.packages.${system}.opencode-model-switcher;
      opencode-sessions = self.packages.${system}.opencode-sessions;
      chunk-files-by-tokens = self.packages.${system}.chunk-files-by-tokens;
      token-count-after-expand = self.packages.${system}.token-count-after-expand;
      opencode-yolo-mode = self.packages.${system}.opencode-yolo-mode;
      rust-llm-tidy = self.packages.${system}.rust-llm-tidy;
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

      opencode-yolo-mode = {
        type = "app";
        program = "${self.packages.${system}.opencode-yolo-mode}/bin/opencode-yolo-mode";
        meta.description = "Toggle external_directory yolo mode";
      };

      rust-llm-tidy = {
        type = "app";
        program = "${self.packages.${system}.rust-llm-tidy}/bin/rust-llm-tidy";
        meta.description = "Reorder and lint Rust source";
      };

      default = opencode-model-switcher;
    });

    # nix develop  →  Rust toolchain + built CLI tools on PATH.
    devShells = eachSystem (system: pkgs: let
      tools = self.packages.${system};
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src"]; # rust-analyzer type info
      };
    in {
      default = pkgs.mkShell {
        packages = [
          # rust-overlay toolchain; standalone rust-analyzer is fresher
          # than the bundled preview.
          rustToolchain
          pkgs.rust-analyzer
          pkgs.pkg-config
          pkgs.stdenv.cc
          (pkgs.python3.withPackages (pythonPackages: [
            pythonPackages.json5
            pythonPackages.pyyaml
          ]))

          # Built CLI tools.
          tools.opencode-model-switcher
          tools.opencode-sessions
          tools.chunk-files-by-tokens
          tools.token-count-after-expand
          tools.opencode-yolo-mode
          tools.rust-llm-tidy
        ];
      };
    });

    # Consumed by the root NixOS flake as:
    #   inputs.opencode-config.homeManagerModules.default
    homeManagerModules.default = homeModule;
  };
}
