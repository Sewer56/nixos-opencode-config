### End-user feature coverage
End-user docs must cover new or changed public features.
Bad: new CLI flag appears in implementation steps but no docs step documents it.
Good: docs step updates the usage page and example command.

### End-user docs fidelity
End-user docs must not contradict implementation.
Bad: docs say default is `true`; code sets default `false`.
Good: docs reflect actual default and behavior.

### End-user docs specificity
Generic `update docs` without file, scope, affected sections, and concrete changes is BLOCKING.
Bad: `Update docs for new feature.`
Good: `Update docs/usage.md Quick Start to add --watch example and describe reload behavior.`

### Frozen-region compliance
Findings on frozen regions are invalid.
Do not flag: version numbers, license blocks, or warnings marked frozen.

### End-user internal links
When multiple docs steps exist, block links to headings another step removes or renames.
Bad: D1 links to `#old-name` while D2 renames it to `#new-name`.
Good: link updated or stable anchor preserved.
