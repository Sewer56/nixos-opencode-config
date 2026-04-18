## Plan Content Rules

- No placeholders (`...`, `TODO`, comment-only test bodies).
- No undefined helpers/types/symbols in snippets.
- Insertions use normal code blocks with `Insert at: <anchor> (~start-end)`.
- Edits/removals use `diff` blocks; deletions include `Remove lines: ~start-end`.
- Include 2+ context lines before and after each change. Context is the authoritative locator; `Lines: ~start-end` indicates approximate location.
- Import changes use a dedicated import `diff` block.
- If layout changes, include target tree and migration order.
