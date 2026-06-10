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

Find public items that should be private. Produce report with exact diffs.

# Inputs

- `$ARGUMENTS`: (optional) file or directory paths. Empty → audit entire repo.
- `repo_root`: from working directory.

# Workflow

## 1. Resolve targets

**Paths given:**
- For each path: if dir, discover source files under it; if file, use directly. Detect language by extension.
- No valid source files → stop, tell user.

**No paths:**
- Ask user: "Audit entire repo? (y/n)". No → stop.
- Spawn `codebase-explorer` to discover all source files. Skip test patterns from `agent/_audit/_templates/lang-*.txt` and files with `Code generated` or `DO NOT EDIT` in first 5 lines.

## 2. Collect

Group targets by language. For each language, pipe file list through:

```
python {{path:./scripts/chunk-files-by-tokens.py}}
```

Default 32k tokens/chunk. Override with `-s`.

Parse output: `chunk N: TOTAL` lines begin groups; `TOKENS FILE_PATH` lines list files. Blank lines separate chunks.

Spawn one `_audit/public-api/collector` per chunk, all in parallel. Each gets:
- `language`: detected language
- `repo_root`: absolute repo root
- `specific_paths`: absolute file paths in chunk

Wait for ALL collectors. Collector output is final — do not re-query or resume.

## 3. Filter

**Paths given:** discard items whose `File` is not under a user-provided path. Cross-reference usage counts from full repo are preserved.

**No paths:** keep all items.

## 4. Classify

Scope: `targeted: <paths>` or `whole repo`. Scope line: `N paths (languages)` or `N files (languages)`.

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
