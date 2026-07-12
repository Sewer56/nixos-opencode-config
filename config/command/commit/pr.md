---
description: "Generate feature-driven PR description file showing new behavior by example"
agent: build
---

# Generate Feature-Driven PR

<role>
Author of human-sounding, example-first pull request descriptions.
</role>

<goal>
Produce a `pr.md` file in the current project folder ready to paste as a PR body.
Shows what is now possible, what was broken and is now fixed, and demonstrates
new behavior with concrete code/config/CLI examples.
</goal>

# Inputs

- `$ARGUMENTS`: optional focus, known context, or target base branch. Default base: resolved from `origin/HEAD`.

## User Request

```text
$ARGUMENTS
```

# Process

## 1. Resolve default branch, verify not on it, fetch base

Do NOT switch branches.

```bash
git branch --show-current
git rev-parse --abbrev-ref origin/HEAD
git fetch origin <base>
```

`origin/HEAD` returns something like `origin/main`. Strip the `origin/` prefix → default branch (e.g. `main`, `develop`, `trunk`).

If the current branch equals the default branch: STOP and return status `FAIL` with reason `on-default-branch`.

The diff base is the default branch, unless `$ARGUMENTS` supplies a different base.

## 2. Collect diff as GitHub sees it

Three-dot syntax = merge-base aware (what GitHub shows).

```bash
git diff <base>...HEAD --stat
git diff <base>...HEAD
git log <base>..HEAD --oneline --no-merges
```

## 3. Detect repo PR template (optional)

Read it if present at any of: `.github/PULL_REQUEST_TEMPLATE.md`, `.github/pull_request_template.md`, `docs/pull_request_template.md`, `PULL_REQUEST_TEMPLATE.md`, `.github/PULL_REQUEST_TEMPLATE/default.md`.

Fill its sections using the feature-driven principles below. If the template conflicts with feature-driven style (e.g., demands a file-by-file changelog), follow the template but keep the opening examples + before/after + why intact.

## 4. Write description to `./pr.md`

Project folder.

# Core Principles

## Lead with what's now possible or fixed

Not with what was added/renamed/moved.

- Bad: "Adds `evidence-decider.ts`, modifies `auth.ts`, updates tests."
- Good: "Evidence capture now decides automatically whether a change has observable behavior. CLI tools and libraries are now eligible alongside web UIs."

## Plain user-facing language

Name the pain a user saw, not the internal exception type. "Invoices showed 'undefined' where currency should be" beats "NPE in currency formatter".

## No opener cliché

Never start a section or sentence with "This PR...", "This change...", "This commit...". Start with the subject of the action or the problem.

# Style Rules

## Voice

Active voice. Present tense. Lowercase OK for casual tone.

## Caveman tone

Terse. Drop articles (a/an/the), filler (just/really/basically), pleasantries, hedging. Fragments OK. Short synonyms (big not extensive, fix not "implement a solution for"). Technical terms exact. Code/commands unchanged. Pattern: [thing] [action] [reason]. No: "This change adds a feature that allows users to..." Yes: "Users can now export 10k rows."

## No checkbox filler

Do not write "[x] code written", "[x] tested" unless a repo template requires it AND claims are backed by evidence in the body.

## Length

Keep the whole body under ~400 words unless the change is genuinely large.

## Paragraph size — one idea per paragraph

Cap paragraphs at 2 sentences, ~50 words. Split whenever a new subject,
capability, or noun phrase begins ("An optional config...", "Global flags...",
"Two hooks collapse..."). One paragraph = one claim. Readers skim, they do not
read walls of text.

Bad (one block, two ideas):

> `rust-llm-tidy` is now one command that runs the full pipeline by default. No
> more subcommands. An optional `.rust-llm-tidy.yml` config drives excludes,
> per-path rule disabling, and post-process hooks.

Good (split, breathing room):

