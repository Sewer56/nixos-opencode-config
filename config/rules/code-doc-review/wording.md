### Sentence flow
Flag choppy, run-on, or awkward sentence construction. Suggest smoother phrasing. ADVISORY.

Bad: `This does X. It also does Y. Which means Z.`
Good: `This does X and Y, which means Z.`

### Passive voice
Flag passive voice when active voice is clearer. BLOCKING for instructions; ADVISORY for descriptive prose.

Bad: `The command should be run by the user.`
Good: `Run the command.`

### Filler
Flag hedging and zero-information phrases: `please note`, `it's important to`, `make sure to`, `ensure that`, `simply`, `just`, `arguably`, `possibly`, `might want to`. BLOCKING.

Bad: `Please note that you should simply run the command.`
Good: `Run the command.`

### Wordiness
Flag phrasing that can be tightened without losing meaning. ADVISORY; block only for egregious inflation.

Bad: `in order to make it possible for users to configure`
Good: `so users can configure`

### Terminology consistency
Flag different terms for the same concept within one source file or step. BLOCKING when ambiguous; ADVISORY for harmless stylistic variation.

Bad: same feature called `settings`, `configuration`, and `preferences` with no distinction.
Good: choose one term or define the distinction.
