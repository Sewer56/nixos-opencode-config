---
description: "Rebase opencode-source production branch onto a new upstream version"
agent: build
---

# Migrate OpenCode to $ARGUMENTS

You are rebasing the `production` branch in `opencode-source/` onto a new upstream version tag: **$ARGUMENTS**

## Step 1: Backup
- Run: `git branch production-backup-$(date +%Y-%m-%d) production` in `opencode-source/`
- Delete old backup branches (`production-backup-pre-rebase` style) if any exist

## Step 2: Find merge base
- Run: `git log --oneline production` and find the last upstream release commit in history (message starts with `release: v`)
- That release tag is the old base. Example: if last release commit is `release: v1.14.25`, the old base is `v1.14.25`
- Verify with: `git merge-base <old-base-tag> production`

## Step 3: Rebase
- Run: `git checkout production` then `git rebase --onto $ARGUMENTS <old-base-tag> production`
- Resolve conflicts using these known patterns:
  - Release commits (message starts with `release: v`) → SKIP with `GIT_EDITOR=true git rebase --skip` (upstream has its own release)
  - `import { Flag } from "@opencode-ai/core/flag/flag"` → resolve to `import { Flag } from "@/flag/flag"`
  - `import { SystemPrompt } from "./system"` → drop the import entirely (replaced by system-prompt-builder)
- After rebase, verify: `git log --oneline $ARGUMENTS..production` shows only custom commits (no release commits)

## Step 4: Install dependencies
- Run: `bun install` in `opencode-source/`

## Step 5: Measure system prompt
- Run: `cd opencode-source/packages/opencode && bun run script/preview-system-prompt.ts`
- Capture everything from the `SUMMARY` line onward

## Step 6: Check tool description lengths
- Read each file in `opencode-source/packages/opencode/src/tool/*.txt`
- Record char count for each

## Step 7: Record metrics
- Update `config/prompt-metrics.log` — replace the `[last-migration]` section with new data

## Step 8: Regression checks — STOP if any fail
- TOTAL tokens must be under 4,600. If over, report immediately and do NOT proceed.
- `bash.txt` must be under 100 chars (single sentence schema description, like all other tool .txt files)
- Any tool `.txt` file over 150 chars is a regression. Long guidance belongs in `system-prompt-builder.ts`, not `.txt` files.
- Compare against reference in `config/tool-lengths-reference.md`

## Architecture reference (for conflict resolution)
- Tool `.txt` files → JSON Schema `description` field (one short sentence)
- `system-prompt-builder.ts` → usage guidance per tool (short bullet lists)
- `tool-prompt-facts.ts` → cross-tool priority rules (common rules section)
- `supplemental/*.txt` → domain workflows (git-workflow.txt, github-cli.txt)
- Git commit protocol → `supplemental/git-workflow.txt`, NOT bash.txt
- PR creation workflow → `supplemental/github-cli.txt`, NOT bash.txt
