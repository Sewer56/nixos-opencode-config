### Artifact paths
- `run_prefix`: `artifact/plan/[[artifact_base]]/review`.
- `artifact_base`: root basename without `.draft.md`.
- Create only runtime `review/`, never alter source members.
- `run_id`: UTC timestamp bound once; reviewer subfolders match agent names.

- Authored `01` maps to runtime evidence key `C01`.
- Start at `r01`; repairs/resume use unused rounds without resetting budgets.

- `validation_path`: `[[run_prefix]]/[[run_id]].Cnn.rNN.quick.validation.md`
- `review_path`: `[[run_prefix]]/<reviewer>/[[run_id]].Cnn.rNN.review.md`
- `verdict_path`: `[[run_prefix]]/verifier/[[run_id]].Cnn.rNN.verdict.md`
- Final paths replace `Cnn` with `final` and omit `quick.`.

- Callers assign exact paths; overwrite only current-round evidence.
- Reject outputs aliasing source, inputs or historical evidence.
- Preserve historical evidence paths on resume.
- Never create stub files; never write any other path.
