#!/usr/bin/env python3
"""OpenCode configuration validator.

Performs deterministic static checks without calling a model. Writes only the
optional path supplied with ``--report``.

Configuration documents:
- Parse active JSON/JSONC configuration and local Caveman plugin package data.
- Require positive tool output limits and a global external-directory policy of
  ask, allow, or a pattern map. Compaction pruning is optional.

Agent frontmatter and permissions:
- Validate IDs, YAML, modes, descriptions, provider-qualified models, permission
  decisions/defaults/order, environment-file denial, and external-directory
  policy of ask, allow, or a pattern map.
- Reject temperature, step-limit, and tools fields.

Commands and task graph:
- Validate command targets, entry points, routes, reachability, cycles, disabled
  agents, and maximum custom task depth of three edges.
- Write config.subagent_depth as (maximum custom task depth + 2) so nested
  Task calls never hit the runtime depth limit.

Prompt structure and imports:
- Check Markdown fences, output contracts, imports, import cycles, rule
  reachability and names, and imported support files.
- Check instruction format: per-line statement cap of 240 characters, em-dash
  ban, and soft split-suggestion warnings over 80 characters across prompt and
  rule Markdown.

Documentation and source syntax:
- Require README.md, EXPLAINER.md, and .opencode/ITERATE.md.
- Check their local links and Markdown anchors.
- Parse project Python and run ``bash -n`` on configured shell scripts.

Result:
- Print Markdown metrics, errors, and warnings. Exit 0 for PASS and 1 for FAIL.
- ``--report PATH`` writes the same report printed to stdout.

Outside scope:
- Nix evaluation/formatting, unit tests, builds, type checks, broader linters,
  OpenCode, plugins, external services, model review, and workflow semantics.
"""
from __future__ import annotations

import argparse
import ast
import re
import subprocess
import sys
from collections import deque
from pathlib import Path
from typing import Any
from urllib.parse import unquote

import json5
import yaml

IMPORT_RE = re.compile(r"\{\{\s*file\s*=\s*['\"]([^'\"]+)['\"]")
FENCE_RE = re.compile(r"^\s*(```|~~~)", re.MULTILINE)
AGENT_ROOTS = (Path("config/agent"), Path(".opencode/agent"))
COMMAND_ROOTS = (Path("config/command"), Path(".opencode/command"))
BUILTIN_AGENTS = {"build", "explore", "general", "plan"}
FORBIDDEN_AGENT_KEYS = {"temperature", "steps", "maxSteps", "tools"}
VALID_AGENT_MODES = {"primary", "subagent", "all"}
VALID_PERMISSION_DECISIONS = {"allow", "ask", "deny"}
CONFIG_PATH_FORMS = (
    "/home/sewer/nixos/users/sewer/home-manager/programs/opencode/**",
    "/home/sewer/opencode/**",
)
BUILTIN_AGENT_ALLOW_EXTERNAL = ("build", "plan")
MAX_CUSTOM_TASK_DEPTH = 3
REQUIRED_PATHS = (
    "README.md",
    "EXPLAINER.md",
    ".opencode/ITERATE.md",
)
LOCAL_LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
REFERENCE_LINK_RE = re.compile(r"(?m)^\[(?!\^)[^\]]+\]:\s*(\S+)")
INSTRUCTION_LIST_RE = re.compile(r"^\s*(?:[-*+]\s+|\d+\.\s+)")
INSTRUCTION_TABLE_RE = re.compile(r"^\s*\|")
INSTRUCTION_URL_RE = re.compile(r"^\s*(https?://\S+|<https?://[^>]+>)\s*$")
INSTRUCTION_TEMPLATE_RE = re.compile(r"^\s*\{\{")
INSTRUCTION_PARAGRAPH_CAP = 240
INSTRUCTION_LINE_TARGET = 80

