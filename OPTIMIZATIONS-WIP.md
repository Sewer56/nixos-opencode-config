
In 183372b2cc375945964d3d1a179973a854cbcee8 , `rust-auto-reorder`. Good example of code to dedupe.

```
let mut cmd = Command::new(binary());
cmd.arg("--dry-run");
cmd.arg(after_path);

let output = cmd.output().unwrap();
assert!(
    output.status.success(),
    "rust-auto-reorder --dry-run failed on {}: {}",
    after_path.display(),
    String::from_utf8_lossy(&output.stderr)
);
```

Markdown tables should be aligned.

Bad: 

```
//! | Code     | Severity | Fires when |
//! |----------|----------|------------|
//! | `DOC001` | Error    | A non-private item has no `///` doc comment. |
//! | `DOC002` | Error    | A `pub fn` returning `Result` has no `# Errors` section. |
//! | `DOC003` | Warning  | A `# Errors` section names no concrete error variant. |
```