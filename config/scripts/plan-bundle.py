#!/usr/bin/env python3
"""Read-only plan routing and path checks, not semantic approval.

Usage: plan-bundle.py --repo-root REPO ROOT [--prospective PATH ...]
ROOT is a Git-root PROMPT-PLAN-<slug>.draft.md, absolute or repo-relative.
Members live in artifact/plan/<root basename without .draft.md>/.
Root Markdown links name execution.md and every human task brief.
execution.md contains exactly one fenced plan-tasks block with rows:
    01 01-name.md -
    02 02-name.md 01
Columns are task ID, brief filename, and comma-separated prerequisite IDs
or '-' for no prerequisites. Each brief links its matching NN-name.exec.md.
These are the only source members. review/ is evidence, never authority.
Other Markdown links are references, not membership declarations.
References may navigate to repository-contained parents; member/write paths
remain traversal-free. Both lexical and symlink-resolved paths are checked.

--prospective checks proposed root/member destinations before creation.
It checks paths only and does not certify an existing bundle or readiness.
Output is JSON on stdout; exit 0 means mechanics pass, 1 means rejection.
No files, Git state, bytecode, or caches are written.
"""
from __future__ import annotations

import argparse
import json
import os
import re
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT_NAME = re.compile(r"PROMPT-PLAN-[A-Za-z0-9][A-Za-z0-9_-]*\.draft\.md\Z")
BRIEF = re.compile(r"([0-9]{2,})-[a-z0-9][a-z0-9-]*\.md\Z")
LINK = re.compile(r"(?<!!)\[[^\]\n]+\]\(([^\n)]*)\)")


