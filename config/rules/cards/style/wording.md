### Writer guidance (lite-caveman)

Professional but tight. Every sentence carries weight.

- Technical terms exact: use the project's defined name for every concept. Code identifiers, commands, paths, and URLs stay verbatim.
- Active voice, one idea per sentence (≤20 words).
- No filler, hedging, or pleasantries: `please note`, `simply`, `just`, `of course`, `certainly`, `basically`.

### Sentence flow
Flag: choppy, run-on, or awkward sentence construction.
Severity: ADVISORY.

### Passive voice
Flag: passive voice when active voice is clearer.
Severity: BLOCKING for instructions; ADVISORY for descriptive prose.
Prefer direct imperatives for instructions.

### Filler and token density
Flag: hedging and zero-information phrases such as `please note`, `it's important to`, `make sure to`, `ensure that`, `simply`, `just`, `arguably`, `possibly`, `might want to`.

Severity: BLOCKING in operational instructions; ADVISORY in narrative prose.

### Wordiness
Flag: phrasing that can be tightened without changing meaning.
Allow: technical terms and identifiers; precise terms over cryptic shortcuts.
Severity: ADVISORY; BLOCKING only for egregious inflation.

### Terminology consistency
Flag: different terms for the same concept within the reviewed artifact or artifact set.
Severity: BLOCKING when ambiguous; ADVISORY for harmless stylistic variation.
Fix by choosing one term or defining the distinction.

### Short synonym
Flag: verbose word choice when a shorter synonym preserves meaning.

Allow: required technical terms, code identifiers, API/CLI names, safety wording, precise technical jargon.

Severity: ADVISORY.

Examples: use not utilize, show not demonstrate, help not facilitate, to not in order to.

### Paragraph length
Flag: paragraphs over 4 sentences or 4 rendered lines.
Severity: ADVISORY.
Split long paragraphs into task-focused paragraphs or lists.

### Bullet atomicity
Flag: Focus, Process, Constraint, or instruction bullets that combine multiple checkable conditions.
Severity: ADVISORY unless combined conditions hide a required action.
Split into one bullet per checkable action.

### Example-prose redundancy
Prose must not restate what the adjacent example shows: the call, literal arguments, or defaults.

Fix: delete the restated clause; keep any non-duplicated remainder, e.g. a trailing cross-reference such as `[Override the policy] shows values that differ:`.

Keep prose for facts the example cannot show: behavior, consequences, ordering, differing values. Keep wins for such facts even when fused with a restated literal; purpose-bearing lead-ins are exempt.

Severity: BLOCKING in end-user and in-code docs; ADVISORY in narrative prose.
