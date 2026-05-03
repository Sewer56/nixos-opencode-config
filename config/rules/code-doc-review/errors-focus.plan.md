### Errors-section ownership
Own all `# Errors` section concerns for changed scope described by I#/T# step files matching `step_pattern`: existence, placement, format, specificity, and completeness.

Bad: public error-returning API has no `# Errors` section or only says `may fail`.
Good: `# Errors` lists each variant with a concrete trigger.

### Specific triggers
Each error bullet must name the condition that produces it. Vague catch-all wording is insufficient.

Bad: `Returns Error if something goes wrong.`
Good: `Returns ParseError when the config file contains invalid TOML.`

### Targeted reads
Ground checks in step file diffs and handoff content. Open target source files only when a step diff is ambiguous or missing context for public API status or reachable variants.

### Scope boundary
Do not check general documentation coverage, inline comments, readability, or implementation correctness except to verify reachable error variants. Do not review D# steps.
