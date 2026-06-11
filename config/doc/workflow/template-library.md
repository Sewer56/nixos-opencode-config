# Template Library

Reusable fragments, `.txt` includes, and schema snippets for `config/agent/`, `config/command/`, and `config/rules/`.

## Rules

1. Use a fragment only when two or more consumers share the same shape. Inline one-consumer rules.
2. Fragment owns reusable structure; caller owns domain-specific goal, paths, criteria, args, and validation.
3. Prefer imports over copied boilerplate.
4. Keep fragments generic and parameterized. Do not hide target-specific instructions in shared templates.
5. Use `[[placeholder]]` for human placeholders inside examples/schemas; reserve `<tag>` for actual XML sections.
6. Validate changed imports/args/conditionals with the renderer when available.

## Include Shape

```markdown
{{ file="./path/to/fragment.txt" }}
```

Parameterized:
```markdown
{{
  file="../config/agent/_templates/review-mission.txt"
  artifact_type="prompt"
  domain="integrity"
}}
```

Args apply to one include. Forward nested args explicitly. Omit false flags; use `1` for true.

## Fragment Groups

### Review lifecycle (`agent/_templates/`)

- `review-mission.txt`: one-line domain mission.
- `review-process/cached.txt`: cached reviewer script; caller supplies delta source and extra reads.
- `review-process/cacheless.txt`: fresh final-gate reviewer script.
- `review-cache-table.txt`: cache record shape.
- `review-footer/cacheless.txt`: inline cacheless output schema.
- `review-finding.txt`: normalized finding shape for actions/cache/inline review output.

Use these for reviewers with stable domains and parseable outputs. Do not paste full reviewer protocol into callers; pass path/id/flag inputs.

### Iterate edit (`agent/_iterate/`)

- `rules/prompt-optimization-contract.txt`: agent-facing prompt optimization baseline imported by `/iterate/edit` and prompt-quality reviewer.
- `rules/iterate-edit-vocabulary.txt`: shared labels for prep, pattern selection, review.
- `rules/edit-log-shape.txt`: fixed edit ledger shape.
- `rules/split-decision-rule.txt`: split/merge/subagent topology decision rules.
- `rules/renderer-syntax.txt`: live renderer syntax reference.
- `rules/renderer-template-use-checks.txt`: renderer import/arg checks.
- `edit-reviewers/_templates/*.txt`: reviewer domain bodies shared by cached/cacheless wrappers.

### Workflow docs (`config/doc/workflow/`)

- `prompt-engineering.md`: prompt quality baseline and harness boundary.
- `design-patterns.md`: OPT topology patterns.
- `optimize-patterns.md`: WOPT existing-workflow optimization tactics.
- `optimize-maintenance.md`: optimizer workflow architecture notes.
- `unproven-patterns.md`: intake for ideas not ready for OPT/WOPT.

## Anti-patterns

- One-consumer rule extracted into a file, then imported once.
- Caller repeats callee role/process/schema around an import.
- Fragment arguments contain paragraphs of target-specific policy.
- Shared template includes provider/model/harness config that belongs in frontmatter/config.
- Placeholder syntax uses angle-bracket slots and looks like XML.
