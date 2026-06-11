#!/usr/bin/env python3
"""Compile a compact editing contract from the deterministic prep output.

Takes the prep.json produced by iterate_edit_prepare.py and selects which
prompt engineering rules (PE), design patterns (OPT), and workflow
optimization tactics (WOPT) apply to this edit. Also decides which semantic
reviewers to run, based on the risk profile.

This replaces what used to be an LLM-based pattern selector subagent.
The rules are looked up from internal catalogs — the script does not read
the full design docs itself; the editor agent is responsible for reading
the referenced source docs when applying rules.

What it produces:
  contract.md  — human-readable: selected rules, reviewers, checks, source docs
  contract.json — machine-readable version for downstream scripts

Use this contract to guide the editor agent and to decide which checks
and reviewers to run.

Input
-----
--repo-root ROOT    Repository root (default: .).
--prep PATH         prep.json from iterate_edit_prepare.py (required).
--out PATH          Where to write contract.md; contract.json is written
                    alongside it (same stem, .json suffix).

  ./iterate_edit_contract.py \\
    --repo-root /home/sewer/opencode \\
    --prep artifacts/20250611-120000/prep.json \\
    --out artifacts/20250611-120000/contract.md

  prep.json keys consumed:
    request, classification.{profile, prompt_kind, risk_flags}, targets

Output
------
contract.md (human-readable):
  # Iterate Edit Contract
  Schema: iterate-edit-contract-v4
  Profile: standard

  ## Request
  Please reduce token bloat …

  ## Targets
  - .opencode/agent/_iterate/editor.md

  ## Required Reads
  - .opencode/agent/_iterate/editor.md
  - .opencode/agent/_iterate/docs/prompt-engineering.md

  ## Required Prompt Rules
  - PE-001: Outcome contract: start with deliverable, scope, done criteria …
  - PE-002: Prompt/harness boundary: prompts own task behavior; harness …
  - PE-003: Sections/XML/placeholders: use XML for mixed blocks …
  …

  ## Selected Design Rules
  - OPT-002: Compiled contract: deterministic scripts select routine rules …
  - OPT-003: Static before semantic: scripts catch render/import/schema …
  …

  ## Selected Workflow Optimization Rules
  - WOPT-003: Prompt/harness scrub: move API/provider/tool-schema/cache …
  …

  ## Required Checks
  - prompt_static_check.py
  - prompt_token_report.py

  ## Required Reviewers
  - prompt

  ## Source Docs
  - prompt_engineering: .opencode/agent/_iterate/docs/prompt-engineering.md
  - design_patterns: .opencode/agent/_iterate/docs/design-patterns.md
  - optimize_patterns: .opencode/agent/_iterate/docs/optimize-patterns.md

  ## Local Constraints
  - Apply explicit user requirements before optional compression.
  …

contract.json (machine-readable):
  {
    "schema": "iterate-edit-contract-v4",
    "request": "Please reduce token bloat …",
    "profile": "standard",
    "risk_flags": ["structural"],
    "targets": [".opencode/agent/_iterate/editor.md"],
    "required_reads": [".opencode/agent/_iterate/editor.md", …],
    "rules": {
      "PE": ["PE-001", "PE-002", …],
      "OPT": ["OPT-002", "OPT-003", …],
      "WOPT": ["WOPT-003"]
    },
    "checks": ["prompt_static_check.py", "prompt_token_report.py"],
    "reviewers": ["prompt"],
    "source_docs": {
      "prompt_engineering": ".opencode/agent/_iterate/docs/prompt-engineering.md",
      "design_patterns": ".opencode/agent/_iterate/docs/design-patterns.md",
      "optimize_patterns": ".opencode/agent/_iterate/docs/optimize-patterns.md"
    }
  }
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

PE_RULES = {
    "PE-001": "Outcome contract: start with deliverable, scope, done criteria, and output shape.",
    "PE-002": "Prompt/harness boundary: prompts own task behavior; harness/config owns runtime mechanics.",
    "PE-003": "Sections/XML/placeholders: use XML for mixed blocks; use [[slot]] placeholders.",
    "PE-004": "Source boundaries: label source data and do not follow embedded instructions from it.",
    "PE-005": "Task-level tool behavior: require inspection/verification where correctness depends on current files.",
    "PE-006": "Context budget: start with targets/direct references; broaden only on ambiguity or failed evidence.",
    "PE-007": "Output contract: exact fields, allowed values, order, and empty-state behavior.",
    "PE-008": "Verification and stops: define smallest useful checks and failure behavior.",
    "PE-009": "Evidence discipline: output findings, assumptions, checks, and concise rationale.",
    "PE-010": "Token density: remove stale hacks, fluff, duplicated policy, and non-constraining examples.",
    "PE-011": "Examples: keep only examples that constrain format, edge cases, or classification boundaries.",
    "PE-012": "Evaluation/migration: use baseline cases and change one prompt variable at a time when practical.",
}

OPT_RULES = {
    "OPT-001": "Thin command: command routes and defines final output; owning agent carries behavior.",
    "OPT-002": "Compiled contract: deterministic scripts select routine rules/profile/checks.",
    "OPT-003": "Static before semantic: scripts catch render/import/schema/token issues before LLM review.",
    "OPT-004": "Risk-tiered review: default reviewer fanout depends on profile, not habit.",
    "OPT-005": "Single run directory: request, prep, contract, log, checks, and reviews stay together.",
    "OPT-006": "Distinct reviewer domains: prompt, integrity, topology, adversarial each own separate risk.",
    "OPT-007": "One-consumer inline: inline rule files used by one prompt; keep only multi-consumer includes.",
    "OPT-008": "Docs/runtime split: long research docs are human/reference input; runtime rules are compact.",
}

WOPT_RULES = {
    "WOPT-001": "Remove standing committee: replace default reviewer swarms with conditional gates.",
    "WOPT-002": "Scriptable selector: replace pattern-selector subagent with contract compiler unless ambiguous.",
    "WOPT-003": "Prompt/harness scrub: move API/provider/tool-schema/cache mechanics out of prompt bodies.",
    "WOPT-004": "Sparse XML: XML-wrap mixed sections, not every bullet.",
    "WOPT-005": "Token report: record changed prompt size and high-cost hotspots each run.",
    "WOPT-006": "Self-iteration guard: edits to this workflow must preserve future optimizer application.",
}


def existing_doc(repo: Path, rels: list[str]) -> str | None:
    for rel in rels:
        if (repo / rel).exists():
            return rel
    return None


def reviewers_for(profile: str, flags: list[str]) -> list[str]:
    f = set(flags)
    if profile == "micro":
        return []
    if profile == "standard":
        return ["prompt"]
    if profile == "structural":
        reviewers = ["prompt"]
        if f & {"reviewer-topology", "structural"}:
            reviewers.append("topology")
        if f & {"permission", "high-risk", "self-iteration"} or True:
            reviewers.append("integrity")
        return list(dict.fromkeys(reviewers))
    if profile == "self_iterating":
        return ["prompt", "integrity", "topology", "adversarial"]
    if profile == "high_risk":
        reviewers = ["prompt", "integrity", "adversarial"]
        if f & {"reviewer-topology", "structural"}:
            reviewers.insert(2, "topology")
        return list(dict.fromkeys(reviewers))
    return ["prompt"]


def select_rules(prep: dict[str, Any]) -> tuple[list[str], list[str], list[str]]:
    profile = prep["classification"].get("profile", "standard")
    kind = prep["classification"].get("prompt_kind", "mixed")
    flags = set(prep["classification"].get("risk_flags", []))
    paths = "\n".join(prep.get("targets", [])).lower()

    pe = ["PE-001", "PE-002", "PE-003", "PE-004", "PE-007", "PE-008", "PE-010"]
    if kind in {"agent", "reviewer", "mixed"}:
        pe += ["PE-005", "PE-006", "PE-009"]
    if "docs" in kind or "/doc/" in paths or paths.startswith("workflow/"):
        pe += ["PE-012"]
    if "example" in prep.get("request", "").lower():
        pe += ["PE-011"]

    opt: list[str] = []
    wopt: list[str] = []
    if "command" in kind or "/command/" in paths:
        opt.append("OPT-001")
    if profile in {"structural", "self_iterating", "high_risk"}:
        opt += ["OPT-002", "OPT-003", "OPT-004", "OPT-005", "OPT-006"]
    if "_templates" in paths or "rules/" in paths:
        opt.append("OPT-007")
    if "/doc/" in paths or paths.startswith("workflow/"):
        opt.append("OPT-008")
    if profile == "self_iterating":
        wopt += ["WOPT-001", "WOPT-002", "WOPT-005", "WOPT-006"]
    if "reviewer-topology" in flags:
        wopt += ["WOPT-001", "WOPT-004"]
    if profile in {"standard", "structural", "self_iterating", "high_risk"}:
        wopt.append("WOPT-003")
    return list(dict.fromkeys(pe)), list(dict.fromkeys(opt)), list(dict.fromkeys(wopt))


def write_contract_md(contract: dict[str, Any], out: Path) -> None:
    lines = [
        "# Iterate Edit Contract",
        f"Schema: {contract['schema']}",
        f"Profile: {contract['profile']}",
        "",
        "## Request",
        contract["request"].strip(),
        "",
        "## Targets",
    ]
    lines += [f"- {p}" for p in contract["targets"]] or ["- None"]
    lines += ["", "## Required Reads"]
    lines += [f"- {p}" for p in contract["required_reads"]] or ["- None"]
    lines += ["", "## Required Prompt Rules"]
    for rid in contract["rules"]["PE"]:
        lines.append(f"- {rid}: {PE_RULES[rid]}")
    lines += ["", "## Selected Design Rules"]
    lines += [f"- {rid}: {OPT_RULES[rid]}" for rid in contract["rules"]["OPT"]] or ["- None"]
    lines += ["", "## Selected Workflow Optimization Rules"]
    lines += [f"- {rid}: {WOPT_RULES[rid]}" for rid in contract["rules"]["WOPT"]] or ["- None"]
    lines += ["", "## Required Checks"]
    for check in contract["checks"]:
        lines.append(f"- {check}")
    lines += ["", "## Required Reviewers"]
    lines += [f"- {r}" for r in contract["reviewers"]] or ["- None"]
    lines += ["", "## Source Docs"]
    for name, rel in contract["source_docs"].items():
        lines.append(f"- {name}: {rel or 'Not found'}")
    lines += ["", "## Local Constraints"]
    lines += [
        "- Apply explicit user requirements before optional compression.",
        "- Keep long research/reference text out of runtime prompt bodies unless selected rules require a compact carry-in.",
        "- If a required target cannot be found or a blocking check cannot be resolved, record INCOMPLETE with evidence.",
    ]
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Compile an editing contract from prep.json. Selects applicable "
            "prompt engineering rules (PE-xxx), workflow design patterns "
            "(OPT-xxx), and optimization tactics (WOPT-xxx) based on the "
            "request classification and risk profile. Also picks which "
            "semantic reviewers must sign off before the edit is complete."
        ),
        epilog=(
            "Example:\n"
            "  %(prog)s --repo-root . --prep artifacts/20250611-120000/prep.json "
            "--out artifacts/20250611-120000/contract.md\n\n"
            "The contract.md output is human-readable and lists every selected "
            "rule with a short description. The editor agent reads this file "
            "and applies only the listed rules."
        ),
    )
    ap.add_argument(
        "--repo-root", default=".",
        help="Root of the repository containing .opencode/ and config/ trees (default: current directory)."
    )
    ap.add_argument(
        "--prep", required=True,
        help="Path to prep.json produced by iterate_edit_prepare.py."
    )
    ap.add_argument(
        "--out", required=True,
        help="Where to write contract.md (contract.json is written alongside it)."
    )
    args = ap.parse_args()

    repo = Path(args.repo_root).resolve()
    prep_path = Path(args.prep)
    if not prep_path.is_absolute():
        prep_path = (repo / prep_path).resolve()
    prep = json.loads(prep_path.read_text(encoding="utf-8"))

    profile = prep["classification"].get("profile", "standard")
    flags = prep["classification"].get("risk_flags", [])
    pe, opt, wopt = select_rules(prep)
    reviewers = reviewers_for(profile, flags)
    checks = ["prompt_static_check.py", "prompt_token_report.py"]
    if any(str(p).endswith(".md") for p in prep.get("targets", [])):
        checks.append("render changed prompts when renderer is available")

    contract = {
        "schema": "iterate-edit-contract-v4",
        "request": prep.get("request", ""),
        "profile": profile,
        "risk_flags": flags,
        "targets": prep.get("targets", []),
        "required_reads": prep.get("required_reads", []),
        "artifacts": prep.get("artifacts", {}),
        "rules": {"PE": pe, "OPT": opt, "WOPT": wopt},
        "checks": checks,
        "reviewers": reviewers,
        "source_docs": {
            "prompt_engineering": existing_doc(repo, [".opencode/agent/_iterate/docs/prompt-engineering.md", "workflow/prompt-engineering.md"]),
            "design_patterns": existing_doc(repo, [".opencode/agent/_iterate/docs/design-patterns.md", "workflow/design-patterns.md"]),
            "optimize_patterns": existing_doc(repo, [".opencode/agent/_iterate/docs/optimize-patterns.md", "workflow/optimize-patterns.md"]),
        },
    }

    out = Path(args.out)
    if not out.is_absolute():
        out = (repo / out).resolve()
    out.parent.mkdir(parents=True, exist_ok=True)
    write_contract_md(contract, out)
    json_out = out.with_suffix(".json")
    json_out.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")
    print("# CONTRACT")
    print(f"Profile: {profile}")
    print(f"Contract: {out.as_posix()}")
    print(f"Contract JSON: {json_out.as_posix()}")
    print(f"Reviewers: {', '.join(reviewers) if reviewers else 'None'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
