## Readability

### Undefined jargon and opaque references
Flag technical, project-specific, or internal taxonomy terms the intended reader cannot resolve nearby.
Allow inline definitions, plain-language rewrites, glossary/link/path pointers, tooltips, or suitable comments.

Severity: BLOCKING if the reader could act incorrectly; otherwise ADVISORY.

Flag nonstandard references to patterns, conventions, pages, or internal systems not defined nearby.
Inline the convention or give a path when navigation suffices.

### Ambiguity and compression
Flag phrases with multiple plausible interpretations that could cause incorrect action as BLOCKING.
Name the exact path, condition, or action.

Flag compressed phrases that sacrifice comprehension as ADVISORY unless they block action.
Prefer plain expansions over stacked shorthand.

### Acronyms
Flag acronyms not expanded on first use.
Severity: BLOCKING for project-specific acronyms; ADVISORY for widely known acronyms.
Use `Expanded Name (ACRONYM)` on first use.

### Exclusions
Do not flag common programming terms, exact code identifiers, path pointers, or terms defined earlier in the artifact.
Do not flag headings, section names, non-prescriptive prose, or standard domain terms.
