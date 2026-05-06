### Broken internal links
Block when one target page links to a heading that another target page removes or renames.
Bad: page A links to `#install`, but page B renames it to `#installation`.
Good: link target is updated or stable anchor preserved.

### Cross-artifact terminology drift
Flag different terms for the same concept across target artifacts.
Severity: ADVISORY unless ambiguity blocks action.
Bad: one page says `workspace`, another says `project root` for the same value.
Good: choose one term or define the distinction.

### Content duplication
Flag verbatim or near-verbatim explanations across artifacts when a cross-reference would serve readers better.
Severity: ADVISORY.
Do not flag: brief inline reminders that improve local comprehension.

### Orphaned references
Flag a target artifact referencing a concept, feature, or page that no target artifact explains and that is not a known external resource.
Severity: ADVISORY.
Bad: `Use profiles` with no page or section explaining profiles.
Good: add explanation or link to the owning page.

### Consistency exclusions
Skip cross-artifact checks for single-file scope. Do not review API reference pages or changelogs for cross-page consistency.