def require(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def safe_path(base: Path, raw: str, boundary: Path) -> Path:
    """Reject lexical and resolved escapes, including prospective symlinks."""
    require(bool(raw) and not any(ord(c) < 32 for c in raw), "invalid path")
    require("\\" not in raw, f"backslash path: {raw}")
    path = Path(raw)
    require(not path.is_absolute() and ".." not in path.parts,
            f"absolute/traversal path: {raw}")
    lexical = base / path
    resolved = lexical.resolve()
    require(lexical.is_relative_to(boundary) and resolved.is_relative_to(boundary),
            f"path escape: {raw}")
    return resolved


def document(path: Path) -> tuple[str, list[str]]:
    require(path.is_file(), f"missing file: {path}")
    text = path.read_text(encoding="utf-8")
    # Fenced examples do not declare source membership.
    lines: list[str] = []
    routes: list[str] = []
    block: list[str] | None = None
    fence = ""
    for line in text.splitlines():
        match = re.match(r"^\s*(`{3,}|~{3,})(.*)$", line)
        if match and not fence:
            fence = match[1]
            if match[2] == "plan-tasks":
                require(line == "```plan-tasks", "routing needs ```plan-tasks")
                block = []
        elif fence and re.fullmatch(re.escape(fence) + r"\s*", line.strip()):
            if block is not None:
                routes.append("\n".join(block))
                block = None
            fence = ""
        elif block is not None:
            block.append(line)
        elif not fence:
            lines.append(line)
    require(not fence, f"unclosed fence: {path}")
    return "\n".join(lines), routes


def anchors(path: Path) -> set[str]:
    found: set[str] = set()
    counts: dict[str, int] = {}
    for title in re.findall(r"(?m)^#{1,6} +(.+?) *#* *$", document(path)[0]):
        slug = re.sub(r"[^\w -]", "", title.lower()).replace(" ", "-")
        n = counts.get(slug, 0)
        counts[slug] = n + 1
        found.add(slug if not n else f"{slug}-{n}")
    return found


def reference_path(base: Path, raw: str, repo: Path) -> Path:
    """Normalize read-only references without relaxing member/write paths."""
    require(not Path(raw).is_absolute() and "\\" not in raw,
            f"absolute/backslash reference: {raw}")
    require(not any(ord(c) < 32 for c in raw), "invalid reference")
    lexical = Path(os.path.normpath(base / raw))
    resolved = (base / raw).resolve()
    require(lexical.is_relative_to(repo) and resolved.is_relative_to(repo),
            f"reference escape: {raw}")
    require(resolved == lexical, f"aliased reference: {raw}")
    return resolved


def links(path: Path, repo: Path, members: Path) -> list[Path]:
    result: list[Path] = []
    text = document(path)[0]
    require(not re.search(r"(?m)^\s*\[[^\]]+\]:", text),
            f"use inline Markdown links: {path}")
    for raw in LINK.findall(text):
        require(bool(raw) and not re.search(r"\s", raw),
                f"encode spaces and omit link titles: {raw}")
        parts = urlsplit(raw.strip("<>"))
        if parts.scheme in {"https", "http", "mailto"}:
            continue
        require(not parts.scheme and not parts.netloc and not parts.query,
                f"unsupported link: {raw}")
        local = unquote(parts.path)
        target = path if not local else reference_path(path.parent, local, repo)
        if target.is_relative_to(members) and not target.is_relative_to(members / "review"):
            # Source-member links remain strict even when a detour resolves back.
            boundary = members if path.parent == members else repo
            safe_path(path.parent, local or path.name, boundary)
        require(target.is_file(), f"missing link: {raw} in {path}")
        if parts.fragment:
            require(target.suffix == ".md" and unquote(parts.fragment) in anchors(target),
                    f"missing anchor: {raw} in {path}")
        result.append(target)
    return result


def check(repo: Path, root_arg: str, proposed: list[str] | None) -> dict:
    repo = repo.resolve(strict=True)
    require(repo.is_dir(), "repository root is not a directory")
    supplied = Path(root_arg)
    if supplied.is_absolute():
        require(".." not in supplied.parts and supplied.parent.resolve() == repo,
                "root must be at repository root")
        root_arg = supplied.name
    require(bool(ROOT_NAME.fullmatch(root_arg)), "unsupported plan root name")
    root = safe_path(repo, root_arg, repo)
    require(root.parent == repo and root.name == root_arg, "aliased plan root")
    member_rel = f"artifact/plan/{root_arg.removesuffix('.draft.md')}"
    members = safe_path(repo, member_rel, repo)
    require(members == repo / member_rel, "aliased member directory")

    def member(raw: str) -> Path:
        target = safe_path(members, raw, members)
        require(target == members / raw, f"aliased source member: {raw}")
        return target

    if proposed is not None:
        require(bool(proposed), "supply every prospective destination")
        checked = set()
        for raw in proposed:
            p = Path(raw)
            if p.is_absolute():
                require(p.is_relative_to(repo), "prospective escape")
                raw = p.relative_to(repo).as_posix()
            target = safe_path(repo, raw, repo)
            require(target == repo / raw, "aliased prospective destination")
            if target != root:
                require(target.parent == members, "prospective member escape")
                name = target.name
                if name.endswith(".exec.md"):
                    name = name.removesuffix(".exec.md") + ".md"
                require(target.name == "execution.md" or bool(BRIEF.fullmatch(name)),
                        "unsupported prospective authority")
                require(target.name == "execution.md" or int(BRIEF.fullmatch(name)[1]) > 0,
                        "invalid prospective task ID")
            require(not target.exists() or target.is_file(), "destination is not a file")
            require(target not in checked, "duplicate prospective destination")
            checked.add(target)
        return {"mode": "prospective", "paths": sorted(str(p) for p in checked)}

    execution = member("execution.md")
    root_links = links(root, repo, members)
    require(root_links.count(execution) == 1, "root must link execution.md once")
    blocks = document(execution)[1]
    require(len(blocks) == 1, "require one plan-tasks routing block")
    tasks: dict[str, dict] = {}
    sources = {root, execution}
    for row in blocks[0].splitlines():
        columns = row.split()
        require(len(columns) == 3, f"invalid routing row: {row}")
        ident, filename, after = columns
        match = BRIEF.fullmatch(filename)
        require(bool(match) and match[1] == ident and int(ident) > 0,
                f"invalid task pair: {row}")
        require(ident not in tasks and all(int(ident) != int(i) for i in tasks),
                f"duplicate/ambiguous task ID: {ident}")
        brief = member(filename)
        machine = member(filename.removesuffix(".md") + ".exec.md")
        require(root_links.count(brief) == 1, f"missing/duplicate root task: {ident}")
        brief_links = links(brief, repo, members)
        exec_links = [p for p in brief_links if p.name.endswith(".exec.md")]
        require(exec_links == [machine], f"missing/ambiguous exec pair: {ident}")
        deps = [] if after == "-" else after.split(",")
        require(len(deps) == len(set(deps)), f"duplicate dependencies: {ident}")
        tasks[ident] = {"brief": str(brief), "exec": str(machine), "after": deps}
        sources.update((brief, machine))
    require(bool(tasks), "missing tasks")
    for ident, task in tasks.items():
        require(all(dep in tasks for dep in task["after"]),
                f"unknown dependency: {ident}")
    order: list[str] = []
    active: set[str] = set()

    def visit(ident: str) -> None:
        require(ident not in active, f"dependency cycle: {ident}")
        if ident in order:
            return
        active.add(ident)
        for dep in tasks[ident]["after"]:
            visit(dep)
        active.remove(ident)
        order.append(ident)

    for ident in tasks:
        visit(ident)
    # Only fixed source members are authoritative; reviews are never traversed.
    for child in members.iterdir():
        if child.name == "review":
            require(child.resolve().is_relative_to(members), "review path escape")
            require(child.is_dir(), "review must be an evidence directory")
            continue
        require(child in sources and child.resolve() == child and child.is_file(),
                f"extra authority/member: {child}")
    for source in sources:
        require(source == execution or not document(source)[1],
                f"extra routing authority: {source}")
        for target in links(source, repo, members):
            if source == root and (BRIEF.fullmatch(target.name)
                                   or target.name.endswith(".exec.md")):
                require(target in sources, f"extra task authority: {target}")
            if target.is_relative_to(members):
                require(target in sources or target.is_relative_to(members / "review"),
                        f"undeclared member: {target}")
            elif target != root:
                require(not ROOT_NAME.fullmatch(target.name), "extra plan authority")
    return {"mode": "bundle", "root": str(root), "execution": str(execution),
            "tasks": tasks, "order": order, "sources": sorted(str(p) for p in sources)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("root")
    parser.add_argument("--prospective", nargs="+")
    args = parser.parse_args()
    try:
        result = check(Path(args.repo_root), args.root, args.prospective)
    except (OSError, ValueError, RuntimeError, RecursionError) as exc:
        print(json.dumps({"status": "FAIL", "error": str(exc)}))
        return 1
    print(json.dumps({"status": "PASS", **result}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
