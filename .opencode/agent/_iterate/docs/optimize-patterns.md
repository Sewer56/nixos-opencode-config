# Prompt Workflow Optimization Tactics

Use these tactics when `/iterate/edit` is asked to reduce or modernize command/agent/reviewer workflows.

## WOPT-001 - Remove Standing Committee

Replace always-on reviewer swarms with profile-gated reviewers. Keep full fanout for self-iteration and high-risk changes.

## WOPT-002 - Scriptable Selector

Replace LLM pattern-selection subagents with deterministic contract compilers when inputs are path/profile/rule based. Keep LLM fallback only for ambiguous semantic routing.

## WOPT-003 - Prompt/Harness Scrub

Move model names, effort parameters, tool schemas, provider reasoning replay, permission tables, sandboxing, egress, and prompt-cache internals out of runtime prompt bodies unless the prompt is specifically editing harness/config docs.

## WOPT-004 - Sparse XML

Use XML tags for major mixed-content boundaries: `agent_contract`, `inputs`, `workflow`, `constraints`, `output_contract`, `verification`, `documents`, `examples`. Do not XML-wrap every bullet.

## WOPT-005 - Token Report

Generate a token/word/char report for changed prompts. Use it to spot prompt bloat, not as the only quality metric.

## WOPT-006 - Self-Iteration Guard

When editing `/iterate/edit`, update runner, editor, reviewers, scripts, docs, and tests if behavior crosses those layers. Preserve future application of the same optimization rules.

## WOPT-007 - Handoff Compression

Pass subagents paths, ids, flags, criteria, and artifact paths. Do not paste complete parent workflows, long catalogs, or examples unless they are necessary input data.

## WOPT-008 - Validation Substitution

If the ideal render/test/check cannot run, require the next-best inspectable substitute and record why it is weaker.
