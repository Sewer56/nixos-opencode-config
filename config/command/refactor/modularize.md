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

## Shared Rules

- `RULES_DIR`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules`
- `GENERAL_RULES_PATH`: `general.md` relative to `RULES_DIR`
- `DOCUMENTATION_RULES_PATH`: `documentation.md` relative to `RULES_DIR`
- `PERFORMANCE_RULES_PATH`: `performance.md` relative to `RULES_DIR`
- `TESTING_RULES_PATH`: `testing.md` relative to `RULES_DIR`
- `TEST_PARAMETERIZATION_RULES_PATH`: `test-parameterization.md` relative to `RULES_DIR`
- `CODE_PLACEMENT_RULES_PATH`: `code-placement.md` relative to `RULES_DIR`

## Workflow

1. Load shared rules
- Read the files in `RULES_DIR` named by `GENERAL_RULES_PATH`, `DOCUMENTATION_RULES_PATH`, `PERFORMANCE_RULES_PATH`, `TESTING_RULES_PATH`, `TEST_PARAMETERIZATION_RULES_PATH`, and `CODE_PLACEMENT_RULES_PATH` once.
- Use them as the source of truth for naming, structure, docs, performance, test coverage, and test shape.

2. Scope targets
- Resolve each provided path.
- If a directory is provided, discover relevant source files under it.
- If no target is provided, stop and ask for explicit path(s).

3. Build a migration map (in memory)
- Map old files/modules to new files/modules.
- Map old exported/public symbols to new names and locations.
- Preserve behavior while improving modular structure.

4. Draft a modularization plan (no file edits)
- Produce:
  - target module/file layout
  - rename map (old symbol -> new symbol)
  - ordered migration steps
  - compatibility strategy (re-exports/shims vs direct break)
  - verification plan

5. Confirmation gate (required)
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

6. Modularize implementation (after `go`)
- Apply `GENERAL_RULES_PATH`, `DOCUMENTATION_RULES_PATH`, `PERFORMANCE_RULES_PATH`, `TESTING_RULES_PATH`, `TEST_PARAMETERIZATION_RULES_PATH`, and `CODE_PLACEMENT_RULES_PATH`.

7. Update references
- Update imports/usages across the codebase for moved or renamed symbols.
- Keep compatibility re-exports/shims only when useful; otherwise complete the rename migration.

8. Verify
- Run formatter, lint, build/type checks, and tests according to repository conventions.
- Iterate until checks pass, or report exact blockers with file/symbol details.

9. Report
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
