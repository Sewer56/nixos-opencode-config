{{ if=shared!=quality }}
{{ file="./rules/groups/style/wording.md" }}

{{ file="./rules/cards/style/adhd-format.md" }}

{{ file="./rules/cards/docs/error-documentation.md" }}

{{ file="./rules/groups/docs/end-user-correctness.md" }}
{{ endif }}

Delete irrelevant implementation detail before shortening.
True detail can be irrelevant to action, decisions or recovery.

Module/file summaries describe organization, not implementations.
Keep traversal differences affecting maintainer decisions.
Leave useful text; impose no quotas or length gates.

Preserve contracts, meaningful exceptions and complete reachable API errors.
Keep identifiers, commands, links, safety wording and frozen regions exact.
Preserve source delimiters, indentation, directives and doctest behavior.

Harmful inaccuracies, unsafe guidance and missing required contracts can block.
Concise replacements are advisory; omit synonym nits.
Keep required/consequential frequency details, like error conditions.
