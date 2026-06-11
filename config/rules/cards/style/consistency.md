### Broken internal links
Block when one target page links to a heading that another target page removes or renames.
Pass when the link target is updated or a stable anchor remains.

### Cross-artifact terminology drift
Flag different terms for the same concept across target artifacts.
Severity: ADVISORY unless ambiguity blocks action.
Fix by choosing one term or defining the distinction.

### Content duplication
Flag verbatim or near-verbatim explanations across artifacts when a cross-reference would serve readers better.
Severity: ADVISORY.
Do not flag: brief inline reminders that improve local comprehension.

### Orphaned references
Flag a target artifact referencing a concept, feature, or page that no target artifact explains and that is not a known external resource.
Severity: ADVISORY.
Fix by adding an explanation or linking to the owning page.

### Consistency exclusions
Skip cross-artifact checks for single-file scope. Do not review API reference pages or changelogs for cross-page consistency.
