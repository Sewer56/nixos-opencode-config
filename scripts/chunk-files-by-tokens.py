#!/usr/bin/env python3
"""Chunk files by token count. Usage: chunk-files-by-tokens.py [-s SIZE] [paths...]

Input: files or dirs (respects .gitignore). No args reads paths from stdin.
Output format:
  chunk 1: 5200
  3000 main.rs
  2200 lib.rs

  chunk 2: 4100
  4100 utils.rs

Env: BYTES_PER_TOKEN (default 3.0)
"""
import os, sys, argparse
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from lib.gitignore import list_files

BYTES_PER_TOKEN = float(os.environ.get("BYTES_PER_TOKEN", 3.0))

def expand(path):
    if os.path.isfile(path):
        yield path
    elif os.path.isdir(path):
        yield from list_files(path)

def paths_input(args):
    if args:
        for a in args: yield from expand(a)
    else:
        for line in sys.stdin:
            b = line.strip()
            if b: yield from expand(b)

def toks(path):
    try: s = os.path.getsize(path)
    except OSError: return None
    if s == 0: return None
    return int(s / BYTES_PER_TOKEN)

def chunk(paths, max_tokens):
    max_bytes = int(max_tokens * BYTES_PER_TOKEN)
    chunks, items, bytes_, tcounts = [], [], 0, []
    for p in paths:
        t = toks(p)
        if t is None: continue
        b = int(t * BYTES_PER_TOKEN)
        if items and bytes_ + b > max_bytes:
            chunks.append((items, tcounts))
            items, bytes_, tcounts = [p], b, [t]
        else:
            items.append(p)
            bytes_ += b
            tcounts.append(t)
    if items: chunks.append((items, tcounts))
    return chunks

def main():
    parser = argparse.ArgumentParser(description="Chunk files by token count")
    parser.add_argument("paths", nargs="*", help="Files or directories")
    parser.add_argument("-s", "--size", type=int, default=32000, metavar="TOKENS",
                        help="Max tokens per chunk (default: 32000)")
    args = parser.parse_args()

    total = files = i = 0
    for i, (items, tcounts) in enumerate(chunk(paths_input(args.paths), args.size), 1):
        sub = sum(tcounts)
        total += sub; files += len(items)
        if i > 1: print()
        print(f"chunk {i}: {sub}")
        for p, t in zip(items, tcounts):
            print(f"{t} {p}")
    print(f"\nchunks: {i}  files: {files}  tokens: {total}")
    print(f"target: {args.size}/chunk  ({BYTES_PER_TOKEN} bytes/token)")

if __name__ == "__main__":
    main()
