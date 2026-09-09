# Architecture and rationale

[README] covers usage; the [Iterate guide] covers instruction edits.

## Architecture

Separate human decisions, execution instructions and evidence.
A readable plan lets people audit what will change without reading tool recipes.

Execution files translate that agreement; they cannot introduce product choices.

### Discussion before documents

The [draft owner] discusses goals, constraints, design and a small task outline.
Explicit agreement and document authorization prevent premature plan writing.

Earlier agreement covers unchanged scope; thanks alone is not approval.

The [explorer] gathers bounded repository evidence for the draft owner.
Discovery follows concrete dependencies; external facts retain version/source.
This keeps research separate from the authority to decide product behavior.

### Human and execution views

- Root: shared outcomes, decisions, boundaries and task links.
- Human brief: task scope and observable completion.
- Shared execution: technical constraints, checks and dependency routing.
- Task exec: bounded targets, references, checks and stops.

Each brief links its exec; the root links shared execution and all briefs.
Shared facts appear once instead of in milestones, matrices or copied requests.
The [bundle rule] defines membership and who reads each view.

The read-only [bundle checker] validates routing, pairs, anchors and paths.
Safe reference backlinks can reach the root or source without adding authority.

Source-member links and prospective writes keep stricter traversal boundaries.
Exact worktree-safe excludes keep plans local without changing `.gitignore`.

Tidy and mechanical checks precede whole-bundle readability/fidelity review.
The [draft verifier] tests candidates, not a second design.
Rejection alone cannot make a plan ready; the latest review must be READY.

Only the new root/shared-execution/task-pair format is supported.
Standalone handoffs and iterate contracts are different artifacts, not adapters.

## Execution and review

The [implementation parent] routes tasks in order without a handoff copy.
It reads root/shared execution and evidence, not every sibling exec file.

Each [task worker] reads human authority and owns writing through commit.
One writer prevents overlap; independent reviewers inspect real changes.

Correctness includes basic test adequacy; quality covers every proposed commit.
Tests and security specialists need grounded risks rather than routine dispatch.

Performance reviews cumulative integration or the complete standalone change.
Task writers still preserve workload requirements and tests.

Local task success does not prove that the whole implementation composes.
Final review therefore covers all human outcomes and cross-task interactions.
Commit calls use immediate HEAD, distinct from the cumulative review base.

The [evidence convention] separates authority, subject and output.
Shared validation avoids rerunning suites in every reviewer.

Compact findings retain consequences and proof; clean reviews skip verification.

The [style verifier] tests QUALITY and EDITORIAL using imported criteria.
The [finding verifier] tests other shared domains without those profiles.

Both independently test evidence, relevance and bounded corrections.
True but irrelevant detail can warrant an advisory; essential caveats stay.

Callers partition by assigned domain and review boundary, not report claims.
Class, boundary and round have distinct verdict paths.

Cumulative editorial never merges with repair-only quality.
Repairs wait for all verdicts, then deduplicate without losing identities.

The total repair budget remains shared across both classes.

CHANGE reviews introduced/exposed defects, not every old problem nearby.
TARGET_AUDIT retains docs/refactor's declared existing-defect scope.

Apply feasible verified advisories by default.
Keep scope, decisions and budgets; skipped advisories remain non-blocking.
Missing required evidence remains INCOMPLETE rather than PASS.

Resume retains original baselines, ownership and consumed repair budgets.
Partial work needs fresh checks/reviews; unclear ownership stops safely.
These boundaries protect unrelated user changes rather than resetting the tree.

## External and interactive workflows

[CodeRabbit] owns verification of its original external findings.
Its resulting edits still re-enter local checks/reviews before scoped commit.

Immutable rounds and bounded re-review preserve evidence of what was checked.

[One-shot] uses one compact handoff for an already-clear bounded request.
[Code] stays interactive, with no review artifacts or delegation by default.
Reviews/commits need explicit requests; later review needs the real baseline.

## Instruction work and validation

The [instruction standard] favors one owner and the smallest useful mechanism.
The [iterate owner] discusses design before creating request or contract files.

Then one editor writes exact frozen targets; only the orchestrator stages.
Self-edits require mechanical checks and architecture/adversarial review.

The [validator] checks syntax, imports, permissions, routes and task depth.
It reports depth mismatches instead of silently rewriting configuration.

[Workflow smoke] exercises routing and path boundaries in temporary fixtures.
Static checks and scenarios are not live-agent execution or usability evidence.

[README]: README.md
[Iterate guide]: .opencode/ITERATE.md
[draft owner]: config/agent/_plan/draft.md
[explorer]: config/agent/_plan/draft/explorer.md
[bundle rule]: config/rules/cards/structure/plan-bundle.md
[bundle checker]: config/scripts/plan-bundle.py
[draft verifier]: config/agent/_plan/draft/verifier.md
[implementation parent]: config/agent/_implement.md
[task worker]: config/agent/_implement/cohort.md
[evidence convention]: config/rules/cards/implementation/review-protocol.md
[finding verifier]: config/agent/_review/verifier.md
[style verifier]: config/agent/_review/style-verifier.md
[CodeRabbit]: config/agent/_review/coderabbit.md
[One-shot]: config/agent/_implement/one-shot.md
[Code]: config/agent/code.md
[instruction standard]: .opencode/rules/instruction-authoring.md
[iterate owner]: .opencode/agent/_iterate/edit.md
[validator]: scripts/validate-opencode-config.py
[Workflow smoke]: scripts/check-workflows.sh
