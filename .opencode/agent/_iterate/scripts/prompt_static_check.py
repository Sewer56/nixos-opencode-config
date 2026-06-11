#!/usr/bin/env python3
"""Run deterministic mechanical checks on changed prompt files.

Called after the editor agent finishes editing. This script does not use an
LLM — it performs fast, reproducible checks that catch common mistakes before
semantic reviewers spend tokens on them:

  - Unresolved {{ file="..." }} imports (broken includes)
  - Imports pointing into opencode-source/ (source-boundary violation)
  - Missing output contracts in runtime prompts
  - Harness/API terms leaking into prompt text (prompt/harness boundary)
  - Angle-bracket placeholders that look like XML tags (use [[slot]] instead)
  - Unclosed XML tags
  - Unclosed markdown fences (``` or ~~~)
  - Consecutive blank lines
  - Trailing whitespace and tab characters
  - Frontmatter that opens but never closes

Exits with code 0 even when findings are BLOCKING — the agent reads the
report file and repairs issues, then reruns this check.

Input
-----
--repo-root ROOT      Repository root (default: .).
--paths P …           Repo-relative file paths to check (primary input).
--contract PATH       Fallback: contract.md/.json for target paths.
--log PATH            Fallback: edit-log.md for changed-file paths.
--out PATH            Where to write the markdown report; stdout if omitted.

  # Direct paths (preferred):
  ./prompt_static_check.py \\
    --repo-root /home/sewer/opencode \\
    --paths .opencode/agent/_iterate/editor.md \\
    --out artifacts/20250611-120000/static-check.md

  # Fallback via contract + log:
  ./prompt_static_check.py \\
    --contract artifacts/20250611-120000/contract.md \\
    --log artifacts/20250611-120000/edit-log.md \\
    --out artifacts/20250611-120000/static-check.md

Output
------
static-check.md (example — PASS):
  # STATIC CHECK
  Decision: PASS

  ## Files Checked
  - .opencode/agent/_iterate/editor.md

  ## Findings
  - None

static-check.md (example — BLOCKING):
  # STATIC CHECK
  Decision: BLOCKING

  ## Files Checked
  - .opencode/agent/_iterate/editor.md

  ## Findings
  - BLOCKING | .opencode/agent/_iterate/editor.md | unresolved prompt import: _iterate/rules/missing.md
  - BLOCKING | .opencode/agent/_iterate/editor.md | runtime prompt has no output contract
  - BLOCKING | .opencode/agent/_iterate/editor.md:42 | unclosed markdown fence '```'
  - ADVISORY | .opencode/agent/_iterate/editor.md:17 | trailing whitespace
  - ADVISORY | .opencode/agent/_iterate/editor.md:55 | consecutive blank lines

Decision is BLOCKING when ≥1 BLOCKING finding exists, PASS otherwise.
Script always exits 0 — the orchestrator reads the report to decide.
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Iterable

RUNTIME_HINTS = ["/agent/", "/command/", "_iterate/"]
DOC_HINTS = ["/doc/", "workflow/", "README", "MIGRATION"]
HARNESS_TERMS = [
    "reasoning_effort",
    "reasoning_content",
    "tool_choice",
    "cache breakpoint",
    "cache_breakpoint",
    "provider replay",
    "preserve thinking block",
    "mcp wiring",
    "tool schema",
]
TRANSCRIPT_TERMS = [
    "full chain of thought",
    "complete chain of thought",
    "private chain of thought",
    "hidden chain of thought",
]
IMPORT_RE = re.compile(r"\{\{\s*file\s*=\s*['\"]([^'\"]+)['\"]")
TAG_RE = re.compile(r"</?([A-Za-z][A-Za-z0-9_-]*)(?:\s[^<>]*)?>")
PLACEHOLDER_RE = re.compile(r"<([A-Za-z][A-Za-z0-9_-]*(?:[_-][A-Za-z0-9_-]+)+)>")


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def is_runtime(rel: str) -> bool:
    low = rel.lower()
    if any(h in low or low.startswith(h.strip("/")) for h in DOC_HINTS):
        return False
    if "/rules/" in low or low.startswith("_iterate/rules/"):
        return False
    return any(h in low or low.startswith(h.strip("/")) for h in RUNTIME_HINTS)


def strip_code_fences(text: str) -> str:
    return re.sub(r"```.*?```", "", text, flags=re.S)


def resolve_import(repo: Path, current: Path, raw: str) -> Path | None:
    raw = raw.strip()
    candidates: list[Path] = []
    if raw.startswith("./"):
        candidates.append(repo / raw[2:])
    candidates.append(repo / raw)
    candidates.append(current.parent / raw)
    if raw.startswith("./.opencode/agent/"):
        candidates.append(repo / raw[len("./.opencode/agent/"):])
    if raw.startswith("../"):
        candidates.append(current.parent / raw)
    for cand in candidates:
        if cand.exists():
            return cand
    return None


def xml_balance_findings(rel: str, text: str) -> list[tuple[str, str, str]]:
    # Light check for prompt files. Ignores fenced examples to avoid blocking docs that explain XML.
    body = strip_code_fences(text)
    stack: list[tuple[str, int]] = []
    findings: list[tuple[str, str, str]] = []
    for m in TAG_RE.finditer(body):
        full = m.group(0)
        name = m.group(1)
        if full.startswith("</"):
            if not stack or stack[-1][0] != name:
                findings.append(("BLOCKING", rel, f"unmatched closing XML tag </{name}>"))
            else:
                stack.pop()
        elif full.endswith("/>"):
            continue
        else:
            stack.append((name, body.count("\n", 0, m.start()) + 1))
    for name, line in stack[-5:]:
        findings.append(("BLOCKING", f"{rel}:{line}", f"unclosed XML tag <{name}>"))
    return findings


def resolve_paths(repo: Path, paths: list[str], contract: Path | None, log: Path | None) -> list[str]:
    """Gather target file paths from explicit list, contract JSON, and edit log."""
    found: list[str] = []
    seen: set[str] = set()

    for p in paths:
        if p and p not in seen:
            seen.add(p)
            found.append(p)

    if contract:
        cj = contract.with_suffix(".json")
        data = read_json(cj) if cj.exists() else {}
        for p in data.get("targets", []) + data.get("required_reads", []):
            if isinstance(p, str) and p not in seen:
                seen.add(p)
                found.append(p)

    if log and log.exists():
        text = log.read_text(encoding="utf-8", errors="ignore")
        for m in re.finditer(r"^-\s+([A-Za-z0-9_@+./-]+)\s+-", text, flags=re.M):
            p = m.group(1)
            if (repo / p).exists() and p not in seen:
                seen.add(p)
                found.append(p)

    return [p for p in found if (repo / p).is_file()]


def check_file(repo: Path, rel: str) -> list[tuple[str, str, str]]:
    path = repo / rel
    findings: list[tuple[str, str, str]] = []
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return [("ADVISORY", rel, "file is not UTF-8 text; skipped prompt checks")]

    lines = text.splitlines()
    if text.startswith("---"):
        end = None
        for i, line in enumerate(lines[1:], start=2):
            if line.strip() == "---":
                end = i
                break
        if end is None:
            findings.append(("BLOCKING", rel, "frontmatter starts but never closes"))

    for i, line in enumerate(lines, start=1):
        if line.rstrip() != line:
            findings.append(("ADVISORY", f"{rel}:{i}", "trailing whitespace"))
        if "\t" in line:
            findings.append(("ADVISORY", f"{rel}:{i}", "tab character"))

    # Consecutive blank lines
    prev_blank = False
    for i, line in enumerate(lines, start=1):
        is_blank = line.strip() == ""
        if prev_blank and is_blank:
            findings.append(("ADVISORY", f"{rel}:{i}", "consecutive blank lines"))
            break
        prev_blank = is_blank

    for raw in IMPORT_RE.findall(text):
        if resolve_import(repo, path, raw) is None:
            findings.append(("BLOCKING", rel, f"unresolved prompt import: {raw}"))
        if "opencode-source/" in raw:
            findings.append(("BLOCKING", rel, f"import points into opencode-source/: {raw} — use local config/ or .opencode/ path"))

    if is_runtime(rel):
        body = strip_code_fences(text)
        low = body.lower()
        if "<output_contract" not in low and "review-output-contract" not in low and "# review" not in low:
            findings.append(("BLOCKING", rel, "runtime prompt has no output contract or shared review output contract import"))
        for term in HARNESS_TERMS:
            if term in low:
                findings.append(("BLOCKING", rel, f"runtime prompt contains harness/API term: {term}"))
        for term in TRANSCRIPT_TERMS:
            if term in low:
                findings.append(("BLOCKING", rel, f"runtime prompt asks for reasoning transcript: {term}"))
        for m in PLACEHOLDER_RE.finditer(body):
            tag = m.group(1)
            # Treat as OK if it has a matching closing tag. Otherwise it is likely an angle placeholder.
            if f"</{tag}>" not in body:
                findings.append(("BLOCKING", rel, f"possible angle-placeholder <{tag}>; use [[{tag}]] or close as XML"))
        findings.extend(xml_balance_findings(rel, text))

    # Unclosed markdown fence (check source, not rendered output)
    fence: str | None = None
    fence_line: int | None = None
    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("```"):
            if fence is None:
                fence = "```"
                fence_line = i
            elif fence == "```":
                fence = None
                fence_line = None
        elif stripped.startswith("~~~"):
            if fence is None:
                fence = "~~~"
                fence_line = i
            elif fence == "~~~":
                fence = None
                fence_line = None
    if fence is not None:
        findings.append(("BLOCKING", f"{rel}:{fence_line}", f"unclosed markdown fence '{fence}'"))

    return findings


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Scan changed prompt files for mechanical problems: broken imports, "
            "opencode-source imports, missing output contracts, harness terms "
            "leaked into prompts, ambiguous placeholders, unbalanced XML tags, "
            "unclosed markdown fences, consecutive blank lines, and whitespace "
            "issues. Reports findings in markdown. Always exits 0 so the "
            "orchestrator can read the report and decide whether to repair "
            "and rerun."
        ),
        epilog=(
            "Example:\n"
            "  %(prog)s --repo-root . --contract artifacts/20250611-120000/contract.md "
            "--log artifacts/20250611-120000/edit-log.md "
            "--out artifacts/20250611-120000/static-check.md\n\n"
            "Blocking findings mean the editor must fix the issues and rerun "
            "this check before semantic review."
        ),
    )
    ap.add_argument(
        "--repo-root", default=".",
        help="Root of the repository (default: current directory)."
    )
    ap.add_argument(
        "--paths", nargs="*", default=[],
        help="Repo-relative file paths to check. Primary input — use this when you already know which files changed."
    )
    ap.add_argument(
        "--contract",
        help="Path to contract.md (or contract.json). Fallback source for target file paths."
    )
    ap.add_argument(
        "--log",
        help="Path to edit-log.md. Fallback source for changed file paths."
    )
    ap.add_argument(
        "--out",
        help="Where to write the static check report (markdown). Prints to stdout if omitted."
    )
    args = ap.parse_args()

    repo = Path(args.repo_root).resolve()
    contract = Path(args.contract).resolve() if args.contract else None
    log = Path(args.log).resolve() if args.log else None
    out = Path(args.out) if args.out else None
    if out and not out.is_absolute():
        out = (repo / out).resolve()
    if out:
        out.parent.mkdir(parents=True, exist_ok=True)

    findings: list[tuple[str, str, str]] = []
    paths = resolve_paths(repo, args.paths, contract, log)
    if not paths:
        findings.append(("ADVISORY", "targets", "no existing target files resolved for static prompt checks"))
    for rel in paths:
        findings.extend(check_file(repo, rel))

    blocking = [f for f in findings if f[0] == "BLOCKING"]
    decision = "BLOCKING" if blocking else "PASS"
    lines = ["# STATIC CHECK", f"Decision: {decision}", "", "## Files Checked"]
    lines += [f"- {p}" for p in paths] or ["- None"]
    lines += ["", "## Findings"]
    if findings:
        for sev, loc, msg in findings:
            lines.append(f"- {sev} | {loc} | {msg}")
    else:
        lines.append("- None")
    report = "\n".join(lines) + "\n"
    if out:
        out.write_text(report, encoding="utf-8")
        print(f"# STATIC CHECK\nDecision: {decision}\nReport: {out.as_posix()}")
    else:
        print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
