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
Keep required/consequential frequency details, like error conditions.

### Documentation concision

Review necessity and accuracy across code and non-code documentation.
For unnecessary content, propose deletion before rewriting.

Identify the removable passage and existing coverage that makes it redundant.
Otherwise explain why it serves no relevant reader purpose.
Truth or possible usefulness cannot rebut redundancy.

Block material clutter from repetition or detail obscuring tasks or contracts.
This overrides advisory-only criteria for such documentation findings.
Minor wording improvements remain advisory; omit synonym nits.
