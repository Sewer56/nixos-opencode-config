#!/bin/sh
# Run mutating tidy on supplied files when the repository opts in.
if git grep -q 'Sewer56/rust-llm-tidy-action' 2>/dev/null \
  || [ -f .rust-llm-tidy.yml ] \
  || [ -z "$(git remote 2>/dev/null)" ] \
  || [ -n "$(find . -maxdepth 3 -type d -name rust-llm-tidy -print -quit 2>/dev/null)" ]; then
  rust-llm-tidy "$@"
  exit $?
fi
echo 'rust-llm-tidy gate: repo not opted in; non-blocking skip'
