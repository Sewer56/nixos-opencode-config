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

Discuss goal, constraints, design and task outline with `/draft` first.
Explicit agreement and authorization unlock documents.

The readable root owns decisions; human task briefs own scope and completion.
Shared `execution.md` and paired task exec files hold technical instructions.

Plans are locally ignored and pass tidy, routing and whole-bundle review.
Approve the ready bundle with `/implement [[plan_path]]`.
Old combined plans are unsupported; nothing silently converts them.

### Implement

Children read human authority, implement, test, review and commit in order.
Say “resume from C03” to continue without discarding prior work.

Unclear ownership or material facts prompt a question.
Final checks include CodeRabbit; nothing is pushed.

## External research

Route external queries through `web-search`; MCP calls can be expensive.

## Outcomes and artifacts

- `SUCCESS`: required implementation and evidence complete.
- `INCOMPLETE`: no known blocker, but required evidence unavailable.
- `NEEDS_INPUT`: material decision or unclear change ownership needs resolution.
- `FAIL`: proven failure remains or protocol integrity failed.

- Validation, reviews, and verdicts live under `artifact/`.
- Instruction-edit evidence uses `artifacts/iterate/`.
- Internal findings need verifier acceptance for scoped repair.
- Apply feasible verified advisories within scope and budget.
- Skipped advisories stay visible with reasons; they never block success.
- CodeRabbit uses its own findings as authority.

## Commands

### Planning and implementation

- `/draft`: discuss a design, then write a human/exec task bundle.
- `/plan/convert-to-draft`: continue conversation toward an agreed draft.
- `/implement`: execute approved tasks in dependency order.
- `/implement/one-shot`: implement and review one bounded request.
- `/code`: interactive coding with review only on request.

### Refactoring

- `/refactor/modularize`: draft behavior-preserving modularization.
- `/refactor/parameterize`: draft safe test parameterization.
- `/refactor/reorder`: preview, then reorder after explicit `go`.
- `/refactor/document`: repair scoped source documentation.
- `/refactor/errors`: trace and repair public error documentation.
- `/cleanup`: clean and review targets while preserving behavior.

### Documentation and review

- `/docs/write`: write scoped end-user documentation.
- `/docs/review`: review and repair scoped end-user documentation.
- `/review/coderabbit`: run CodeRabbit and apply scoped repairs.

### Repository maintenance

- `/commit/main`: create semantic commits with explicit staging.
- `/write/issue`: write a repository-grounded issue.
- `/write/pr`: generate grounded `pr.md` from the branch diff.
- `/iterate/edit`: discuss and edit or verify instruction artifacts.
- `/migrate`: run the separate pinned-source migration workflow.

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

- Setup offers cargo, bun, git, Node.js LTS, Yarn and Docker Desktop.
- Each gets a y/N winget prompt; cargo/bun/yarn have installer fallbacks.
- Failed or declined installs warn without stopping setup.
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
bash scripts/check-workflows.sh
```

Use `nix develop` when local Python lacks `json5` or `PyYAML`.
See the [validator docstring] for check scope.

Validation writes only an explicitly requested report, never configuration.
The shell smoke checks routing, pairs, cycles and path safety in temp fixtures.

- Credentials, plugins, CodeRabbit, and OpenCode need environment checks.
- Static checks cannot certify stochastic model behavior.

[OpenCode]: https://opencode.ai
[architecture and rationale]: EXPLAINER.md#architecture
[practical iterate guide]: .opencode/ITERATE.md
[validator docstring]: scripts/validate-opencode-config.py
