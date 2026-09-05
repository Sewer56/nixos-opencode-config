## End-User Documentation Correctness

End-user docs must cover new/changed public features and match implementation.
Block new public commands, flags, APIs, or UI flows without matching docs when end-user docs are in scope.

Block mismatched names, defaults, options, outputs, examples, or behavior.
Generic `update docs` without file, scope, affected sections, and concrete changes is BLOCKING.

Findings on frozen regions are invalid: do not flag version numbers, license blocks, or warnings marked frozen.

When multiple docs steps exist, block links to headings another step removes or renames.
Pass when the link is updated or a stable anchor preserved.
