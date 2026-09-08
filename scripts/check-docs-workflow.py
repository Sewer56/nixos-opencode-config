#!/usr/bin/env python3
"""Check documentation-review topology, permissions and textual wiring.

Uses the config validator's parser and import resolver without writing files.
Permission probes model last-match wildcard rules, not tool execution.
Text assertions check wiring only, not model routing or parallel dispatch.
"""

from __future__ import annotations

import re
import runpy
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = runpy.run_path(str(ROOT / "scripts/validate-opencode-config.py"))
EDITORIAL = "_docs/reviewers/editorial"
CALLERS = ("_implement/cohort", "_implement/one-shot", "_implement", "code")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def matches(value: str, pattern: str) -> bool:
    """Model documented '*' and '?' wildcards, including path separators."""
    regex = re.escape(pattern).replace(r"\*", ".*").replace(r"\?", ".")
    return re.fullmatch(regex, value, re.DOTALL) is not None


def decision(permission: dict, tool: str, value: str) -> str:
    result = "ask"
    for action, rules in permission.items():
        if not matches(tool, action):
            continue
        if isinstance(rules, str):
            result = rules
            continue
        for pattern, effect in rules.items():
            if matches(value, pattern):
                result = effect
    return result


def imports(path: Path, ancestors: tuple[Path, ...] = ()) -> set[Path]:
    require(path not in ancestors, f"import cycle: {path}")
    found = {path}
    for raw in VALIDATOR["IMPORT_RE"].findall(path.read_text()):
        target = VALIDATOR["resolve_import"](ROOT, path, raw)
        require(target is not None, f"unresolved import: {path}: {raw}")
        found.update(imports(target, (*ancestors, path)))
    return found


def check_topology(agents: dict) -> None:
    reviewer = agents[EDITORIAL]
    require(reviewer.get("hidden") is True, "editorial must be hidden")
    require(reviewer.get("mode") == "subagent", "editorial must be a subagent")
    require(not reviewer.get("disable"), "editorial must be enabled")

    for caller in CALLERS:
        targets = VALIDATOR["task_targets"](agents[caller])
        require(EDITORIAL in targets, f"unreachable editorial from {caller}")
        require("_review/verifier" in targets, f"missing verifier: {caller}")
        require(not any(t.startswith("_docs/") and t != EDITORIAL for t in targets),
                f"unexpected documentation delegate: {caller}")

    sources = imports(ROOT / f"config/agent/{EDITORIAL}.md")
    required = (
        "groups/style/wording.md", "cards/style/adhd-format.md",
        "cards/docs/error-documentation.md", "groups/docs/end-user-correctness.md",
        "groups/implementation/review-findings.md",
        "groups/implementation/implementation-review.md",
        "cards/implementation/review-protocol.md",
        "cards/structure/writable-surface.md",
    )
    for rule in required:
        require(ROOT / f"config/rules/{rule}" in sources, f"missing rule: {rule}")
    require(ROOT / "config/rules/groups/docs/code-docs.md" not in sources,
            "editorial imports code-body grouping")
    require(all(p == ROOT / f"config/agent/{EDITORIAL}.md"
                or p.is_relative_to(ROOT / "config/rules") for p in sources),
            "editorial imports a standalone workflow")

    for caller in CALLERS:
        require(ROOT / "config/rules/groups/implementation/implementation-review.md"
                in imports(ROOT / f"config/agent/{caller}.md"),
                f"caller lacks shared transport: {caller}")
    print("PASS parsed topology: four caller edges, verifier edges and imports")


