# Documentation writing

## Reader and purpose

Write human, simple documentation that is easy to understand.

Assume stated prerequisites, not familiarity with this implementation.
Explain unfamiliar concepts as needed.
Classify by reader and purpose, not file extension.

- API reference explains behavior, inputs, results and constraints.
- Maintainer docs explain non-obvious mechanisms, decisions and invariants.
- Guides explain prerequisites, actions, expected outcomes and recovery.

## Coverage and structure

Document new/changed public items; update affected docs in place.
Document private APIs only if nontrivial.

Preserve required API sections and their language/project equivalents.
Public APIs need Arguments for parameters and Returns for returned values.

Keep entries concise; explain roles, units, constraints and special cases.
List each public API error, its cause and a concise explanation.
Use Remarks for miscellaneous caveats.

Preserve contracts, conditions, guarantees and useful explanations.
Never invent behavior, errors or examples to fill a section.

## Human, simple wording

Write as if explaining the subject to another person.

Use ordinary words, concrete nouns and direct verbs.
Describe what happens before naming abstractions.
Keep technical terms when they add precision; explain unfamiliar ones.

Give the purpose first, then the details readers need.
Unpack dense phrases into natural sentences, not clipped fragments.
Remove filler, repetition and irrelevant detail.

Link existing coverage instead of repeating it.
Keep unrequested patch history in commits/PRs.

Prefer easy reading over fewer words.
Sound natural, not chatty or artificially informal.
Keep established terms consistent rather than varying them for style.
Leave already-clear passages unchanged.

## Examples

Explain complex mechanisms with small concrete examples when useful.
Show what happens, then connect it to concepts and types.

Reuse a coherent scenario without repeating explanations.
Use diagrams when they clarify relationships better than text alone.

Do not add examples to obvious operations to fill a template.
Use verified behavior, real APIs and hermetic fixtures for runnable examples.

Style references, not implementation evidence:

- Simple purpose: "Read, write, and organize files within a storage directory."
- A choice: "Chooses what happens if a file already exists when writing."
- A mechanism: "A calls `0x2000`. The call arrow adds B as a separate function."

The mechanism example shows the event before naming its representation.
Keep useful examples even when they make the documentation longer.

{{ file="./rules/adhd-communication.md" }}
