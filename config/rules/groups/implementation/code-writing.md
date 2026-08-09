## RULE GROUP: IMPLEMENTATION / CODE WRITING
Read: approved plan, compiled cohort or final failures, changed/referenced files, direct consumers, and applicable path instructions. Repo search: evidence-triggered only.
Owns: minimal code changes, repository conventions, naming, tests, comments/docs/errors, placement, performance, security, and unrelated-change avoidance.

### Lint gate

Before reviewer or parent validation handoff, run from `PATH`:
`rust-llm-tidy`

Auto mode checks repository-wide tracked staged and unstaged `.rs`/`.md` changes. It may include unrelated tracked changes; untracked files are excluded until staged. No eligible tracked changes is a successful skip. Non-zero blocks handoff; repair and rerun within the caller's bounded writer loop, returning its failure status on exhaustion.

### Writer gate

Before staging: committed code/comments/tests/docs/commit messages never cite internal ids (`AC-1`) — apply the 'Self-contained committed content' rule.

{{ file="./rules/groups/quality/general.md" }}

{{ file="./rules/groups/tests/test-strategy.md" }}

{{ file="./rules/groups/docs/code-docs.md" }}

{{ file="./rules/groups/docs/error-docs.md" }}

{{ file="./rules/groups/quality/placement.md" }}

{{ file="./rules/groups/performance/performance.md" }}

{{ file="./rules/groups/security/security.md" }}
