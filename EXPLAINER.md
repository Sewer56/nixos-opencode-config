# Architecture and rationale

Configuration uses selected context, one owner per decision, deterministic evidence, bounded review, and exact commit boundaries. [README][readme] covers commands; [Iterate guide][iterate-guide] covers instruction edits.

## Architecture

```text
request
  -> repository-grounded behavioral draft
  -> independent draft review
  -> human approval
  -> dependency-ordered cohorts
  -> one cohort owner per cohort:
       code -> checks -> correctness + quality -> optional specialists
            -> finding verifier -> bounded repair -> exact commit
  -> full validation + cumulative integration review
```

Approved draft owns behavior. [Implementation orchestrator][implement] creates cohorts and calls [cohort agent][cohort] exactly once per cohort. Cohort agent owns writing, checks, review, repair, and commit, which keeps local failures in one context while reviewers remain read-only and independent.

## Context selection

Agents begin from requested behavior and exact targets, map direct producers and consumers, inspect one dependency hop, and expand only when a call, import, schema, manifest, migration, test, trace, or trust-boundary clue can change decision. They pass paths and evidence references instead of copied repositories or transcripts.

This aims to improve relevant context density. [Repoformer][repoformer] reports that selective retrieval brought “as much as 70% inference speedup ... without harming the performance” on repository-level completion benchmarks.[^repoformer-scope] [Lost in the Middle][lost-middle] found performance “significantly degrades” when relevant information sits in middle of long contexts. [CodeRabbit path instructions][coderabbit-paths] similarly document that excluding irrelevant files “keeps reviews focused and fast.” These sources support selection, not this repository's exact one-hop rule.

[Draft explorer][draft-explorer] builds impact map. [Cohort planner][create-cohorts] groups source, tests, and required documentation by observable outcome and orders groups by repository dependency. Cohort structure is local design choice, not a claimed reproduction of external planning research.

## Implementation and commits

[Cohort loop][cohort] stages only cohort paths in real Git index and runs repository-native quick checks. Correctness and quality always review exact proposed commit. Tests, security, and performance live under [optional review subtree][optional-reviews] and run only when change triggers their domain.

Candidate findings from internal reviewers go to [shared verifier][review-verifier]. Accepted blockers may enter repair; advisories remain visible and do not trigger edits. Every selected reviewer must complete before commit. [Commit agent][commit-agent] commits only approved staged paths while preserving unrelated staged and unstaged changes.

Final gate runs full validation and reviews cumulative base-to-final implementation. A final repair has two identities:

- cumulative base-to-final diff for integration review;
- staged repair diff for correctness, quality, and commit.

This matters because separately valid changes can compose badly. [MOSAIC-Bench][mosaic-bench] reports 53–86% exploit success across 199 three-ticket chains and about 25% neutral-review evasion on confirmed-vulnerable cumulative diffs; pentester framing reduced evasion to 3.0–17.6% in evaluated subset.[^mosaic-preprint] Local workflow therefore reviews composition and routes cross-cohort security risk explicitly.

Implementation is only workflow that treats dirty target as ambiguous ownership and commits automatically. Documentation/refactor workflows edit named current contents without staging or committing, so repeated passes on already modified files remain valid.

## Deterministic evidence

Writers run format checks, parsers, type/build checks, and targeted tests before semantic review. Full validation follows cohort commits. Executed product failures enter bounded repair; unavailable tools, services, credentials, or fixtures produce `INCOMPLETE`.

This separates observations from model claims. [Agentless][agentless] describes “a simplistic three-phase process of localization, repair, and patch validation” and reports 32.0% resolution at $0.70 average cost on SWE-bench Lite with its evaluated setup.[^agentless-scope] Local workflow uses same broad ordering, not Agentless patch sampling or benchmark configuration.

