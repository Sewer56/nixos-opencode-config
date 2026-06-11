#!/usr/bin/env python3
"""Estimate token count after prompt expansion.

Usage: token-count-after-expand.py [paths...]

Renders each file via md-expand, strips YAML frontmatter, estimates tokens.
No args reads paths from stdin. Output: "tokens  path", one per line, then total.
Diagnostics go to stderr.

Env: BYTES_PER_TOKEN (default 3.0)
"""

import os
import subprocess
import sys

BYTES_PER_TOKEN = float(os.environ.get("BYTES_PER_TOKEN", 3.0))
SCRIPTS_DIR = os.path.dirname(os.path.abspath(__file__))
RENDER_SCRIPT = os.path.join(SCRIPTS_DIR, "render-file.sh")


def strip_frontmatter(text: str) -> str:
    """Strip YAML frontmatter delimited by --- or ..."""
    lines = text.splitlines(keepends=True)
    if not lines or lines[0].strip() != "---":
        return text
    for i in range(1, len(lines)):
        stripped = lines[i].strip()
        if stripped in ("---", "..."):
            return "".join(lines[i + 1 :])
    # No closing delimiter found — return as-is (malformed)
    return text


def render_and_count(path: str) -> tuple[str, int, str | None]:
    """Returns (path, char_count, error_msg)."""
    try:
        result = subprocess.run(
            ["bash", RENDER_SCRIPT, path],
            capture_output=True,
            text=True,
            timeout=30,
            cwd=os.path.dirname(SCRIPTS_DIR),
        )
    except subprocess.TimeoutExpired:
        return (path, 0, "timeout (30s)")
    except FileNotFoundError:
        return (path, 0, "render-file.sh not found")

    if result.returncode != 0:
        err = result.stderr.strip() or f"exit code {result.returncode}"
        return (path, 0, err)

    cleaned = strip_frontmatter(result.stdout)
    chars = len(cleaned)
    return (path, chars, None)


def paths_from_args(args: list[str]):
    if args:
        for a in args:
            if os.path.isfile(a):
                yield a
            elif os.path.isdir(a):
                for root, _, files in os.walk(a):
                    for f in sorted(files):
                        if f.endswith(".md"):
                            yield os.path.join(root, f)
            else:
                print(f"skip: not found: {a}", file=sys.stderr)
    else:
        for line in sys.stdin:
            p = line.strip()
            if p:
                yield p


def main():
    args = sys.argv[1:]
    paths = list(paths_from_args(args))
    if not paths:
        if args:
            print("error: no valid paths found", file=sys.stderr)
        else:
            print("Usage: token-count-after-expand.py [paths...]", file=sys.stderr)
        sys.exit(1)

    total = 0
    count = 0
    errors = 0

    for path in paths:
        path, chars, err = render_and_count(path)
        if err:
            print(f"! {path}  render failed: {err}", file=sys.stderr)
            errors += 1
            continue
        tokens = int(chars / BYTES_PER_TOKEN)
        total += tokens
        count += 1
        print(f"{tokens}  {path}")

    if count > 0:
        print(f"{total}  total")
    print(f"# {BYTES_PER_TOKEN} chars/token")
    if errors:
        print(f"! {errors} errors", file=sys.stderr)


if __name__ == "__main__":
    main()
