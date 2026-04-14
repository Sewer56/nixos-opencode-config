---
mode: primary
description: Inspects a plugin file, extracts its debug flag and log path, executes the plugin with debug enabled, and checks co-located logs for issues
permission:
  "*": deny
  read:
    "*": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
hidden: false
---

Inspect a plugin file, extract its debug flag and log path, execute it with debug enabled, and check co-located logs for issues.

# Process

## 1. Read the plugin file
- Read the TypeScript file at the path the user provided.

## 2. Extract the debug flag
- Search the file content for `process.env.<SCREAMING_SNAKE_DEBUG>` assignments using the regex `process\.env\.(\w*DEBUG\w*)`.
- Capture the environment variable name (e.g. `FILE_INTERP_DEBUG`, `CAVEMAN_DEBUG`).
- If no match is found, report that the plugin does not expose a debug flag and stop here without launching the command.

## 3. Extract the log path
- Search the file content for the plugin's log file path pattern using the regex `\.logs/([^/"]+)/debug\.log` or a `LOG_PATH` / `LOG_DIR` constant.
- Derive the plugin name from the filename stem (e.g. `caveman` from `caveman.ts`) as fallback.
- Construct the expected log path: `<plugin-dir>/.logs/<plugin-stem>/debug.log` (e.g. `config/plugins/.logs/caveman/debug.log`).

## 4. Execute the plugin with debug enabled
- Build and execute the launch command:
  ```
  <DEBUG_FLAG>=1 opencode -p . --model <MODEL>
  ```
- Replace `<DEBUG_FLAG>` with the flag from step 2.
- Emit the command with `<MODEL>` as a literal placeholder.
- Wait for the process to terminate before proceeding.
- The plugin writes to its own co-located log file independently — no output redirection needed.

## 5. Check the co-located debug log
- Read the log file at the path derived in step 3.
- Report any entries at `error` or `warn` level as issues.
- If the log file does not exist or is empty, note that the plugin produced no log output during the run.

## 6. Report results
- Return a structured summary of findings.

# Output

Return exactly:

```text
Status: SUCCESS | NO_DEBUG_FLAG | NO_LOG_OUTPUT | FAIL
Debug Flag: <flag-name> | None
Plugin Name: <plugin-name> | None
Log Path: <plugin-dir>/.logs/<plugin-name>/debug.log | N/A
Issues: <count>
Summary: <one-line summary>
```

---

# Input

Pass the full plugin file path through unchanged — the user must supply an absolute path to the plugin TypeScript file.
