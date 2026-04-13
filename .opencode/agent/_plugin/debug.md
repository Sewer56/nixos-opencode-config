---
mode: primary
description: Inspects a plugin file, extracts its debug flag and service name, executes the plugin with debug enabled, and checks logs for issues
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

Inspect a plugin file, extract its debug flag and service name, execute it with debug enabled, and check logs for issues.

# Input

Pass the full plugin file path through unchanged — the user must supply an absolute path to the plugin TypeScript file.

# Workflow

## 1. Read the plugin file
- Read the TypeScript file at the path the user provided.

## 2. Extract the debug flag
- Search the file content for `process.env.<SCREAMING_SNAKE_DEBUG>` assignments using the regex `process\.env\.(\w*DEBUG\w*)`.
- Capture the environment variable name (e.g. `FILE_INTERP_DEBUG`, `CAVEMAN_DEBUG`).
- If no match is found, report that the plugin does not expose a debug flag and stop here without launching the command.

## 3. Extract the service name
- Search the file content for the service identifier inside `client.app.log` calls using the regex `service:\s*"([^"]+)"`.
- Capture the service string (e.g. `file-interp`, `caveman`).
- If no service name is found, default to the plugin filename stem (e.g. `file-interp` from `file-interp.ts`).

## 4. Create the debug log directory
- Run via bash: `mkdir -p /tmp/opencode-debug`

## 5. Execute the plugin with debug enabled
- Build and execute the launch command:
  ```
  <DEBUG_FLAG>=1 opencode -p . --model <MODEL> &> /tmp/opencode-debug/<service>-$(date +%s).log
  ```
- Replace `<DEBUG_FLAG>` with the flag from step 2.
- Replace `<service>` with the name from step 3.
- Emit the command with `<MODEL>` as a literal placeholder.
- Wait for the process to terminate before proceeding.

## 6. Check the debug log
- Read the temp log file created in step 5.
- Grep for the service name to find plugin-produced entries.
- Report any entries at `error` or `warn` level as issues.
- If the log file is empty or contains no entries for the service, note that the plugin produced no log output during the run.

## 7. Report results
- Return a structured summary of findings.

# Output
Return exactly:

```text
Status: SUCCESS | NO_DEBUG_FLAG | NO_LOG_OUTPUT | FAIL
Debug Flag: <flag-name> | None
Service: <service-name> | None
Issues: <count>
Log Path: /tmp/opencode-debug/<service>-<timestamp>.log | N/A
Summary: <one-line summary>
```
