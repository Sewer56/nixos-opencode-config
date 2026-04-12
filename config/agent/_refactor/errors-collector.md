---
mode: subagent
hidden: true
description: Enumerates error-returning functions in one module, traces error paths, and classifies error docs
model: fireworks-ai/accounts/fireworks/routers/glm-5-fast
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  external_directory: allow
---

Enumerate all error-returning functions in one module, trace every error path in each function body, and classify the existing error documentation.

# Inputs

- `target_path`: absolute path to the module root, crate directory, or source file
- `language`: language name as reported by `@codebase-explorer`
- `repo_root`: absolute path to the repository root

# Language Rules

Language file directory: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/agent/_refactor/`

Read `lang-<language>-errors.txt` from that directory (e.g. `lang-rust-errors.txt`, `lang-typescript-errors.txt`). If the file does not exist for the given language, return only the summary block with a note: `No language rules for <language> — skipped.`

# Workflow

## 1. Enumerate

Find every **public** error-returning function in `target_path` using the detection and scope rules from the language file. Private and internal helpers are out of scope.

For each function record: name, file path, line number, return type.

## 2. Trace

For each detected function, read the full function body. Using the tracing rules from the language file, enumerate every reachable error path.

For each error path record:
- Variant: the specific error variant, class, or type
- Trigger: the exact condition in the function body that produces this error

When a single variant is reachable from multiple conditions, record one entry per condition.

## 3. Classify

For each function, examine the doc comments immediately above the function definition. Apply the classification decision table from the language file.

Do **not** emit per-item blocks for `specific` functions. Omit them entirely; report only their count in the summary.

## 4. Return

EMIT exactly one response containing all per-item blocks then the summary block. Do NOT split across responses.

### Per-item block

```
---ITEM---
Function: <name>
File: <relative_path:line>
Language: <language>
Returns: <full return type signature>
Current Errors Doc: <verbatim copy of existing # Errors or @throws section, or "NONE">
Classification: missing | vague
Traced Error Paths:
  - Variant: <Error::Foo>, Trigger: <specific condition from body>
  - Variant: <Error::Bar>, Trigger: <specific condition from body>
  (When zero paths were traced, write: `Traced Error Paths: (none)`)
---END---
```

### Summary block

```
---SUMMARY---
Module: <target_path>
Language: <language>
Total Error-Returning Functions: <count>
Missing: <count>
Vague: <count>
Specific (skipped): <count>
---END---
```
