#!/usr/bin/env python3
"""Emit compact metrics-first digest for one opencode-sessions export bundle.

Reads an export directory produced by `opencode-sessions export` and prints a
structured digest to stdout. The optimizer uses this to decide whether a
candidate workflow change helped or hurt.

Output sections (in order):
  # EXPORT DIGEST           — root session metadata (ID, title, status, turn count, etc.)
  ## Tree Token Totals      — aggregate tokens summed across root + all child sessions
  ## Root Token Totals      — tokens for the root (orchestrator) session only
  ## Child Generated Spread — max/min/spread of child output+reasoning tokens
                               (high spread → unbalanced reviewer workload)
  ## Child Sessions         — per-child breakdown: agent, status, generated tokens,
                               input, output, reasoning, cache, tools, decision
  ## Top Child Generated Tokens — top 5 children by output+reasoning (cost hotspots)
  ## Duplicate Child Reads  — files read by multiple child agents (waste signal)
  Child Summary Files       — relative paths to each child's summary.json

Key metric: output_plus_reasoning_tokens.
  - Tree level  → primary: did the change reduce total generated tokens?
  - Child level → secondary: which reviewer dominates cost? is spread balanced?

Duplicate reads signal redundant context loading across reviewers — a target
for scope-boundary or reviewer-merging strategies.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def main() -> int:
    args = parse_args()
    export_path = Path(args.export_path).resolve()
    data = load_json(export_path / "index.json")

    session_index = data.get("session_index") or []
    if not session_index:
        raise SystemExit("index.json missing session_index")

    root = session_index[0]
    root_summary_file = root["summary_file"]
    root_summary = read_summary(export_path, root_summary_file)
    child_files = child_summary_files(data)
    child_rows, duplicate_reads = build_child_rows(export_path, data, child_files)

    print_header(export_path, data, root, root_summary_file, child_files)
    print_token_totals("Tree", data.get("totals") or {})
    print_token_totals("Root", totals_from_summary(root_summary))
    print_child_spread(child_rows)
    print_child_sessions(child_rows)
    print_top_child_generated(child_rows)
    print_duplicate_child_reads(duplicate_reads)
    print_child_summary_files(child_files)
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("export_path")
    return parser.parse_args()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def read_summary(export_path: Path, rel_path: str) -> dict:
    try:
        return load_json(export_path / rel_path)
    except Exception as exc:
        return {"_read_error": str(exc)}


def child_summary_files(index: dict) -> list[str]:
    files = []
    for child in (index.get("tree") or {}).get("children") or []:
        summary_file = child.get("summary_file")
        if summary_file:
            files.append(summary_file)
    return files


def build_child_rows(export_path: Path, index: dict, child_files: list[str]) -> tuple[list[dict], dict[str, dict]]:
    summary_meta = session_meta_by_summary(index)
    child_rows = []
    duplicate_reads: dict[str, dict] = {}

    for rel_path in child_files:
        summary = read_summary(export_path, rel_path)
        meta = summary_meta.get(rel_path, {})
        row = child_row(rel_path, summary, meta)
        child_rows.append(row)
        collect_duplicate_reads(duplicate_reads, row["agent"], summary)

    return child_rows, duplicate_reads


def session_meta_by_summary(index: dict) -> dict[str, dict]:
    result = {}
    for item in index.get("session_index") or []:
        summary_file = item.get("summary_file")
        if summary_file:
            result[summary_file] = item
    return result


def child_row(rel_path: str, summary: dict, meta: dict) -> dict:
    totals = totals_from_summary(summary)
    session = summary.get("session") or {}
    agent = meta.get("agent") or session.get("agent") or "unknown"
    title = meta.get("title") or session.get("title") or "unknown"
    return {
        "path": meta.get("session_path") or session.get("session_path") or "?",
        "agent": agent,
        "title": title,
        "status": meta.get("session_status") or summary.get("session_status") or "unknown",
        "duration_ms": meta.get("duration_ms") or session.get("duration_ms") or 0,
        "tools": meta.get("tool_call_count") or totals.get("tool_calls") or 0,
        "input": token_value(totals, "input_tokens"),
        "output": token_value(totals, "output_tokens"),
        "reasoning": token_value(totals, "reasoning_tokens"),
        "cache_read": token_value(totals, "cache_read_tokens"),
        "generated": output_plus_reasoning(totals),
        "total": total_no_cache(totals),
        "decision": decision_from_summary(summary),
        "summary_file": rel_path,
    }


def collect_duplicate_reads(duplicate_reads: dict[str, dict], agent: str, summary: dict) -> None:
    for access in summary.get("file_access_rollup") or []:
        path = access.get("path")
        if not path:
            continue
        entry = duplicate_reads.setdefault(path, {"read_count": 0, "readers": set()})
        entry["read_count"] += access.get("read_count") or 0
        entry["readers"].add(agent)


def totals_from_summary(summary: dict) -> dict:
    return summary.get("totals") or {}


def token_value(totals: dict, key: str) -> int:
    value = totals.get(key, 0)
    return value if isinstance(value, int) else 0


def output_plus_reasoning(totals: dict) -> int:
    return token_value(totals, "output_tokens") + token_value(totals, "reasoning_tokens")


def total_no_cache(totals: dict) -> int:
    return token_value(totals, "input_tokens") + output_plus_reasoning(totals)


def decision_from_summary(summary: dict) -> str:
    text = "\n".join(
        turn.get("final_assistant_text_preview", "")
        for turn in summary.get("hot_turns") or []
        if turn.get("final_assistant_text_preview")
    )
    match = re.search(r"Decision:\s*([A-Z_ -]+)", text)
    if match:
        parts = match.group(1).strip().split()
        if parts:
            return parts[0]
    if "Status: SUCCESS" in text:
        return "SUCCESS"
    if "Status: FAIL" in text:
        return "FAIL"
    return "unknown"


def print_header(export_path: Path, data: dict, root: dict, root_summary_file: str, child_files: list[str]) -> None:
    print("# EXPORT DIGEST")
    print(f"Export Path: {export_path}")
    print(f"Root Session ID: {data.get('root_session_id', 'unknown')}")
    print(f"Root Title: {data.get('root_title', root.get('title', 'unknown'))}")
    print(f"Root Status: {data.get('root_session_status', root.get('session_status', 'unknown'))}")
    print(f"Root Duration Ms: {root.get('duration_ms', 'unknown')}")
    print(f"Root Turn Count: {root.get('turn_count', 'unknown')}")
    print(f"Root Tool Calls: {root.get('tool_call_count', 'unknown')}")
    print(f"Root Summary File: {root_summary_file}")
    print(f"Root Turns Compact File: {root['turns_compact_file']}")
    print(f"Root Messages Compact File: {root['messages_compact_file']}")
    print(f"Child Sessions: {len(child_files)}")


def print_token_totals(label: str, totals: dict) -> None:
    print(f"\n## {label} Token Totals")
    print(f"- output_plus_reasoning_tokens: {output_plus_reasoning(totals)}")
    print(f"- output_tokens: {token_value(totals, 'output_tokens')}")
    print(f"- reasoning_tokens: {token_value(totals, 'reasoning_tokens')}")
    print(f"- input_tokens: {token_value(totals, 'input_tokens')}")
    print(f"- cache_read_tokens: {token_value(totals, 'cache_read_tokens')}")
    print(f"- total_tokens_no_cache: {total_no_cache(totals)}")


def print_child_spread(child_rows: list[dict]) -> None:
    values = [row["generated"] for row in child_rows]
    max_generated = max(values) if values else 0
    min_generated = min(values) if values else 0
    print("\n## Child Generated Spread")
    print(f"- max_child_output_plus_reasoning_tokens: {max_generated}")
    print(f"- min_child_output_plus_reasoning_tokens: {min_generated}")
    print(f"- spread_child_output_plus_reasoning_tokens: {max_generated - min_generated if values else 0}")


def print_child_sessions(child_rows: list[dict]) -> None:
    print("\n## Child Sessions")
    if not child_rows:
        print("- None")
        return
    for row in child_rows:
        print(
            f"- path={row['path']} agent={row['agent']} status={row['status']} "
            f"generated={row['generated']} output={row['output']} reasoning={row['reasoning']} "
            f"input={row['input']} cache_read={row['cache_read']} tools={row['tools']} "
            f"duration_ms={row['duration_ms']} decision={row['decision']} "
            f"title=\"{compact_title(row['title'])}\" summary={row['summary_file']}"
        )


def print_top_child_generated(child_rows: list[dict]) -> None:
    print("\n## Top Child Generated Tokens")
    if not child_rows:
        print("- None")
        return
    for row in sorted(child_rows, key=lambda item: item["generated"], reverse=True)[:5]:
        print(f"- {row['agent']} path={row['path']} generated={row['generated']} tools={row['tools']} title=\"{compact_title(row['title'], 60)}\"")


def print_duplicate_child_reads(duplicate_reads: dict[str, dict]) -> None:
    print("\n## Duplicate Child Reads")
    duplicates = []
    for path, entry in duplicate_reads.items():
        readers = sorted(entry["readers"])
        if len(readers) > 1:
            duplicates.append((path, entry["read_count"], readers))
    duplicates.sort(key=lambda item: (len(item[2]), item[1]), reverse=True)
    if not duplicates:
        print("- None")
        return
    for path, read_count, readers in duplicates[:10]:
        print(f"- readers={len(readers)} reads={read_count} path={path} agents={','.join(readers[:6])}")


def print_child_summary_files(child_files: list[str]) -> None:
    print("\nChild Summary Files:")
    if not child_files:
        print("- None")
        return
    for path in child_files:
        print(f"- {path}")


def compact_title(value: str, limit: int = 80) -> str:
    value = " ".join((value or "unknown").split())
    if len(value) <= limit:
        return value
    return value[: limit - 1] + "…"


if __name__ == "__main__":
    raise SystemExit(main())
