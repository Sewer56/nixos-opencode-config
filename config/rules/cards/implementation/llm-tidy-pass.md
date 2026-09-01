### LLM tidy pass
Draft, review, and refine prose artifacts through `rust-llm-tidy`.
Run the pass after drafting and after each repair of the artifact.

1. Review non-mutating with `rust-llm-tidy --no-config --dry-run --json <file>`.
2. Apply every finding to the artifact, then save it.
3. Rerun the review on the saved artifact.
4. Repeat until the JSON output is `[]`.

Keep every repair inside the artifact's declared scope and frozen regions.
Never widen scope or cross a frozen-region boundary to satisfy a finding.
