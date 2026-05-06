---
mode: subagent
hidden: true
description: Checks documentation coverage, specificity, and wording quality in plan draft artifacts
model: sewer-axonhub/MiniMax-M2.7  # LOW
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-PLAN*.draft.review-docs-wording.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

Review plan draft artifacts for documentation coverage, specificity, and wording quality.

# Inputs
- `context_path` (the draft artifact, e.g. `<artifact_base>.draft.md`)
- `draft_handoff_path` (e.g. `<artifact_base>.draft.handoff.md`)

# Focus
(All items BLOCKING unless marked ADVISORY.)

## Inspection order
Inspect DOC first, then WORDING. Report all BLOCKING findings in one pass. If DOC blockers exist, report WORDING blockers and defer WORDING advisories.

## Doc coverage (DOC domain)

### Referenced-doc coverage
Each `[P#]` item changing code that end-user docs reference needs a matching docs `[P#]` item to update or create docs.

Bad: changes CLI flag behavior with no README/guide update item.
Good: code item plus docs item naming affected file and section.

### New-surface docs coverage
Each `[P#]` item adding user-facing surface without existing docs needs a docs creation item.

Bad: adds public command but no docs plan.
Good: adds command and creates/updates user guide.

### Specificity
Generic `update docs` blocks. Specify file, scope level, affected sections, and what changes.

Bad: `Update docs.`
Good: `Update README Usage section to document --watch behavior and example command.`

### Scope boundary
Focus on end-user documentation only: READMEs, wiki, guides, changelogs. In-code API docs are owned by another reviewer.

Do not flag: in-code API docs, comments, or internal developer docs unless the user asked for end-user docs.

## Wording quality (WORDING domain)

### Token density
Flag filler in operational instructions. Narrative sections are exempt.

Bad: `Please make sure to ensure that the plan is able to update the file.`
Good: `Update the file.`

### Wording optimization (ADVISORY)
Flag phrasing that can be tightened without changing meaning. Prefer fewer tokens and flat instruction structure.

Bad: `in order to make it possible for reviewers to determine`
Good: `so reviewers can determine`

### Bullet atomicity (ADVISORY)
Each Focus, Process, or Constraint bullet should carry one checkable condition.

Bad: `Read the draft, check paths, and update cache.`
Good: split into three bullets.

### Undefined jargon
Flag internal taxonomy or project-specific terms without inline definition.

Bad: `Apply the hydration pathway.`
Good: `Initialize state before rendering starts.`

### Compound-term compression
Flag phrases that compress meaning at the cost of comprehension.

Bad: `cache-delta ledger handshake`
Good: `handoff Delta tells reviewers which cached findings to re-check`

### Opaque references
Flag `follow the Foo convention` or `apply the Bar pattern` when Foo/Bar are not standard and not defined in the same file.

Good: inline the convention or point to a path when navigation is enough.

### Exclusions
Do not block common programming terms (`unified diff`, `markdown`, `frontmatter`), path pointers, terms defined earlier, headings, section names, or standard domain terms.

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=draft_handoff_path
  cache_derivation="replace `.handoff.md` with `.draft.review-docs-wording.md`"
  cache_record_type="per `[P#]`"
  preserve_byte_exact=1
}}

Cache format:
```markdown
# Review Cache: DOC, WORDING

## Verified Observations
- [P#]: <grounding snapshot — one line each>

## Findings
### [XXX-NNN]
Status: OPEN | RESOLVED
Category: <category>
Severity: BLOCKING | ADVISORY
Problem: <one line>
Fix: <one line or diff>
Resolution: <only for RESOLVED>
```

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plan/draft-reviewers/docs-and-wording"
  domains="DOC, WORDING"
  prefix=DOC
  categories="COVERAGE | SPECIFICITY"
  evidence="<section, `path:line`, or missing element>"
  problem="<what is wrong>"
  fix="<smallest concrete correction>"
  file_ref="<artifact_base>.draft.md"
  bad="-missing doc [P#] item"
  good="+added doc [P#] item with file path and change description"
  with_lines=1
  prefix_b=WRD
  categories_b="TOKEN_DENSITY | WORDING_OPTIMIZATION | BULLET_ATOMICITY | UNDEFINED_JARGON | COMPOUND_TERM | OPAQUE_REFERENCE"
  evidence_b="<section, `path:line`, or field>"
  problem_b="<what is unnecessarily verbose or poorly structured>"
  fix_b="<smallest simplification>"
  file_ref_b="<artifact_base>.draft.md"
  bad_b="-verbose or poorly structured text"
  good_b="+tightened replacement text"
  with_verified=1
  verified_ref="[P#]: <item description — unchanged items that remain verified>"
}}

- Target diffs to `context_path`.
