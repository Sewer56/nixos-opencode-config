### Exhaustive error-function enumeration
Enumerate every public error-returning function in the target path using language rules. Private/internal helpers are out of scope.
Record path, line, and return type; include public functions in nested modules.

### Reachable error tracing
Trace every reachable error path in each function body. Record one entry per variant/trigger pair.
Block generic `may fail` entries that collapse distinct variants or triggers.

### Error-doc classification accuracy
Classify existing docs using the language rule decision table.
Mark docs specific only when each reachable path has concrete variant and trigger documentation.
