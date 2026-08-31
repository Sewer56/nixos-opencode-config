#!/bin/sh
# rust-llm-tidy gate: run when the repository opts in; exit 0 (skip) otherwise.
if git grep -q 'Sewer56/rust-llm-tidy-action' 2>/dev/null \
  || [ -f .rust-llm-tidy.yml ] \
  || [ -z "$(git remote 2>/dev/null)" ] \
  || [ -n "$(find . -maxdepth 3 -type d -name rust-llm-tidy -print -quit 2>/dev/null)" ]; then
  echo 'rust-llm-tidy gate: tool executed (auto mode: tracked staged/unstaged .rs/.md; untracked excluded until staged)'
  rust-llm-tidy
  rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "rust-llm-tidy gate: exit $rc; blocks handoff; repair and rerun"
    exit "$rc"
  fi
  exit 0
fi
echo 'rust-llm-tidy gate: tool not ran (OK); repo not opted in; non-blocking'
exit 0
