{
  pkgs,
  config,
  inputs,
  ...
}: let
  opencodeSource = "${config.home.homeDirectory}/nixos/users/sewer/home-manager/programs/opencode/opencode-source";
  opencodeBin = "${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin/opencode";
  opencodeScript = pkgs.writeShellScriptBin "opencode" ''
    export OPENCODE_ENABLE_EXA=1
    if [ "$#" -eq 0 ]; then
      exec ${opencodeBin} .
    else
      exec ${opencodeBin} "$@"
    fi
  '';
  opencodeBuildScript = pkgs.writeShellScriptBin "opencode-build" ''
    set -euo pipefail
    pushd ${opencodeSource}/packages/opencode > /dev/null
    bun install
    bun run build --single
    popd > /dev/null
    chmod -R +x ${opencodeSource}/packages/opencode/dist/opencode-linux-x64/bin
  '';
in {
  # Install OpenCode from the dedicated flake
  home.packages = with pkgs; [
    # OpenCode from the flake
    # inputs.opencode-flake.packages.${pkgs.system}.default

    opencodeScript
    opencodeBuildScript
    inputs.llm-agents.packages.${pkgs.stdenv.hostPlatform.system}.coderabbit-cli

    # Dependencies for MCP servers
    nodejs
    yarn
    docker
    # TypeScript for plugin development
    typescript
    # Bun for local development
    bun
    # Go for local development
    go
  ];

  # Symlink our configuration folder to OpenCode's expected location
  home.file.".config/opencode".source = config.lib.file.mkOutOfStoreSymlink "/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config";

  # Symlink for convenient access to opencode directory
  home.file."opencode".source = config.lib.file.mkOutOfStoreSymlink "${config.home.homeDirectory}/nixos/users/sewer/home-manager/programs/opencode";
}
