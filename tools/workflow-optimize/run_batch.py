#!/usr/bin/env python3
"""Run opencode sessions for workflow optimization experiments.

Two modes:
  Worktree mode (default): creates git worktrees, runs N samples in
  parallel, captures metadata, cleans up worktrees. Designed for
  multi-sample averaging to handle LLM non-determinism.

  Direct mode (--no-worktree): runs a single session in the current
  working directory without worktrees. Simpler but no isolation.

Usage (parallel, 3 samples):
  python3 run_batch.py --samples 3 --run 1 --task help-page \\
    --command plan/finalize --title "baseline run-1" \\
    --model sewer-axonhub/wafer/GLM-5.1 \\
    --file PROMPT-PLAN-help-page.draft.md \\
    --prompt "Finalize the plan..." \\
    --meta-dir .opencode/workflow-optimize/slug/events \\
    --repo /home/sewer/Project/nixos-homelab --slug help-page

Usage (direct, single session, no worktree):
  python3 run_batch.py --samples 1 --run 1 --task help-page \\
    --command plan/finalize --title "baseline run-1" \\
    --model sewer-axonhub/wafer/GLM-5.1 \\
    --file PROMPT-PLAN-help-page.draft.md \\
    --prompt "Finalize the plan..." \\
    --meta-dir .opencode/workflow-optimize/slug/events \\
    --repo /home/sewer/Project/nixos-homelab --slug help-page \\
    --no-worktree
"""

import argparse
import json
import shutil
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path


# ---------------------------------------------------------------------------
# Shared session runner (parses opencode JSON stream)
# ---------------------------------------------------------------------------

def run_opencode_session(cmd, cwd=None):
    """Run an opencode command and parse its JSON event stream.

    Returns a metadata dict with token counts, tool calls, session ID, etc.
    """
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
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            cwd=cwd,
        )

        for line in proc.stdout:
            line = line.strip()
            if not line:
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue

            if session_id is None:
                sid = event.get("sessionID")
                if sid:
                    session_id = sid

            evt_type = event.get("type", "")
            part = event.get("part", {})

            if evt_type == "step_start":
                step_count += 1
            elif evt_type == "tool_use":
                tool_use_count += 1
            elif evt_type == "text":
                text = part.get("text", "")
                if text:
                    last_text = text
            elif evt_type == "step_finish":
                tokens = part.get("tokens", {})
                if tokens:
                    input_tokens += tokens.get("input", 0)
                    output_tokens += tokens.get("output", 0)
                    reasoning_tokens += tokens.get("reasoning", 0)
                    cache = tokens.get("cache", {})
                    cache_read_tokens += cache.get("read", 0)
                    cache_write_tokens += cache.get("write", 0)
                reason = part.get("reason", "")
                if reason == "stop":
                    final_status = "COMPLETED"
                    stop_reason = "stop"

        proc.wait()
        elapsed = time.time() - start_time

        if proc.returncode != 0:
            final_status = f"ERROR(rc={proc.returncode})"
        elif final_status == "UNKNOWN":
            final_status = "INTERRUPTED"

    except Exception as e:
        elapsed = time.time() - start_time
        final_status = f"EXCEPTION: {e}"

    return {
        "session_id": session_id,
        "status": final_status,
        "stop_reason": stop_reason,
        "elapsed_seconds": round(elapsed, 2),
        "step_count": step_count,
        "tool_use_count": tool_use_count,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "reasoning_tokens": reasoning_tokens,
        "cache_read_tokens": cache_read_tokens,
        "cache_write_tokens": cache_write_tokens,
        "total_tokens": input_tokens + output_tokens + reasoning_tokens + cache_read_tokens + cache_write_tokens,
        "status_line": last_text[:500],
    }


# ---------------------------------------------------------------------------
# Worktree mode: one sample per worktree, parallel execution
# ---------------------------------------------------------------------------

ARTIFACT_CLEANUP_PATTERNS = [
    "PROMPT-PLAN-*.handoff.md",
    "PROMPT-PLAN-*.step.*.md",
    "PROMPT-PLAN-*.review-*.md",
]


