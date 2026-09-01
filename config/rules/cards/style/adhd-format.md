### ADHD-aware comments and docs
Shape comments and docs for an ADHD reader: answer first, one action per
step, no filler.

Defers to wording (sentence economy, fillers/closers), code-documentation
(purpose-first, current-behavior-only, release-note back-compat), and
error-documentation (variant completeness) cards.

Those plus accuracy requirements win on conflicts.

- Open with the point or next action; context follows.
- Headers: one capability line plus one-fact bullets; never narrative.
- Numbered steps, fewest, no double "and then".
- Mark resulting states where intent is unclear (`// After this line:
  ...`), never trivial code.
- End with `Next:` or checkable `Done when:`; API docs put errors and
  returns last.
- Errors: condition, cause, fix in one line (`Error: condition. Fix:
  action.`).
- Concrete units (`~2 min`, not "a bit"), non-trivial work only.
- 80 chars/line soft, never BLOCKING; 240 chars/paragraph, BLOCKING.
  Split over-cap paragraphs at idea changes; exempt code, URLs, tables,
  headings, signatures, exact text.
- No intro or outro; start at the answer, stop when done.
- No em dashes; use colons or periods.
- Full-explain requests, destructive actions, real ambiguity, harness or
  accuracy rules override shape; keep the lead, drop closers.
