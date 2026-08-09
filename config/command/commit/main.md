---
description: "Create commits using Keep a Changelog prefixes"
agent: commit
---

Optional scope, issue reference, grouping preference, or message guidance:
$ARGUMENTS

If no instructions are given above, default the scope in this order:
1. Changes made during the current conversation/session context.
2. Otherwise, the current unstaged changes.

Commits are created immediately without a confirmation prompt; the response includes a copy of each commit message.
