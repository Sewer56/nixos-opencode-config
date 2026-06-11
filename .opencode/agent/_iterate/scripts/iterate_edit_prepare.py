#!/usr/bin/env python3
"""Prepare a single /iterate/edit run by inspecting the user's request.

Called by the /iterate/edit orchestrator before any editing happens.
This script is purely deterministic — it reads the user request, extracts
file paths mentioned in it, classifies what kind of change this is, picks a
risk profile, and lays out the artifact directory structure.

What it produces:
  prep.json   — machine-readable classification, targets, required reads
  prep.md     — human-readable version of the same data

If no target files can be found in the request, it asks the user which files
to edit instead of guessing.

It never edits target files. Use iterate_edit_contract.py next to compile
editing rules from the prep output.

Input
-----
--repo-root ROOT          Repository root (default: .).
--request-file PATH       Markdown file containing the verbatim edit request.
--run-dir DIR             Artifact directory to create (prep.json, prep.md land here).

  ./iterate_edit_prepare.py \\
    --repo-root /home/sewer/opencode \\
    --request-file artifacts/20250611-120000/request.md \\
    --run-dir artifacts/20250611-120000

  request.md (example content):
    > Please reduce token bloat in .opencode/agent/_iterate/editor.md.
    > Remove redundant policy paragraphs and collapse the pre-edit checklist.

Output
------
prep.json (machine-readable):
  {
    "schema": "iterate-edit-prep-v4",
    "generated_at": "2025-06-11T12:00:00Z",
    "slug": "reduce-token-bloat-in",
    "decision": "READY",
    "question": null,
    "request": "Please reduce token bloat in …",
    "artifacts": {
      "run_dir": "artifacts/20250611-120000",
      "prep_json": "artifacts/20250611-120000/prep.json",
      "prep_md":   "artifacts/20250611-120000/prep.md",
      "contract_md": "artifacts/20250611-120000/contract.md",
      …
    },
    "targets": [".opencode/agent/_iterate/editor.md"],
    "required_reads": [".opencode/agent/_iterate/editor.md", …],
    "classification": {
      "prompt_kind": "agent",
      "consumer": "LLM-runtime",
      "profile": "standard",
      "risk_flags": ["structural"]
    },
    "notes": []
  }

prep.md (human-readable):
  # Iterate Edit Prep
  Schema: iterate-edit-prep-v4
  Decision: READY
  Question: None

  ## Request
  Please reduce token bloat in .opencode/agent/_iterate/editor.md.
  Remove redundant policy paragraphs and collapse the pre-edit checklist.

  ## Artifacts
  - run_dir: artifacts/20250611-120000
  - prep_json: artifacts/20250611-120000/prep.json
  …

  ## Classification
  - prompt_kind: agent
  - consumer: LLM-runtime
  - profile: standard
  - risk_flags: structural

  ## Targets
  - .opencode/agent/_iterate/editor.md

  ## Required Reads
  - .opencode/agent/_iterate/editor.md
  …

  ## Notes
  - None

When no targets found:
  decision = "NEEDS_INPUT"
  question = "Which repo-relative prompt, command, agent, reviewer, template,
              or workflow files should /iterate/edit modify?"
  notes    = ["No existing target path could be resolved deterministically …"]
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

DENY_PARTS = {"opencode-source"}
PROMPT_EXTS = {".md", ".txt"}


def rel_to_repo(path: Path, repo: Path) -> str | None:
    try:
        return path.resolve().relative_to(repo.resolve()).as_posix()
    except Exception:
        return None


def clean_token(token: str) -> str:
    return token.strip().strip("'\"`.,;:()[]{}")


def slugify(text: str) -> str:
    words = re.findall(r"[A-Za-z0-9]+", text.lower())[:5]
    slug = "-".join(words) or "edit"
    return slug[:60]


def path_exists(repo: Path, rel: str) -> bool:
    return (repo / rel).exists()


def resolve_repo_path(repo: Path, raw: str) -> str | None:
    token = clean_token(raw)
    if not token or "://" in token:
        return None
    token = token.replace("\\", "/")
    if token.startswith(repo.as_posix()):
        token = token[len(repo.as_posix()):].lstrip("/")
    token = token.lstrip("./")
    if not token or any(part in DENY_PARTS for part in token.split("/")):
        return None
    candidates = [token]
    if token.startswith("config/agent/"):
        candidates.append(token.replace("config/agent/", ".opencode/agent/", 1))
    if token.startswith("agent/"):
        candidates.append(".opencode/" + token)
    if token.startswith("_iterate/"):
        candidates.append(".opencode/agent/" + token)
    if token.startswith("workflow/"):
        candidates.append("config/doc/" + token)
    for cand in candidates:
        if path_exists(repo, cand):
            return cand
    return token if "/" in token and Path(token).suffix in PROMPT_EXTS else None


def extract_path_hints(repo: Path, request: str) -> list[str]:
    raw_tokens: list[str] = []
    raw_tokens += re.findall(r"`([^`]+)`", request)
    raw_tokens += re.findall(r"(?:^|\s)([./A-Za-z0-9_@+-][A-Za-z0-9_@+./-]*\.(?:md|txt|json|yaml|yml|py|sh))(?:\s|$)", request)
    raw_tokens += re.findall(r"(?:^|\s)([./]?(?:\.opencode|config|scripts|tests|workflow|_iterate)/[A-Za-z0-9_@+./-]+)(?:\s|$)", request)
    seen: set[str] = set()
    paths: list[str] = []
    for raw in raw_tokens:
        rel = resolve_repo_path(repo, raw)
        if rel and rel not in seen:
            seen.add(rel)
            paths.append(rel)

    lower = request.lower()
    def add_existing(cands: list[str]) -> None:
        for cand in cands:
            if path_exists(repo, cand) and cand not in seen:
                seen.add(cand)
                paths.append(cand)
                return

    if "/iterate/edit" in lower or "iterate edit" in lower or "edit workflow" in lower:
        for rel in [
            ".opencode/agent/_iterate/edit.md",
            "_iterate/edit.md",
            ".opencode/agent/_iterate/editor.md",
            "_iterate/editor.md",
            ".opencode/agent/_iterate/review.md",
            "_iterate/review.md",
        ]:
            if path_exists(repo, rel) and rel not in seen:
                seen.add(rel)
                paths.append(rel)
    if "workflow" in lower or "prompt-engineering" in lower or "prompt engineering" in lower:
        for rel in [
            "config/doc/workflow/prompt-engineering.md",
            "workflow/prompt-engineering.md",
            "config/doc/workflow/design-patterns.md",
            "workflow/design-patterns.md",
            "config/doc/workflow/optimize-patterns.md",
            "workflow/optimize-patterns.md",
        ]:
            if path_exists(repo, rel) and rel not in seen:
                seen.add(rel)
                paths.append(rel)
    return paths


def classify_prompt_kind(paths: list[str]) -> str:
    kinds: set[str] = set()
    for p in paths:
        low = p.lower()
        if "/command/" in low:
            kinds.add("command")
        elif "/review" in low or "reviewer" in low:
            kinds.add("reviewer")
        elif "/agent/" in low or low.startswith("_iterate/"):
            kinds.add("agent")
        elif "/doc/" in low or low.startswith("workflow/"):
            kinds.add("docs")
    if not kinds:
        return "mixed"
    return next(iter(kinds)) if len(kinds) == 1 else "mixed"


def classify_profile(paths: list[str], request: str) -> tuple[str, list[str]]:
    lower = request.lower()
    joined = "\n".join(paths).lower()
    flags: set[str] = set()

    if any(x in joined for x in ["_iterate/", "/iterate/", "iterate_edit_", "prompt_static_check", "prompt_token_report"]):
        flags.add("self-iteration")
    if any(x in joined for x in ["reviewer", "/review", "_templates", "template", "contract", "compiler"]):
        flags.add("reviewer-topology")
    if any(x in lower for x in ["permission", "sandbox", "egress", "secret", "destructive", "external", "security"]):
        flags.add("high-risk")
    if any(x in joined for x in ["permission", "sandbox", "secrets"]):
        flags.add("high-risk")
    if any(x in lower for x in ["import", "template", "frontmatter", "schema", "output contract", "subagent", "reviewer", "routing", "topology", "split", "merge"]):
        flags.add("structural")
    if all((p.startswith("workflow/") or "/doc/" in p) for p in paths) and paths:
        flags.add("docs-only")

    if "high-risk" in flags:
        profile = "high_risk"
    elif "self-iteration" in flags:
        profile = "self_iterating"
    elif "structural" in flags or "reviewer-topology" in flags:
        profile = "structural"
    elif "docs-only" in flags and any(x in lower for x in ["wording", "compress", "reduce", "typo", "docs"]):
        profile = "micro"
    else:
        profile = "standard"
    return profile, sorted(flags)


def direct_imports(repo: Path, rels: list[str]) -> list[str]:
    imports: list[str] = []
    seen = set(rels)
    pattern = re.compile(r"\{\{\s*file\s*=\s*['\"]([^'\"]+)['\"]")
    for rel in rels:
        path = repo / rel
        if not path.is_file() or path.suffix not in PROMPT_EXTS:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            text = path.read_text(errors="ignore")
        for raw in pattern.findall(text):
            cand = raw.strip()
            candidates = []
            if cand.startswith("./"):
                candidates.append(cand[2:])
            candidates.append(cand)
            if cand.startswith("./.opencode/agent/"):
                candidates.append(cand[len("./.opencode/agent/"):])
            if cand.startswith("../"):
                candidates.append((Path(rel).parent / cand).as_posix())
            for c in candidates:
                c = c.lstrip("./")
                if path_exists(repo, c) and c not in seen:
                    seen.add(c)
                    imports.append(c)
                    break
    return imports


def write_markdown(prep: dict[str, Any], out: Path) -> None:
    lines = [
        "# Iterate Edit Prep",
        f"Schema: {prep['schema']}",
        f"Decision: {prep['decision']}",
        f"Question: {prep.get('question') or 'None'}",
        "",
        "## Request",
        prep["request"],
        "",
        "## Artifacts",
    ]
    for k, v in prep["artifacts"].items():
        lines.append(f"- {k}: {v}")
    lines += ["", "## Classification"]
    for k, v in prep["classification"].items():
        if isinstance(v, list):
            v = ", ".join(v) if v else "None"
        lines.append(f"- {k}: {v}")
    lines += ["", "## Targets"]
    lines += [f"- {p}" for p in prep["targets"]] or ["- None"]
    lines += ["", "## Required Reads"]
    lines += [f"- {p}" for p in prep["required_reads"]] or ["- None"]
    lines += ["", "## Notes"]
    lines += [f"- {n}" for n in prep["notes"]] or ["- None"]
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Inspect a user edit request and classify what kind of prompt edit "
            "is needed. Extracts file paths from the request text, determines "
            "risk profile (micro/standard/structural/self_iterating/high_risk), "
            "and creates the run directory with prep.json and prep.md."
        ),
        epilog=(
            "Example:\n"
            "  %(prog)s --repo-root . --request-file artifacts/20250611-120000/request.md "
            "--run-dir artifacts/20250611-120000\n\n"
            "Next step: feed prep.json into iterate_edit_contract.py to compile "
            "the editing contract."
        ),
    )
    ap.add_argument(
        "--repo-root", default=".",
        help="Root of the repository containing .opencode/ and config/ trees (default: current directory)."
    )
    ap.add_argument(
        "--request-file", required=True,
        help="Path to a markdown file containing the verbatim user edit request."
    )
    ap.add_argument(
        "--run-dir", required=True,
        help="Directory where prep.json, prep.md, and all subsequent artifacts will be written."
    )
    args = ap.parse_args()

    repo = Path(args.repo_root).resolve()
    run_dir = Path(args.run_dir)
    if not run_dir.is_absolute():
        run_dir = (repo / run_dir).resolve()
    run_dir.mkdir(parents=True, exist_ok=True)
    (run_dir / "reviews").mkdir(exist_ok=True)

    request_path = Path(args.request_file)
    if not request_path.is_absolute():
        request_path = (repo / request_path).resolve()
    request = request_path.read_text(encoding="utf-8")
    slug = slugify(request)

    targets = extract_path_hints(repo, request)
    profile, risk_flags = classify_profile(targets, request)
    prompt_kind = classify_prompt_kind(targets)
    required_reads = list(dict.fromkeys(targets + direct_imports(repo, targets)))

    decision = "READY" if targets else "NEEDS_INPUT"
    question = None if targets else "Which repo-relative prompt, command, agent, reviewer, template, or workflow files should /iterate/edit modify?"
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    artifacts = {
        "run_dir": run_dir.as_posix(),
        "request_path": request_path.as_posix(),
        "prep_json": (run_dir / "prep.json").as_posix(),
        "prep_md": (run_dir / "prep.md").as_posix(),
        "contract_md": (run_dir / "contract.md").as_posix(),
        "contract_json": (run_dir / "contract.json").as_posix(),
        "edit_log": (run_dir / "edit-log.md").as_posix(),
        "static_check": (run_dir / "static-check.md").as_posix(),
        "token_report": (run_dir / "token-report.md").as_posix(),
        "reviews_dir": (run_dir / "reviews").as_posix(),
    }
    prep: dict[str, Any] = {
        "schema": "iterate-edit-prep-v4",
        "generated_at": now,
        "slug": slug,
        "decision": decision,
        "question": question,
        "request": request,
        "artifacts": artifacts,
        "targets": targets,
        "required_reads": required_reads,
        "classification": {
            "prompt_kind": prompt_kind,
            "consumer": "mixed" if prompt_kind in {"mixed", "docs"} else "LLM-runtime",
            "profile": profile,
            "risk_flags": risk_flags,
        },
        "notes": [] if targets else ["No existing target path could be resolved deterministically from the request."],
    }

    (run_dir / "prep.json").write_text(json.dumps(prep, indent=2) + "\n", encoding="utf-8")
    write_markdown(prep, run_dir / "prep.md")
    print("# PREP")
    print(f"Decision: {decision}")
    print(f"Run Dir: {run_dir.as_posix()}")
    print(f"Prep: {(run_dir / 'prep.json').as_posix()}")
    print(f"Profile: {profile}")
    print(f"Targets: {', '.join(targets) if targets else 'None'}")
    print(f"Question: {question or 'None'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
