# Documentation writing

Write only what the reader needs for correct use or safe maintenance.
Leave docs unchanged when they already fit the reader's task.

## Match the reader's task

Assume stated prerequisites only.
By default, readers know the language, not this domain or implementation.

Domain includes platforms, hardware, formats, protocols and project terms.

Classify by reader and purpose, not file extension.

- API reference: caller-visible contracts needed for correct use.
- Maintainer docs: internals needed for safe changes and operation.
- Guides: prerequisites, actions, expected outcomes and recovery.

## Structure source docs

<!---
```rust
/// Files are renamed when both folders share a drive.
/// Otherwise they are copied.
```
--->
In comments, start a sentence mid-line only if it ends on that line.

<!---
```rust
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
///
/// # Remarks
///
/// Files are read on commit, so later edits to `source` are included.
```

```csharp
/// <summary>Queue one file to copy into the mods folder on <see cref="Staging.Commit"/>.</summary>
/// <param name="source">File to copy.</param>
/// <param name="target">Path relative to the mods folder.</param>
/// <returns>The number of files now queued.</returns>
/// <exception cref="IOException"><paramref name="source"/> cannot be read.</exception>
/// <exception cref="StagingConflictException"><paramref name="target"/> is already queued.</exception>
/// <remarks>Files are read on commit, so later edits to <paramref name="source"/> are included.</remarks>
```
--->
Open public API docs with the item's purpose and key contract.

Use only needed sections, in this order:

- Rust: Arguments, Returns, Examples, Errors, Panics, Safety, Remarks.
- C#: summary, typeparam, param, returns, value, exception, remarks, example,
  seealso.

Give each parameter, return case and error one line where possible.

Put exact rules, edge cases and needed internals in Remarks.
Put error cases under Returns or Errors, not Remarks.

<!---
```rust
/// Unlike [`Staging::copy`], replaces files already queued.
```

```csharp
/// Unlike <see cref="Staging.Copy"/>, replaces files already queued.
```
--->
Link code items in docs: rustdoc [`Item`], C# `<see cref>`.

## Separate user docs from implementation work

User-facing docs describe public behavior, not development progress.

Omit unsolicited internal wiring, refactor notes and migration markers.
Omit TODOs, implementation status and pending-work lists unless requested.

Describe user-visible limitations as current behavior, not unfinished work.
Keep progress and remaining work in task artifacts or handoffs.
Keep unrequested patch history in commits/PRs.

Include internals only when requested or needed for correct use.

## Keep coverage minimal

Add docs only for unmet reader needs or explicit project requirements.
Update affected docs in place.

Omit boilerplate and repetition of signatures or obvious behavior.
Link existing coverage instead of repeating it.
Preserve required sections, contracts, warnings and requested explanations.

Never invent behavior, errors or examples to fill a section.

## Use human, simple wording

Write like a person explaining to a colleague.
Match the tone and terms of agreed style examples or the project's best docs.

Keep sentences short, ideally under 25 words.
Write full sentences, not clipped fragments.
Cut what readers don't need, not explanations they do.
Choose easy reading over fewer words.

Use everyday words and direct verbs.
Say what happens before naming the concept.

Replace each domain term with its practical meaning, or explain it on first use.
Skip only terms the user says readers know; if unsure, explain.
Use one term for each thing.

## Use examples to make abstract ideas concrete

Show an abstract mechanism with one real example, such as input and output.
Skip examples that repeat what the text already makes clear.

Reuse a coherent scenario without repeating explanations.
Use diagrams when they clarify relationships better than text alone.

Use verified behavior, real APIs and hermetic fixtures for runnable examples.

Keep useful examples despite their length.

{{ file="./rules/adhd-communication.md" }}
