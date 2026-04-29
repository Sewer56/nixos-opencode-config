#!/usr/bin/env python3
"""Emit compact root-session digest for one export bundle."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("export_path")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    export_path = Path(args.export_path).resolve()
    index_path = export_path / "index.json"
    data = json.loads(index_path.read_text(encoding="utf-8"))

    session_index = data.get("session_index") or []
    if not session_index:
        raise SystemExit("index.json missing session_index")

    root = session_index[0]
    child_summary_files = []
    tree = data.get("tree") or {}
    for child in tree.get("children") or []:
        summary_file = child.get("summary_file")
        if summary_file:
            child_summary_files.append(summary_file)

    print("# EXPORT DIGEST")
    print(f"Export Path: {export_path}")
    print(f"Root Session ID: {data.get('root_session_id', 'unknown')}")
    print(f"Root Title: {data.get('root_title', root.get('title', 'unknown'))}")
    print(f"Root Status: {data.get('root_session_status', root.get('session_status', 'unknown'))}")
    print(f"Root Duration Ms: {root.get('duration_ms', 'unknown')}")
    print(f"Root Turn Count: {root.get('turn_count', 'unknown')}")
    print(f"Root Tool Calls: {root.get('tool_call_count', 'unknown')}")
    print(f"Root Summary File: {root['summary_file']}")
    print(f"Root Turns Compact File: {root['turns_compact_file']}")
    print(f"Root Messages Compact File: {root['messages_compact_file']}")
    print(f"Child Sessions: {len(child_summary_files)}")
    print("Child Summary Files:")
    if child_summary_files:
        for path in child_summary_files:
            print(f"- {path}")
    else:
        print("- None")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
