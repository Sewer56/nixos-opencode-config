#!/usr/bin/env sh
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
path="${1:?Usage: render-file.sh <config/path.md | .opencode/path.md> [render-options...]}"
shift
cd "$root"
exec bun plugins/opencode-plugin-md-expand/src/cli/cli.ts render "${path}" "$@"