def run_worktree_sample(args_tuple):
    """Run a single opencode session in a git worktree."""
    (sample_idx, repo_root, branch, worktree_base, command, title,
     model, files, prompt, slug) = args_tuple

    worktree_dir = worktree_base / f"wt-sample-{sample_idx}"

    # Create worktree
    try:
        subprocess.run(
            ["git", "worktree", "add", str(worktree_dir), branch],
            cwd=str(repo_root),
            capture_output=True,
            text=True,
            timeout=30,
        )
    except Exception as e:
        return (sample_idx, {"error": f"worktree create failed: {e}"})

    # Clean artifacts from worktree
    for pattern in ARTIFACT_CLEANUP_PATTERNS:
        for f in worktree_dir.glob(pattern):
            try:
                f.unlink()
            except Exception:
                pass

    # Copy untracked attached files into worktree
    for f in files:
        src = repo_root / f
        dst = worktree_dir / f
        if src.exists() and not dst.exists():
            try:
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(str(src), str(dst))
            except Exception:
                pass

    # Build opencode command
    cmd = [
        "opencode", "run",
        "--format", "json",
        "--command", command,
        "--title", f"{title} sample-{sample_idx}",
        "--dir", str(worktree_dir),
    ]
    if model:
        cmd.extend(["--model", model])
    for f in files:
        cmd.extend(["--file", f])
    cmd.append("--")
    cmd.append(prompt)

    meta = run_opencode_session(cmd)
    meta["sample"] = sample_idx
    meta["command"] = command
    meta["title"] = f"{title} sample-{sample_idx}"
    meta["model"] = model

    # Clean up worktree
    try:
        subprocess.run(
            ["git", "worktree", "remove", "--force", str(worktree_dir)],
            cwd=str(repo_root),
            capture_output=True,
            text=True,
            timeout=15,
        )
    except Exception:
        pass

    return (sample_idx, meta)


# ---------------------------------------------------------------------------
# Direct mode: run in cwd, no worktree
# ---------------------------------------------------------------------------

def run_direct_sample(sample_idx, repo_root, command, title, model, files, prompt):
    """Run a single opencode session directly in the repo directory."""
    # Clean artifacts from repo
    for pattern in ARTIFACT_CLEANUP_PATTERNS:
        for f in repo_root.glob(pattern):
            try:
                f.unlink()
            except Exception:
                pass

    cmd = [
        "opencode", "run",
        "--format", "json",
        "--command", command,
        "--title", f"{title} sample-{sample_idx}",
        "--dir", str(repo_root),
    ]
    if model:
        cmd.extend(["--model", model])
    for f in files:
        cmd.extend(["--file", f])
    cmd.append("--")
    cmd.append(prompt)

    meta = run_opencode_session(cmd)
    meta["sample"] = sample_idx
    meta["command"] = command
    meta["title"] = f"{title} sample-{sample_idx}"
    meta["model"] = model

    return (sample_idx, meta)


