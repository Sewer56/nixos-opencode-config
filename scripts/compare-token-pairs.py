#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = ["tiktoken"]
# ///
"""Compare all ID.before.txt / ID.after.txt pairs in one directory.

Usage: python3 scripts/compare-token-pairs.py DIRECTORY
On Windows, use py -3 or python with Python 3.10+.
Quote paths containing spaces in the active shell.

Print compact JSON with cl100k_base raw snippet counts and savings.
Positive savings mean fewer tokens; counts are not whole-context estimates.

Escape non-ASCII output for compatibility with Windows console encodings.

Requires prepared dependencies and tokenizer data.
See count-instruction-tokens.py for tokenizer cache setup.

Read UTF-8 text without expansion and reject incomplete pairs.
Ignore unrelated files; pair files must be regular files, not symlinks.
Write no files.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location(
    "instruction_tokens", Path(__file__).with_name("count-instruction-tokens.py")
)
counter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(counter)


def read_pairs(directory: Path) -> list[tuple[str, str, str]]:
    """Validate the complete batch before counting any pairs."""
    sides: dict[str, dict[str, Path]] = {}
    for path in sorted(directory.iterdir()):
        for side in ("before", "after"):
            suffix = f".{side}.txt"
            if not path.name.endswith(suffix):
                continue
            name = path.name[:-len(suffix)]
            if not name or path.is_symlink() or not path.is_file():
                raise ValueError(f"invalid pair file: {path.name}")
            sides.setdefault(name, {})[side] = path
    if not sides:
        raise ValueError("no token pairs found")
    pairs = []
    for name, files in sorted(sides.items()):
        if set(files) != {"before", "after"}:
            raise ValueError(f"incomplete pair: {name}")
        pairs.append((name, *(
            files[side].read_bytes().decode("utf-8") for side in ("before", "after")
        )))
    return pairs


def compare(pairs, encoder) -> dict:
    rows = []
    for name, before, after in pairs:
        before_count = len(encoder.encode_ordinary(before))
        after_count = len(encoder.encode_ordinary(after))
        rows.append({
            "id": name, "before": before_count, "after": after_count,
            "saved": before_count - after_count,
        })
    return {"encoding": "cl100k_base", "measurement": "raw-snippet", "pairs": rows}


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    try:
        pairs = read_pairs(args.directory)
        try:
            encoder = counter.load_encoder()
        except RuntimeError as exc:
            raise RuntimeError("prepared tiktoken and cl100k_base cache required") from exc
        report = compare(pairs, encoder)
        print(json.dumps(report, ensure_ascii=True, separators=(",", ":")))
        return 0
    except (OSError, UnicodeError, RuntimeError, ValueError) as exc:
        message = f"compare-token-pairs: {exc}"
        print(message.encode("ascii", "backslashreplace").decode("ascii"), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
