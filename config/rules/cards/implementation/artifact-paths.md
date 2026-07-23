### Artifact paths
`run_prefix` is `artifact/[[artifact_base]].[[run_id]].implement` — a filename prefix, never a directory; never `mkdir`.

Bind each path variable by substitution: `Cnn` = cohort id (`C01`, `C02`, …), `rNN` = round starting `r01`, +1 per repair round, `<domain>` = reviewer domain (`correctness`, `quality`, `tests`, `security`, `performance`, `integration`).

- `handoff_path`: `[[run_prefix]].handoff.md`
- `cohort_path`: `[[run_prefix]].Cnn.md`
- `validation_path`: `[[run_prefix]].Cnn.rNN.quick.validation.md`
- `review_path`: `[[run_prefix]].Cnn.rNN.<domain>.review.md`
- `verdict_path`: `[[run_prefix]].Cnn.rNN.verdict.md`
- final gate: `validation_path` = `[[run_prefix]].final.rNN.validation.md`, `review_path` = `[[run_prefix]].final.rNN.<domain>.review.md`, `verdict_path` = `[[run_prefix]].final.rNN.verdict.md`

The caller computes the exact path from these templates and passes it as the matching input; the addressed writer creates or overwrites that exact file. Never create placeholder or stub files (e.g. `touch`); never write any other path.
