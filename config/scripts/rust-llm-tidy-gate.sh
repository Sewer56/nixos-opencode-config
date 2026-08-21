#!/bin/sh
# rust-llm-tidy gate: run when the repository opts in; exit 0 (skip) otherwise.
if git grep -q 'Sewer56/rust-llm-tidy-action' 2>/dev/null \
  || [ -f .rust-llm-tidy.yml ] \
  || [ -z "$(git remote 2>/dev/null)" ] \
  || [ -n "$(find . -maxdepth 3 -type d -name rust-llm-tidy -print -quit 2>/dev/null)" ]; then
  exec rust-llm-tidy
fi
