#!/usr/bin/env sh
# Usage:
#   tools/validate-file-interp.sh [paths...]
#
# Validates md-expand file/env/template references in .opencode and config by
# default. Pass paths to narrow the scan.
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
# Benchmark fixtures intentionally include unresolved-reference cases.
bun config/plugins/opencode-plugin-md-expand/src/cli/cli.ts validate \
  --exclude renderer-syntax.txt \
  --exclude renderer-template-use-checks.txt \
  --exclude design-patterns.md \
  --exclude template-library.md \
  --exclude config/plugins/opencode-plugin-md-expand/README.md \
  --exclude config/plugins/opencode-plugin-md-expand/bench/fixtures \
  ${*:-.opencode config}