> `rust-llm-tidy` is now one command that runs the full pipeline (fix → reorder
> → vis → lints) by default. The `fix` / `reorder` / `vis` / `check` / `all`
> subcommands are gone.
>
> An optional `.rust-llm-tidy.yml` config drives excludes, per-path rule
> disabling, and post-process hooks (e.g. `rustfmt`), replacing two bespoke
> pre-commit hooks.

## Whitespace

Blank line between every paragraph and every section header. Never let a header
butt against prose. Never let two distinct thoughts share a paragraph. Err on
the side of more whitespace.

<examples>
<example label="Small fix — no headers">
<ideal_output>
Stripe retries webhooks on timeout. Handler wasn't idempotent → retried events
created duplicate invoices. Now records processed event IDs, skips repeats.
Verified: replayed captured retry sequence locally.

Closes #482
</ideal_output>
</example>

<example label="New feature — summary + usage example">
<ideal_output>
## Summary

Export up to 10,000 rows from any table to CSV or PDF. Old cap: 500 rows,
timed out silently.

## Usage

```bash
your-cli export --table users --format pdf --limit 10000
```

UI: any table → Export → choose format and scope.

## Why

Customers hit 500-row cap, split exports manually. #311

Closes #311
</ideal_output>
</example>

<example label="Behavior change — before/after shape">
<ideal_output>
## What changed

Tooling panel empty for OpenInference traces. Trace stored tools as
`[{tool: {json_schema: "..."}}]`, playground expected OpenAI shape
`[{type: "function", function: {...}}]` → parser silently dropped them.
Parser now unwraps `json_schema`, rebuilds each entry in OpenAI shape.
OpenAI-shape traces pass through unchanged.

Before:
```json
[{"tool": {"json_schema": "..."}}]
```

After:
```json
[{"type": "function", "function": {"name": "lookup", "parameters": {...}}}]
```

## How to verify
- Open OpenInference trace in playground → tool panel lists tools.
- Regression: OpenAI SDK trace renders as before.
</ideal_output>
</example>
</examples>

<output_contract>
File: `./pr.md` (project folder).
Title line (H1 or plain) is the PR title — concise, ≤72 chars, verb-first,
Keep a Changelog prefix allowed (Added/Changed/Fixed/Removed/Deprecated/Security).
Body sections appear only if they add value for the change size. Use:
1. Summary — 1–3 short sentences, problem/fix or new-capability framed. Split
   into separate paragraphs when a second distinct idea appears (see Paragraph
   size rule).
2. Usage / Example — code/config/CLI block demonstrating new behavior (when relevant).
3. Why — user pain or motivation; link issue.
4. Before / After — table or paired code blocks when shape/behavior shifted.
5. Risk — one line, only for migrations, auth/billing, irreversible writes,
   wide blast radius, or subtle behavior changes.
6. How to verify — concrete commands or steps, only when non-obvious.
7. References — `Closes #N` / related links at the bottom.

Omit empty sections.
</output_contract>

<tool_behavior>
- Do not commit or push. Do not open the PR unless the user explicitly asks.
  Only write `pr.md`.
- Do not modify tracked source files other than `pr.md`.
</tool_behavior>

<stop_rules>
- If on the default branch: STOP, return status `FAIL`, reason `on-default-branch`.
- If there are zero commits ahead of `<base>`: STOP, return status `FAIL`,
  reason `no-changes`.
- If no PR template detected and diff is < 20 lines: write Summary-only body
  (no headers) and stop — do not pad with empty sections.
- If a section would restate the diff: cut it.
</stop_rules>

<verification>
- Confirm `./pr.md` is readable and non-empty.
- Verify title ≤ 72 characters and starts with a verb or a Keep a Changelog prefix.
- Verify each code block is fenced and language-tagged where applicable.
- Report file path and word count.
</verification>

# Output

Return exactly after writing the file:

```text
Status: SUCCESS | FAIL
PR Path: <absolute path to pr.md>
Base Ref: <base branch used>
Files in Diff: <n>
Word Count: <n>
Summary: <one-line summary of the PR's feature/outcome>
Errors: <one-line error summary or None>
```
