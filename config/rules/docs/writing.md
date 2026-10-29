# Documentation writing

## Match the reader

Assume readers know the programming language, not the domain or implementation.

Help future maintainers use and safely change the code.

## Keep guides simple

<example name="guide-recovery">
```markdown
!!! tip "Check the CLI error message before retrying"

An interrupted update can leave `state/corpus.lock` behind.

If this file blocks the next update:

1. Confirm the previous updater has stopped.
2. Run with `--force-update`.
3. Wait until the update saves state and removes `state/corpus.lock` before
   reading.
```
</example>

Open with a plain purpose line, then actions or choices.
In overviews, explain the problem before implementation.

Use short bullets, examples and links instead of dense setup paragraphs.

Use supported callouts selectively, with the takeaway as the title.
Keep necessary warning explanations with the warning.

Put example-specific guidance in code comments without repeating it in text.
Use two trailing spaces `  ` for Markdown line breaks.

## Keep API docs focused

<example name="api-sections">
```rust
/// Creates a folder.
///
/// # Arguments
///
/// - `path`: path of the new folder.
///
/// # Returns
///
/// [`Ok`] when the folder is created.
///
/// # Errors
///
/// - [`io::ErrorKind::AlreadyExists`]: `path` exists.
/// - Other I/O errors if the folder cannot be created.
///
/// # Remarks
///
/// Create any parent folders first.
pub fn create_folder(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir(path)
}
```
</example>

Open with what the item does.
Keep type summaries short; put method-specific contracts on methods.

Document public methods with a summary, Arguments for non-`self` parameters,
Returns for meaningful results, and Errors for fallible operations.

Describe `Result<()>` success; omit Returns for plain unit returns.
Use equivalent sections in other languages.

Keep required sections brief, not absent.
Omit mechanics established by the reader's prerequisites.

Add other sections only when useful, keeping this order:

- Rust: Arguments, Returns, Examples, Errors, Panics, Safety, Remarks.
- C#: summary, typeparam, param, returns, value, exception, remarks, example,
  seealso.

Give each parameter, return case and error one line where possible.
Start each documentation sentence on a new source line.

Put error conditions under Errors.

<example name="purpose-and-limit">
Before:
```rust
/// Writes a chunk without going over the expected file size.
```

After, with Arguments, Returns and Errors retained between these sections:
```rust
/// Writes a chunk.
///
/// # Remarks
///
/// Does not write beyond the expected file size.
```
</example>

Put useful type-wide failure behavior, limits and guarantees in Remarks.
Omit irrelevant detail rather than moving it there.
Keep essential contracts and safety instructions even when they add length.

<example name="simple-module">
```rust
//! Writes the generated C++ files to disk or memory.
//!
//! - [`DiskWriter`] saves them to the output directory.
//! - [`MemoryWriter`] saves them to memory.
```
</example>

Open module docs with a one-line purpose.
Crate and entry-module docs then list public entry points, one brief role each.
Skip the list for single-item modules.

Omit private wiring from module headers.
Entry headers omit workflows and API comparisons.
Keep call sequences on the relevant type or focused module, not both.

List call sequences as bullets, one call per bullet, never a paragraph.
For several limits, rules or cases, use a lead-in and short bullets.
Summarize categories unless readers need their members.

Keep required return/error conditions without cataloging inferred consequences.
Link code items: rustdoc [`Item`] and [`None`], C# `<see cref>`.

## Keep coverage minimal

<example name="restate-short-condition">
Before:
```rust
/// The result always stays under `self`, because `child` must itself pass
/// the checks in [`StoragePath::new`]. Absolute and `..` children fail.
```

After:
```rust
/// Absolute paths and `..` segments are not allowed.
```
</example>

Explain each contract once: restate short conditions, link longer explanations.
Keep method docs specific to that operation, not the surrounding workflow.

Omit generic trait remarks about implementations choosing names or storage.
Document specific constraints callers or implementers must follow instead.

<example name="irrelevant-qualification">
Before:
```rust
/// Creates a cache without storing any values.
```

After:
```rust
/// Creates a cache.
```
</example>

Omit boilerplate, obvious consequences and irrelevant edge cases.
Include qualifications only when they help a decision or prevent a mistake.

Do not add staging tutorials, responsibility reminders or durability disclaimers
unless needed for the reader's task.

Never document unimplemented work unless the user explicitly asks.
Update existing status notes only when their work changes.

Describe current caller-visible behavior, not internals or patch history.
Omit progress notes.
Explain internals only for maintainer tasks or correct use.

Skip private-helper rustdoc when names and code explain it.
Comment only on non-obvious reasons or invariants needed for safe changes.

## Use human wording

Write like a person explaining to a colleague.
Preserve clear author voice; do not invent a persona.

Use "we" naturally in walkthroughs, not as a compulsory subject.

