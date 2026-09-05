## Error Collection

Enumerate every public error-returning function in the target path, including nested modules, using language rules.
Private/internal helpers are out of scope.
Record path, line, and return type.

Trace every reachable error path in each function body, recording one entry per variant/trigger pair.
Block generic `may fail` entries that collapse distinct variants or triggers.

Classify existing docs using the language rule decision table.
Mark docs specific only when each reachable path documents its concrete variant and trigger.
