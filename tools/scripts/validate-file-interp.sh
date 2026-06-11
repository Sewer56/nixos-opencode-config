#!/usr/bin/env sh
# Usage:
#   tools/scripts/validate-file-interp.sh [paths...]
#
# Validates md-expand file/env/template references in .opencode and config by
# default. Pass paths to narrow the scan.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
bun plugins/opencode-plugin-md-expand/src/cli/cli.ts validate \
  --exclude renderer-syntax.txt \
  --exclude renderer-template-use-checks.txt \
  --exclude design-patterns.md \
  --exclude template-library.md \
  ${*:-.opencode config}
