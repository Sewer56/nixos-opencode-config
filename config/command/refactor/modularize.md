---
description: "Refactor existing code into a modular layout"
agent: build
---

# Modularize Code

Refactor existing code into smaller, cohesive modules/files with clearer names.
Aggressive renaming of modules, files, structs/classes, and functions is allowed when it improves readability and maintainability.

This command is **plan-first** and requires explicit user confirmation before any code edits.

## User Input

```text
$ARGUMENTS
```

Use the user input as the target scope (one or more files/directories).

## Workflow

1. Scope targets
- Resolve each provided path.
- If a directory is provided, discover relevant source files under it.
- If no target is provided, stop and ask for explicit path(s).

2. Build a migration map (in memory)
- Map old files/modules to new files/modules.
- Map old exported/public symbols to new names and locations.
- Preserve behavior while improving modular structure.

3. Draft a modularization plan (no file edits)
- Produce:
  - target module/file layout
  - rename map (old symbol -> new symbol)
  - ordered migration steps
  - compatibility strategy (re-exports/shims vs direct break)
  - verification plan

4. Confirmation gate (required)
- Present the plan and stop.
- Use this format:
```text
Proposed Modularization Plan

Targets: <paths>

Layout:
<tree>

Rename Map:
- old_symbol -> new_symbol

Migration Steps:
1. ...
2. ...

Verification:
- <commands>

Say "go" to apply this plan, or suggest changes.
```
- Continue only when user says `go`.
- If user suggests changes, revise the plan and re-run this gate.

5. Modularize implementation (after `go`)

6. Update references
- Update imports/usages across the codebase for moved or renamed symbols.
- Keep compatibility re-exports/shims only when useful; otherwise complete the rename migration.

7. Verify
- Run formatter, lint, build/type checks, and tests according to repository conventions.
- Iterate until checks pass, or report exact blockers with file/symbol details.

8. Report
- Summarize:
  - Files/modules created, removed, and moved
  - Symbol renames and final locations
- Any compatibility shims or intentional breaking changes
- Verification command results

## Example

Example target layout:
```text
src/config/
  mod.rs
  models/
    binding_profile.rs
    config_binding.rs
    device_mapping.rs
```

Example rename map:
- `ConfigData` -> `InputConfig`
- `helpers.rs` -> `device_selector.rs`
- `parse` -> `parse_binding_profile`

# Rules

{{ file="./rules/quality/general.md" }}
{{ file="./rules/quality/performance.md" }}
{{ file="./rules/testing/testing.md" }}
{{ file="./rules/testing/test-parameterization.md" }}
{{ file="./rules/quality/code-placement.md" }}
{{ file="./rules/docs/documentation.md" }}
