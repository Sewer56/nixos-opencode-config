### Undefined jargon
Flag technical, project-specific, or internal taxonomy terms used without inline definition, plain-language rewrite, glossary link, or tooltip.

Bad: `Enable the hydration seam.`
Good: `Enable the startup hook that initializes state before rendering.`

### Ambiguous language
Flag phrases with multiple plausible interpretations where a reader could act incorrectly. BLOCKING.

Bad: `Update the nearby config when needed.`
Good: `Update config/app.toml when the new flag is enabled.`

### Compound-term compression
Flag compressed phrases that sacrifice comprehension.

Bad: `hot-reload DX pipeline`
Good: `developer workflow that reloads the app after source changes`

### Opaque reference
Flag references to patterns, conventions, or pages that are not standard and not defined nearby.

Bad: `Follow the adapter convention.`
Good: `Wrap external calls in an adapter module so callers depend on one local interface.`

### Acronym without expansion
Flag acronyms not expanded on first use. BLOCKING for project-specific acronyms; ADVISORY for widely known acronyms.

Bad: `SSR must stay enabled.`
Good: `Server-side rendering (SSR) must stay enabled.`

### Scope boundary
Review linguistic comprehensibility only. Do not judge correctness, duplication, or wording style unless unclear language causes the issue.

Bad finding: `This API call is wrong.`
Good finding: `The text says "bridge" without explaining which module or behavior it means.`

Bad: flag a wrong hook name as clarity.
Good: flag undefined wording that prevents knowing which hook is meant.

### Review Blocking Criteria
- Block for ambiguous language and project-specific acronyms without expansion.
- Do not block for undefined jargon, compound-term compression, opaque references, widely known acronyms, or exclusions listed below.

### Exclusions
Do not block these as clarity issues:
- Common programming terms such as `API`, `HTTP`, `markdown`, `frontmatter`.
- Path-based pointers to other docs.
- Terms defined earlier in the same page/file/ticket.
- Headings, section names, and non-prescriptive prose.
- Standard domain terms known to practitioners in the documentation's subject domain.
