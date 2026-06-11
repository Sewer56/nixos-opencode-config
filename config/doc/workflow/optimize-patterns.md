# Workflow Optimize Patterns

Approved tactics for reducing existing workflow token cost without losing quality. Use after observing focus signals from export/digest/logs; do not force-fit tactics.

Refs: `WOPT-###` for existing-workflow optimization, `OPT-###` for design shape, `LOCAL:[[name]]` for one-run hypotheses.

## Use

1. Start from observed signals and counterevidence.
2. Select the fewest WOPT/OPT refs that explain the waste.
3. Convert refactor moves into direct edits; do not paste this catalog into prompts.
4. Keep quality guards and rerun representative checks.

## Signal Map

| Signal | Usually Apply |
| --- | --- |
| generated hotspot | WOPT-003, WOPT-004, WOPT-005 |
| overbroad handoff | WOPT-001, WOPT-003, OPT-002 |
| duplicate reads/reasoning | WOPT-001, WOPT-003, OPT-003 |
| review-loop churn | WOPT-001, WOPT-002, WOPT-006 |
| cache/delta failure | WOPT-001, WOPT-005 |
| output bloat | WOPT-005, OPT-004, OPT-005 |
| model/risk mismatch | WOPT-004 |
| prompt/context bloat | WOPT-001, WOPT-003, OPT-018 |

## WOPT-001 - Structural Withholding

Problem: runner gives every child full context/rules, causing duplicate reasoning.

Refactor: parent keeps orchestration; child prompt owns role/process/schema; handoff carries paths, ids, flags, decisions, cache/action paths, and success criteria only. Move repeated child instructions into child prompt or shared multi-consumer include.

Guard: child still has enough access to verify its domain. Do not withhold required source paths or safety constraints.

## WOPT-002 - Review Loop Restructuring

Problem: every fix reruns every reviewer.

Refactor: order high-risk blocking domains before presentation/style domains. After a fix, rerun only touched domains; recompute set when paths, risk flags, or scope changes. Advisory-only findings do not force full-loop reruns unless target workflow requires them.

Guard: final gate requires zero unresolved blocking findings.

## WOPT-003 - Reviewer Topology Shaping

Problem: reviewers overlap, reread same artifacts, or split without context savings.

Refactor: merge reviewers with same read scope and overlapping findings. Split overloaded reviewer only when subdomains are independent and each child receives smaller scoped input. Update caller routing, cache ownership, output parsing, docs, and review ledger.

Guard: do not merge away correctness/security/data-loss ownership. Reject splits where children still read full context.

## WOPT-004 - Risk-Based Reviewer Tiering

Problem: low-risk mechanical checks use high-cost models or high-risk checks use weak ones.

Refactor: assign model tier by domain risk, judgment load, and failure cost. Keep correctness, security, data-loss, migration, and ambiguous semantic review on strong models. Move narrow mechanical checks down only with evidence.

Guard: never downgrade from token cost alone. Keep escalation path when risk flags appear.

## WOPT-005 - Reviewer Action/Cache Split

Problem: reviewer responses carry full evidence/history every loop.

Refactor: cached reviewer response is pointer-only: `Decision`, `Actions`, `Cache`, current IDs. Actions file contains current OPEN fixes. Cache contains durable evidence, statuses, resolved/deferred history, and verified observations. Cacheless reviewers return findings inline and do not use sidecars.

Guard: every current ID exists in response, actions, and cache. Missing/malformed sidecar = protocol failure.

## WOPT-006 - Coupled-Loop Header Pairing

Problem: two phases implicitly re-dispatch each other and cross-references are unclear.

Refactor: group them under one top-level step with named substeps and a preamble stating trigger and re-entry direction. Keep loop caps at the grouped level.

Guard: pair only phases with structural re-dispatch. Independent phases remain separate.
