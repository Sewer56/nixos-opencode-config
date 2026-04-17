---
mode: primary
description: Reorders declarations within source files to follow visibility tiers and reading order
permission:
  "*": deny
  read: allow
  glob: allow
  grep: allow
  list: allow
  edit: allow
  write: allow
  bash: allow
  external_directory: allow
---

Reorder declarations so reading top-to-bottom follows the call flow. This command is **plan-first** and requires explicit user confirmation before any code edits.

## Workflow

1. Load shared rules
- Read `CODE_PLACEMENT_RULES_PATH` once (defined in `## Shared Rules`).
- Use it as the source of truth for ordering policy.

2. Scope targets
- If user input includes file paths, use those paths directly.
- If no paths provided, collect changed files with `git status --porcelain`.
- Skip generated files, vendored code, lockfiles, snapshots, and binary assets.
- Skip files that are not source code (e.g. markdown, json, yaml without executable declarations).

3. Analyze each file
- Read the full file.
- Identify all top-level declarations (functions, methods, classes, structs, enums, constants, etc.).
- Determine the visibility tier of each declaration (public/exported vs private/internal).
- Build a call-dependency graph: note which declarations call which others.
- Determine the entry point(s) (e.g. `main`, the module's primary public API).
- Compute the target ordering using the rules described in `## Ordering Rules`.

4. Draft a reorder plan (no file edits)
- For each file, list the current declaration order (symbol names in file order).
- For each file, list the target declaration order (symbol names after reordering).
- Note when the order changes (e.g. "moved `_parse_csv` after `_cargo_metadata` to match call order").
- If a file is already in correct order, note it explicitly and skip it in the plan.

5. Confirmation gate (REQUIRED - DO NOT SKIP)
   - Present the plan using the format below.
   - STOP HERE. Do not proceed to step 6.
   - Wait for explicit user confirmation.
   - Use this format:
   ```text
   Proposed Reorder Plan
   
   Targets: <paths>
   
   <path>:
     Current: symbol_a, symbol_b, symbol_c, ...
     Target:  symbol_a, symbol_c, symbol_b, ...
     Changes: moved symbol_c before symbol_b (caller before callee)
   
   ...
   
   Files already in order: <paths>
   
   Say "go" to apply this plan, or suggest changes.
   ```
   - Continue ONLY when user says exactly "go".
   - If user suggests changes, revise the plan and re-run this gate.
   - DO NOT use edit or write tools before receiving "go".

6. Apply reorder (after `go`)
- Rewrite each affected file with declarations in the target order.
- Preserve all imports, module-level statements, doc comments, and trailing content.
- Do not change any logic, signatures, or documentation—only declaration order.

7. Verify
- Run formatter, lint, build/type checks, and tests according to repository conventions.
- Iterate until checks pass, or report exact blockers with file/symbol details.

8. Report
- Summarize files reordered and files already in order.
- Summarize symbol movements per file.
- Summarize verification command results.

## Shared Rules

- `RULES_DIR`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules`
- `CODE_PLACEMENT_RULES_PATH`: `code-placement.md` relative to `RULES_DIR`

## Ordering Rules

1. **Visibility tier**: public/entry-point declarations before private/internal ones.
2. **Reading order**: within each visibility tier, callers before callees.
3. **Entry point first**: place the entry point (e.g. `main`, the primary exported function) first; then direct callees in the order called; then their callees, and so on.
4. **Stability**: when two declarations have equal priority (same tier, no call dependency between them), preserve the existing relative order.

## Constraints

- Do not reorder across files; only reorder within a single file.
- Do not change logic, types, imports, or documentation.
- Preserve blank lines and section comments between declarations.
- When the language has a conventional entry point (e.g. `main`, `__init__`, `mod.rs` with `pub fn`), place it first regardless of other rules.
