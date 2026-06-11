# Prompt Template Library

These are compact shapes to reuse when creating new command/agent/reviewer files.

## Agent Contract

```xml
<agent_contract id="[[agent_id]]">
Goal: [[deliverable]].
Inputs: [[inputs]].
Done: [[done_criteria]].
</agent_contract>
```

## Workflow

```xml
<workflow>
1. Read [[required_inputs]].
2. Inspect [[target_paths]] and direct references.
3. Apply [[selected_rules]].
4. Run [[required_checks]].
5. Return [[output_contract]].
</workflow>
```

## Reviewer

```text
# REVIEW
Decision: PASS | ADVISORY | BLOCKING
Reviewer: [[domain]]
Profile: [[profile]]

## Findings
- None
- [[ID]] | BLOCKING | [[path:line_or_section]] | [[problem]] | Fix: [[minimal_fix]]

## Verified
- [[evidence_or_check]]
```

## Run Directory

```text
.opencode/runs/[[command_name]]/[[timestamp_slug]]/
  request.md
  prep.json
  contract.md
  edit-log.md
  static-check.md
  token-report.md
  reviews/
```

## Placeholder Convention

Use `[[slot_name]]` for variable placeholders in prompt text, examples, and schemas. Use `<tag>` only for real XML-style sections.
