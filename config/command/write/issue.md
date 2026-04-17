---
description: "Write an issue using a repo template or built-in fallback"
agent: build
---

# Write Issue

Do one very quick repo pass before drafting:
- inspect the repo root for obvious context
- skim `README.md` if present
- look for issue templates in `.github/ISSUE_TEMPLATE/` and `.github/ISSUE_TEMPLATE.md`

Then write the issue directly.

## User Request

```text
$ARGUMENTS
```

## Template Selection

1. If repo issue templates exist, read them and choose the single best match for the user's request.
   - Use the template's name, labels, and sections to decide.
   - Ignore `.github/ISSUE_TEMPLATE/config.yml`.
2. If no repo template exists, use one built-in fallback template below.
   - Use the bug template for bugs, regressions, crashes, and broken behavior.
   - Use the feature template for features, enhancements, requests, and improvements.
3. If the intent is still unclear, ask one short clarifying question. Otherwise decide and proceed.

## Built-in Templates

### Bug Template

```markdown
---
title: "[Bug Title]"
labels: [bug]
---

## Description
[Bug description]

## Steps to Reproduce
1. [Step 1]
2. [Step 2]

## Expected Behaviour
[What should happen]

## Actual Behaviour
[What actually happens]

## Additional Context
[Any additional information]
```

### Feature Template

```markdown
---
title: "[Feature Title]"
labels: [enhancement]
---

## Description
[Feature description]

## Use Case
[Why this feature is needed]

## Proposed Solution
[How this could be implemented]

## Alternatives Considered
[Other approaches considered]

## Additional Context
[Any additional information]
```

## Output

- Create `ISSUE-[brief-description].md`.
- Write a tight initial issue draft that fits the chosen template.
- Briefly state which template was used.
- After writing the initial draft, offer an optional deep repo pass via `@general` to refine likely affected areas, solution shape, and risks.
