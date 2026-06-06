### Writer guidance (lite-caveman)

Professional but tight. Every sentence carries weight.

- Technical terms exact: use the project's defined name for every concept. Code identifiers, commands, paths, and URLs stay verbatim.
- Short synonyms: use not utilize, show not demonstrate, help not facilitate, to not in order to.
- Active voice, one idea per sentence (≤20 words).
- No filler, hedging, or pleasantries: no `please note`, `simply`, `just`, `of course`, `certainly`, `basically`.

### Sentence flow
Flag: choppy, run-on, or awkward sentence construction.
Severity: ADVISORY.
Bad: `This does X. It also does Y. Which means Z.`
Good: `This does X and Y, which means Z.`

### Passive voice
Flag: passive voice when active voice is clearer.
Severity: BLOCKING for instructions; ADVISORY for descriptive prose.
Bad: `The command should be run by the user.`
Good: `Run the command.`

### Filler and token density
Flag: hedging and zero-information phrases such as `please note`, `it's important to`, `make sure to`, `ensure that`, `simply`, `just`, `arguably`, `possibly`, `might want to`.
Severity: BLOCKING in operational instructions; ADVISORY in narrative prose.
Bad: `Please make sure to ensure that the plan is able to update the file.`
Good: `Update the file.`

### Wordiness
Flag: phrasing that can be tightened without changing meaning; use the fewest words that preserve exact meaning.
Allow: necessary technical terms and identifiers; prefer precise terms over cryptic shortcuts.
Severity: ADVISORY; BLOCKING only for egregious inflation.
Bad: `in order to make it possible for reviewers to determine`
Good: `so reviewers can determine`

### Terminology consistency
Flag: different terms for the same concept within the reviewed artifact or artifact set.
Severity: BLOCKING when ambiguous; ADVISORY for harmless stylistic variation.
Bad: same feature called `settings`, `configuration`, and `preferences` with no distinction.
Good: choose one term or define the distinction.

### Short synonym
Flag: verbose word choice when a shorter synonym preserves meaning.
Allow: required technical terms, code identifiers, API/CLI names, safety wording, precise technical jargon.
Severity: ADVISORY.
Bad: `utilize`, `demonstrate`, `facilitate`, `in order to`, `as a means of`
Good: `use`, `show`, `help`, `to`, `to`

### Paragraph length
Flag: paragraphs over 4 sentences or 4 rendered lines.
Severity: ADVISORY.
Bad: one paragraph covers setup, usage, caveats, and troubleshooting.
Good: split into short paragraphs or list items by task.

### Bullet atomicity
Flag: Focus, Process, Constraint, or instruction bullets that combine multiple checkable conditions.
Severity: ADVISORY unless combined conditions hide a required action.
Bad: `Read the draft, check paths, and update cache.`
Good: split into one bullet per checkable action.
