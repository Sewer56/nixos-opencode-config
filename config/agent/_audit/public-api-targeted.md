---
mode: primary
description: Audits specific files or folders for unnecessarily public APIs with privatization diffs
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "PROMPT-API-AUDIT.md": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "codebase-explorer": "allow"
    "_audit/collector": "allow"
---

Audit specific files or folders for items that are public/exported but should not be. Cross-reference usage across the entire repository. Produce a report with exact diffs.

# Inputs

- `$ARGUMENTS`: one or more file or directory paths to audit.
- `repo_root`: determined from the working directory.

# ORCHESTRATION

## 1. Resolve targets

The user-provided paths are in `$ARGUMENTS`.

For each path:

- If a directory: discover source files under it
- If a file: use directly
- Detect the language from the file extension
- Walk up from the file to find the parent module boundary (nearest Cargo.toml, package.json, `__init__.py`, `pyproject.toml`, `setup.py`, `build.gradle`, or `build.gradle.kts`). For Go, use the file's own directory as the module boundary (Go packages are per-directory per lang-go.txt, not per `go.mod`). If no marker is found, use the file's parent directory.

`repo_root`: the git repository root (parent directory containing `.git`).

If no valid source paths remain after resolution, stop and tell the user.

## 2. Collect

Group resolved paths by parent module and language. When a module directory contains source files in multiple languages, group by (parent module, language) so each language gets its own collector. Spawn one `_audit/collector` per group in a single parallel call.

Per collector, pass:

- `target_path`: absolute path to the **parent module directory** (not individual files)
- `language`: detected language
- `repo_root`: absolute path to the repository root
- `specific_paths`: list of absolute paths to the user-provided files/directories within the module

The collector enumerates public items in the module (limited to `specific_paths` if the module exceeds 80 items). After collection, step 4 filters to only user-requested files.

Collectors cross-reference against the entire repo - not just the target paths - because usage may come from anywhere.

## 3. Gate

Wait for ALL collectors to return before proceeding. Do not begin any analysis until every collector has reported.

Collector output is final — per-item blocks for candidates/review, then summary. Do not re-query or resume.

# ANALYSIS

## 4. Filter

Discard any collector items whose `File` field is not within or equal to one of the user-provided target paths. Only user-requested files enter classification. Usage counts from the full repo cross-reference are preserved.

## 5. Classify

Use `targeted: <paths>` as the scope value and `N paths (languages)` as the scope line.

### Rules

{{ file="./rules/groups/audit/search-public-api-analysis.md" }}

{{ file="./agent/_audit/_templates/analysis-report.txt" }}

# Output

Return exactly:

```text
Status: SUCCESS | FAIL
Report Path: <absolute path to PROMPT-API-AUDIT.md>
Targets Audited: <n>
Candidates: <n>
Summary: <one-line summary>
```
