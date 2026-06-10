---
mode: primary
description: Audits unnecessarily public APIs with privatization diffs — targeted files or entire repo
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
    "_audit/public-api/collector": "allow"
---

Audit items that are public/exported but should not be. Produce report with exact diffs.

# Inputs

- `$ARGUMENTS`: (optional) file or directory paths. Empty → audit entire repo.
- `repo_root`: determined from working directory.

# Orchestration

## 1. Resolve targets

**If `$ARGUMENTS` has paths:**

For each path:
- If directory: discover source files under it
- If file: use directly
- Detect language from file extension

If no valid source files found, stop and tell user.

**If `$ARGUMENTS` is empty:**

Ask user: "Audit entire repo? (y/n)". If no, stop.

Spawn `codebase-explorer` to discover all source files in repo. Skip files matching test patterns from language files (`agent/_audit/_templates/lang-*.txt`) and files with `Code generated` or `DO NOT EDIT` in first 5 lines.

## 2. Collect

Group targets by detected language. For each language, pipe file list through:

```
python {{path:./scripts/chunk-files-by-tokens.py}}
```

Spawn one `_audit/public-api/collector` per chunk in single parallel call. Each chunk is <64k estimated tokens.

Per collector:
- `language`: detected language
- `repo_root`: absolute path to repo root
- `specific_paths`: comma-separated list of absolute file paths in chunk

## 3. Gate

Wait for ALL collectors to return. Collector output is final — per-item blocks for candidates/review, then summary. Do not re-query or resume.

# Analysis

## 4. Filter

If `$ARGUMENTS` had paths: discard collector items whose `File` is not within a user-provided target path. Only user-requested files enter classification. Usage counts from full repo cross-reference preserved.

If `$ARGUMENTS` was empty: keep all items (no filter).

## 5. Classify

Scope: `targeted: <paths>` if paths given, otherwise `whole repo`. Scope line: `N paths (languages)` or `N files (languages)`.

### Rules

{{ file="./config/rules/groups/audit/search-public-api-analysis.md" }}

{{ file="./config/agent/_audit/_templates/analysis-report.txt" }}

# Output

```text
Status: SUCCESS | FAIL
Report Path: <absolute path to PROMPT-API-AUDIT.md>
Files Audited: <n>
Candidates: <n>
Summary: <one-line summary>
```
