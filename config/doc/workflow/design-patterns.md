# Workflow Design Patterns

Reusable patterns for creating/refining command, agent, reviewer, and prompt-template workflows. For prompt text quality, first apply `config/doc/workflow/prompt-engineering.md`; this catalog covers workflow topology.

## Use

1. Classify traits and risks before selecting patterns.
2. Select only patterns with a direct behavior or topology match.
3. Convert carry-ins into target-specific instructions; do not paste this catalog into runtime prompts.
4. Prefer a smaller prompt plus stronger artifact contract over long prose.
5. Use `[[placeholder]]` for placeholders in examples/schemas.

## Trait Map

| Trait | Patterns |
| --- | --- |
| command delegates to agent | OPT-001 |
| primary runner + reviewers | OPT-002, OPT-003, OPT-011, OPT-012, OPT-014, OPT-016 |
| review loop | OPT-003, OPT-004, OPT-005, OPT-009, OPT-011, OPT-018, OPT-019 |
| subagent coordination | OPT-002, OPT-006, OPT-012 |
| machine-readable output | OPT-004, OPT-008, OPT-018 |
| diff/action artifacts | OPT-007, OPT-009 |
| failure-path validation | OPT-013 |
| central pattern selection | OPT-015 |
| pipeline decomposition | OPT-017 |
| prompt fragment inclusion | OPT-018 |

## OPT-001 - Thin Command Templates

Apply when a command mainly routes user input. Command body contains frontmatter plus `$ARGUMENTS` or one small routing note. Agent owns role, process, constraints, output, examples, and checks. Do not duplicate agent-owned rules in command text.

## OPT-002 - Tight Subagent Inputs

Caller passes only run data: artifact paths, scoped ids/paths, trigger flags, user notes, changed decisions. Callee owns role, focus, process, output schema, and examples. If every call repeats a rule, move it into callee. Pass content only when callee cannot read the path.

## OPT-003 - Repeated Subagent Cache

Use when the same reviewer/helper may run multiple times for stable inputs. Caller passes `cache_path` and smallest invalidation basis: revision, changed paths, changed ids, delta, or decisions. Callee preserves unchanged verified records, reopens stale/touched records, writes cache before responding.

## OPT-004 - Fixed Structured Output Blocks

Use one exact fenced `text` block for machine-consumed outputs. Field names, order, allowed values, and empty sections stay stable. Use JSON only for explicit JSON consumers.

## OPT-005 - Reference Instead of Requote

Reference shared context by path, heading, item id, or finding id. Quote only the smallest exact snippet. Do not duplicate requirements, deltas, rule catalogs, reviewer outputs, or design docs across artifacts.

## OPT-006 - Shared Context File

When multiple agents need the same stable context, write one context artifact and pass its path. Keep it factual: scope, assumptions, paths, ids, decisions. Do not hide mutable instructions inside shared context.

## OPT-007 - Diff Line Locators

For review/fix actions, cite path plus stable locator: line, heading, item id, or diff hunk. Use smallest patch-like snippet when exact change matters.

## OPT-008 - Nested Code Fence Safety

When output may contain fenced code, define fence strategy. Prefer outer `~~~` around markdown that itself contains triple-backtick code.

## OPT-009 - Reviewer Inline Diffs When Exact

Reviewer findings include exact diff only when the fix must be applied verbatim. Otherwise use concise problem/fix. Keep evidence separate from patch text.

## OPT-010 - Inline Path Variables

Define derived artifact/path variables once near inputs, then reuse variable names. Avoid long repeated paths and ambiguous shorthand.

## OPT-011 - Triggered Reviewer Sets

Route reviewers by changed paths, risk flags, and decisions. Always run required blocking domains; skip unrelated domains. If a fix changes scope or risk, recompute reviewer set.

## OPT-012 - Explicit Reviewer Scope Boundaries

Each reviewer owns one domain, in-scope artifacts, blocking criteria, advisory criteria, and output protocol. Avoid reviewers with overlapping findings unless one is an adjudicator.

## OPT-013 - Fast-Fail Preconditions

Before expensive work, verify required inputs exist and machine artifacts parse. Missing/malformed prerequisite = fail or ask, not best-effort guessing.

## OPT-014 - Per-File Step Scoping

When reviews churn on large artifacts, pass changed file/id slices plus direct references instead of full workspace context. Preserve reviewer access to broader files only when evidence requires it.

## OPT-015 - Central Pattern Selector

For workflows that apply reusable patterns, use one selector to write a compact pattern contract. Runner and reviewers consume that contract; they do not reread full catalogs except selected sections.

## OPT-016 - Adjudicated High-Risk Review

For correctness, security, data-loss, migration, or irreversible-action risk, use independent reviewers plus an adjudicator when findings conflict. Adjudicator resolves current findings only; it does not become another broad reviewer.

## OPT-017 - Pipeline Decomposition

Split pipeline stages when they have distinct inputs, outputs, failure modes, or deterministic checks. Merge stages when handoff cost exceeds saved context or stages read the same material.

## OPT-018 - Prompt Fragment Inclusion

Use template/include fragments for boilerplate with two or more consumers. Fragment owns reusable shape; caller supplies domain, paths, criteria, and validation. Inline one-consumer rules.

## OPT-019 - Paired Substep Loop Grouping

When two phases share one re-dispatch loop, make one top-level step with substeps (`6a`, `6b`) and state trigger direction. Keep independent phases as separate top-level steps.
