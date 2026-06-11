# Prompt Workflow Design Patterns

Use these as source material for `scripts/iterate_edit_contract.py`. Runtime agents receive selected ids and compact carry-ins, not this full catalog.

## OPT-001 - Thin Command Router

A command should route, define artifacts, and specify final output. Behavioral complexity belongs in the owning agent or deterministic script. Use when command and agent duplicate policy.

## OPT-002 - Compiled Contract

Convert request, target paths, risk flags, and source docs into one compact contract before editing. The editor reads the contract instead of re-solving pattern selection.

## OPT-003 - Static Before Semantic

Render, import, placeholder, schema, token, and obvious boundary checks are script work. Reviewers should spend tokens on semantic correctness and risk.

## OPT-004 - Risk-Tiered Review

Use reviewer fanout by profile: `micro`, `standard`, `structural`, `self_iterating`, `high_risk`. Full independent review is for self-iteration or high-risk changes, not every wording edit.

## OPT-005 - Single Run Directory

Keep request, prep, contract, log, checks, and reviews under one run directory. This reduces path-passing errors and makes debugging reproducible.

## OPT-006 - Distinct Reviewer Domains

Use reviewers only when their context or judgment is distinct:
- prompt: selected rules, output, verification, density
- integrity: frontmatter, wiring, permissions, imports, source boundaries
- topology: split/merge/template/reviewer architecture
- adversarial: bypass, injection, self-disable, high-risk failure modes

## OPT-007 - One-Consumer Inline

Inline a rule/include used by exactly one runtime prompt. Keep shared includes only when two or more consumers need identical text.

## OPT-008 - Docs/Runtime Split

Keep long research and examples in docs. Runtime prompts import short contracts or selected rules.

## OPT-009 - Positive Scoped Rules

Prefer `if/then` and concrete positive instructions. Reserve absolute prohibitions for invariants such as source-boundary and safety constraints.

## OPT-010 - Evidence-Oriented Completion

Final responses and reviewer outputs should show changed files, checks, evidence, and remaining risks. Avoid plan-only completion when the edit path is safe and available.
