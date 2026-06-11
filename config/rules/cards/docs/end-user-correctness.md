### End-user feature coverage
End-user docs must cover new or changed public features.
Block new public commands, flags, APIs, or UI flows without matching docs when end-user docs are in scope.

### End-user docs fidelity
End-user docs must not contradict implementation.
Block mismatched names, defaults, options, outputs, examples, or behavior.

### End-user docs specificity
Generic `update docs` without file, scope, affected sections, and concrete changes is BLOCKING.

### Frozen-region compliance
Findings on frozen regions are invalid.
Do not flag: version numbers, license blocks, or warnings marked frozen.

### End-user internal links
When multiple docs steps exist, block links to headings another step removes or renames.
Pass when the link is updated or a stable anchor is preserved.
