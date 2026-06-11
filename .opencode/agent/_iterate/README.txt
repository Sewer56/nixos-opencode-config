_iterate/edit — How It Works
===========================

Pipeline for editing OpenCode prompt files (agents, commands, reviewers, docs).
Deterministic prep + contract compilation → editor subagent → static checks →
semantic reviewers → repair loops → done.

Stages
------

1. PREP (iterate_edit_prepare.py)
   - Extracts file paths from user request.
   - Classifies: prompt kind (agent/command/reviewer/docs), risk profile.
   - Discovers direct imports (file-include references to other prompt files).
   - Outputs: prep.json + prep.md.
   - If no targets found → NEEDS_INPUT, asks user.

2. CONTRACT (iterate_edit_contract.py)
   - Selects applicable rules from 3 catalogs:
     PE  = prompt engineering rules (output contract, boundaries, density, ...)
     OPT = design patterns (thin command, static-before-semantic, ...)
     WOPT= optimization tactics (scrub harness, sparse XML, token report, ...)
   - Picks required reviewers by risk profile.
   - Outputs: contract.md (human) + contract.json (machine).

3. EDITOR (editor.md subagent)
   - Reads request + prep + contract.
   - Applies only selected PE/OPT/WOPT rules + explicit user requirements.
   - Directly edits target files (no draft/finalize outside run_dir).
   - Writes edit-log.md using edit-log-shape.txt template.

4. STATIC CHECKS (deterministic, blocking)
   - prompt_static_check.py  → render/import/schema/structural issues.
   - prompt_token_report.py  → size deltas, high-cost hotspots.
   - BLOCKING → repair changed files, update log, rerun. Max 3 same-domain retries.

5. SEMANTIC REVIEWERS (risk-tiered, by profile)
   - prompt      (review.md):              rule adherence, output contract, density, boundaries.
   - integrity   (reviewers/integrity):    source boundaries, safety, permission preservation.
   - topology    (reviewers/topology):     wiring, imports, agent routing, reviewer fanout.
   - adversarial (reviewers/adversarial):  self-iteration guard, optimizer future-proofing.

   Profile → reviewers:
     micro:           none
     standard:        prompt
     structural:      prompt + integrity + topology (by domain)
     self_iterating:  prompt + integrity + topology + adversarial
     high_risk:       prompt + integrity + adversarial (+ topology if structural)

6. REPAIR
   - Fix BLOCKING reviewer findings (correctness, boundaries, output contracts,
     selected rules, verification, wiring, self-iteration).
   - Rerun static/token + affected reviewers. Max 2 review repair rounds.

7. FINALIZE
   - Update edit-log.md to edit-log-shape.txt spec.
   - Return output contract: Status, Run Dir, Profile, Files Changed, Checks,
     Reviews, Summary, Remaining Risks.

Risk Profiles
-------------
micro:          docs-only wording/compression, no schema/permission/import change.
standard:       normal command/agent/reviewer/template prompt edit.
structural:     imports, templates, output protocol, agent boundaries, reviewer routing.
self_iterating: changes under .opencode/agent/_iterate/** or related scripts/docs.
high_risk:      permissions, destructive actions, security, secrets, sandbox, egress.

Key Design Rules
----------------
- Deterministic work in scripts; semantic judgment in agents/reviewers.
- Single run directory: artifacts/[[timestamp_slug]]/ contains everything.
- Risk-tiered reviewer fanout — no standing committee.
- Static checks before semantic review.
- Preserve output schemas, source boundaries, permissions, imports, verification gates.
- Inline one-consumer rules; keep multi-consumer includes.
- Keep commands thin; put behavior in owning agent.
- Prompt text owns behavior; harness/config owns runtime mechanics.
