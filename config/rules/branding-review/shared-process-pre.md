1. Load cache
- Derive cache path from `handoff_path`: replace the `.handoff.md` suffix with `.review-<domain>.md`. Read the cache file if it exists. Treat missing or malformed cache as empty.
- Treat the cache as one record per candidate name or brand element with fields `last_decision`, `open_findings`, `evidence`, and `verified`.

2. Read handoff
- Read `## Delta` for change tracking.
- Read `### Decisions` only when non-empty.
- When the reviewer's Focus includes search-findings references (distinctiveness, availability): also read the search findings section for external data.

3. Select in-scope content
- Carry forward Verified entries that are Unchanged in Delta.
- Re-evaluate Changed and New entries.
- Re-evaluate own Open entries from cache and decision-referenced entries.
