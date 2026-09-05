### Artifact paths
- `run_prefix`: `artifact/[[artifact_base]].[[run_id]].implement`.
- It is never a directory.

- Authored `01` maps to runtime evidence key `C01`.
- Start at `r01`; repairs/resume use unused rounds without resetting budgets.

- `validation_path`: `[[run_prefix]].Cnn.rNN.quick.validation.md`
- `review_path`: `[[run_prefix]].Cnn.rNN.<domain>.review.md`
- `verdict_path`: `[[run_prefix]].Cnn.rNN.verdict.md`
- Final paths replace `Cnn` with `final` and omit `quick.`.

- Callers assign exact paths; overwrite only current-round evidence.
- Preserve historical evidence paths on resume.
- Never create stub files; never write any other path.
