#!/usr/bin/env python3
"""Combine one PB_DUMP capture into a private Markdown report."""

import argparse
import os
from pathlib import Path
import re
import sys


CAPTURES = (
    ("raw.txt", "System prompt before", "text"),
    ("final.txt", "System prompt after", "text"),
    ("tools.raw.json", "Tool descriptions and schemas before", "json"),
    ("tools.final.json", "Tool descriptions and schemas after", "json"),
    ("tools.txt", "Final character counts (not tokens)", "text"),
)


def build_report(prefix: str, hook: str) -> str:
    """Read all five captures before writing anything; preserve their contents."""
    parts = [
        "# Prompt and tool capture\n",
        f"Hook: {hook}\n",
        "These snapshots show this plugin's input and output, not the final network request.\n",
        "They may contain private project instructions and tool metadata.\n",
    ]
    for suffix, title, language in CAPTURES:
        source = Path(f"{prefix}.{hook}.{suffix}")
        with source.open(encoding="utf-8", newline="") as capture:
            content = capture.read()

        # A prompt can contain Markdown fences. Use a longer fence around it.
        longest = max((len(run) for run in re.findall(r"`+", content)), default=0)
        fence = "`" * max(3, longest + 1)
        ending = "" if content.endswith("\n") else "\n"
        parts.append(f"## {title}\n\n{fence}{language}\n{content}{ending}{fence}\n")

    return "\n".join(parts)


def write_report(output: Path, report: str) -> None:
    """Create an owner-only report; never replace an existing file or symlink."""
    descriptor = os.open(output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as destination:
        destination.write(report)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("prefix", help="PB_DUMP path prefix used by the OpenCode server")
    parser.add_argument(
        "--hook", default="context", help="hook to read (default: context)",
    )
    parser.add_argument(
        "-o", "--output", type=Path,
        help="new report path (default: PREFIX.HOOK.report.md); existing files are refused",
    )
    args = parser.parse_args()
    output = args.output if args.output is not None else Path(f"{args.prefix}.{args.hook}.report.md")

    try:
        report = build_report(args.prefix, args.hook)
        write_report(output, report)
    except FileExistsError:
        print(f"Report already exists: {output}. Choose a new --output path.", file=sys.stderr)
        return 1
    except (OSError, UnicodeError) as error:
        print(
            f"Cannot create report: {error}. Check the capture prefix, hook, UTF-8 files "
            "and directory permissions. If a partial report was created, remove it before retrying.",
            file=sys.stderr,
        )
        return 1

    print(f"Report written to {output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
