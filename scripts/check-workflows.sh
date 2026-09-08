#!/usr/bin/env bash
# Read-only routing smoke; fixtures and cleanup stay in our private temp dir.
set -euo pipefail
script_dir="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
checker="$script_dir/../config/scripts/plan-bundle.py"
tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT
export PYTHONDONTWRITEBYTECODE=1
repo="$tmp/repo"
members="$repo/artifact/plan/PROMPT-PLAN-smoke"
mkdir -p "$members" "$repo/src" "$tmp/outside"
printf '# Input parser\n## Contract\nReject invalid input without partial output.\n' > "$repo/src/parser.md"
root="$repo/PROMPT-PLAN-smoke.draft.md"
cat > "$root" <<'EOF'
# Safe input parsing
Status: DRAFT
## Goal
Reject invalid input without partial output; preserve valid input behavior.
[Execution](artifact/plan/PROMPT-PLAN-smoke/execution.md)
[01: Validate input](artifact/plan/PROMPT-PLAN-smoke/01-first.md#done)
[02: Report failure](artifact/plan/PROMPT-PLAN-smoke/02-second.md)
EOF
cat > "$members/execution.md" <<'EOF'
# Execution
Run the parser checks after each task; stop on changed valid-input behavior.
```plan-tasks
01 01-first.md -
02 02-second.md 01
```
EOF
cat > "$members/01-first.md" <<'EOF'
# Validate input
Reject invalid input before producing output; leave reporting to task 02.
## Done
Invalid input produces no partial output; valid input remains unchanged.
[Execution](01-first.exec.md)
EOF
printf '# Report failure\nSurface the parser failure to callers.\n[Execution](02-second.exec.md)\n' > "$members/02-second.md"
printf '# Validation execution\nTarget src/parser.md; check invalid and valid input.\n' > "$members/01-first.exec.md"
printf '# Reporting execution\nStop if reporting changes parsing semantics.\n' > "$members/02-second.exec.md"
check() { python3 "$checker" --repo-root "$repo" "$root" "$@"; }
reject() {
  if check "$@" > "$tmp/rejected.json"; then
    printf 'FAIL: unsafe fixture accepted\n' >&2
    exit 1
  fi
}
check > "$tmp/result.json"
python3 - "$tmp/result.json" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))
assert data['order'] == ['01', '02']
assert len(data['sources']) == 6
assert data['tasks']['02']['after'] == ['01']
PY
# References navigate out of the member directory without adding authorities.
printf '\n[Plan](../../../PROMPT-PLAN-smoke.draft.md#goal)\n' >> "$members/01-first.md"
printf '\n[Parser contract](../../../src/parser.md#contract)\n' >> "$members/01-first.exec.md"
check > "$tmp/references.json"
cmp "$tmp/result.json" "$tmp/references.json"
cp "$members/01-first.exec.md" "$tmp/exec.md"
printf '\n[Escape](../../../../outside/secret.md)\n' >> "$members/01-first.exec.md"
reject
cp "$tmp/exec.md" "$members/01-first.exec.md"
printf '\n[Encoded escape](%%2e%%2e/%%2e%%2e/%%2e%%2e/%%2e%%2e/outside/secret.md)\n' >> "$members/01-first.exec.md"
reject
cp "$tmp/exec.md" "$members/01-first.exec.md"
printf 'private fixture\n' > "$tmp/outside/secret.md"
ln -s "$tmp/outside/secret.md" "$repo/src/escape.md"
printf '\n[Symlink](../../../src/escape.md)\n' >> "$members/01-first.exec.md"
reject
cp "$tmp/exec.md" "$members/01-first.exec.md"
reject --prospective 'src/parser.md'
cp "$members/01-first.md" "$tmp/strict-brief.md"
sed 's|(01-first.exec.md)|(../PROMPT-PLAN-smoke/01-first.exec.md)|' "$tmp/strict-brief.md" > "$members/01-first.md"
reject
cp "$tmp/strict-brief.md" "$members/01-first.md"
check --prospective "$root" "$members/03-third.md" "$members/03-third.exec.md" > /dev/null
mv "$members/01-first.exec.md" "$tmp/saved.exec.md"
reject
ln -s "$tmp/saved.exec.md" "$members/01-first.exec.md"
reject
reject --prospective "$members/01-first.exec.md"
rm "$members/01-first.exec.md"
mv "$tmp/saved.exec.md" "$members/01-first.exec.md"
cp "$members/execution.md" "$tmp/execution.md"
sed 's/01 01-first.md -/01 01-first.md 02/' "$tmp/execution.md" > "$members/execution.md"
reject
sed 's/02 02-second.md 01/02 02-second.md 99/' "$tmp/execution.md" > "$members/execution.md"
reject
sed 's/02 02-second.md 01/01 01-first.md -/' "$tmp/execution.md" > "$members/execution.md"
reject
sed 's/01 01-first.md -/01 01-first.exec.md -/' "$tmp/execution.md" > "$members/execution.md"
reject
cp "$tmp/execution.md" "$members/execution.md"
cp "$root" "$tmp/root.md"
sed 's/#done/#missing/' "$tmp/root.md" > "$root"
reject
cp "$tmp/root.md" "$root"
printf '\n[Duplicate](artifact/plan/PROMPT-PLAN-smoke/01-first.md)\n' >> "$root"
reject
cp "$tmp/root.md" "$root"
cp "$members/01-first.md" "$tmp/brief.md"
printf '\n[Wrong execution](02-second.exec.md)\n' >> "$members/01-first.md"
reject
cp "$tmp/brief.md" "$members/01-first.md"
ln -s 01-first.md "$members/03-alias.md"
reject
rm "$members/03-alias.md"
printf '\n[Encoded escape](%%2e%%2e/escape.md)\n' >> "$root"
reject
cp "$tmp/root.md" "$root"
printf '\n[Escape](../../../../outside.md)\n' >> "$root"
reject
cp "$tmp/root.md" "$root"
printf '\n[Escape](/etc/passwd)\n' >> "$root"
reject
cp "$tmp/root.md" "$root"
reject --prospective 'artifact/plan/PROMPT-PLAN-smoke/../escape.md'
reject --prospective "$tmp/outside/01-escape.md"
printf '# Extra authority\n' > "$members/contract.md"
reject
rm "$members/contract.md"
mkdir "$members/review"
printf '# Evidence, not routing\n' > "$members/review/run.md"
check > /dev/null
mv "$members" "$tmp/saved-members"
ln -s "$tmp/saved-members" "$members"
reject
reject --prospective "$members/03-third.md"
rm "$members"
mv "$tmp/saved-members" "$members"
# Legacy combined roots have no new routing authority and fail closed.
printf '# Old combined plan\n## Cohorts\n' > "$root"
reject
cp "$tmp/root.md" "$root"
before="$(find "$repo" -type f -exec sha256sum {} + | sort)"
check > /dev/null
after="$(find "$repo" -type f -exec sha256sum {} + | sort)"
test "$before" = "$after"
test -z "$(find "$repo" -name __pycache__ -print)"
printf 'PASS: routing, pairs, IDs, cycles, anchors, escapes and read-only checks\n'
