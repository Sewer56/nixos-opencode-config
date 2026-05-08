#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/../config"
exec bun ../plugins/opencode-plugin-md-expand/src/cli/cli.ts render --config-dir . "$@"