Trim wordiness without losing natural sentences or useful explanations.
State what happens without unprompted "X, not Y" contrasts.

<example name="direct-outcome">
Before:
```rust
/// [`Ok`] when the implementation finishes the file successfully.
/// Returns an I/O error if the implementation cannot finish the file.
```

After:
```rust
/// [`Ok`] when the file is complete.
/// I/O error if the file cannot be finished.
```
</example>

Describe outcomes directly, without implementation-focused wording.
Under Returns and Errors, start with the outcome or condition, not "Returns".
Use "I/O error if..." without generic implementation-defined qualifications.

Use full, short sentences, one idea each.
Short labels are enough where a sentence would add nothing.

<example name="concrete-wording">
Before:
```rust
//! Converts source data into caller-selected representations.
// Materialize the computed result for subsequent reuse.
```

After:
```rust
//! Converts data to the format you choose.
// Save the result.
```
</example>

Use ordinary words for ordinary things, with concrete verbs and exact meaning.
Explain unfamiliar terms the task needs, respecting agreed reader knowledge.

Do not explain simple assignments or error recording; omit the comment or
name the action briefly, without a failure-mechanism narrative.

Expand unfamiliar acronyms on first use as `Expanded Name (ACRONYM)`.
Exempt known terms, identifiers, paths, headings and non-instructional text.
Use one term for each thing.

<example name="known-comparison">
```markdown
Basically a switch table, but `call` instead of `jmp`.
```
</example>

Relate a new idea to one the reader knows, then give the difference.

Give reliability estimates only with evidence.

## Use examples where they help

<example name="virtual-size-warning">
```markdown
!!! warning "A section's virtual size can exceed the bytes stored in the `.xbe`."

    For example a section at `0x1000` can have:

    - Stored size of `0x20`
    - Virtual size of `0x100`

    When this is loaded into memory, the remainder of the size not backed by
    stored bytes is zero-filled.

    We must be careful during generation to account for this.
```
</example>

State the behavior or invariant; show a small case if readers must simulate it.

Skip examples that repeat a clear explanation or another example.
Keep useful ones despite length.

<example name="api-usage">
````rust
/// Sends messages between parts of a program.
///
/// # Examples
///
/// ```rust
/// use std::sync::mpsc;
///
/// // Send a message.
/// let (sender, receiver) = mpsc::channel();
/// sender.send("ready")?;
///
/// // Read it back.
/// assert_eq!(receiver.recv()?, "ready");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Remarks
///
/// Messages arrive in send order.
````
</example>

For APIs, add one short usage example only when callers must combine operations.
Prefer usage over machinery; use verified APIs and hermetic fixtures.
Use short comments to group example steps when that helps readers follow them.

Let the example show the sequence; do not repeat it in text.
Keep type introductions to purpose; put useful shared rules in brief Remarks.

<example name="test-phases">
```rust
// Arrange
let text = "twelve"; // <- not a number

// Act
let result = text.parse::<u32>(); // <- fails here

// Assert
assert!(result.is_err()); // <- rejected input
```
</example>

Separate test phases with standalone `// Arrange`, `// Act` and `// Assert`.
Put phase explanations below the label.

Use short trailing `// <-` arrows only to trace the faulting input.
Leave other lines unmarked.

Mark that input, including parameterized case inputs, then each line that
processes it, up to its assertion.
Do not replace the marker with a paragraph about expected failure.

Group related operations with blank lines.
Open each group with a one-line comment naming its action.
Use longer comments only for invariants or worked inputs.
Leave simple bodies alone when their names explain them.

Use ordering words for non-obvious dependencies, not simple two-step bodies.

<example name="markdown-transformation">
```markdown
## Collision renames

!!! note "Archives can contain several files with the same name."

We keep them all instead of overwriting one with another.

We add `~1`, `~2`, and so on until we find an unused name.

| Archive path | Stored path                    |
| ------------ | ------------------------------ |
| `readme.txt` | `readme.txt`                   |
| `readme.txt` | `readme.txt~1`                 |
| `setup.exe`  | `setup.exe.malware.donotrun`   |
| `setup.exe`  | `setup.exe~1.malware.donotrun` |
```
</example>

Use before/after values or tables for transformations.
Use diagrams for relationships.
Include meaningful cases; reuse one scenario without explaining it twice.

<example name="maintainer-invariant">
```rust
// Put boundaries at both ends without changing their owner sets.
// This lets us edit start..end without affecting bytes outside it.
```
</example>

In maintainer docs, explain the mechanism or invariant needed for safe changes.
Name the function and file when they help readers navigate.

## Apply the editorial profile

Before delivery, check wording against these rules, audience and style examples.

Apply user-corrected style patterns throughout the authorized scope.
Prefer readability and useful explanations over shorter wording.
Lint success alone does not establish reader fit.

{{ file="./rules/adhd-communication.md" }}
