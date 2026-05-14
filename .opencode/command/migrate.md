---
description: "Rebase opencode-source production branch onto a new upstream version"
agent: build
---

# Migrate OpenCode to $ARGUMENTS

You are rebasing the `production` branch in `opencode-source/` onto a new upstream version tag: **$ARGUMENTS**

# Inputs

- `$ARGUMENTS`: upstream version tag or commit to rebase `production` onto.

## Step 1: Backup
- Run: `git branch production-backup-$(date +%Y-%m-%d) production` in `opencode-source/`
- Delete old backup branches (`production-backup-pre-rebase` style) if any exist

## Step 2: Find merge base
- Run: `git log --oneline production` and find the last upstream release commit in history (message starts with `release: v`)
- That release tag is the old base. Example: if last release commit is `release: v1.14.25`, the old base is `v1.14.25`
- Verify with: `git merge-base <old-base-tag> production`

## Step 3: Rebase (commit-by-commit cherry-pick)
- Do NOT use `git rebase --exec`. Apply commits one at a time and test manually between each.
- Run: `git log --oneline --reverse <old-base-tag>..production` to list custom commits in oldest-first order
- Run: `git checkout -b production-rebase $ARGUMENTS` to create a working branch at the new base
- Run `bun install` once before starting the loop
- For each commit in the list:
  1. Skip release commits (message starts with `release: v`) — upstream has its own release, do not cherry-pick these
  2. `git cherry-pick <sha>`
  3. If conflict → resolve using known patterns below, `git add`, `git cherry-pick --continue`
  4. `bun install && cd packages/opencode && bun run typecheck && bun test`
  5. If test failure → fix the breakage, `git add -A && git commit --amend --no-edit`, re-run tests (amend only to fold fixes into the same commit — do not squash multiple commits or reorganize)
  6. If unfixable → `git cherry-pick --abort`, emit FAIL output below, stop immediately — do NOT continue to Steps 4–8
- After all commits: `git branch -f production production-rebase && git checkout production && git branch -d production-rebase`
- Verify: `git log --oneline $ARGUMENTS..production` shows only custom commits (no release commits)
- Do NOT squash commits or reorganize the commit sequence — preserve each commit as-is
- Resolve conflicts using these known patterns:
  - `import { Flag } from "@opencode-ai/core/flag/flag"` → resolve to `import { Flag } from "@/flag/flag"`
  - `import { SystemPrompt } from "./system"` → drop the import entirely (replaced by system-prompt-builder)

## Step 4: Install dependencies
- Run `bun install` in `opencode-source/`

## Step 5: Measure system prompt
- Run: `cd opencode-source/packages/opencode && bun run script/preview-system-prompt.ts`
- Capture everything from the `SUMMARY` line onward

## Step 6: Check tool description lengths
- Read each file in `opencode-source/packages/opencode/src/tool/*.txt`
- Record char count for each

## Step 7: Record metrics
- Update `config/prompt-metrics.log` — replace the `[last-migration]` section with new data

## Step 8: Regression checks — STOP if any fail
- Compare TOTAL tokens against previous migration in `config/prompt-metrics.log`. If TOTAL increased, report immediately and do NOT proceed.
- `bash.txt` must be under 100 chars (single sentence schema description, like all other tool .txt files)
- Tool `.txt` files: keep schema descriptions short. Some tools (apply_patch, skill, plan-enter) need slightly longer descriptions to explain their envelope/format. That's fine as long as TOTAL tokens don't increase.
- Compare against reference in `config/tool-lengths-reference.md`

## Architecture reference (for conflict resolution)
- Tool `.txt` files → JSON Schema `description` field (one short sentence)
- `system-prompt-builder.ts` → usage guidance per tool (short bullet lists)
- `tool-prompt-facts.ts` → cross-tool priority rules (common rules section)
- `supplemental/*.txt` → domain workflows (git-workflow.txt, github-cli.txt)
- Git commit protocol → `supplemental/git-workflow.txt`, NOT bash.txt
- PR creation workflow → `supplemental/github-cli.txt`, NOT bash.txt

# Output

Return exactly:

```text
Status: SUCCESS | FAIL | BLOCKED
Target Version: $ARGUMENTS
Backup Branch: <branch name | None>
Prompt Metrics: PASS | FAIL | NOT_RUN
Summary: <one-line summary>
```
