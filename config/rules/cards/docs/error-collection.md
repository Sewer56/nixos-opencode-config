### Exhaustive error-function enumeration
Enumerate every public error-returning function in the target path using language rules. Private/internal helpers are out of scope.
Bad: scanning only exported files and missing public functions in nested modules.
Good: all public error-returning functions recorded with path, line, and return type.

### Reachable error tracing
Trace every reachable error path in each function body. Record one entry per variant/trigger pair.
Bad: one generic `may fail` entry for several branches.
Good: separate entries for parse failure, missing file, and permission error.

### Error-doc classification accuracy
Classify existing docs using the language rule decision table.
Bad: mark vague docs as specific because an `# Errors` header exists.
Good: mark specific only when each reachable path has concrete variant and trigger docs.
