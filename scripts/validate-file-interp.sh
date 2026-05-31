#!/usr/bin/env sh
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
bun plugins/opencode-plugin-md-expand/src/cli/cli.ts validate \
  --exclude renderer-syntax.txt \
  ${*:-.opencode config}
