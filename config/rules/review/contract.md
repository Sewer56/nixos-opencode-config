{{ file="./rules/review/reporting.md" }}

`<review-inputs>`:
- authority_paths; authorized targets, comparison and exclusions.
- scope: TASK:[[ID]]|FINAL|STANDALONE.
- boundary: STAGED|WORKTREE|COMMITTED; base_commit, head_commit.
- Repo-relative paths; cwd; current validation_path.
- prior_verdict_paths[]; assigned review_path.
- Assigned round.