def load_frontmatter(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---\n", 4)
    if end < 0:
        raise ValueError("frontmatter opens but does not close")
    data = yaml.safe_load(text[4:end])
    return data if isinstance(data, dict) else {}


def agent_id(root: Path, path: Path) -> str:
    return path.relative_to(root).with_suffix("").as_posix()


def resolve_import(repo: Path, current: Path, raw: str) -> Path | None:
    raw = raw.strip()
    candidates: list[Path] = []
    if current.is_relative_to(repo / "config"):
        prompt_root = repo / "config"
    else:
        prompt_root = repo
    if raw.startswith("./"):
        candidates.extend((prompt_root / raw[2:], repo / raw[2:]))
    else:
        candidates.extend((prompt_root / raw, repo / raw))
    candidates.append(current.parent / raw)
    for candidate in candidates:
        try:
            resolved = candidate.resolve()
        except OSError:
            continue
        if resolved.is_file():
            return resolved
    return None


def task_targets(frontmatter: dict[str, Any]) -> set[str]:
    permission = frontmatter.get("permission")
    if not isinstance(permission, dict):
        return set()
    task = permission.get("task")
    if not isinstance(task, dict):
        return set()
    return {
        str(name)
        for name, decision in task.items()
        if name != "*" and str(decision).lower() == "allow"
    }


def active_json_files(repo: Path) -> list[Path]:
    paths = [*sorted((repo / "config").glob("*.json")), *sorted((repo / "config").glob("*.jsonc"))]
    paths.extend(sorted((repo / ".opencode").rglob("*.json")))
    paths.extend(sorted((repo / ".opencode").rglob("*.jsonc")))
    local_plugin = repo / "config/plugins/caveman/package.json"
    if local_plugin.is_file():
        paths.append(local_plugin)
    return sorted(set(paths))


def active_prompt_files(repo: Path) -> list[Path]:
    paths: list[Path] = []
    for root in (
        repo / "config/agent",
        repo / "config/command",
        repo / "config/rules",
        repo / ".opencode/agent",
        repo / ".opencode/command",
    ):
        if root.exists():
            paths.extend(root.rglob("*.md"))
            paths.extend(root.rglob("*.txt"))
    return sorted(set(paths))


def instruction_format_issues(text: str) -> list[tuple[str, str]]:
    """Return ``(severity, message)`` instruction-format issues for one text.

    Each line is one statement; consecutive lines are never joined. Error: a
    non-exempt line whose statement (list marker excluded) exceeds 240
    characters. Error: any em dash. Warning: a non-exempt line over 80
    characters, suggesting a split into simpler separate statements. Exempt
    from all three rules: YAML frontmatter, fenced code content, table rows,
    URL-only lines, ``{{ ... }}`` template directive lines, and blank lines.
    """
    issues: list[tuple[str, str]] = []
    lines = text.split("\n")
    frontmatter_lines = 0
    if text.startswith("---\n"):
        end = text.find("\n---\n", 4)
        if end >= 0:
            frontmatter_lines = text[: end + 5].count("\n")

    def exempt(number: int, line: str) -> bool:
        if number <= frontmatter_lines:
            return True
        if not line.strip():
            return True
        return bool(
            INSTRUCTION_TABLE_RE.match(line)
            or INSTRUCTION_URL_RE.match(line)
            or INSTRUCTION_TEMPLATE_RE.match(line)
        )

    fenced = False
    for number, line in enumerate(lines, 1):
        if FENCE_RE.match(line):
            fenced = not fenced
            continue
        if fenced or exempt(number, line):
            continue
        if "—" in line:
            issues.append(("error", f"line {number}: em dash"))
        statement = INSTRUCTION_LIST_RE.sub("", line, count=1)
        if len(statement) > INSTRUCTION_PARAGRAPH_CAP:
            issues.append(
                ("error", f"line {number}: statement exceeds 240 characters ({len(statement)})")
            )
        elif len(line) > INSTRUCTION_LINE_TARGET:
            issues.append(
                (
                    "warning",
                    f"line {number}: {len(line)} characters; "
                    "split into simpler separate statements",
                )
            )
    return issues


def validate_config_path_pairing(ident: str, external: Any, errors: list[str]) -> None:
    """Both the physical config path and its symlink must be allowed together."""
    if not isinstance(external, dict):
        return
    for form in CONFIG_PATH_FORMS:
        others = [f for f in CONFIG_PATH_FORMS if f != form]
        if form in external and not any(o in external for o in others):
            errors.append(
                f"{ident} external_directory allows {form!r} without the other config path spelling"
            )


def validate_permission_map(ident: str, permission: Any, errors: list[str]) -> None:
    if not isinstance(permission, dict):
        errors.append(f"agent {ident} has no explicit permission mapping")
        return
    if str(permission.get("*", "")).lower() != "deny":
        errors.append(f"agent {ident} permission must start from top-level '*: deny'")
    valid = VALID_PERMISSION_DECISIONS
    for tool, rules in permission.items():
        if isinstance(rules, str):
            if rules.lower() not in valid:
                errors.append(f"agent {ident} permission {tool!r} has invalid decision {rules!r}")
            continue
        if not isinstance(rules, dict):
            errors.append(f"agent {ident} permission {tool!r} must be a decision or mapping")
            continue
        if "*" in rules and next(iter(rules)) != "*":
            errors.append(f"agent {ident} permission {tool!r} wildcard must be first; last match wins")
        for pattern, decision in rules.items():
            if str(decision).lower() not in valid:
                errors.append(
                    f"agent {ident} permission {tool!r} pattern {pattern!r} has invalid decision {decision!r}"
                )
    external = permission.get("external_directory")
    if external is not None and not isinstance(external, dict) and str(external).lower() not in {"ask", "allow"}:
        errors.append(f"agent {ident} external_directory must be ask, allow, or a pattern mapping")
    validate_config_path_pairing(f"agent {ident}", external, errors)
    read = permission.get("read")
    if not isinstance(read, dict):
        errors.append(f"agent {ident} read permission must be a mapping")
    else:
        for pattern in ("*.env", "*.env.*"):
            if str(read.get(pattern, "")).lower() != "deny":
                errors.append(f"agent {ident} read permission must deny {pattern!r}")
    task = permission.get("task")
    if isinstance(task, str) and task.lower() != "deny":
        errors.append(f"agent {ident} task permission may only use scalar 'deny'")
    elif isinstance(task, dict) and str(task.get("*", "")).lower() != "deny":
        errors.append(f"agent {ident} task permission must default to deny")


def markdown_anchors(path: Path) -> set[str]:
    anchors: set[str] = set()
    counts: dict[str, int] = {}
    fenced = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.lstrip().startswith(("```", "~~~")):
            fenced = not fenced
            continue
        if fenced or not line.startswith("#"):
            continue
        heading = line.lstrip("#").strip().lower()
        anchor = re.sub(r"[^a-z0-9 _-]", "", heading).replace(" ", "-")
        anchor = re.sub(r"-+", "-", anchor).strip("-")
        suffix = counts.get(anchor, 0)
        counts[anchor] = suffix + 1
        anchors.add(anchor if suffix == 0 else f"{anchor}-{suffix}")
    return anchors


def validate_doc_links(repo: Path, paths: list[Path], errors: list[str]) -> int:
    checked = 0
    anchors: dict[Path, set[str]] = {}
    for source in paths:
        body = source.read_text(encoding="utf-8")
        destinations = [*LOCAL_LINK_RE.findall(body), *REFERENCE_LINK_RE.findall(body)]
        for raw in destinations:
            destination = raw.strip().split(maxsplit=1)[0].strip("<>")
            if not destination or destination.startswith(("http://", "https://", "mailto:")):
                continue
            checked += 1
            target_raw, _, fragment = destination.partition("#")
            target = source if not target_raw else (source.parent / unquote(target_raw)).resolve()
            try:
                target.relative_to(repo)
            except ValueError:
                errors.append(f"{source.relative_to(repo)} has external local link {destination}")
                continue
            if not target.exists():
                errors.append(f"{source.relative_to(repo)} has stale link {destination}")
                continue
            if fragment and target.suffix.lower() == ".md":
                anchors.setdefault(target, markdown_anchors(target))
                if unquote(fragment).lower() not in anchors[target]:
                    errors.append(f"{source.relative_to(repo)} has stale heading link {destination}")
    return checked


def longest_depth(graph: dict[str, set[str]], roots: set[str]) -> tuple[int, list[str], list[list[str]]]:
    best_depth = 0
    best_path: list[str] = []
    cycles: list[list[str]] = []

    def visit(node: str, path: list[str]) -> None:
        nonlocal best_depth, best_path
        if node in path:
            cycles.append(path[path.index(node):] + [node])
            return
        new_path = path + [node]
        depth = len(new_path) - 1
        if depth > best_depth:
            best_depth = depth
            best_path = new_path
        for child in sorted(graph.get(node, set())):
            visit(child, new_path)

    for root in sorted(roots):
        visit(root, [])
    return best_depth, best_path, cycles


def find_import_cycles(graph: dict[Path, set[Path]]) -> list[list[Path]]:
    """Return unique directed cycles in a prompt-import graph."""
    state: dict[Path, int] = {}
    stack: list[Path] = []
    cycles: list[list[Path]] = []
    seen: set[tuple[str, ...]] = set()

    def visit(node: Path) -> None:
        marker = state.get(node, 0)
        if marker == 2:
            return
        if marker == 1:
            start = stack.index(node)
            cycle = stack[start:] + [node]
            names = [item.as_posix() for item in cycle[:-1]]
            rotations = [tuple(names[i:] + names[:i]) for i in range(len(names))]
            key = min(rotations) if rotations else tuple()
            if key not in seen:
                seen.add(key)
                cycles.append(cycle)
            return

        state[node] = 1
        stack.append(node)
        for child in sorted(graph.get(node, set())):
            visit(child)
        stack.pop()
        state[node] = 2

    nodes = set(graph) | {child for children in graph.values() for child in children}
    for node in sorted(nodes):
        visit(node)
    return cycles


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--report", help="Optional Markdown report path")
    args = parser.parse_args()

    repo = Path(args.repo_root).resolve()
    errors: list[str] = []
    warnings: list[str] = []
    details: list[str] = []

    config_path = repo / "config/opencode.json"
    try:
        config = json5.loads(config_path.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"cannot parse config/opencode.json as JSONC: {exc}")
        config = {}

    json_paths = active_json_files(repo)
    for path in json_paths:
        try:
            json5.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:
            errors.append(f"{path.relative_to(repo)} is invalid JSON/JSONC: {exc}")

    agent_files: dict[str, Path] = {}
    agent_frontmatter: dict[str, dict[str, Any]] = {}
    for rel_root in AGENT_ROOTS:
        root = repo / rel_root
        if not root.exists():
            continue
        for path in sorted(root.rglob("*.md")):
            if not path.read_text(encoding="utf-8").startswith("---\n"):
                continue
            ident = agent_id(root, path)
            if ident in agent_files:
                errors.append(f"duplicate agent id {ident}: {agent_files[ident]} and {path}")
                continue
            agent_files[ident] = path
            try:
                fm = load_frontmatter(path)
            except Exception as exc:
                errors.append(f"{path.relative_to(repo)}: {exc}")
                fm = {}
            agent_frontmatter[ident] = fm
            forbidden = sorted(FORBIDDEN_AGENT_KEYS.intersection(fm))
            if forbidden:
                errors.append(
                    f"{path.relative_to(repo)} sets forbidden agent field(s): {', '.join(forbidden)}"
                )
            mode = fm.get("mode")
            if mode not in VALID_AGENT_MODES:
                errors.append(f"{path.relative_to(repo)} has invalid agent mode {mode!r}")
            description = fm.get("description")
            if not isinstance(description, str) or not description.strip():
                errors.append(f"{path.relative_to(repo)} has no agent description")
            model = fm.get("model")
            if isinstance(model, str) and "/" in model:
                provider = model.split("/", 1)[0]
                if provider not in (config.get("provider") or {}):
                    errors.append(f"{path.relative_to(repo)} uses undeclared model provider {provider}")
            validate_permission_map(ident, fm.get("permission"), errors)

    command_roots: set[str] = set()
    command_count = 0
    for rel_root in COMMAND_ROOTS:
        root = repo / rel_root
        if not root.exists():
            continue
        for path in sorted(root.rglob("*.md")):
            command_count += 1
            try:
                fm = load_frontmatter(path)
            except Exception as exc:
                errors.append(f"{path.relative_to(repo)}: {exc}")
                continue
            description = fm.get("description")
            if not isinstance(description, str) or not description.strip():
                errors.append(f"{path.relative_to(repo)} has no command description")
            # Agent-less commands run in-context under the current agent/model
            # (e.g. /commit/current) and contribute no task-graph root.
            target = fm.get("agent")
            if target is not None and not isinstance(target, str):
                errors.append(f"{path.relative_to(repo)} agent target must be a string")
            elif isinstance(target, str) and target:
                if target not in agent_files and target not in BUILTIN_AGENTS:
                    errors.append(f"{path.relative_to(repo)} targets missing agent {target}")
                elif target in agent_files:
                    if agent_frontmatter[target].get("mode") == "subagent":
                        errors.append(f"{path.relative_to(repo)} targets subagent-only agent {target}")
                    command_roots.add(target)

    graph: dict[str, set[str]] = {ident: set() for ident in agent_files}
    for ident, fm in agent_frontmatter.items():
        for target in task_targets(fm):
            if target in agent_files:
                graph[ident].add(target)
            elif target not in BUILTIN_AGENTS:
                errors.append(f"agent {ident} allows missing task target {target}")

    disabled_agents = {
        ident for ident, fm in agent_frontmatter.items() if fm.get("disable") is True
    }
    for ident, children in graph.items():
        for child in children & disabled_agents:
            errors.append(f"agent {ident} routes to disabled agent {child}")

    # Built-in agent task permissions are additional entry points into custom subagents.
    for builtin_cfg in (config.get("agent") or {}).values() if isinstance(config.get("agent"), dict) else []:
        if not isinstance(builtin_cfg, dict):
            continue
        for target in task_targets(builtin_cfg):
            if target in agent_files:
                command_roots.add(target)
            elif target not in BUILTIN_AGENTS:
                errors.append(f"built-in agent config allows missing task target {target}")

    reachable: set[str] = set()
    queue = deque(sorted(command_roots))
    while queue:
        node = queue.popleft()
        if node in reachable:
            continue
        reachable.add(node)
        queue.extend(sorted(graph.get(node, set()) - reachable))
    orphans = sorted(set(agent_files) - reachable)
    if orphans:
        errors.append("unreachable custom agents: " + ", ".join(orphans))

    max_depth, max_path, cycles = longest_depth(graph, command_roots)
    for cycle in cycles:
        errors.append("custom task cycle: " + " -> ".join(cycle))
    if max_depth > MAX_CUSTOM_TASK_DEPTH:
        errors.append(f"custom task depth {max_depth} exceeds policy maximum {MAX_CUSTOM_TASK_DEPTH}")
    details.append(
        "Maximum custom task depth: "
        + str(max_depth)
        + (" (" + " -> ".join(max_path) + ")" if max_path else "")
    )

    required_subagent_depth = max_depth + 2
    if config.get("subagent_depth") != required_subagent_depth:
        raw = config_path.read_text(encoding="utf-8")
        match = re.search(r'"subagent_depth"\s*:\s*\d+', raw)
        if match:
            updated = raw[: match.start()] + f'"subagent_depth": {required_subagent_depth}' + raw[match.end() :]
        else:
            updated = raw.replace(
                '"autoupdate": false,',
                f'"autoupdate": false,\n  "subagent_depth": {required_subagent_depth},',
                1,
            )
        config_path.write_text(updated, encoding="utf-8")
        config["subagent_depth"] = required_subagent_depth
    details.append(f"config.subagent_depth: {required_subagent_depth}")

    all_prompt_files = active_prompt_files(repo)
    imported_targets: set[Path] = set()
    import_edges: dict[Path, set[Path]] = {}
    import_count = 0
    for path in all_prompt_files:
        text = path.read_text(encoding="utf-8")
        fences = FENCE_RE.findall(text)
        if len(fences) % 2:
            errors.append(f"{path.relative_to(repo)} has an unbalanced Markdown fence")
        if path.is_relative_to(repo / "config/agent") or path.is_relative_to(repo / ".opencode/agent"):
            if text.startswith("---\n") and not re.search(r"(?mi)^#{1,3}\s+(output|result)\b|return (?:exactly|only):", text):
                errors.append(f"{path.relative_to(repo)} has no explicit output/result contract")
        edges = import_edges.setdefault(path.resolve(), set())
        for raw in IMPORT_RE.findall(text):
            import_count += 1
            resolved = resolve_import(repo, path, raw)
            if resolved is None:
                errors.append(f"{path.relative_to(repo)} has unresolved import {raw}")
            else:
                resolved = resolved.resolve()
                imported_targets.add(resolved)
                edges.add(resolved)

    for cycle in find_import_cycles(import_edges):
        errors.append(
            "prompt import cycle: "
            + " -> ".join(str(path.relative_to(repo)) for path in cycle)
        )

    # Rule cards/groups are runtime modules. Every module must be imported by
    # an active frontmatter agent or command.
    runtime_roots = {
        path.resolve()
        for rel_root in (*AGENT_ROOTS, *COMMAND_ROOTS)
        for path in (repo / rel_root).rglob("*.md")
        if path.read_text(encoding="utf-8").startswith("---\n")
    }
    reachable_imports: set[Path] = set()
    import_queue = deque(sorted(runtime_roots))
    while import_queue:
        path = import_queue.popleft()
        if path in reachable_imports:
            continue
        reachable_imports.add(path)
        import_queue.extend(sorted(import_edges.get(path, set()) - reachable_imports))
    for rule_root in (repo / "config/rules/cards", repo / "config/rules/groups"):
        for path in sorted(rule_root.rglob("*.md")):
            if path.resolve() not in reachable_imports:
                errors.append(f"unreachable rule module: {path.relative_to(repo)}")
    prefixed_groups = sorted((repo / "config/rules/groups").rglob("target-*.md"))
    if prefixed_groups:
        errors.append(
            "rule group filenames use obsolete target- prefix: "
            + ", ".join(path.relative_to(repo).as_posix() for path in prefixed_groups)
        )

    for path in sorted((repo / "config/agent").rglob("*")):
        if path.is_file() and path.suffix != ".md" and path.resolve() not in imported_targets:
            errors.append(f"unreferenced prompt support file: {path.relative_to(repo)}")

    instruction_targets = sorted({*all_prompt_files, *(repo / ".opencode/rules").glob("*.md")})
    instruction_format_error_count = 0
    instruction_format_warning_count = 0
    for path in instruction_targets:
        for severity, message in instruction_format_issues(path.read_text(encoding="utf-8")):
            entry = f"{path.relative_to(repo)}: {message}"
            if severity == "error":
                instruction_format_error_count += 1
                errors.append(entry)
            else:
                instruction_format_warning_count += 1
                warnings.append(entry)
    details.append(
        f"Instruction format: {instruction_format_error_count} statement error(s), "
        f"{instruction_format_warning_count} split-suggestion warning(s)"
    )

    for part in REQUIRED_PATHS:
        if not (repo / part).is_file():
            errors.append(f"required documentation is missing: {part}")

    documentation = [repo / "README.md", repo / "EXPLAINER.md", repo / ".opencode/ITERATE.md"]
    link_count = validate_doc_links(repo, [path for path in documentation if path.is_file()], errors)

    python_paths = sorted((repo / ".opencode").rglob("*.py")) + sorted((repo / "scripts").rglob("*.py")) + sorted(
        (repo / "tests").rglob("*.py")
    )
    for path in python_paths:
        try:
            ast.parse(path.read_text(encoding="utf-8"), filename=str(path.relative_to(repo)))
        except SyntaxError as exc:
            errors.append(f"{path.relative_to(repo)} has Python syntax error: {exc}")

    shell_paths = [
        repo / ".githooks/pre-commit",
        *sorted((repo / "scripts").rglob("*.sh")),
        *sorted((repo / "tools").glob("*.sh")),
    ]
    for path in shell_paths:
        if not path.is_file():
            continue
        result = subprocess.run(["bash", "-n", str(path)], text=True, capture_output=True, check=False)
        if result.returncode:
            errors.append(f"{path.relative_to(repo)} has shell syntax error: {result.stderr.strip()}")

    tool_output = config.get("tool_output")
    if not isinstance(tool_output, dict):
        errors.append("config.tool_output is missing")
    else:
        if not isinstance(tool_output.get("max_lines"), int) or tool_output["max_lines"] <= 0:
            errors.append("config.tool_output.max_lines must be a positive integer")
        if not isinstance(tool_output.get("max_bytes"), int) or tool_output["max_bytes"] <= 0:
            errors.append("config.tool_output.max_bytes must be a positive integer")
    # Compaction pruning is optional by policy; prune:false preserves old
    # tool-call contents in context, so it is not mandated here.
    permission = config.get("permission")
    external = permission.get("external_directory") if isinstance(permission, dict) else None
    if not isinstance(permission, dict) or (
        not isinstance(external, dict) and str(external).lower() not in {"ask", "allow"}
    ):
        errors.append("config.permission.external_directory must be ask, allow, or a pattern mapping")
    validate_config_path_pairing("config", external, errors)
    if isinstance(permission, dict):
        for tool, rules in permission.items():
            if not isinstance(rules, dict):
                continue
            for pattern, decision in rules.items():
                if str(decision).lower() not in VALID_PERMISSION_DECISIONS:
                    errors.append(
                        f"config.permission {tool!r} pattern {pattern!r} has invalid decision {decision!r}"
                    )

    # Default modes (build, plan) run user-driven, so they inherit a global
    # allow policy instead of the ask-everywhere default for subagents.
    agents = config.get("agent")
    if not isinstance(agents, dict):
        errors.append("config.agent must be a mapping")
    else:
        for ident in BUILTIN_AGENT_ALLOW_EXTERNAL:
            agent_perm = agents.get(ident, {}).get("permission") if isinstance(agents.get(ident), dict) else None
            got = agent_perm.get("external_directory") if isinstance(agent_perm, dict) else None
            if str(got).lower() != "allow":
                errors.append(
                    f"config.agent.{ident}.permission.external_directory must be allow (default mode)"
                )

    details.extend(
        [
            f"JSON/JSONC documents: {len(json_paths)}",
            f"Custom agents: {len(agent_files)}",
            f"Commands: {command_count}",
            f"Prompt imports: {import_count}",
            f"Reachable custom agents: {len(reachable)}/{len(agent_files)}",
            f"Documentation links checked: {link_count}",
            f"Forbidden temperature/step fields: 0" if not any("forbidden agent field" in e for e in errors) else "Forbidden temperature/step fields: present",
        ]
    )

    decision = "PASS" if not errors else "FAIL"
    lines = [
        "# OpenCode configuration validation",
        f"Decision: {decision}",
        "",
        "## Summary",
        *[f"- {item}" for item in details],
        "",
        "## Errors",
        *([f"- {item}" for item in errors] or ["- None"]),
        "",
        "## Warnings",
        *([f"- {item}" for item in warnings] or ["- None"]),
        "",
    ]
    report = "\n".join(lines)
    if args.report:
        report_path = Path(args.report)
        if not report_path.is_absolute():
            report_path = repo / report_path
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(report, encoding="utf-8")
    print(report, end="")
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
