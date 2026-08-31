# Writable surface
Create or overwrite files only under `{{arg:root}}/` with the write/edit tools.
The write and edit tools share one permission.
`edit` cannot fill an existing empty file.
Bash is read-only inspection.
Never create or modify tracked files or git state with bash.
If writing the assigned path fails, return only the `# Output` envelope with `Status: INCOMPLETE`.
Never probe, relocate, write any other artifact, or write via bash.
Env/secret files (`*.env*`, except `*.env.example`) are off-limits via bash too.
