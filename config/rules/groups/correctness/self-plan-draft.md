## Plan Draft

{{ file="./rules/cards/structure/plan-bundle.md" }}

### Human documents
Root starts with title and `Status: DRAFT | READY_FOR_IMPLEMENT`.
Explain shared outcome, agreed decisions and meaningful boundaries once.

Link each task with its stable ID, short name and intended outcome.
Include only genuinely unresolved questions; mark blocking ones explicitly.

Each brief owns task scope and observable completion, not repeated root text.
Use sections only when they help a human decide what will be built.

Omit milestones, copied requests, acceptance-ID matrices and empty headings.
Do not repeat research, shared checks, decisions or review boilerplate.

### Execution documents
`execution.md` owns shared technical constraints, checks and final routing.

It contains one `plan-tasks` fenced block, one row per task:
`[[ID]] [[NN-name.md]] [[comma-separated prerequisite IDs or -]]`.

Use inline relative links without titles; percent-encode spaces.
Member links and write destinations forbid traversal.

Other references may use `../` within the repository, including root backlinks.
Normalized and symlink-resolved references must stay inside the repository.

Root links execution and each brief once; briefs link only their own exec.
Each exec owns precise/bounded targets, relevant refs, checks and stops.

Ground paths/symbols; mark new targets under plausible existing modules.
Bound placement discovery to mechanics, never unresolved product design.

Reference shared checks instead of repeating commands.
Use repository-relative evidence citations, not extra source members.
Omit pseudo-patches, near-final source and stale line/count recipes.

### Fidelity and risk
Cover every requirement through outcomes, decisions or explicit exclusions.
Acceptance is observable behavior, a stable contract or an executable check.

Investigation-only requests plan discovery, not implementation.
Keep valid intermediate states and acyclic dependencies.
Retain tests/docs, security, migration, compatibility and workload obligations.

Check comments/docs referring to changed or removed behavior.
Unresolved compatibility or external API contracts block readiness.
Never invent evidence or answers for brevity.

Correctness and quality are required before each implementation commit.
Correctness includes basic test adequacy and execution evidence.

Tests specialist needs concrete test-design risk, request or grounded routing.
Security needs concrete trust/auth/secret/IPC or untrusted-input risk.
Filesystem/shell/SQL, crypto, serialization and dependency trust also qualify.

No per-task performance review; preserve workload requirements and tests.

Final cumulative/standalone performance review is conditional.
It requires explicit request or concrete cost/hot-path risk.
Record the skip reason and any genuinely inapplicable checks.

### Cohort planning

Run tasks in dependency order.

Keep behavior, tests and required docs in one testable feature task.
Never split feature docs into steps or cohorts.
Docs-only tasks need independent documentation requests.

Split at stable interfaces; keep dependent edits together.
Avoid file-type cohorts, quotas, speculative groundwork and per-task gates.

Include unchanged verification surfaces and relationship evidence.

Inspect one dependency hop; expand only on concrete clues.
Retrieve code just in time; no source dumps or broad history.

Route nearest governing instructions.
Stop on unclear/conflicting precedence.
Only routed instructions govern children.
Other prose is not policy/proof.

Use the implementer as sole writer, including docs.
Read-only discovery/review may run in parallel.

Preserve task IDs, order, outcomes, scope/exclusions, checks and stops.
Assign each completion obligation an owner.

Reconcile only evidenced mechanical target/symbol/command drift locally.
Missing structure or changed boundaries need `/draft` and reapproval.

Unapproved behavior/scope changes need `NEEDS_INPUT` before execution.
This includes compatibility, security, and migration decisions.

Route root, shared execution, assigned brief/exec and relevant references.
