{{ file="./rules/review/reporting.md" }}

`<review-inputs>`:
- authority_paths; authorized review scope and exclusions.
- scope: TASK:[[ID]]|FINAL|STANDALONE.
- boundary: STAGED|WORKTREE|COMMITTED; base_commit, head_commit.
- Repo-relative paths; cwd; current validation_path.
- prior_verdict_paths[]; assigned review_path.

Prioritize deterministic failures and verifier-accepted blockers.
Apply feasible accepted advisories.

After acceptance, check source for repairs, not to re-adjudicate findings.
Reuse verified edits or follow verified requirements.
Never use rejected or unresolved corrections.

Send contradictory repair evidence back to the assigned verifier.
Delegate reverification of material departures from verified corrections.

Explain nonblocking skips; preserve scope, decisions and budgets.
Recheck/review edits.

### Output
Return Status: PASS|CANDIDATES|INCOMPLETE|FAIL.
Domain, Review Path, Finding Count (all), one-line Summary.
