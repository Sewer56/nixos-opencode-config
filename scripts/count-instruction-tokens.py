#!/usr/bin/env python3
"""Count explicit UTF-8 files with tiktoken cl100k_base; print JSON.

Usage: python3 scripts/count-instruction-tokens.py config/agent/code.md
Add --report PATH to create a JSON report (existing files are never replaced).

Requires Python 3.10+, tiktoken and its preloaded cl100k_base cache.
Markdown also needs Bun and the existing md-expand plugin dependencies.

Prepare dependencies separately; this command never installs or downloads them.
Preload the cache separately with:
  python3 -c 'import tiktoken; tiktoken.get_encoding("cl100k_base")'

TIKTOKEN_CACHE_DIR and DATA_GYM_CACHE_DIR select an existing cache.
Markdown counts use the existing renderer, including its whitespace handling.

Other file types are raw-only: expanded equals raw, labeled unrendered.
Counts include frontmatter; special-token spellings count as ordinary text.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from unittest.mock import patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "config/plugins/opencode-plugin-md-expand/src/cli/cli.ts"


def load_encoder():
    """Use tiktoken's encoder with a cache-only loader, without cache writes."""
    try:
        import tiktoken
        import tiktoken.load
    except ImportError as exc:
        raise RuntimeError("tiktoken is unavailable; use a prepared Python environment") from exc

    cache = os.environ.get(
        "TIKTOKEN_CACHE_DIR",
        os.environ.get("DATA_GYM_CACHE_DIR", str(Path(tempfile.gettempdir()) / "data-gym-cache")),
    )

    def cached_only(blobpath: str, expected_hash: str | None = None) -> bytes:
        if not cache:
            raise RuntimeError("a preloaded tiktoken cache is required; see --help")
        key = hashlib.sha1(blobpath.encode()).hexdigest()
        try:
            data = (Path(cache) / key).read_bytes()
        except OSError as exc:
            raise RuntimeError("cl100k_base cache is unavailable; preload it separately") from exc
        if expected_hash and hashlib.sha256(data).hexdigest() != expected_hash:
            raise RuntimeError("cl100k_base cache hash mismatch; repair it separately")
        return data

    with patch.object(tiktoken.load, "read_file_cached", cached_only):
        return tiktoken.get_encoding("cl100k_base")


def render(path: Path) -> str:
    # Render alone silently drops missing imports; validate with the same owner.
    for action in ("validate", "render"):
        result = subprocess.run(
            ["bun", str(RENDERER), action, str(path)],
            cwd=ROOT, text=True, encoding="utf-8", capture_output=True, check=False,
        )
        if result.returncode:
            detail = (result.stderr + result.stdout).strip()
            raise RuntimeError(f"{action} failed for {path}: {detail}")
    return result.stdout


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("paths", nargs="+", type=Path, help="explicit input files, relative to cwd")
    parser.add_argument("--report", type=Path, help="create this report only; parent must exist")
    args = parser.parse_args()
    try:
        sources = [(path.resolve(strict=True), path.read_bytes().decode("utf-8")) for path in args.paths]
        encoder = load_encoder()
        rows = []
        for path, raw in sources:
            rendered = path.suffix.lower() == ".md"
            expanded = render(path) if rendered else raw
            rows.append({
                "path": str(path),
                "expansion": "md-expand" if rendered else "unrendered",
                "raw": len(encoder.encode_ordinary(raw)),
                "expanded": len(encoder.encode_ordinary(expanded)),
            })
        report = {
            "encoding": "cl100k_base",
            "files": rows,
            "totals": {key: sum(row[key] for row in rows) for key in ("raw", "expanded")},
        }
        output = json.dumps(report, indent=2) + "\n"
        if args.report:
            with args.report.open("x", encoding="utf-8") as destination:
                destination.write(output)
        print(output, end="")
        return 0
    except (OSError, UnicodeError, RuntimeError, ValueError) as exc:
        print(f"count-instruction-tokens: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
