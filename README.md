# OpenCode Config

Personal [OpenCode] configuration for focused repository-scale coding. Main flow turns a request into one human-approved behavioral draft, then implements it in dependency-ordered, validated, reviewed cohorts.

```text
/draft <request>
# Review or edit PROMPT-PLAN-<slug>.draft.md
/implement PROMPT-PLAN-<slug>.draft.md
```

Design favors selective context, deterministic evidence, precise review, and few clear roles—not maximum agents or prompt volume. See [architecture and rationale](EXPLAINER.md#architecture).

> [!WARNING]
> Personal provider names, secret paths, Nix assumptions, plugins, and pinned submodules need adaptation before reuse.

## Main workflow

### Draft and approve

`/draft` creates or refines one root `PROMPT-PLAN-<slug>.draft.md` containing behavioral goal, decisions, invariants, non-goals, acceptance criteria, logical work, validation, review routes, and unresolved questions.

Draft describes behavior, decisions, invariants, non-goals, acceptance criteria, and validation. Review it until status is `READY_FOR_IMPLEMENT`; approved file becomes downstream behavioral authority.

### Implement

```text
/implement PROMPT-PLAN-example.draft.md
```

`/implement` reconciles approved draft with live repository, creates dependency-ordered cohorts, and calls one cohort agent per cohort:

```text
one writer -> deterministic checks -> focused review -> finding verification
           -> bounded repair when needed -> exact local commit
```

Correctness and quality review every proposed commit. Test, security, and performance specialists run only for matching risk. Final gate validates and reviews complete base-to-final result. Workflow never pushes.

Each implementation invocation starts from approved draft. Existing unrelated user changes are preserved; a planned target already changed by user requires input.

For low-ambiguity work:

```text
/implement/one-shot <request>
```

It routes through same draft and implementation pipeline.

## Outcomes and artifacts

- `SUCCESS`: required implementation and evidence complete.
- `INCOMPLETE`: no known blocker, but required evidence unavailable.
- `NEEDS_INPUT`: material human decision or ambiguous pre-existing target change requires resolution.
- `FAIL`: proven failure remains after bounded repair or protocol integrity failed.

Implementation artifacts under `artifact/` include handoff/cohorts, validation ledgers, candidate reviews, and verifier verdicts. Iterate artifacts use `artifacts/iterate/`.

Internal review findings are hypotheses; blockers and advisories accepted by the shared verifier enter automatic repair within approved plan scope. CodeRabbit uses its own structured findings as authority.

Validator scope is documented in module docstring at top of `scripts/validate-opencode-config.py`.

## Commands

### Planning and implementation

| Command | Purpose |
|---|---|
| `/draft` | Create or refine human-reviewed implementation draft. |
| `/plan/convert-to-draft` | Convert useful conversation context into same draft format. |
| `/implement` | Implement approved draft in validated logical cohorts. |
| `/implement/one-shot` | Draft, review, and implement bounded request. |

### Refactoring

| Command | Purpose |
|---|---|
| `/refactor/modularize` | Draft behavior-preserving modularization. |
| `/refactor/parameterize` | Draft safe test parameterization. |
| `/refactor/reorder` | Preview and reorder declarations after explicit `go`. |
| `/refactor/document` | Repair scoped source documentation. |
| `/refactor/errors` | Trace and repair public error documentation. |

### Documentation, review, and audits

| Command | Purpose |
|---|---|
| `/docs/write` | Write scoped end-user documentation. |
| `/docs/review` | Review and repair scoped end-user documentation. |
| `/review/coderabbit` | Run CodeRabbit and repair its blocking findings. |
| `/audit/public-api` | Audit unnecessarily public APIs. |

### Repository maintenance

| Command | Purpose |
|---|---|
| `/commit/main` | Create intentional semantic commits with explicit staging. |
| `/write/issue` | Write repository-grounded issue file. |
| `/write/pr` | Generate evidence-backed `pr.md` from branch diff. |
| `/iterate/edit` | Create, edit, move, delete, or verify instruction artifacts. |
| `/migrate` | Run separate pinned-source migration workflow. |

See [practical iterate guide](.opencode/ITERATE.md) for instruction work.

## Layout

```text
config/       installed OpenCode config, agents, commands, rules, plugins
.opencode/    repository-local iterate and migration workflows
scripts/      deterministic validation and platform setup
tests/        implementation workflow contract tests
flake.nix     Home Manager module, tools, and reproducible validation deps
EXPLAINER.md  architecture, authoring guidance, research, tradeoffs
```

Agents use least-privilege permissions and receive only tools and paths required for their role.

## Installation

Full checkout:

```bash
git submodule update --init --recursive
```

Nix/Home Manager setup links `~/.config/opencode` to editable `config/` and provides `opencode`, `opencode-build`, local tools, CodeRabbit CLI, and validation dependencies.

```bash
opencode-build
opencode /path/to/project
```

Windows:

```powershell
pwsh ./scripts/windows/setup.ps1
```

Pre-flight offers to install missing prerequisites (cargo, bun, git, Node.js LTS, Yarn, Docker Desktop) via `winget`, with a y/N prompt per tool and a direct-installer fallback for cargo/bun/yarn. Installs are never fatal; declines fall back to the old detect-and-warn behaviour. Pass `-NoInstallPrereqs` to force detect-only.

CodeRabbit CLI is intentionally not installed on Windows: upstream ships Linux/macOS binaries only (WSL-only on Windows). The `/review/coderabbit` command returns `INCOMPLETE` when `cr`/`coderabbit` is absent. To use it, install under WSL: `wsl -c 'curl -fsSL https://cli.coderabbit.ai/install.sh | sh'`.

## Validation

From repository root:

```bash
python3 scripts/validate-opencode-config.py --repo-root .
python3 -m unittest discover -s tests -p 'test_*.py'
```

Use `nix develop` when local Python lacks `json5` or `PyYAML`.

Static checks cannot certify provider credentials, plugin loading, CodeRabbit service, installed OpenCode runtime, or stochastic model behavior. Those remain environment checks.

[OpenCode]: https://opencode.ai
