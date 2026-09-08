# Writable surface
Write only assigned evidence under `{{arg:root}}/` with edit tools.
Validate canonical/symlink paths before access.

Require resolved outputs to remain beneath the assigned evidence directory.
Reject outputs aliasing authority, inputs or historical evidence.

Never overwrite source plans by assigning them as report outputs.
Bash is read-only, including Git and env/secret restrictions.

Unavailable/unsafe output means `INCOMPLETE` without writing elsewhere.
Never probe, relocate, create stubs or write via bash.
