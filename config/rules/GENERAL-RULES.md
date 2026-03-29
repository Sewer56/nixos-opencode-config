# General Rules

Use these rules for planning, implementation, and review unless a more specific rules file overrides them.

- Keep changes minimal.
- Prefer plain code and names; avoid jargon and cleverness.
- Write the least new code that fully satisfies the requirement.
- Reuse existing patterns.
- Keep visibility minimal.
- Within files, order declarations from most public to most private; within each visibility level, define callers before callees (reading order).
- Avoid broad refactors unless required or requested.
- Avoid dead code, debug/dev-only logging, and unnecessary abstractions.
