### ADHD-aware comments and docs
Shape comments and docs for an ADHD reader.

Wording, code-documentation, error-documentation, and accuracy win conflicts.

- Open with the point or next action, not an intro; context follows.
- Headers: one capability line plus one-fact bullets; never narrative.
- Use the fewest numbered steps, one action each; no double "and then".
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
- Stop when done; no outro.
- No em dashes; use colons or periods.
- Full-explain requests, destructive actions, real ambiguity, harness or
  accuracy rules override shape; keep the lead, drop closers.
