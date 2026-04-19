---
description: "Run the orchestrator against a built prompt pack"
agent: _orchestrator/orchestrator
---

## User Input

```text
$ARGUMENTS
```

## Path Resolution

- If `$ARGUMENTS` is a path to a `PROMPT-ORCHESTRATOR.md` file, use it.
- If `$ARGUMENTS` is a directory, look for `PROMPT-ORCHESTRATOR.md` inside it.
- If empty, look for `PROMPT-ORCHESTRATOR.md` in the current working directory.
- If no orchestrator file is found, tell the user to run `/orchestrator/prompt-pack` first.