# ---------------------------------------------------------------------------
# Main: parse args, dispatch to worktree or direct mode, compute averages
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Run opencode sessions for workflow optimization")
    parser.add_argument("--samples", required=True, type=int, help="Number of parallel samples")
    parser.add_argument("--run", required=True, type=int, help="Batch/run number")
    parser.add_argument("--task", required=True, help="Task label")
    parser.add_argument("--command", required=True, help="CLI command (slashless)")
    parser.add_argument("--title", required=True, help="Base session title")
    parser.add_argument("--model", default=None, help="Model override")
    parser.add_argument("--file", action="append", default=[], help="Files to attach (repo-relative)")
    parser.add_argument("--prompt", required=True, help="Prompt message")
    parser.add_argument("--meta-dir", required=True, help="Directory for metadata JSON outputs")
    parser.add_argument("--repo", required=True, help="Absolute path to git repo root")
    parser.add_argument("--slug", required=True, help="Experiment slug for worktree naming")
    parser.add_argument("--no-worktree", action="store_true",
                        help="Run directly in repo dir instead of git worktrees")

    args = parser.parse_args()
    repo_root = Path(args.repo)

    results = {}
    overall_start = time.time()

    if args.no_worktree:
        # Direct mode: run samples sequentially (no isolation)
        for i in range(1, args.samples + 1):
            idx, meta = run_direct_sample(
                i, repo_root, args.command, args.title,
                args.model, args.file, args.prompt,
            )
            results[idx] = meta
            sid = meta.get("session_id", "N/A")
            status = meta.get("status", "UNKNOWN")
            elapsed = meta.get("elapsed_seconds", 0)
            tokens = meta.get("total_tokens", 0)
            tools = meta.get("tool_use_count", 0)
            print(f"  Sample {i}: session={sid} status={status} "
                  f"elapsed={elapsed:.1f}s tokens={tokens} tools={tools}")
    else:
        # Worktree mode: run samples in parallel
        worktree_base = repo_root / ".opencode" / "workflow-optimize" / args.slug / "worktrees"

        branch_result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            cwd=str(repo_root),
            capture_output=True,
            text=True,
        )
        branch = branch_result.stdout.strip() or "HEAD"
        worktree_base.mkdir(parents=True, exist_ok=True)

        sample_args = []
        for i in range(1, args.samples + 1):
            sample_args.append((
                i, repo_root, branch, worktree_base,
                args.command, args.title, args.model,
                args.file, args.prompt, args.slug,
            ))

        with ProcessPoolExecutor(max_workers=args.samples) as executor:
            futures = {
                executor.submit(run_worktree_sample, sa): sa[0]
                for sa in sample_args
            }
            for future in as_completed(futures):
                sample_idx = futures[future]
                try:
                    idx, meta = future.result()
                    results[idx] = meta
                    sid = meta.get("session_id", "N/A")
                    status = meta.get("status", "UNKNOWN")
                    elapsed = meta.get("elapsed_seconds", 0)
                    tokens = meta.get("total_tokens", 0)
                    tools = meta.get("tool_use_count", 0)
                    print(f"  Sample {sample_idx}: session={sid} status={status} "
                          f"elapsed={elapsed:.1f}s tokens={tokens} tools={tools}")
                except Exception as e:
                    results[sample_idx] = {"error": str(e)}
                    print(f"  Sample {sample_idx}: FAILED - {e}")

    overall_elapsed = time.time() - overall_start

    # Write individual meta files
    meta_dir = Path(args.meta_dir)
    meta_dir.mkdir(parents=True, exist_ok=True)

    for idx, meta in sorted(results.items()):
        meta["run"] = args.run
        meta["task"] = args.task
        meta_path = meta_dir / f"run-{args.run:03d}-sample-{idx:03d}.meta.json"
        meta_path.write_text(json.dumps(meta, indent=2) + "\n")

    # Compute averages
    completed = [m for m in results.values() if m.get("status") == "COMPLETED"]
    discarded = {idx: m for idx, m in results.items() if m.get("status") != "COMPLETED"}

    if completed:
        avg_tokens = sum(m["total_tokens"] for m in completed) / len(completed)
        avg_output = sum(m["output_tokens"] for m in completed) / len(completed)
        avg_input = sum(m["input_tokens"] for m in completed) / len(completed)
        avg_reasoning = sum(m["reasoning_tokens"] for m in completed) / len(completed)
        avg_cache_read = sum(m["cache_read_tokens"] for m in completed) / len(completed)
        avg_elapsed = sum(m["elapsed_seconds"] for m in completed) / len(completed)
        avg_tools = sum(m["tool_use_count"] for m in completed) / len(completed)
        avg_steps = sum(m["step_count"] for m in completed) / len(completed)
        session_ids = [m["session_id"] for m in completed if m.get("session_id")]
    else:
        avg_tokens = avg_output = avg_input = avg_reasoning = 0
        avg_cache_read = avg_elapsed = avg_tools = avg_steps = 0
        session_ids = []

    summary = {
        "run": args.run,
        "task": args.task,
        "slug": args.slug,
        "mode": "direct" if args.no_worktree else "worktree",
        "samples_requested": args.samples,
        "samples_completed": len(completed),
        "samples_discarded": len(discarded),
        "discarded_details": {str(k): v.get("status", "UNKNOWN") for k, v in discarded.items()},
        "session_ids": session_ids,
        "avg_total_tokens": round(avg_tokens, 1),
        "avg_output_tokens": round(avg_output, 1),
        "avg_input_tokens": round(avg_input, 1),
        "avg_reasoning_tokens": round(avg_reasoning, 1),
        "avg_cache_read_tokens": round(avg_cache_read, 1),
        "avg_elapsed_seconds": round(avg_elapsed, 1),
        "avg_tool_calls": round(avg_tools, 1),
        "avg_steps": round(avg_steps, 1),
        "wall_elapsed_seconds": round(overall_elapsed, 1),
    }

    summary_path = meta_dir / f"run-{args.run:03d}-summary.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n")

    print(f"\nRun {args.run} complete: {len(completed)}/{args.samples} samples succeeded"
          f" ({'direct' if args.no_worktree else 'worktree'} mode)")
    print(f"  Avg tokens: {avg_tokens:.0f} (out={avg_output:.0f}, in={avg_input:.0f}, "
          f"reason={avg_reasoning:.0f}, cache={avg_cache_read:.0f})")
    print(f"  Avg elapsed: {avg_elapsed:.1f}s | Wall time: {overall_elapsed:.1f}s")
    print(f"  Avg tool calls: {avg_tools:.0f} | Avg steps: {avg_steps:.0f}")
    print(f"  Sessions: {session_ids}")

    return 0 if completed else 1


if __name__ == "__main__":
    sys.exit(main())
