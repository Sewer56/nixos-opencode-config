{{ file="./rules/review/reporting.md" }}

`<review-inputs>`:
- authority_paths; authorized review scope and exclusions.
- scope: TASK:[[ID]]|FINAL|STANDALONE.
- boundary: STAGED|WORKTREE|COMMITTED; base_commit, head_commit.
- Repo-relative paths; cwd; current validation_path.
- prior_verdict_paths[]; assigned review_path.
- Local candidate/verdict schema: justified-v1; assigned round.
