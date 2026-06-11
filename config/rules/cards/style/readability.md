### Undefined jargon
Flag: technical, project-specific, or internal taxonomy terms the intended reader cannot resolve nearby.
Allow: inline definition, plain-language rewrite, glossary/link/path pointer, tooltip, or suitable comment.
Severity: BLOCKING when the reader could act incorrectly; otherwise ADVISORY.

### Ambiguous language
Flag: phrases with multiple plausible interpretations where a reader could act incorrectly.
Severity: BLOCKING.
Fix by naming the exact path, condition, or action.

### Compound-term compression
Flag: compressed phrases that sacrifice comprehension.
Severity: ADVISORY unless the compressed phrase blocks action.
Prefer plain expansions over stacked shorthand.

### Opaque reference
Flag: references to patterns, conventions, pages, or internal systems that are not standard and not defined nearby.
Allow: inline the convention or point to a path when navigation is enough.

### Acronym without expansion
Flag: acronyms not expanded on first use.
Severity: BLOCKING for project-specific acronyms; ADVISORY for widely known acronyms.
Fix with `Expanded Name (ACRONYM)` on first use.

### Readability exclusions
Do not flag common programming terms, exact code identifiers, path pointers, terms defined earlier in the same artifact, headings, section names, non-prescriptive prose, or standard domain terms.
