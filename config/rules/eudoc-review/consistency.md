### Broken internal links
Block when one target page links to a heading that another target page removes or renames.

Bad: page A links to `#install`, but page B renames it to `#installation`.
Good: link target is updated or stable anchor preserved.

### Terminology drift
Flag different terms for the same concept across target pages. ADVISORY.

Bad: one page says `workspace`, another says `project root` for the same value.
Good: choose one term or define the distinction.

### Content duplication
Flag verbatim or near-verbatim explanations across pages when a cross-page link would serve readers better. ADVISORY.

Do not flag: brief inline reminders that improve local comprehension.

### Orphaned references
Flag a target page referencing a concept, feature, or page that no target page explains and that is not a known external resource. ADVISORY.

Bad: `Use profiles` with no page or section explaining profiles.
Good: add explanation or link to the owning page.

### Review Blocking Criteria
- Block for broken internal links between target pages.
- Do not block for terminology drift, content duplication, or orphaned references — ADVISORY only.

### Exclusions
Skip entirely for single-file scope. Do not review API reference pages or changelogs for cross-page consistency.
