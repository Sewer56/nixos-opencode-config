## Wording

Use the project's defined name for every concept.
Keep technical terms, code identifiers, API/CLI names, commands, paths, and URLs exact.
Use professional prose with one idea per sentence (≤20 words).

### Flow and concision
Flag choppy, run-on, or awkward sentences as ADVISORY.
Flag filler, hedging, pleasantries, and zero-information phrases.
Severity: BLOCKING in operational instructions; ADVISORY in narrative prose.

Flag phrasing that can be tightened without changing meaning.
Use shorter synonyms only when meaning and safety wording stay exact.
Prefer precise terms over cryptic shortcuts.
Wordiness is ADVISORY; only egregious inflation is BLOCKING.

### Terminology and bullets
Flag different terms for the same concept within the reviewed artifact or artifact set.
Severity: BLOCKING when ambiguous; ADVISORY for harmless stylistic variation.
Choose one term or define the distinction.

Split Focus, Process, Constraint, or instruction bullets into one checkable action each.
Combined conditions are ADVISORY unless they hide a required action.

### Example-prose redundancy
Prose must not restate an adjacent example's call, literal arguments, or defaults.
Delete restated clauses; keep non-duplicated facts, even when fused with restated literals.

Preserve behavior, effects, order, and differing values absent from the example.
Purpose-bearing lead-ins are exempt.
Severity: BLOCKING in end-user and in-code docs; ADVISORY in narrative prose.
