{{ file="./rules/groups/quality/general.md" }}

{{ file="./rules/groups/docs/code-docs.md" }}

{{ file="./rules/groups/docs/error-docs.md" }}

{{ file="./rules/groups/docs/end-user-correctness.md" }}

{{ file="./rules/groups/style/readability.md" }}

{{ file="./rules/groups/style/wording.md" }}

## Documentation and editorial

Apply code-body layout only to scoped code changes, not editorial findings.

Module/file summaries describe organization, not implementations.
Keep traversal differences affecting maintainer decisions.

Preserve contracts, meaningful exceptions and complete reachable API errors.
Preserve source delimiters, indentation, directives and doctest behavior.

Harmful inaccuracies, unsafe guidance and missing required contracts can block.
Concise replacements are advisory; omit synonym nits.
Keep required/consequential frequency details, like error conditions.
