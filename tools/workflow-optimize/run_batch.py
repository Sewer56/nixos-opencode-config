#!/usr/bin/env python3
"""Run three identical OpenCode sessions in parallel, isolated workspace copies.

What it does (end to end):
  1. Copies the source repo to /tmp per sample so sessions never share state.
  2. Cleans old workflow artifacts from the source tree (before copy) and each
     workspace (after copy) using caller-supplied glob patterns.
  3. Spawns three opencode sessions in parallel via ProcessPoolExecutor.
  4. Streams JSON events from each session; captures token breakdowns, step
     counts, tool-use counts, elapsed time, and exit status.
  5. Writes per-sample <prefix>.meta.json files and a <prefix>-summary.json
     with avg/median/spread for every metric.
  6. Deletes each workspace immediately after its session finishes.

Output files:
  <meta-dir>/run-<run:03d>-<task>-sample-<sample:03d>.meta.json  — per session
  <meta-dir>/run-<run:03d>-<task>-summary.json                   — aggregate

Per-sample meta.json captures:
  session_id, status, stop_reason, elapsed_seconds,
  step_count, tool_use_count,
  input_tokens, output_tokens, reasoning_tokens,
  output_plus_reasoning_tokens,  ← primary optimizer metric (early signal)
  cache_read_tokens, cache_write_tokens,
  total_tokens, total_with_cache_tokens,
  status_line (last 500 chars of session output).

Summary aggregates across completed samples:
  avg/median/spread/spread_pct for every token and tool/step metric,
  session_ids, samples_completed, samples_discarded, wall_elapsed_seconds.

The summary is the primary input for the optimizer's compare-and-decide step.
Final decisions use export-derived tree output+reasoning (see export_digest.py).
Session stdout/stderr is interleaved (stderr merged to stdout) so the caller
sees live progress per sample.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path
from statistics import median


SAMPLES = 3


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo).resolve()
    cleanup_patterns = args.cleanup_pattern
    task_label = safe_label(args.task)
    overall_start = time.time()

    cleanup_artifacts(repo_root, cleanup_patterns)

    results = run_samples(args, repo_root, cleanup_patterns, task_label)

    wall_elapsed = round(time.time() - overall_start, 1)
    summary = write_results(args, results, task_label, cleanup_patterns, wall_elapsed)
    print_summary(summary)
    return 0 if summary["samples_completed"] else 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run three opencode sessions for workflow optimization")
    parser.add_argument("--run", required=True, type=int, help="Batch/run number")
    parser.add_argument("--task", required=True, help="Task label")
    parser.add_argument("--command", required=True, help="CLI command (slashless)")
    parser.add_argument("--title", required=True, help="Base session title")
    parser.add_argument("--model", default=None, help="Model to use")
    parser.add_argument("--file", action="append", default=[], help="Files to attach (repo-relative)")
    parser.add_argument("--prompt", required=True, help="Prompt message")
    parser.add_argument("--meta-dir", required=True, help="Directory for metadata JSON outputs")
    parser.add_argument("--repo", required=True, help="Absolute path to source workspace root")
    parser.add_argument("--slug", required=True, help="Experiment slug for workspace naming")
    parser.add_argument("--cleanup-pattern", action="append", default=[], help="Artifact glob to remove before each sample")
    return parser.parse_args()


def safe_label(value: str) -> str:
    cleaned = []
    for ch in value.lower():
        if ch.isalnum() or ch in {"-", "_", "."}:
            cleaned.append(ch)
        else:
            cleaned.append("-")
    label = "".join(cleaned).strip("-._")
    while "--" in label:
        label = label.replace("--", "-")
    return label or "task"


def cleanup_artifacts(root: Path, patterns: list[str]) -> None:
    for pattern in patterns:
        for path in root.glob(pattern):
            try:
                if path.is_dir():
                    shutil.rmtree(path)
                else:
                    path.unlink()
            except Exception:
                pass


def run_samples(args: argparse.Namespace, repo_root: Path, cleanup_patterns: list[str], task_label: str) -> dict[int, dict]:
    workspace_base = Path("/tmp") / f"workflow-optimize-{args.slug}-workspaces" / f"run-{args.run:03d}-{task_label}"
    workspace_base.mkdir(parents=True, exist_ok=True)

    sample_args = [
        (sample, args, repo_root, workspace_base, cleanup_patterns)
        for sample in range(1, SAMPLES + 1)
    ]

    results = {}
    with ProcessPoolExecutor(max_workers=SAMPLES) as executor:
        futures = {executor.submit(run_workspace_sample, item): item[0] for item in sample_args}
        for future in as_completed(futures):
            sample = futures[future]
            try:
                idx, meta = future.result()
            except Exception as exc:
                idx, meta = sample, {"error": str(exc), "status": f"EXCEPTION: {exc}"}
            results[idx] = meta
            print_sample(idx, meta)
    return results


def run_workspace_sample(args_tuple) -> tuple[int, dict]:
    sample, args, repo_root, workspace_base, cleanup_patterns = args_tuple
    workspace_dir = workspace_base / f"sample-{sample}"

    try:
        copy_workspace(repo_root, workspace_dir)
    except Exception as exc:
        return sample, {"error": f"workspace copy failed: {exc}", "status": f"EXCEPTION: {exc}"}

    try:
        cleanup_artifacts(workspace_dir, cleanup_patterns)
        cmd = build_opencode_cmd(args, sample, workspace_dir)
        meta = run_opencode_session(cmd)
        add_sample_fields(meta, sample, args)
        return sample, meta
    finally:
        shutil.rmtree(workspace_dir, ignore_errors=True)


def copy_workspace(repo_root: Path, workspace_dir: Path) -> None:
    if workspace_dir.exists():
        shutil.rmtree(workspace_dir, ignore_errors=True)
    workspace_dir.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(repo_root, workspace_dir, symlinks=True)


def build_opencode_cmd(args: argparse.Namespace, sample: int, run_dir: Path) -> list[str]:
    opencode_bin = shutil.which("opencode") or "opencode"
    cmd = [
        opencode_bin,
        "run",
        "--format",
        "json",
        "--command",
        args.command,
        "--title",
        f"{args.title} sample-{sample}",
        "--dir",
        str(run_dir),
    ]
    if args.model:
        cmd.extend(["--model", args.model])
    for file_path in args.file:
        cmd.extend(["--file", file_path])
    cmd.extend(["--", args.prompt])
    return cmd


def run_opencode_session(cmd: list[str]) -> dict:
    start_time = time.time()
    session_id = None
    input_tokens = 0
    output_tokens = 0
    reasoning_tokens = 0
    cache_read_tokens = 0
    cache_write_tokens = 0
    tool_use_count = 0
    step_count = 0
    final_status = "UNKNOWN"
    stop_reason = None
    last_text = ""

    try:
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )

        for raw_line in proc.stdout:
            line = raw_line.strip()
            if not line:
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue

            if session_id is None and event.get("sessionID"):
                session_id = event["sessionID"]

            event_type = event.get("type", "")
            part = event.get("part", {})

            if event_type == "step_start":
                step_count += 1
            elif event_type == "tool_use":
                tool_use_count += 1
            elif event_type == "text" and part.get("text"):
                last_text = part["text"]
            elif event_type == "step_finish":
                tokens = part.get("tokens", {})
                input_tokens += tokens.get("input", 0)
                output_tokens += tokens.get("output", 0)
                reasoning_tokens += tokens.get("reasoning", 0)
                cache = tokens.get("cache", {})
                cache_read_tokens += cache.get("read", 0)
                cache_write_tokens += cache.get("write", 0)
                if part.get("reason") == "stop":
                    final_status = "COMPLETED"
                    stop_reason = "stop"

        proc.wait()
        if proc.returncode != 0:
            final_status = f"ERROR(rc={proc.returncode})"
        elif final_status == "UNKNOWN":
            final_status = "INTERRUPTED"
    except Exception as exc:
        final_status = f"EXCEPTION: {exc}"

    elapsed = round(time.time() - start_time, 2)
    return {
        "session_id": session_id,
        "status": final_status,
        "stop_reason": stop_reason,
        "elapsed_seconds": elapsed,
        "step_count": step_count,
        "tool_use_count": tool_use_count,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "reasoning_tokens": reasoning_tokens,
        "output_plus_reasoning_tokens": output_tokens + reasoning_tokens,
        "cache_read_tokens": cache_read_tokens,
        "cache_write_tokens": cache_write_tokens,
        "total_tokens": input_tokens + output_tokens + reasoning_tokens,
        "total_with_cache_tokens": input_tokens + output_tokens + reasoning_tokens + cache_read_tokens + cache_write_tokens,
        "status_line": last_text[:500],
    }


def add_sample_fields(meta: dict, sample: int, args: argparse.Namespace) -> None:
    meta["sample"] = sample
    meta["command"] = args.command
    meta["title"] = f"{args.title} sample-{sample}"
    meta["model"] = args.model


def print_sample(sample: int, meta: dict) -> None:
    sid = meta.get("session_id", "N/A")
    status = meta.get("status", "UNKNOWN")
    elapsed = meta.get("elapsed_seconds", 0)
    tokens = meta.get("total_tokens", 0)
    tools = meta.get("tool_use_count", 0)
    print(f"  Sample {sample}: session={sid} status={status} elapsed={elapsed:.1f}s tokens={tokens} tools={tools}")


def write_results(args: argparse.Namespace, results: dict[int, dict], task_label: str, cleanup_patterns: list[str], wall_elapsed: float) -> dict:
    meta_dir = Path(args.meta_dir)
    meta_dir.mkdir(parents=True, exist_ok=True)

    for idx, meta in sorted(results.items()):
        meta["run"] = args.run
        meta["task"] = args.task
        meta_path = meta_dir / f"run-{args.run:03d}-{task_label}-sample-{idx:03d}.meta.json"
        meta_path.write_text(json.dumps(meta, indent=2) + "\n")

    completed = [meta for meta in results.values() if meta.get("status") == "COMPLETED"]
    discarded = {idx: meta for idx, meta in results.items() if meta.get("status") != "COMPLETED"}
    session_ids = [meta["session_id"] for meta in completed if meta.get("session_id")]

    summary = {
        "run": args.run,
        "task": args.task,
        "slug": args.slug,
        "cleanup_patterns": cleanup_patterns,
        "samples_requested": SAMPLES,
        "samples_completed": len(completed),
        "samples_discarded": len(discarded),
        "discarded_details": {str(idx): meta.get("status", "UNKNOWN") for idx, meta in discarded.items()},
        "session_ids": session_ids,
        "wall_elapsed_seconds": wall_elapsed,
    }

    for key in [
        "total_tokens",
        "total_with_cache_tokens",
        "output_plus_reasoning_tokens",
        "output_tokens",
        "reasoning_tokens",
        "input_tokens",
        "cache_read_tokens",
        "tool_use_count",
        "step_count",
        "elapsed_seconds",
    ]:
        summary.update(metric_stats(completed, key))

    summary["avg_tool_calls"] = summary["avg_tool_use_count"]
    summary["median_tool_calls"] = summary["median_tool_use_count"]
    summary["spread_tool_calls"] = summary["spread_tool_use_count"]
    summary["spread_pct_tool_calls"] = summary["spread_pct_tool_use_count"]
    summary["avg_steps"] = summary["avg_step_count"]
    summary["median_steps"] = summary["median_step_count"]
    summary["spread_steps"] = summary["spread_step_count"]
    summary["spread_pct_steps"] = summary["spread_pct_step_count"]

    summary_path = meta_dir / f"run-{args.run:03d}-{task_label}-summary.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def metric_stats(samples: list[dict], key: str) -> dict:
    values = [meta.get(key, 0) for meta in samples]
    avg = average(values)
    rng = spread(values)
    return {
        f"avg_{key}": rounded(avg),
        f"median_{key}": rounded(median(values)) if values else 0,
        f"spread_{key}": rounded(rng),
        f"spread_pct_{key}": rounded((rng / avg) * 100) if avg else 0,
    }


def average(values: list[float]) -> float:
    return sum(values) / len(values) if values else 0


def spread(values: list[float]) -> float:
    return max(values) - min(values) if values else 0


def rounded(value: float) -> float:
    return round(value, 1)


def print_summary(summary: dict) -> None:
    print(f"\nRun {summary['run']} complete: {summary['samples_completed']}/{SAMPLES} samples succeeded")
    print(
        f"  Avg output+reasoning: {summary['avg_output_plus_reasoning_tokens']:.0f} "
        f"(spread={summary['spread_output_plus_reasoning_tokens']:.0f}, "
        f"spread_pct={summary['spread_pct_output_plus_reasoning_tokens']:.1f}%)"
    )
    print(
        f"  Avg total: {summary['avg_total_tokens']:.0f} "
        f"(out={summary['avg_output_tokens']:.0f}, in={summary['avg_input_tokens']:.0f}, "
        f"reason={summary['avg_reasoning_tokens']:.0f}, cache={summary['avg_cache_read_tokens']:.0f})"
    )
    print(f"  Avg elapsed: {summary['avg_elapsed_seconds']:.1f}s | Wall time: {summary['wall_elapsed_seconds']:.1f}s")
    print(f"  Avg tool calls: {summary['avg_tool_use_count']:.0f} | Avg steps: {summary['avg_step_count']:.0f}")
    print(f"  Sessions: {summary['session_ids']}")


if __name__ == "__main__":
    sys.exit(main())
