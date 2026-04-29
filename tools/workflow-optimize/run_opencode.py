#!/usr/bin/env python3
"""Run `opencode run --format json` and save compact metadata."""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
import threading
import time
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=int, required=True)
    parser.add_argument("--task", required=True)
    parser.add_argument("--command", dest="target_command", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--model")
    parser.add_argument("--file", dest="files", action="append", default=[])
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--meta-out", required=True)
    parser.add_argument(
        "--raw-out",
        help="Optional debug file for full nested stdout stream. Keep unset by default.",
    )
    return parser.parse_args()


def build_argv(args: argparse.Namespace) -> list[str]:
    argv = [
        "opencode",
        "run",
        "--format",
        "json",
        "--command",
        args.target_command,
        "--title",
        args.title,
    ]
    if args.model:
        argv.extend(["--model", args.model])
    for file_path in args.files:
        argv.extend(["--file", file_path])
    argv.append(args.prompt)
    return argv


def drain_stderr(stream: Any, sink: list[str], counter: list[int]) -> None:
    if stream is None:
        return
    for line in stream:
        counter[0] += 1
        text = line.rstrip("\n")
        if text and len(sink) < 5:
            sink.append(text[:200])


def main() -> int:
    args = parse_args()
    argv = build_argv(args)
    started_at = time.time()

    raw_handle = None
    if args.raw_out:
        raw_path = pathlib.Path(args.raw_out)
        raw_path.parent.mkdir(parents=True, exist_ok=True)
        raw_handle = raw_path.open("w", encoding="utf-8")

    proc = subprocess.Popen(
        argv,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    stderr_preview: list[str] = []
    stderr_count = [0]
    stderr_thread = threading.Thread(
        target=drain_stderr,
        args=(proc.stderr, stderr_preview, stderr_count),
        daemon=True,
    )
    stderr_thread.start()

    session_id = None
    event_counts: dict[str, int] = {}
    event_errors: list[str] = []
    non_json_preview: list[str] = []
    line_count = 0
    json_event_count = 0
    json_parse_failures = 0

    assert proc.stdout is not None
    for raw_line in proc.stdout:
        if raw_handle is not None:
            raw_handle.write(raw_line)

        line_count += 1
        line = raw_line.strip()
        if not line:
            continue

        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            json_parse_failures += 1
            if len(non_json_preview) < 5:
                non_json_preview.append(line[:200])
            continue

        json_event_count += 1
        if session_id is None and "sessionID" in event:
            session_id = event["sessionID"]

        event_type = event.get("type", "unknown")
        event_counts[event_type] = event_counts.get(event_type, 0) + 1

        if event.get("type") == "error" or event.get("error"):
            if len(event_errors) < 5:
                event_errors.append(str(event.get("error", event))[:200])

    nested_return_code = proc.wait()
    stderr_thread.join(timeout=5)
    finished_at = time.time()
    elapsed_ms = int((finished_at - started_at) * 1000)

    if raw_handle is not None:
        raw_handle.close()

    errors = list(event_errors)
    if nested_return_code != 0:
        if stderr_preview:
            errors.extend(f"stderr: {line}" for line in stderr_preview)
        elif non_json_preview:
            errors.extend(f"non-json: {line}" for line in non_json_preview)
    errors = errors[:5]

    validation_errors: list[str] = []
    if nested_return_code != 0:
        validation_errors.append(f"nested return code {nested_return_code}")
    if session_id is None:
        validation_errors.append("missing sessionID")
    if json_event_count == 0:
        validation_errors.append("no JSON events parsed")

    meta = {
        "run": args.run,
        "task_case": args.task,
        "title": args.title,
        "target_command": args.target_command,
        "model": args.model,
        "files": args.files,
        "session_id": session_id,
        "started_at": int(started_at * 1000),
        "finished_at": int(finished_at * 1000),
        "elapsed_ms": elapsed_ms,
        "event_counts": event_counts,
        "line_count": line_count,
        "json_event_count": json_event_count,
        "json_parse_failures": json_parse_failures,
        "stderr_count": stderr_count[0],
        "stderr_preview": stderr_preview,
        "error_count": len(errors),
        "errors": errors,
        "validation_errors": validation_errors,
        "return_code": nested_return_code,
    }

    meta_path = pathlib.Path(args.meta_out)
    meta_path.parent.mkdir(parents=True, exist_ok=True)
    meta_path.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")

    print(
        "RUN"
        f" {args.run}"
        f" task={args.task}"
        f" session={session_id or 'none'}"
        f" rc={nested_return_code}"
        f" elapsed_ms={elapsed_ms}"
        f" events={json.dumps(event_counts, sort_keys=True)}"
    )

    return 0 if not validation_errors else 1


if __name__ == "__main__":
    sys.exit(main())
