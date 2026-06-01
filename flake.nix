{
  description = "OpenCode config utilities";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {self, nixpkgs, ...}: let
    systems = ["x86_64-linux"];
    eachSystem = fn:
      nixpkgs.lib.genAttrs systems (system: fn system nixpkgs.legacyPackages.${system});
  in {
    # Buildable tools for this config repo. Keep utility source under tools/*;
    # scripts/* are thin wrappers plus runtime data files.
    packages = eachSystem (system: pkgs: rec {
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

      default = opencode-model-tiers;
    });

    # `nix flake check` builds the Go utility and runs its tests through
    # buildGoModule's checkPhase.
    checks = eachSystem (system: _pkgs: {
      opencode-model-tiers = self.packages.${system}.opencode-model-tiers;
    });

    # Root-level invocation:
    #   nix run .#opencode-model-tiers -- status
    #   nix run .#opencode-work-mode -- --dry-run
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

      default = opencode-model-tiers;
    });

    # direnv/nix develop shell for Go tool development. CGO is disabled because
    # the utility is pure Go and this avoids host C toolchain linker surprises.
    devShells = eachSystem (_system: pkgs: {
      default = pkgs.mkShell {
        packages = with pkgs; [
          go
          gopls
          gotools
        ];

        CGO_ENABLED = "0";
      };
    });
  };
}
