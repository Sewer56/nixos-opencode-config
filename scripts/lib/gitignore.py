"""Git-aware file listing, respecting .gitignore patterns.

Provides functions to list files in directories while respecting gitignore rules,
without vendoring a gitignore parser. Uses `git ls-files` where possible.
"""
import os
import subprocess


def git_files(dir_path):
    """Return absolute paths of tracked/ignored files under dir_path.

    Uses `git ls-files --cached --others --exclude-standard` to respect all
    .gitignore rules (including nested ones). Returns None if dir_path is not
    inside a git repository.

    Args:
        dir_path: Directory to scan.

    Returns:
        List of absolute file paths, or None if not in a git repo.
    """
    try:
        result = subprocess.run(
            ["git", "-C", dir_path, "ls-files", "--cached", "--others", "--exclude-standard"],
            capture_output=True,
            text=True,
            check=True,
        )
        paths = []
        for line in result.stdout.splitlines():
            if not line:
                continue
            # git ls-files gives paths relative to git worktree root
            abs_path = os.path.normpath(os.path.join(dir_path, line))
            if os.path.exists(abs_path):
                paths.append(abs_path)
        return paths
    except subprocess.CalledProcessError:
        return None


def list_files(dir_path):
    """Yield file paths under dir_path, respecting .gitignore if in a git repo.

    Falls back to os.walk if not in a git repository.
    """
    git_paths = git_files(dir_path)
    if git_paths is not None:
        for path in git_paths:
            yield path
    else:
        for root, _dirs, files in os.walk(dir_path):
            for f in files:
                yield os.path.join(root, f)
