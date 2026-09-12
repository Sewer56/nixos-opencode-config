#!/bin/sh
# Run mutating tidy on supplied files when the repository opts in.
repo_root=$(git rev-parse --show-toplevel 2>/dev/null)
if [ -n "$repo_root" ]; then
  # Probe for opt-in markers from the repository root without changing cwd,
  # so relative path arguments still resolve for rust-llm-tidy.
  if git -C "$repo_root" grep -q 'Sewer56/rust-llm-tidy-action' 2>/dev/null \
    || [ -f "$repo_root/.rust-llm-tidy.yml" ] \
    || [ -z "$(git -C "$repo_root" remote 2>/dev/null)" ] \
    || [ -n "$(find "$repo_root" -maxdepth 3 -type d -name rust-llm-tidy -print -quit 2>/dev/null)" ]; then
    rust-llm-tidy "$@"
    exit $?
  fi
  echo 'rust-llm-tidy gate: repo not opted in; non-blocking skip'
  exit 0
fi
if git grep -q 'Sewer56/rust-llm-tidy-action' 2>/dev/null \
  || [ -f .rust-llm-tidy.yml ] \
  || [ -z "$(git remote 2>/dev/null)" \
  || [ -n "$(find . -maxdepth 3 -type d -name rust-llm-tidy -print -quit 2>/dev/null)" ]; then
  rust-llm-tidy "$@"
  exit $?
fi
echo 'rust-llm-tidy gate: repo not opted in; non-blocking skip'
