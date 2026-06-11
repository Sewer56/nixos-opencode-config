### Hook-first content
First 50 words should answer what this is, why it is different, and who it is for.
Severity: BLOCKING for landing/index pages; ADVISORY for inner reference pages.
Block openings that start with history, welcome text, or implementation detail on landing/index pages.

### Hook-first length
The hook should fit in roughly 50 words or 3 short sentences.
Severity: ADVISORY.

### Show-don't-tell
Getting-started and guide pages need a concrete example, command, terminal output, or visual within the first screenful.
Severity: BLOCKING for guides; ADVISORY for reference pages.

### Scannability
Prefer short paragraphs, tables/grids for feature lists, and bold key terms.
Severity: ADVISORY; BLOCKING only for egregious landing-page walls of text.

### Peer points as bullets
Three or more parallel explanatory points should be a list.
Severity: ADVISORY.

### Bullet spacing
Use a blank line before the first bullet after prose and between multi-line bullet items.
Severity: ADVISORY.
Do not flag: compact single-line enum or flag lists.

### Progressive complexity
Content order should be: one-line what → minimal example → common usage → configuration → advanced patterns → edge cases.
Severity: BLOCKING when advanced material appears before basics.

### No fluff
Block zero-information text: `welcome to`, `made with love`, generic `Contributions Welcome` without steps, purposeless emoji.

### Quick start feasibility
Quick starts should be ≤3 steps, copy-pasteable, and reach running code within 30 seconds of reading.
Severity: BLOCKING for quick-start sections.

### Engagement exclusions
API reference pages are exempt from hook-first, progressive complexity, and quick start. Changelogs and migration guides are exempt from progressive complexity.
