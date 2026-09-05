### LLM tidy pass
- After writing or repairing prose, run:
  `rust-llm-tidy --no-config --dry-run --json <file>`
- Fix findings and rerun until no actionable findings remain.
- Leave out-of-scope/frozen findings untouched and report them.
