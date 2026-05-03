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

### Exclusions
Do not block common programming terms, exact code identifiers, terms defined earlier in the same file/step, headings, section names, non-prescriptive prose, or standard domain terms.