def check_permissions(permission: dict) -> None:
    require(permission.get("*") == "deny", "reviewer must default deny")
    require(permission.get("task") == "deny", "reviewer must not delegate")
    require(permission.get("grep") == "deny", "content search must be denied")
    require(set(permission) <= {
        "*", "external_directory", "read", "edit", "grep", "glob", "list",
        "bash", "task",
    }, "unexpected reviewer tool grant")
    expected_outputs = {
        "*": "deny",
        "artifact/review/CODE-*/editorial/*.review.md": "allow",
        "artifact/review/ONESHOT-*/editorial/*.review.md": "allow",
        "artifact/plan/*/review/editorial/*.review.md": "allow",
    }
    require(permission.get("edit") == expected_outputs, "output grants widened")
    for path in (
        "src/lib.rs", "README.md", ".git/index", "pr.md",
        "artifact/review/CODE-demo/quality/r01.quality.review.md",
        "artifact/review/ONESHOT-demo/r01.quick.validation.md",
        "artifact/plan/PROMPT-PLAN-demo/01-first.md",
    ):
        require(decision(permission, "edit", path) == "deny", f"product write: {path}")

    for path in (".env", "src/.env.local", "/home/sewer/.secrets/key",
                 "/home/sewer/projects/nixos-secrets/key",
                 "/home/sewer/.config/gh/hosts.yml",
                 "/home/sewer/.config/app/credentials.json",
                 "/home/sewer/.local/share/opencode/auth.json"):
        require(decision(permission, "read", path) == "deny", f"secret read: {path}")
    # Grep authorizes regex patterns, not include globs or matched read paths.
    # The verifier's include="credentials.json" counterexample asks for ".".
    for pattern in (".", ".*", "token", "password"):
        require(decision(permission, "grep", pattern) == "deny",
                f"content-search bypass: {pattern}")
    for path in ("README.md", "src/lib.rs"):
        require(decision(permission, "read", path) == "allow",
                f"scoped read denied: {path}")
    for command in (
        "git add README.md", "git commit -m bad", "git reset --hard",
        "git diff --output=README.md", "git show HEAD:.env",
        "git diff --no-ext-diff --no-textconv HEAD -- .env",
        "git diff --no-ext-diff --no-textconv --ext-diff HEAD",
        "git diff --no-ext-diff --no-textconv --no-index /etc/passwd README.md",
        "git status --short > README.md", "git status --short; touch README.md",
        "git status --short\ntouch README.md", "git status $(touch README.md)",
        "python3 scripts/check-docs-workflow.py", "rust-llm-tidy README.md",
        "touch README.md", "cat .env",
    ):
        require(decision(permission, "bash", command) == "deny", f"unsafe bash: {command}")
    for command in (
        "git diff --no-ext-diff --no-textconv --cached HEAD -- README.md",
        "git show --no-ext-diff --no-textconv HEAD:README.md",
        "git status --short", "git rev-parse HEAD",
    ):
        require(decision(permission, "bash", command) == "allow", f"blocked read: {command}")
    print("PASS permission-model probes: report-only edits, grep denial, reads and shell")


def check_output_transport(permission: dict) -> None:
    """Render actual caller templates with fixture values, then probe grants."""
    card = text("config/rules/cards/implementation/artifact-paths.md")
    plan_template = re.search(r"`review_path`: `([^`]+)`", card)
    require(plan_template is not None, "missing plan review-path template")
    one_shot = text("config/agent/_implement/one-shot.md")
    standalone_template = re.search(r"`review_path = ([^`]+)`", one_shot)
    require(standalone_template is not None, "missing one-shot review-path template")
    code = text("config/agent/code.md")
    code_template = re.search(r"Assign `([^`]+\.review\.md)`", code)
    require(code_template is not None, "missing code review-path assignment")

    fixtures = (
        (plan_template[1], "artifact/plan/PROMPT-PLAN-demo/review", "C01"),
        (plan_template[1], "artifact/plan/PROMPT-PLAN-demo/review", "final"),
        (standalone_template[1], "artifact/review/ONESHOT-demo", "C01"),
        (code_template[1], "artifact/review/CODE-demo", "C01"),
    )
    outputs = []
    for template, directory, scope in fixtures:
        output = template
        for key, value in {
            "[[run_prefix]]": directory, "[[review_dir]]": directory,
            "[[run_id]]": "20260908T000000Z", "<reviewer>": "editorial",
            "<domain>": "editorial", "[[domain]]": "editorial",
            "Cnn": scope, "rNN": "r01",
        }.items():
            output = output.replace(key, value)
        require(decision(permission, "edit", output) == "allow",
                f"caller output denied: {output}")
        outputs.append(output)
    require(len(set(outputs)) == 4, "caller outputs collide")
    print("PASS parsed output templates: task, final, one-shot and code grants")


