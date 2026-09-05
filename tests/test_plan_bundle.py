"""Exercise links and Git ignore/worktree mechanics."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from test_instruction_format import load_validator


class PlanMechanicsTests(unittest.TestCase):
    def test_local_links_and_anchors(self):
        validator = load_validator()
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            root = repo / "PROMPT-PLAN-demo.draft.md"
            members = repo / "artifact/plan/PROMPT-PLAN-demo"
            members.mkdir(parents=True)
            cohort = members / "01-demo.md"
            contract = members / "contract.md"
            root.write_text("# Demo\n[Contract](artifact/plan/PROMPT-PLAN-demo/contract.md)\n"
                            "[01](artifact/plan/PROMPT-PLAN-demo/01-demo.md#checks)\n")
            contract.write_text("# Contract\n")
            cohort.write_text("# 01\n## Checks\n[Contract](contract.md)\n")
            errors = []
            validator.validate_doc_links(repo, [root, contract, cohort], errors)
            self.assertEqual([], errors)
            cohort.write_text("# 01\n[Missing](missing.md)\n")
            validator.validate_doc_links(repo, [root, contract, cohort], errors)
            self.assertTrue(any("stale heading link" in error for error in errors))
            self.assertTrue(any("stale link missing.md" in error for error in errors))

    def test_exact_ignore_patterns_and_tracked_paths_in_worktrees(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            repo.mkdir()
            env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
            env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)

            def git(*args, check=True):
                return subprocess.run(["git", *args], cwd=repo, env=env,
                                      capture_output=True, check=check)

            git("init", "-q")
            git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                "commit", "-q", "--allow-empty", "-m", "fixture")
            exclude = (repo / os.fsdecode(git("rev-parse", "--git-path",
                                             "info/exclude").stdout).strip()).resolve()
            before = b"# preserve local entries\nkeep-me"
            exclude.write_bytes(before)
            with exclude.open("ab") as output:
                output.write(b"\n/PROMPT-PLAN-\\[demo\\].draft.md\n"
                             b"/artifact/plan/PROMPT-PLAN-\\[demo\\]/\n")
            self.assertTrue(exclude.read_bytes().startswith(before + b"\n"))
            worktree = Path(directory) / "worktree"
            git("worktree", "add", "-q", "-b", "fixture-worktree", str(worktree))
            for location in (repo, worktree):
                repo = location
                resolved = (repo / os.fsdecode(git("rev-parse", "--git-path",
                                                  "info/exclude").stdout).strip()).resolve()
                self.assertEqual(exclude, resolved)
                for path, ignored in (
                    ("PROMPT-PLAN-[demo].draft.md", True),
                    ("artifact/plan/PROMPT-PLAN-[demo]/contract.md", True),
                    ("artifact/plan/PROMPT-PLAN-[demo]/schema/details.md", True),
                    ("PROMPT-PLAN-d.draft.md", False),
                    ("nested/PROMPT-PLAN-[demo].draft.md", False),
                    ("artifact/plan/other/contract.md", False),
                ):
                    with self.subTest(location=location, path=path):
                        self.assertEqual(0 if ignored else 1,
                                         git("check-ignore", "-q", "--", path,
                                             check=False).returncode)
                path = "PROMPT-PLAN-[demo].draft.md"
                (repo / path).write_text("accepted content\n")
                git("add", "-f", "--", path)
                self.assertEqual(path, git("--literal-pathspecs", "ls-files", "--",
                                           path).stdout.decode().strip())
                self.assertEqual(1, git("check-ignore", "-q", "--", path,
                                        check=False).returncode)
                self.assertFalse((repo / ".gitignore").exists())
            self.assertTrue((worktree / ".git").is_file())
            self.assertFalse((worktree / ".git/info/exclude").exists())


if __name__ == "__main__":
    unittest.main()
