#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
rules_dir="$repo_root/config/rules"
output_path="$rules_dir/all.md"

sources=(
  "plan-content.md"
  "general.md"
  "performance.md"
  "testing.md"
  "test-parameterization.md"
  "code-placement.md"
  "documentation.md"
  "orchestration-plan.md"
  "orchestration-revision.md"
)

for source in "${sources[@]}"; do
  if [[ ! -f "$rules_dir/$source" ]]; then
    printf 'Missing rules source: %s\n' "$rules_dir/$source" >&2
    exit 1
  fi
done

{
  for source in "${sources[@]}"; do
    if [[ "$source" != "${sources[0]}" ]]; then
      printf '\n'
    fi
    cat "$rules_dir/$source"
  done
} > "$output_path"

printf 'Wrote %s\n' "$output_path"
