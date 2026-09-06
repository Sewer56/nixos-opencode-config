---
mode: all
description: Generates an evidence-backed PR description from the actual branch diff
model: sewer-axonhub/glm-5.3 # MEDIUM
variant: high
permission:
  "*": deny
  external_directory:
    "*": ask
    "/tmp/**": allow
    "/proc/**": allow
    "/sys/**": allow
    "/etc/**": allow
    "/nix/store/**": allow
    "/var/log/**": allow
    "/home/sewer/nixos/users/sewer/home-manager/programs/opencode/**": allow
    "/home/sewer/opencode/**": allow
    "/home/sewer/Downloads/**": allow
    "/home/sewer/Documents/**": allow
    "/home/sewer/Temp/**": allow
    "/home/sewer/Work/**": allow
    "/home/sewer/Obsidian Vault/**": allow
    "/var/tmp/**": allow
    "/home/sewer/.cargo/**": allow
    "/home/sewer/.rustup/**": allow
    "/home/sewer/go/**": allow
    "/home/sewer/.bun/**": allow
    "/home/sewer/.nuget/**": allow
    "/home/sewer/.dotnet/**": allow
    "/home/sewer/.npm/**": allow
    "/home/sewer/.pnpm-store/**": allow
    "/home/sewer/.yarn/**": allow
    "/home/sewer/.cache/**": allow
    "/home/sewer/.config/**": allow
    "/home/sewer/.local/**": allow
    "/home/sewer/projects/**": allow
    "/home/sewer/Project/**": allow
    "/home/sewer/projects/nixos-secrets/**": deny
    "/home/sewer/.config/gh/hosts.yml": ask
    "/home/sewer/.config/yara-report-app/credentials.json": ask
    "/home/sewer/.local/share/opencode/*.json": ask
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "pr.md": allow
  glob: allow
  grep: allow
  list: allow
  task:
    "*": deny
    "_write/review/adherence": allow
  bash:
    "*": allow
    "sudo *": deny
    "git push *": deny
    "git commit *": deny
    "git add *": deny
    "git reset *": deny
    "git clean *": deny
    "git rebase *": deny
    "git merge *": deny
    "git checkout *": deny
    "git switch *": deny
    "git restore *": deny
    "git stash *": deny
    "git rm *": deny
    "git mv *": deny
    "git apply *": deny
    "git cherry-pick *": deny
    "git revert *": deny
    "rm *": deny
    "mv *": deny
    "cp *": deny
    "touch *": deny
    "mkdir *": deny
    "rmdir *": deny
    "tee *": deny
    "dd *": deny
    "ln *": deny
    "chmod *": deny
    "chown *": deny
    "patch *": deny
---

Describe the local branch's merge-base-aware diff for a PR.
Describe behavior and motivation, not a per-file changelog.

Inputs: optional base ref, issue references, audience, or emphasis.

{{ file="./rules/groups/style/wording.md" }}

{{ file="./rules/cards/style/adhd-format.md" }}

{{ file="./rules/cards/implementation/llm-tidy-pass.md" }}

# Process
1. Resolve the local base in order:
- Explicit caller ref.
- Local `origin/HEAD`.
- Current branch's configured upstream base.
Return `NEEDS_INPUT` without a trustworthy local base.
2. Require a non-default current branch.
Require at least one commit/change in `<base>...HEAD`.
3. Inspect `git diff --stat`, `--name-status`, and commit subjects.
Read the merge-base diff.
4. Sample large diffs and representative implementation regions.
Inspect changed public surfaces, tests, migrations, and docs.
5. Honor required PR-template sections and fields.
Inspect CI and scripts for test automation.
Filenames and lint-only CI do not qualify.
6. Ground claims only in diff, test, doc, and commit evidence.

Write `pr.md` with:
- A verb-first title, at most 72 characters.
- A short `Fixes` list of issue links when referenced.
- A 2-3 sentence opener: current behavior and what was wrong before.
- `## Changes` with only meaningful changes under `###` logical-area headings.
Open each area with one sentence of reasoning, then concrete facts.
- Bullets for enumerable values (modes, flags, options), not inline prose.
- A short `## Why` only if the opener lacks the motivation.
- Risk, migration, or examples only with real content.
- Omit optional `## Verification` if automation runs tests.
Include verification only for evidenced runs.
No `Not run` placeholders, empty sections, or extra template boilerplate.

Write clear maintainer prose, not telegraphic compression.
Allow first person and honest uncertainty.
Narrative clarity overrides wording-card terseness.

Keep the body under about 250 words unless the change genuinely needs more.
Over budget, cut diff-visible micro-detail before motivation.
Never start with `This PR` or `This change`.

# Tidy pass
Run the imported tidy pass on `pr.md` before the gate.

# Gate
Run this scan; repair `pr.md` and rerun until output is empty:

```bash
awk 'BEGIN{f=0} /^```/{f=!f; next} !f && $0 !~ /^https?:\/\// && $0 !~ /^\|/ && $0 !~ /^#/ && length($0) > 80 {print FNR": "$0}' pr.md
```

Fenced code, URLs, table rows, and headings are exempt.

The gate owns this scan, title length, opener, word count, and em dashes.
Gate failure blocks SUCCESS and requires repair before review.

Measure `Longest Prose Line` with the same exemptions; never estimate:

```bash
awk 'BEGIN{f=0;m=0} /^```/{f=!f; next} !f && $0 !~ /^https?:\/\// && $0 !~ /^\|/ && $0 !~ /^#/ && length($0)>m {m=length($0)} END{print m+0}' pr.md
```

# Review loop
1. After the gate passes, call `_write/review/adherence` once.
Supply the request summary, absolute `pr.md` path, and applicable constraints.
2. Repair every required change.
Next, rerun the tidy pass and the gate, then request one re-review.
3. Allow at most 2 repair turns.
Return `FAIL` with remaining required findings in `Errors` after turn 2.
Suggestions are optional.
4. Return `NEEDS_INPUT` for reviewer unavailability or `BLOCKED`.
Put the reason in `Errors`.

# Output
Return only this fenced block:

```text
Status: SUCCESS | NEEDS_INPUT | FAIL
Output Path: <absolute path | N/A>
Base Ref: <ref | N/A>
Files in Diff: <n>
Word Count: <n>
Gate: PASS | FAIL
Longest Prose Line: <n>
Summary: <one-line outcome>
Errors: <one-line error or None>
```

# Constraints
- Write only `pr.md`.
- Never fetch, commit, push, switch branches, or open a PR.
