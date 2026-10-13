# Documentation writing

## Match the reader

Assume readers know the programming language, not the domain or implementation.

## Keep guides simple

Open with a plain purpose line, then actions or choices.
In overviews, explain the problem before implementation.

Use short bullets, examples and links instead of dense setup paragraphs.

Use supported callouts selectively; a title can headline the text below it.
Keep necessary warning explanations with the warning.

Put example-specific guidance in code comments without repeating it in text.
Use two trailing spaces `  ` for Markdown line breaks.

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

## Keep API docs focused

Open with the item's purpose and key contract.
Keep type summaries short; put method-specific contracts on methods.

Omit mechanics established by the signature or the reader's prerequisites.

Use only needed sections, in this order:

- Rust: Arguments, Returns, Examples, Errors, Panics, Safety, Remarks.
- C#: summary, typeparam, param, returns, value, exception, remarks, example,
  seealso.

Give each parameter, return case and error one line where possible.
In comments, start a sentence mid-line only if it ends on that line.

<example name="api-sections">
````rust
/// Queue one file to copy into the mods folder on [`Staging::commit`].
///
/// # Arguments
///
/// - `source`: file to copy.
/// - `target`: path relative to the mods folder.
///
/// # Returns
///
/// The number of files now queued.
///
/// # Examples
///
/// ```no_run
/// staging.copy(&texture, "textures/sky.dds")?;
/// ```
///
/// # Errors
///
/// - [`StagingError::Conflict`]: `target` is already queued.
/// - [`StagingError::Io`]: `source` cannot be read.
///
/// # Panics
///
/// Panics if `target` is an absolute path.
pub fn copy(&mut self, source: &Path, target: &str) -> Result<usize, StagingError> {
````
</example>

For several limits or cases, use a plain lead-in and short bullets.
Summarize categories unless readers need their members.

<example name="return-limits">
```rust
/// Returns the section's stored bytes.
///
/// # Arguments
///
/// - `data`: The executable's complete file buffer.
///
/// # Returns
///
/// A slice of `data` starting at `raw_address`.
/// Reads stop at the smallest of:
/// - Raw size.
/// - Mapped size.
/// - Available file bytes.
pub fn file_bytes<'a>(&self, data: &'a [u8]) -> &'a [u8] {
```
</example>

Keep required return/error conditions without cataloging inferred consequences.
Put errors under Returns or Errors; needed caveats and their reasons in Remarks.
Link code items: rustdoc [`Item`] and [`None`], C# `<see cref>`.

<example name="error-contract">
```rust
/// Parses one page from an already fetched response.
///
/// # Arguments
/// - `status`: HTTP status code returned with `body`.
/// - `body`: Response body, read only when `status` is 200.
///
/// # Returns
/// The parsed page. `next_cursor` is [`None`] on the last page.
///
/// # Errors
/// - [`PageError::AuthenticationDenied`] when `status` is 401.
/// - [`PageError::CorpusAccessDenied`] when `status` is 403.
/// - [`PageError::InvalidRequest`] when `status` is 422.
/// - [`PageError::MalformedData`] when `status` is 200 and `body` is not
///   a valid page.
/// - [`PageError::TransientStatus`] when `status` is 408, 429, 500, 502,
///   503 or 504.
/// - [`PageError::UnexpectedStatus`] for any other `status`.
pub fn from_response(status: u16, body: &[u8]) -> Result<Self, PageError> {
```
</example>

## Keep coverage minimal

Explain each contract once; link existing coverage instead of repeating it.
Omit boilerplate, obvious consequences and irrelevant edge cases.

User-facing docs describe current behavior, not patch history or progress.
Include internals only when requested or needed for correct use.

## Use human wording

Write like a person explaining to a colleague.
Preserve clear author voice; do not invent a persona.

Use "we" naturally in walkthroughs, not as a compulsory subject.

Trim wordiness without losing natural sentences or useful explanations.

<example name="trim-unnecessary-detail">
Before:
```rust
/// The slice is a view into `data`, not a copy.
/// A short `data` gives fewer bytes, not an error.
```

After:
```rust
/// Returns the section's stored bytes.
/// Reads stop at the end of `data`.
```
</example>

Use full, short sentences, one idea each.
Short labels are enough where a sentence would add nothing.

Prefer everyday words and concrete verbs without losing precision.
Explain unfamiliar terms the task needs, respecting agreed reader knowledge.

Expand unfamiliar acronyms on first use as `Expanded Name (ACRONYM)`.
Exempt known terms, identifiers, paths, headings and non-instructional text.
Use one term for each thing.

Relate a new idea to one the reader knows, then give the difference.

<example name="known-comparison">
```markdown
Basically a switch table, but `call` instead of `jmp`.
```
</example>

State relevant limitations; give reliability estimates only with evidence.

## Use examples where they help

State the behavior or invariant; show a small case if readers must simulate it.
Skip examples that repeat a clear explanation; keep useful ones despite length.

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

For API docs, prefer usage examples over machinery demonstrations.
Use verified APIs and hermetic fixtures for runnable examples.

<example name="api-usage">
````rust
/// Inputs for a page request, serialized as URL query parameters.
///
/// Pass [`None`] for the first page.
/// To continue, pass the returned cursor unchanged.
///
/// # Examples
///
/// ```
/// # use malware_corpus_client::{Page, PageRequest, InvalidPageLimit};
/// # fn example(page: Page) -> Result<(), InvalidPageLimit> {
/// let first = PageRequest::new(500, None)?;
/// let next = PageRequest::new(500, page.next_cursor)?;
/// # Ok(())
/// # }
/// ```
````
</example>

Use before/after values or tables for transformations.
Use diagrams for relationships.
Include meaningful cases; reuse one scenario without explaining it twice.

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

In maintainer docs, explain the mechanism or invariant needed for safe changes.
Name the function and file when they help readers navigate.

<example name="maintainer-invariant">
```rust
// Put boundaries at both ends without changing their owner sets.
// This lets us edit start..end without affecting bytes outside it.
```
</example>

{{ file="./rules/adhd-communication.md" }}
