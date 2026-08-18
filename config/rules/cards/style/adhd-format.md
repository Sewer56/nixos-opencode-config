### ADHD-aware comments and docs
Shape comments and documentation for an ADHD reader: answer first, one action per step, no filler. Defers to the wording card (sentence economy, fillers and closers), the code-documentation card (purpose-first, current-behavior-only, release-note back-compat exception), and the error-documentation card (variant completeness). Those rules and task or accuracy requirements win on any conflict.

- **Answer first**: open comments and docs with the point or next action; context follows.
- **Header shape**: file, module, and example headers are one capability
  line plus bullets, one fact each. Never open with a narrative paragraph.
- **One action per step**: numbered steps, fewest steps, no double "and then".
- **State markers**: name the resulting state where identifiers leave intent unclear, like `// After this line: ...`; never on trivial code.
- **End with next**: docs end with `Next:` or a checkable `Done when:`; API docs surface errors and returns last.
- **Cause then fix**: errors read condition, cause, fix in one line, like `Error: condition. Fix: action.`
- **Concrete units**: `~2 min`, never "a bit"; in comments only for non-trivial work.
- **Short prose blocks**: target 80 characters per line and 240 per paragraph.
  Start a new paragraph when the idea changes. Exempt code, URLs, tables,
  headings, signatures, and exact required text. Clarity and valid formatting win.
- **Cap lists at 5**: split optional lists into must vs nice-to-have or do now vs later. Required coverage wins: error variants, parameters, features, and execution steps may exceed 5 when omission loses required detail.
- **No intro or outro**: no "In this guide", no "Hope this helps", no "let me know if". Start at the answer; stop when done.
- **No em dashes**: no em-dashes in comments or docs; use colons or periods instead.
- **Shape never wins**: full-explain requests, destructive actions, real ambiguity, and harness, task, accuracy, or fidelity rules override shape. Keep the compact lead; drop closers.