Execution evidence includes command, working directory, result, exit code, and decisive output. It does not manufacture screenshots or logs. [Greptile's TREX article][greptile-trex] states, “Bad evidence is worse than no evidence,” and backs findings with scripts, logs, traces, and screenshots in disposable sandboxes. This configuration has no disposable sandbox, so it runs only authorized repository-native checks and reports unavailable proof as `INCOMPLETE`.

## Finding quality

[Review-finding rules][review-findings] require violated contract, exact location, reachable failure path, material impact, and falsifiable verification for internal candidate review. Severity, confidence, reviewer count, and repetition are metadata—not proof.

[Refute-or-Promote][refute-promote] reports killing about 79% of roughly 171 candidates; its clearest failure had ten dedicated reviewers unanimously endorse nonexistent vulnerability that one empirical test rejected.[^refute-preprint] This supports refutation and deterministic evidence over reviewer voting, while remaining single-operator 2026 preprint evidence rather than controlled proof of this workflow.

Signal budget also matters. [Greptile reports][greptile-filtering] its own comments were 19% useful, 2% incorrect, and 79% nits; team-feedback filtering raised reported address rate from 19% to 55%+ in two weeks.[^greptile-vendor] Local reviewers cap low-value output, keep micro-optimizations advisory unless material evidence exists, and never auto-apply advisories.

Reviewers inspect code and repository evidence rather than trusting surrounding prose. [Sevra-Bench][sevra-bench] holds vulnerable diff fixed across 15 social-engineering framings and reports review agents “are susceptible to narrative manipulation” on retained challenge set of 1,062 adversarial PRs.[^sevra-preprint] Plans define approved intent; PR text, comments, summaries, and tool narration remain claims to check.

## CodeRabbit

`/review/coderabbit` invokes official structured CLI mode for `all`, `committed`, or `uncommitted` scope, requires successful terminal completion, stores exact diff identity, repairs blocking findings, validates repairs, and permits one re-review. CodeRabbit is external review authority, so its findings do not pass through local verifier.

[CodeRabbit CLI reference][coderabbit-cli] defines review types and base comparison. [CodeRabbit path instructions][coderabbit-paths] say targeted instructions “work best as a targeted supplement, not a replacement,” matching local path-scoped rule approach.

## Git boundaries

Implementation assumes one repository writer:

- record base commit and unrelated changed paths;
- reject dirty planned targets because ownership is ambiguous;
- stage only workflow-owned paths;
- review exact cached diff;
- pass reviewed staged paths to commit agent;
- commit explicit paths only;
- verify committed bytes and unchanged unrelated state.

Global `external_directory` policy is `allow`. Agent prompts use deny-by-default tool maps and inherit global policy.

## Instruction authoring and iterate

[Instruction standard][instruction-standard] chooses smallest mechanism:

| Need | Mechanism |
|---|---|
| User entry point | Thin command routing `$ARGUMENTS` to one owner. |
| Distinct privilege, context, or independent judgment | Agent. |
| Reusable on-demand guidance | Skill. |
| Shared runtime policy | Rule group. |
| Mechanical invariant | Existing script, test, or direct check. |
| Usage and rationale | Human documentation. |

Runtime prompts state trigger, objective, inputs, authority, decisions, stop conditions, evidence, failure behavior, and exact output. They pass paths rather than pasted context, use `[[placeholder]]`, and request evidence instead of private reasoning transcripts.

`/iterate/edit` uses one compact flow:

```text
contract -> one editor -> exact staging -> validator/tests
         -> focused reviewer -> verifier -> at most two repairs
```

[Iterate orchestrator][iterate-agent] runs validator and workflow tests for self-edits and requests architecture/adversarial review when contract requires those lenses.

## Design boundaries

- Context expands from concrete dependency evidence rather than exhaustive graph or embedding retrieval. [Repoformer][repoformer] and [Lost in the Middle][lost-middle] motivate selectivity; they do not prescribe local thresholds.
- No learned reviewer memory feeds automatic decisions. [Greptile's reported feedback-clustering gain][greptile-filtering] depends on team votes; this configuration has no governed feedback corpus.[^greptile-vendor]
- No fixed context percentage, reviewer vote, or finding quota determines correctness. [Refute-or-Promote's][refute-promote] unanimous false positive shows why agreement alone is weak evidence.[^refute-preprint]
- Advisories never enter automatic repair. [Greptile's reported 79% nit share][greptile-filtering] illustrates cost of treating every comment as action.[^greptile-vendor]
- Repair loops are bounded: five turns per cohort, two at final integration, two in iterate, and one CodeRabbit re-review.
- Runtime execution stays within available repository environment. [Greptile describes][greptile-trex] each TREX review using “a disposable sandboxed environment”; local workflow does not claim equivalent isolation.
- Implementation uses real Git index and one-writer assumption. Each invocation starts from approved draft and creates fresh evidence artifacts.

## Validation

[Configuration validator][validator] documents its checks in its module docstring and checks parseability, frontmatter, routes, reachability, task depth, permissions, imports, rule reachability, Markdown structure, local documentation links, Python/shell syntax, and required global options. [Workflow tests][workflow-tests] cover implementation-specific contracts.

[^repoformer-scope]: Repoformer evaluates repository-level code completion, not issue-resolution workflow.
[^agentless-scope]: Agentless result comes from its 2024 SWE-bench Lite experiment and does not estimate this configuration's success rate.
[^refute-preprint]: Refute-or-Promote is arXiv preprint and retrospective field study; authors report no autonomous vulnerability discovery and no component ablation.
[^greptile-vendor]: Greptile numbers are vendor-reported internal metrics, not independent evaluation.
[^mosaic-preprint]: MOSAIC-Bench is 2026 arXiv preprint on oracle-backed synthetic chains.
[^sevra-preprint]: Sevra-Bench is 2026 arXiv preprint built from reversed historical vulnerability fixes.

[readme]: README.md
[iterate-guide]: .opencode/ITERATE.md
[implement]: config/agent/_implement.md
[cohort]: config/agent/_implement/cohort.md
[optional-reviews]: config/agent/_implement/cohort/review/optional/
[review-verifier]: config/agent/_review/verifier.md
[commit-agent]: config/agent/commit.md
[draft-explorer]: config/agent/_plan/draft/explorer.md
[create-cohorts]: config/agent/_implement/create-cohorts.md
[review-findings]: config/rules/groups/implementation/review-findings.md
[instruction-standard]: .opencode/agent/_iterate/rules/instruction-authoring.md
[iterate-agent]: .opencode/agent/_iterate/edit.md
[validator]: scripts/validate-opencode-config.py
[workflow-tests]: tests/test_implement_workflow.py
[repoformer]: https://proceedings.mlr.press/v235/wu24a.html
[lost-middle]: https://aclanthology.org/2024.tacl-1.9/
[agentless]: https://arxiv.org/abs/2407.01489
[refute-promote]: https://arxiv.org/abs/2604.19049
[mosaic-bench]: https://arxiv.org/abs/2605.03952
[sevra-bench]: https://arxiv.org/abs/2606.13757
[greptile-filtering]: https://www.greptile.com/blog/make-llms-shut-up
[greptile-trex]: https://www.greptile.com/blog/trex-code-execution
[coderabbit-cli]: https://docs.coderabbit.ai/cli/reference
[coderabbit-paths]: https://docs.coderabbit.ai/configuration/path-instructions
