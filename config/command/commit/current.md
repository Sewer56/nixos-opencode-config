---
description: "Commit the work done in the current context window, in-context under the current agent"
---

Commit the work this conversation produced, entirely within this session and under the current agent/model. Do not delegate, switch agents, or re-derive from scratch.

Optional scope, issue reference, grouping preference, or message guidance:

```
$ARGUMENTS
```

If no instructions are given above, commit the changes made during this conversation/session.

Resolve the exact set by mapping the conversation's known work onto the working tree, and confirm that mapping with `git status` / `git diff` reads.

If nothing the conversation produced maps onto a change, fall back to the current unstaged changes.

# Rules

{{ file="./rules/cards/implementation/self-contained-content.md" }}
{{ file="./rules/cards/implementation/commit-message.md" }}

# Process
1. Inspect `git status`, `git diff`, `git diff --check`, and recent commits in parallel.
2. Map this session's work to specific paths; exclude workflow evidence and generated local artifacts: `artifact/`, `artifacts/`, `PROMPT-*.md`, review ledgers, build outputs, secrets, and anything outside the resolved scope.
3. Stage explicit paths or hunks per logical change. Never use blanket `git add -A` or `git add .`.
4. Re-read the staged diff, then run the message tidy pass above to draft, refine, and commit.
5. Run `git status` to confirm and report what remains.

# Safety
- Never push, `git reset --hard`, `git clean`, or `git commit --no-verify` unless the user explicitly requested that exact operation.
- Do not create empty commits, commit suspected secrets, or commit inside a dirty submodule unless the user explicitly requested that exact operation.
- If hooks fail or modify files, fix the result and create a NEW commit; do not amend a failed one.
- Stop with `NEEDS_INPUT` for suspected secrets, unresolved conflicts, a dirty submodule that must be committed first, or ambiguous unrelated changes.

# Output
Report each resulting commit as `<hash> <first line>`, the number of files committed, and the remaining-changes count. If nothing is eligible, say so instead of committing.
