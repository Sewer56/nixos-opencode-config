## Performance Rules

Use these rules when choosing between correct implementations that differ in performance.

### Rules

- Prefer the highest-performance correct implementation.
- Then simplify for readability and reviewability, but never trade meaningful performance for brevity or superficial simplicity.

### Example

```rust
// Prefer: partial-sort + sort only the slice you keep
entries.select_nth_unstable_by(limit - 1, |a, b| b.1.cmp(&a.1));
entries[..limit].sort_by(|a, b| b.1.cmp(&a.1));

// Avoid when dataset can exceed limit: sorts entries you'll discard
entries.sort_by(|a, b| b.1.cmp(&a.1));
```
