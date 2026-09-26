# Documentation writing

Write only what the reader needs for correct use or safe maintenance.
Leave docs unchanged when they already fit the reader's task.

## Match the reader's task

Assume stated prerequisites, not familiarity with this implementation.
Classify by reader and purpose, not file extension.

- API reference: caller-visible contracts needed for correct use.
- Maintainer docs: internals needed for safe changes and operation.
- Guides: prerequisites, actions, expected outcomes and recovery.

Caller-facing docs describe constraints, not implementation details.

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
