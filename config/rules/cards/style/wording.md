### Writer guidance (lite-caveman)

- Use the project's defined name for every concept.
- Keep code identifiers, commands, paths, and URLs verbatim.
- Use professional prose with one idea per sentence (≤20 words).

### Sentence flow
Flag: choppy, run-on, or awkward sentence construction.
Severity: ADVISORY.

### Passive voice
Flag: passive voice when active voice is clearer.
Severity: BLOCKING for instructions; ADVISORY for descriptive prose.
Prefer direct imperatives for instructions.

### Filler and token density
Flag: filler, hedging, pleasantries, and zero-information phrases.
Severity: BLOCKING in operational instructions; ADVISORY in narrative prose.

### Wordiness
Flag: phrasing that can be tightened without changing meaning.
Use shorter synonyms only when meaning and safety wording stay exact.
Preserve technical terms, identifiers, and API/CLI names.
Prefer precise terms over cryptic shortcuts.

Severity: ADVISORY; BLOCKING only for egregious inflation.

### Terminology consistency
Flag: different terms for the same concept within the reviewed artifact or artifact set.
Severity: BLOCKING when ambiguous; ADVISORY for harmless stylistic variation.
Fix by choosing one term or defining the distinction.

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

Fix: delete the restated clause; keep any non-duplicated remainder.

Keep facts absent from the example: behavior, effects, order, differing values.
Keep those facts even when fused with a restated literal.
Purpose-bearing lead-ins are exempt.

Severity: BLOCKING in end-user and in-code docs; ADVISORY in narrative prose.
