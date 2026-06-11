#!/usr/bin/env sh
# Usage:
#   tools/scripts/render-file.sh <config/path.md | .opencode/path.md> [render-options...]
#
# Renders one Markdown prompt through the md-expand CLI so file/env/template
# expansions can be inspected without starting opencode.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
path="${1:?Usage: render-file.sh <config/path.md | .opencode/path.md> [render-options...]}"
shift
cd "$root"
exec bun plugins/opencode-plugin-md-expand/src/cli/cli.ts render "${path}" "$@"
