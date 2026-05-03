### Errors-section ownership
Own all `# Errors` section concerns for public error-returning APIs in source files listed in `## Target Files`: existence, placement, format, specificity, and completeness.

Bad: public error-returning API has no `# Errors` section or only says `may fail`.
Good: `# Errors` lists each variant with a concrete trigger.

### Specific triggers
Each error bullet must name the condition that produces it. Vague catch-all wording is insufficient.

Bad: `Returns Error if something goes wrong.`
Good: `Returns ParseError when the config file contains invalid TOML.`

### Targeted reads
Read only repo files needed to ground error-doc checks.

### Scope boundary
Do not check general documentation coverage, inline comments, readability, or implementation correctness except to verify reachable error variants.
