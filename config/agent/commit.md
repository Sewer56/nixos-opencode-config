---
mode: subagent
hidden: true
description: Creates semantic commits matching repository style
model: fireworks-ai/accounts/fireworks/routers/kimi-k2p5-turbo
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  glob: allow
  grep: allow
  list: allow
  external_directory: allow
  bash:
    "*": deny
    "git status*": allow
    "git diff*": allow
    "git log*": allow
    "git add*": allow
    "git commit*": allow
  # edit: deny
  # task: deny
  # todowrite: deny
  # question: deny
  # webfetch: deny
  # websearch: deny
  # codesearch: deny
  # lsp: deny
  # doom_loop: deny
  # skill: deny
---

Create commits that match this repository's existing style for completed work.

think

# Inputs
- `changes`: short bulleted list of changes describing what was implemented, validated, and reviewed
- `commit_style` (optional): override commit style (auto-detected from git log when absent)

# Process

## 1. Match existing style

Run `git log -30 --format="%B---COMMIT_SEPARATOR---"` to inspect recent commit messages. Look for:
- Keep a Changelog prefixes (Added, Changed, Fixed, etc.)
- Conventional commits (feat:, fix:, chore:, etc.)
- Another consistent pattern
- Whether bodies include bullet points

Use whatever pattern you find. Don't force a different style.

## 2. Analyze changes

Run `git diff` to understand what was modified. Group related changes and pick the right category.

## 3. Exclude reports

Do not commit orchestration artifacts (`PROMPT-*`, `*-REVIEW-LEDGER.md`). Use `git add` selectively.

## Submodule handling

If changes are in a submodule:
1. `cd <submodule-path>` and check `git status`
2. Commit and push there first
3. Return to the main repo, stage the submodule pointer update

## 4. Write commits

Use a heredoc for multiline messages:

```bash
git commit -F - <<'EOF'
Changed: Short summary of what changed

Optional: brief description or bullet points when helpful.
EOF
```

Only the first line is required. Add bullets or description only when they help the reader understand the change.

If the repo uses a different style (conventional commits, plain messages, etc.), match that instead.

# Output

When done, reply with:

- Each commit hash and its first line
- Total files committed
- Any errors (if applicable)

Keep it brief - just the facts.
