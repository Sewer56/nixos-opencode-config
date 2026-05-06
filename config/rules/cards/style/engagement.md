### Hook-first content
First 50 words should answer what this is, why it is different, and who it is for.
Severity: BLOCKING for landing/index pages; ADVISORY for inner reference pages.
Bad: starts with history, welcome text, or implementation detail.
Good: starts with value, audience, and differentiator.

### Hook-first length
The hook should fit in roughly 50 words or 3 short sentences.
Severity: ADVISORY.
Bad: first screen has several paragraphs before the value statement.
Good: concise opening followed by details.

### Show-don't-tell
Getting-started and guide pages need a concrete example, command, terminal output, or visual within the first screenful.
Severity: BLOCKING for guides; ADVISORY for reference pages.
Bad: long conceptual intro before any command or example.
Good: minimal example appears immediately after the hook.

### Scannability
Prefer short paragraphs, tables/grids for feature lists, and bold key terms.
Severity: ADVISORY; BLOCKING only for egregious landing-page walls of text.
Bad: dense paragraph lists five features.
Good: feature grid or bullet list.

### Peer points as bullets
Three or more parallel explanatory points should be a list.
Severity: ADVISORY.
Bad: reasons A, B, and C as inline clauses.
Good: bullets for A, B, and C.

### Bullet spacing
Use a blank line before the first bullet after prose and between multi-line bullet items.
Severity: ADVISORY.
Do not flag: compact single-line enum or flag lists.

### Progressive complexity
Content order should be: one-line what → minimal example → common usage → configuration → advanced patterns → edge cases.
Severity: BLOCKING when advanced material appears before basics.
Bad: advanced configuration appears before any minimal example.
Good: basic example appears before configuration and edge cases.

### No fluff
Block zero-information text: `welcome to`, `made with love`, generic `Contributions Welcome` without steps, purposeless emoji.
Bad: `Welcome to our amazing project, made with love!`
Good: `Install the CLI and run your first command.`

### Quick start feasibility
Quick starts should be ≤3 steps, copy-pasteable, and reach running code within 30 seconds of reading.
Severity: BLOCKING for quick-start sections.
Bad: quick start has six conceptual steps before first command.
Good: three copy-pasteable steps reach running code.

### Engagement exclusions
API reference pages are exempt from hook-first, progressive complexity, and quick start. Changelogs and migration guides are exempt from progressive complexity.
