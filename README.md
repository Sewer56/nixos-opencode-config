# OpenCode Config

Personal [OpenCode] configuration for repository-scale coding.

```text
/draft [[request]]
# Review PROMPT-PLAN-[[slug]].draft.md and its declared members
/implement PROMPT-PLAN-[[slug]].draft.md
```

See [architecture and rationale].

> [!WARNING]
> Adapt personal providers, secret paths, Nix assumptions, and plugins.
> Check pinned submodules too.

## Main workflow

### Draft and approve

Use `/draft` to define an ignored contract and small cohorts.
Review their goals, scope, exclusions, and completion checks.
Approve the ready bundle with `/implement [[plan_path]]`.

### Implement

Children implement, test, review, and commit cohorts in order.
Say “resume from C03” to continue without discarding prior work.
Unclear ownership or material facts prompt a question.
Final checks include CodeRabbit; nothing is pushed.

## Outcomes and artifacts

- `SUCCESS`: required implementation and evidence complete.
- `INCOMPLETE`: no known blocker, but required evidence unavailable.
- `NEEDS_INPUT`: material decision or unclear change ownership needs resolution.
- `FAIL`: proven failure remains or protocol integrity failed.

- Validation, reviews, and verdicts live under `artifact/`.
- Instruction-edit evidence uses `artifacts/iterate/`.
- Internal findings need verifier acceptance for bounded, in-scope repair.
- CodeRabbit uses its own findings as authority.

## Commands

### Planning and implementation

| Command                  | Purpose                                                              |
| ------------------------ | -------------------------------------------------------------------- |
| `/draft`                 | Define cohorts in a human-reviewed plan bundle.                      |
| `/plan/convert-to-draft` | Convert useful conversation context into same draft format.          |
| `/implement`             | Execute approved cohorts in dependency order.                        |
| `/implement/one-shot`    | Implement a bounded request in one writer-review-verify-repair loop. |
| `/code`                  | General rules-baked coding agent with on-request reviewer/verifier.  |

### Refactoring

| Command                  | Purpose                                                                         |
| ------------------------ | ------------------------------------------------------------------------------- |
| `/refactor/modularize`   | Draft behavior-preserving modularization.                                       |
| `/refactor/parameterize` | Draft safe test parameterization.                                               |
| `/refactor/reorder`      | Preview and reorder declarations after explicit `go`.                           |
| `/refactor/document`     | Repair scoped source documentation.                                             |
| `/refactor/errors`       | Trace and repair public error documentation.                                    |
| `/cleanup`               | Clean existing code to current standards through the implement review gauntlet. |

### Documentation, review, and audits

| Command              | Purpose                                          |
| -------------------- | ------------------------------------------------ |
| `/docs/write`        | Write scoped end-user documentation.             |
| `/docs/review`       | Review and repair scoped end-user documentation. |
| `/review/coderabbit` | Run CodeRabbit and repair its blocking findings. |
| `/audit/public-api`  | Audit unnecessarily public APIs.                 |

### Repository maintenance

| Command         | Purpose                                                      |
| --------------- | ------------------------------------------------------------ |
| `/commit/main`  | Create intentional semantic commits with explicit staging.   |
| `/write/issue`  | Write repository-grounded issue file.                        |
| `/write/pr`     | Generate evidence-backed `pr.md` from branch diff.           |
| `/iterate/edit` | Create, edit, move, delete, or verify instruction artifacts. |
| `/migrate`      | Run separate pinned-source migration workflow.               |

See [practical iterate guide] for instruction work.

## Installation

Full checkout:

```bash
git submodule update --init --recursive
```

- Nix/Home Manager links `~/.config/opencode` to editable `config/`.
- It supplies OpenCode, local tools, CodeRabbit CLI, and validation tooling.

```bash
opencode-build
opencode /path/to/project
```

Windows:

```powershell
pwsh ./scripts/windows/setup.ps1
```

- Setup offers missing cargo, bun, git, Node.js LTS, Yarn, and Docker Desktop.
- Each tool gets a y/N `winget` prompt; cargo/bun/yarn have installer fallbacks.
- Installation failures and declines warn without stopping setup.
- Pass `-NoInstallPrereqs` for detect-only.

- CodeRabbit CLI ships Linux/macOS binaries; Windows setup does not install it.
- `/review/coderabbit` returns `INCOMPLETE` without `cr`/`coderabbit`.
- Install under WSL:

```bash
wsl -c 'curl -fsSL https://cli.coderabbit.ai/install.sh | sh'
```

## Validation

From repository root:

```bash
python3 scripts/validate-opencode-config.py --repo-root .
python3 -m unittest discover -s tests -p 'test_*.py'
```

Use `nix develop` when local Python lacks `json5` or `PyYAML`.
See the [validator docstring] for check scope.

- Credentials, plugins, CodeRabbit, and OpenCode need environment checks.
- Static checks cannot certify stochastic model behavior.

[OpenCode]: https://opencode.ai
[architecture and rationale]: EXPLAINER.md#architecture
[practical iterate guide]: .opencode/ITERATE.md
[validator docstring]: scripts/validate-opencode-config.py
