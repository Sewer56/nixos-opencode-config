#!/usr/bin/env python3
"""
Usage:
    iterate-static-check.py <artifact_base>

Runs deterministic checks for a direct /iterate/edit run and writes
<artifact_base>.static-check.md plus the result on stdout.

Lints ( BLOCKING when found ):
- Import target        : {{ file="...opencode-source/..." }} → use local config/ path
- Missing frontmatter  : Agent/command .md does not start with '---' on line 1
- Unresolved @agent    : @audit/collector with no config/agent/audit/collector.md
- Render failure       : bun cli.ts render exits non-zero for an agent/command file
- Trailing whitespace  : Rendered output lines end with spaces or tabs
- Double blank lines   : Consecutive empty lines in rendered output
- Unclosed fence       : ``` or ~~~ opened in rendered output but never closed
- Git whitespace       : git diff --check reports trailing whitespace or conflict markers
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Tuple


def generate_id() -> str:
    """Generate a finding ID matching the original STAT- suffix style."""
    # Original: date +%s%N | tail -c 6  -> last 6 digits of epoch+nanoseconds
    ns = time.time_ns()
    return f"STAT-{str(ns)[-6:]}"


def run_git(args: List[str]) -> Tuple[int, str, str]:
    """Run a git command and return (returncode, stdout, stderr)."""
    proc = subprocess.run(
        ["git"] + args,
        capture_output=True,
        text=True,
    )
    return proc.returncode, proc.stdout, proc.stderr


def add_finding(
    findings: List[str],
    ids: List[str],
    sev: str,
    path: str,
    problem: str,
    fix: str,
) -> None:
    """Append a formatted row to findings and record the ID."""
    fid = generate_id()
    findings.append(f"| {fid} | {sev} | {path} | {problem} | {fix} |\n")
    ids.append(fid)


def shell_quote_join(paths: List[str]) -> str:
    """Join paths with commas, matching original bash paste -sd ',' behavior."""
    return ",".join(paths)


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: iterate-static-check.py <artifact_base>", file=sys.stderr)
        return 2

    artifact_base = sys.argv[1]

    # Resolve repository root (script_dir/..)
    script_dir = Path(__file__).resolve().parent
    root = script_dir.parent.resolve()
    os.chdir(root)

    state_path = f"{artifact_base}.prep.md"
    log_path = f"{artifact_base}.md"
    result_path = f"{artifact_base}.static-check.md"

    # ------------------------------------------------------------------
    # 1. State and log presence
    # ------------------------------------------------------------------
    if not os.path.isfile(state_path) or not os.path.isfile(log_path):
        print(
            f"FAIL: missing prep state ({state_path}) or edit log ({log_path})",
            file=sys.stderr,
        )
        return 2

    findings_rows: List[str] = []
    finding_ids: List[str] = []

    # ------------------------------------------------------------------
    # 2. Derive changed paths from git
    # ------------------------------------------------------------------
    _, changed_stdout, _ = run_git(["diff", "--name-only", "HEAD"])
    changed_lines = [
        ln
        for ln in changed_stdout.splitlines()
        if ln and not re.match(rf"^{re.escape(artifact_base)}\\.", ln)
    ]

    _, untracked_stdout, _ = run_git(["ls-files", "--others", "--exclude-standard"])
    untracked_lines = [
        ln
        for ln in untracked_stdout.splitlines()
        if ln and not re.match(rf"^{re.escape(artifact_base)}\\.", ln)
    ]

    all_paths_set = set(changed_lines) | set(untracked_lines)
    all_paths = sorted(all_paths_set)

    # ------------------------------------------------------------------
    # 3. Validate references and frontmatter
    # ------------------------------------------------------------------
    # Examples:
    #   - Reject {{ file="...opencode-source/..." }} imports (must use local paths)
    #   - Agent/command .md must start with '---' frontmatter
    #   - @agent/name references must resolve to existing .opencode/agent/ or config/agent/ files
    # ------------------------------------------------------------------
    agent_or_command_prefixes = (
        ".opencode/agent/",
        "config/agent/",
        ".opencode/command/",
        "config/command/",
    )

    for path in all_paths:
        if not os.path.isfile(path):
            continue

        if not path.endswith(".md"):
            continue

        with open(path, "r", encoding="utf-8") as fh:
            content = fh.read()

        # Reject imports pointing into opencode-source
        if re.search(r'\{\{[^}]*file="[^"]*opencode-source/', content):
            add_finding(
                findings_rows,
                finding_ids,
                "BLOCKING",
                path,
                "renderer import points into opencode-source/",
                "use a local config/ or .opencode/ path or remove the import",
            )

        # Frontmatter delimiters for agent/command files
        is_agent_or_cmd = any(
            path.startswith(prefix) for prefix in agent_or_command_prefixes
        )
        if is_agent_or_cmd:
            first_line = content.splitlines()[0] if content else ""
            if first_line != "---":
                add_finding(
                    findings_rows,
                    finding_ids,
                    "BLOCKING",
                    path,
                    "missing opening frontmatter delimiter",
                    "add '---' as the first line",
                )

        # Resolve local @agent/name references
        refs = set(re.findall(r"@[A-Za-z][A-Za-z0-9_./-]+", content))
        for raw_ref in refs:
            ref = raw_ref[1:]  # strip leading @
            name = ref.split("/", 1)[1] if "/" in ref else ref
            agent_in_dotopencode = f".opencode/agent/{name}.md"
            agent_in_config = f"config/agent/{name}.md"
            if not os.path.isfile(agent_in_dotopencode) and not os.path.isfile(
                agent_in_config
            ):
                add_finding(
                    findings_rows,
                    finding_ids,
                    "BLOCKING",
                    path,
                    f"unresolved @agent reference: {ref}",
                    "create the agent file or fix the reference",
                )

    # ------------------------------------------------------------------
    # 4. Render every changed agent/command file
    # ------------------------------------------------------------------
    for path in all_paths:
        if not os.path.isfile(path):
            continue

        if not (
            path.startswith(".opencode/agent/")
            or path.startswith(".opencode/command/")
            or path.startswith("config/agent/")
            or path.startswith("config/command/")
        ):
            continue

        if not path.endswith(".md"):
            continue

        rendered = f"{path}.rendered"
        rendered_err = f"{path}.rendered.err"

        with open(rendered, "w", encoding="utf-8") as out_fh, open(
            rendered_err, "w", encoding="utf-8"
        ) as err_fh:
            proc = subprocess.run(
                [
                    "bun",
                    "plugins/opencode-plugin-md-expand/src/cli/cli.ts",
                    "render",
                    path,
                ],
                stdout=out_fh,
                stderr=err_fh,
            )

        if proc.returncode != 0:
            err_msg = ""
            if os.path.isfile(rendered_err):
                with open(rendered_err, "r", encoding="utf-8") as fh:
                    err_msg = fh.readline().strip()
            add_finding(
                findings_rows,
                finding_ids,
                "BLOCKING",
                path,
                f"renderer failed: {err_msg}",
                "fix the source template or import, then re-render",
            )
            os.remove(rendered)
            os.remove(rendered_err)
            continue

        with open(rendered, "r", encoding="utf-8") as fh:
            rendered_lines = fh.read().splitlines()

        # Trailing whitespace lint
        for line in rendered_lines:
            if line.rstrip("\r\n") != line.rstrip():
                add_finding(
                    findings_rows,
                    finding_ids,
                    "BLOCKING",
                    path,
                    "rendered output has trailing whitespace",
                    "remove trailing spaces from the source prompt",
                )
                break

        # Consecutive blank lines lint
        prev_blank = False
        for line in rendered_lines:
            is_blank = line.strip() == ""
            if prev_blank and is_blank:
                add_finding(
                    findings_rows,
                    finding_ids,
                    "BLOCKING",
                    path,
                    "rendered output has consecutive blank lines",
                    "collapse repeated blank lines in the source prompt",
                )
                break
            prev_blank = is_blank

        # Unclosed markdown fence lint
        fence: str | None = None
        opened_line: int | None = None
        for idx, line in enumerate(rendered_lines, start=1):
            stripped = line.strip()
            if stripped.startswith("```"):
                if fence is None:
                    fence = "```"
                    opened_line = idx
                elif fence == "```":
                    fence = None
                    opened_line = None
            elif stripped.startswith("~~~"):
                if fence is None:
                    fence = "~~~"
                    opened_line = idx
                elif fence == "~~~":
                    fence = None
                    opened_line = None

        if fence is not None:
            add_finding(
                findings_rows,
                finding_ids,
                "BLOCKING",
                path,
                f"rendered output has an unclosed markdown fence starting at line {opened_line}",
                "close the fence or switch inner examples to the other fence marker",
            )

        os.remove(rendered)
        os.remove(rendered_err)

    # ------------------------------------------------------------------
    # 5. git diff --check
    # ------------------------------------------------------------------
    proc = subprocess.run(
        ["git", "diff", "--check", "HEAD"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if proc.returncode != 0:
        for line in proc.stdout.splitlines():
            if not line.strip():
                continue
            add_finding(
                findings_rows,
                finding_ids,
                "BLOCKING",
                "git-diff",
                line,
                "fix whitespace in the named file",
            )

    # ------------------------------------------------------------------
    # 6. Write result
    # ------------------------------------------------------------------
    finding_table = (
        "| ID | Severity | Path | Problem | Fix |\n"
        "|----|----------|------|---------|-----|\n"
    )

    if not findings_rows:
        decision = "PASS"
        finding_table += "| None | none | None | no findings | None |\n"
        changed_lines_out = [f"- {p}" for p in all_paths[:50]]
        if not changed_lines_out:
            changed_block = "- None"
        else:
            changed_block = "\n".join(changed_lines_out)
        verified_block = "- None"
        ids_out = "None"
        summary = "static check passed"
    else:
        decision = "BLOCKING"
        changed_lines_out = [f"- {p}" for p in all_paths[:50]]
        if not changed_lines_out:
            changed_block = "- None"
        else:
            changed_block = "\n".join(changed_lines_out)
        verified_block = "- None"
        ids_str = ", ".join(finding_ids)
        # Replicate bash behaviour: ${ids%%,*} – remove shortest suffix from last comma
        ids_out = ids_str.rsplit(",", 1)[0] if "," in ids_str else ids_str
        summary = f"static check found {ids_out} (and possibly more) BLOCKING findings"

    result_body = (
        "# Iterate Edit Static Check\n"
        "Schema: v1\n"
        f"Decision: {decision}\n\n"
        "## Changed Paths\n"
        f"{changed_block}\n\n"
        "## Findings\n"
        f"{finding_table}"
        f"{''.join(findings_rows)}\n"
        "## Verified\n"
        f"{verified_block}\n"
    )

    with open(result_path, "w", encoding="utf-8") as fh:
        fh.write(result_body)

    # ------------------------------------------------------------------
    # Stdout verdict for the primary
    # ------------------------------------------------------------------
    changed_csv = shell_quote_join([p for p in all_paths if p])
    if not changed_csv:
        changed_csv = "(none)"

    print("# STATIC CHECK")
    print(f"Decision: {decision}")
    print(f"Result: {os.path.join(os.getcwd(), result_path)}")
    print(f"Changed Paths: {changed_csv}")
    print(f"IDs: {ids_out}")
    print(f"Summary: {summary}")

    return 1 if decision == "BLOCKING" else 0


if __name__ == "__main__":
    sys.exit(main())
