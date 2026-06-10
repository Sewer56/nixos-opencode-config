#!/usr/bin/env python3
"""Chunk files by estimated token count for collector dispatch.

Usage:
  echo "file1.py\nfile2.py" | chunk-files-by-tokens.py
  chunk-files-by-tokens.py dir1/ dir2/ file3.py

Input paths can be files or directories (directories expanded recursively).
Outputs one chunk per line, comma-separated paths.
Each chunk targets < MAX_CHUNK_TOKENS tokens (default 64000, ~3 chars/token).
"""
import os
import sys

MAX_TOKENS = int(os.environ.get("MAX_CHUNK_TOKENS", 64000))
BYTES_PER_TOKEN = float(os.environ.get("BYTES_PER_TOKEN", 3.0))
MAX_BYTES = int(MAX_TOKENS * BYTES_PER_TOKEN)


def expand(path):
    """Yield file paths from a file or directory."""
    if os.path.isfile(path):
        yield path
    elif os.path.isdir(path):
        for root, _dirs, files in os.walk(path):
            for f in files:
                yield os.path.join(root, f)


def paths_from_args(args):
    """Yield all file paths from CLI arguments (files or directories)."""
    for arg in args:
        yield from expand(arg)


def paths_from_stdin():
    """Yield all file paths read from stdin lines (files or directories)."""
    for line in sys.stdin:
        base = line.strip()
        if not base:
            continue
        yield from expand(base)


def chunk(paths):
    """Group paths into chunks under MAX_BYTES each, yielding comma-separated lines."""
    chunk_list = []
    chunk_bytes = 0
    for path in paths:
        try:
            size = os.path.getsize(path)
        except OSError:
            continue
        if size == 0:
            continue
        if chunk_list and chunk_bytes + size > MAX_BYTES:
            print(",".join(chunk_list))
            chunk_list = [path]
            chunk_bytes = size
        elif chunk_list:
            chunk_list.append(path)
            chunk_bytes += size
        else:
            chunk_list = [path]
            chunk_bytes = size
    if chunk_list:
        print(",".join(chunk_list))


def main():
    if len(sys.argv) > 1:
        paths = paths_from_args(sys.argv[1:])
    else:
        paths = paths_from_stdin()
    chunk(paths)


if __name__ == "__main__":
    main()
