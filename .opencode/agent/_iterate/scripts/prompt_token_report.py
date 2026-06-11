#!/usr/bin/env python3
"""Measure the size of every changed prompt file.

Called after the editor finishes editing and static checks pass. Produces a
simple table showing word count, character count, and estimated token count
for each file. The estimate uses chars/4 — a portable approximation that
works across languages without needing a provider-specific tokenizer.

This report helps spot prompt bloat (files that grew unexpectedly) and
confirms that compression edits actually reduced size. It is not a quality
metric on its own — use it alongside the semantic reviewers' findings.

Input
-----
--repo-root ROOT      Repository root (default: .).
--paths P …           Repo-relative file paths to measure (primary input).
--contract PATH       Fallback: contract.md/.json for target paths.
--log PATH            Fallback: edit-log.md for changed-file paths.
--out PATH            Where to write the markdown table; stdout if omitted.

  # Direct paths:
  ./prompt_token_report.py \\
    --paths .opencode/agent/_iterate/edit.md config/doc/workflow/prompt-engineering.md \\
    --out artifacts/20250611-120000/token-report.md

  # Fallback via contract + log:
  ./prompt_token_report.py \\
    --contract artifacts/20250611-120000/contract.md \\
    --log artifacts/20250611-120000/edit-log.md \\
    --out artifacts/20250611-120000/token-report.md

Output
------
token-report.md (example):
  # TOKEN REPORT
  Estimator: chars/4 approximate; use provider tokenizer for exact billing.

  | File | Words | Chars | Est Tokens |
  | --- | ---: | ---: | ---: |
  | `.opencode/agent/_iterate/edit.md` | 1203 | 8734 | 2184 |
  | `config/doc/workflow/prompt-engineering.md` | 24510 | 187200 | 46800 |
  | `.opencode/agent/_iterate/editor.md` | 3421 | 23891 | 5973 |

  Total words: 29134
  Total chars: 219825
  Estimated tokens: 54957

Stdout (when --out set):
  # TOKEN REPORT
  Report: artifacts/20250611-120000/token-report.md
  Estimated Tokens: 54957
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def resolve_paths(repo: Path, paths: list[str], contract: Path | None, log: Path | None) -> list[str]:
    """Gather target file paths from explicit list, contract JSON, and edit log."""
    found: list[str] = []
    seen: set[str] = set()

    for p in paths:
        if p and p not in seen:
            seen.add(p)
            found.append(p)

    if contract and contract.with_suffix(".json").exists():
        data = read_json(contract.with_suffix(".json"))
        for p in data.get("targets", []):
            if isinstance(p, str) and p not in seen:
                seen.add(p)
                found.append(p)

    if log and log.exists():
        text = log.read_text(encoding="utf-8", errors="ignore")
        for m in re.finditer(r"^-\s+([A-Za-z0-9_@+./-]+)\s+-", text, flags=re.M):
            p = m.group(1)
            if p not in seen:
                seen.add(p)
                found.append(p)

    return [p for p in found if (repo / p).is_file()]


def estimate_tokens(text: str) -> int:
    # Portable approximation: modern English/code prompts are often 3.5-4 chars/token.
    return max(1, round(len(text) / 4)) if text else 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Count words, characters, and estimate tokens for changed "
            "prompt files. Uses the chars/4 heuristic — quick and portable but "
            "not billing-accurate. For exact token counts, pipe through a "
            "provider tokenizer instead."
        ),
        epilog=(
            "Examples:\n"
            "  %(prog)s --paths .opencode/agent/_iterate/edit.md config/doc/workflow/prompt-engineering.md --out report.md\n"
            "  %(prog)s --contract artifacts/run/contract.md --log artifacts/run/edit-log.md --out artifacts/run/token-report.md\n\n"
            "At least one of --paths, --contract, or --log should be provided."
        ),
    )
    ap.add_argument(
        "--repo-root", default=".",
        help="Root of the repository (default: current directory)."
    )
    ap.add_argument(
        "--paths", nargs="*", default=[],
        help="Repo-relative file paths to measure. Primary input — use this when you already know which files changed."
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
        help="Where to write the token report (markdown table). Prints to stdout if omitted."
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

    rows = []
    total_chars = total_words = total_est = 0
    for rel in resolve_paths(repo, args.paths, contract, log):
        text = (repo / rel).read_text(encoding="utf-8", errors="ignore")
        words = len(text.split())
        chars = len(text)
        est = estimate_tokens(text)
        total_chars += chars
        total_words += words
        total_est += est
        rows.append((rel, words, chars, est))

    lines = [
        "# TOKEN REPORT",
        "Estimator: chars/4 approximate; use provider tokenizer for exact billing.",
        "",
        "| File | Words | Chars | Est Tokens |",
        "| --- | ---: | ---: | ---: |",
    ]
    for rel, words, chars, est in rows:
        lines.append(f"| `{rel}` | {words} | {chars} | {est} |")
    if not rows:
        lines.append("| None | 0 | 0 | 0 |")
    lines += [
        "",
        f"Total words: {total_words}",
        f"Total chars: {total_chars}",
        f"Estimated tokens: {total_est}",
    ]
    report = "\n".join(lines) + "\n"
    if out:
        out.write_text(report, encoding="utf-8")
        print(f"# TOKEN REPORT\nReport: {out.as_posix()}\nEstimated Tokens: {total_est}")
    else:
        print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
