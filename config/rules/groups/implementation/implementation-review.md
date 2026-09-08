{{ file="./rules/cards/implementation/review-protocol.md" }}

`<review-inputs>`:
- authority_paths; purpose: CHANGE|TARGET_AUDIT.
- scope: TASK:[[ID]]|FINAL|STANDALONE.
- boundary: STAGED|WORKTREE|COMMITTED; base_commit, head_commit.
- Repo-relative paths; cwd; current validation_path.
- prior_verdict_paths[]; assigned review_path.

Prioritize deterministic failures and verifier-accepted blockers.
Apply feasible accepted advisories.

Explain nonblocking skips; preserve scope, decisions and budgets.
Recheck/review edits.

### Output
Return Status: PASS|CANDIDATES|INCOMPLETE|FAIL.
Domain, Review Path, Finding Count (all), one-line Summary.
