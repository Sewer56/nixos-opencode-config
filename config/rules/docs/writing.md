# Documentation writing

Write only what the reader needs for correct use or safe maintenance.
Leave docs unchanged when they already fit the reader's task.

## Match the reader's task

Assume stated prerequisites, not familiarity with this implementation.
Classify by reader and purpose, not file extension.

- API reference: caller-visible contracts needed for correct use.
- Maintainer docs: internals needed for safe changes and operation.
- Guides: prerequisites, actions, expected outcomes and recovery.

## Structure source docs

In comments, start a sentence mid-line only if it ends on that line.

Open API docs with the item's purpose and key contract.
Put exact rules, edge cases and needed internals in a final Remarks section.
Put error cases under Returns or Errors, not Remarks.

## Separate user docs from implementation work

User-facing docs describe public behavior, not development progress.

Omit unsolicited internal wiring, refactor notes and migration markers.
Omit TODOs, implementation status and pending-work lists unless requested.

Describe user-visible limitations as current behavior, not unfinished work.
Keep progress and remaining work in task artifacts or handoffs.

Include internals only when requested or needed for correct use.

## Keep coverage minimal

Add docs only for unmet reader needs or explicit project requirements.
Update affected docs in place.

Omit boilerplate and repetition of signatures or obvious behavior.
Preserve required sections, contracts, warnings and requested explanations.

Never invent behavior, errors or examples to fill a section.

## Use human, simple wording

Use ordinary words, concrete nouns and direct verbs.
Describe what happens before naming abstractions.

Keep technical terms when they add precision; explain unfamiliar ones.

Unpack dense phrases into natural sentences, not clipped fragments.
Remove filler, repetition and irrelevant detail.

Link existing coverage instead of repeating it.
Keep unrequested patch history in commits/PRs.

Prefer easy reading over fewer words.
Sound natural, not chatty or artificially informal.
Keep established terms consistent.

## Use examples only to resolve reader-relevant ambiguity

Reuse a coherent scenario without repeating explanations.
Use diagrams when they clarify relationships better than text alone.

Use verified behavior, real APIs and hermetic fixtures for runnable examples.

Keep useful examples despite their length.

{{ file="./rules/adhd-communication.md" }}