def check_text_wiring() -> None:
    """These assertions cannot prove policy compliance or live execution."""
    for path in (ROOT / "config").rglob("*.md"):
        if not (path.is_relative_to(ROOT / "config/agent")
                or path.is_relative_to(ROOT / "config/rules")):
            continue
        body = path.read_text()
        require("_docs/editor" not in body and "<editor-inputs>" not in body,
                f"obsolete editor wiring: {path}")
        require('file="./rules/cards/docs/editorial.md"' not in body,
                f"obsolete shared card: {path}")
    for obsolete in ("config/agent/_docs/editor.md", "config/rules/cards/docs/editorial.md"):
        require(not (ROOT / obsolete).exists(), f"obsolete file remains: {obsolete}")

    expectations = {
        "config/rules/groups/implementation/implementation-review.md": (
            "authority_paths", "base_commit", "head_commit", "validation_path",
            "prior_verdict_paths[]", "review_path", "cwd", "TASK:", "FINAL",
            "STANDALONE", "STAGED", "COMMITTED", "Apply feasible accepted advisories",
        ),
        "config/rules/groups/implementation/cohort-planning.md": (
            "sole writer, including docs", "behavior, tests and required docs",
            "Never split feature docs into steps or cohorts",
            "independent documentation requests",
        ),
        f"config/agent/{EDITORIAL}.md": (
            "CHANGE", "TASK", "STANDALONE", "FINAL", "STAGED", "COMMITTED",
            "Each finding needs location and exact `Before:`/`After:` text.",
            "Deletions use `After: DELETE`.",
            "Insertions use `Before: EMPTY` with an exact anchor and before/after placement.",
            "Keep concise reader consequences outside edits.",
            "No vague or whole-document rewrites.",
            "Preserve source delimiters, indentation, directives and doctest behavior.",
            "EDT-NNN",
            "Never mutate Git", "independent checks", "frozen regions",
            "behavioral bookkeeping", "required contract", "Keep frequency details",
        ),
        "config/agent/code.md": (
            "By default, write no review artifacts and make no delegations",
            "explicit user request for review", "not extension", "limited named-reviewer",
            "including refactors", "Runnable examples need correctness even in Markdown",
            "permissions", "dependency trust",
            "newly required", "five repair turns", "Require explicit user request to commit",
        ),
        "config/agent/_implement.md": (
            "Never edit code", "cumulative boundary, including staged repairs",
            "valid prior editorial reuse", "unchanged text/claims",
            "Modified Paths", "Section 3 steps 3–8", "two final repair turns",
        ),
        "config/agent/_implement/cohort/review/quality.md": (
            "fidelity", "complete reachable error conditions", "defer duplicate editorial",
        ),
    }
    for caller in CALLERS:
        path = f"config/agent/{caller}.md"
        expectations[path] = (*expectations.get(path, ()), "parallel", "stable diff",
                              "newly required", "INCOMPLETE", "verdict_path")
    for caller in CALLERS[:2]:
        path = f"config/agent/{caller}.md"
        expectations[path] += (
            "sole code/tests/docs writer", "docs/comments or public-behavior docs",
            "Honor explicit reviewer requests", "selection/skip reasons",
            "validation_path", "quick", "repair_turn_limit",
        )
    for path, fragments in expectations.items():
        body = text(path)
        for fragment in fragments:
            require(fragment in body, f"text wiring missing in {path}: {fragment}")
    require("unless docs-only" not in text("config/agent/code.md"),
            "code restored blanket performance routing")
    print("PASS text-wiring assertions: ownership, boundaries, routing and repair")
    print("LIMIT: no model routing, parallel dispatch, symlink or live tool enforcement tested")


def main() -> int:
    try:
        agent_root = ROOT / "config/agent"
        agents = {
            VALIDATOR["agent_id"](agent_root, path): VALIDATOR["load_frontmatter"](path)
            for path in agent_root.rglob("*.md")
        }
        check_topology(agents)
        permission = agents[EDITORIAL]["permission"]
        check_permissions(permission)
        check_output_transport(permission)
        check_text_wiring()
    except (ValueError, KeyError, OSError) as error:
        print(f"FAIL: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
