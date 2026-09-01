"""Static contract tests for the /cleanup workflow.

Input: the cleanup command and agent prompt files and the README command table.
Output: unittest PASS/FAIL; repository remains unchanged.

Run: ``python3 -m unittest discover -s tests -p 'test_*.py'``.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
COMMAND = ROOT / "config/command/cleanup.md"
AGENT = ROOT / "config/agent/_cleanup.md"
README = ROOT / "README.md"
RULE_IMPORT = '{{ file="./rules/groups/implementation/code-writing.md" }}'
REVIEWERS = (
    "_implement/cohort/review/correctness",
    "_implement/cohort/review/quality",
    "_implement/cohort/review/optional/tests",
    "_implement/cohort/review/optional/security",
    "_implement/cohort/review/optional/performance",
)
TASK_ALLOWLIST = {
    *REVIEWERS,
    "_review/verifier",
}
SHARED_BASH_DENIES = (
    '"sudo *": deny',
    '"git push *": deny',
    '"git reset --hard *": deny',
    '"git clean *": deny',
    '"git commit --no-verify *": deny',
    '"git commit *": deny',
)


def text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def frontmatter(path: Path) -> str:
    source = text(path)
    if not source.startswith("---\n"):
        raise AssertionError(f"{path} has no frontmatter")
    return source.split("---", 2)[1]


def body(path: Path) -> str:
    return text(path).split("---", 2)[2]


def permission_block(frontmatter_text: str, tool: str) -> str:
    lines = frontmatter_text.splitlines(keepends=True)
    start = next(i for i, line in enumerate(lines) if line == f"  {tool}:\n")
    end = next(
        (
            i
            for i in range(start + 1, len(lines))
            if lines[i].startswith("  ") and not lines[i].startswith("    ")
        ),
        len(lines),
    )
    return "".join(lines[start:end])


class CleanupWorkflowTests(unittest.TestCase):
    def test_command_is_thin_entrypoint(self) -> None:
        self.assertTrue(COMMAND.is_file())
        meta = frontmatter(COMMAND)
        self.assertIn("agent: _cleanup", meta)
        description = re.search(r"(?m)^description:[ \t]*(.+)$", meta)
        self.assertIsNotNone(description)
        self.assertTrue(description.group(1).strip().strip('"').strip())
        prompt = body(COMMAND)
        self.assertIn("Target paths and cleanup focus:", prompt)
        self.assertIn("$ARGUMENTS", prompt)
        self.assertEqual([], re.findall(r"(?m)^#{1,6}[ \t]", prompt))

    def test_agent_frontmatter_matches_one_shot_shape(self) -> None:
        meta = frontmatter(AGENT)
        one_shot_meta = frontmatter(ROOT / "config/agent/_implement/one-shot.md")
        self.assertIn("mode: primary", meta)
        for field in ("model:", "variant:"):
            self.assertEqual(
                re.search(rf"(?m)^{re.escape(field)}.*$", one_shot_meta).group(),
                re.search(rf"(?m)^{re.escape(field)}.*$", meta).group(),
            )
        edit = permission_block(meta, "edit")
        self.assertIn('"artifact/CLEANUP-*.handoff.md": allow', edit)
        self.assertIn('"artifact/CLEANUP-*.r??.quick.validation.md": allow', edit)
        self.assertIn('"artifact/**": deny', edit)
        self.assertIn('"artifacts/**": deny', edit)
        self.assertIn('"*PROMPT-*.md": deny', edit)

    def test_agent_bash_keeps_shared_and_commit_denies(self) -> None:
        bash = permission_block(frontmatter(AGENT), "bash")
        for deny in SHARED_BASH_DENIES:
            self.assertIn(deny, bash)

    def test_agent_tasks_allow_exactly_six_cleanup_delegations(self) -> None:
        task = permission_block(frontmatter(AGENT), "task")
        allowed = set(re.findall(r'"([^"]+)": allow', task))
        self.assertEqual(TASK_ALLOWLIST, allowed)
        self.assertIn('"_review/verifier": allow', task)
        self.assertNotIn("commit", task)
        self.assertNotIn("coderabbit", task)

    def test_agent_body_imports_rules_and_runs_full_gauntlet(self) -> None:
        agent = body(AGENT)
        self.assertIn(RULE_IMPORT, agent)
        for name in (*REVIEWERS, "_review/verifier"):
            self.assertIn(name, agent)
        self.assertIn("Scope: STANDALONE", agent)
        self.assertIn("reviewer-declared `COHORT_STAGED` for security and performance", agent)
        self.assertIn("at most five repair turns", agent)

    def test_agent_requires_targets_and_leaves_work_staged(self) -> None:
        agent = body(AGENT)
        self.assertIn("`NEEDS_INPUT` when no target paths are supplied", agent)
        self.assertRegex(agent, r"(?i)never commit")
        self.assertIn("staged", agent)

    def test_agent_has_no_coderabbit_or_commit_gate(self) -> None:
        agent = body(AGENT)
        self.assertNotIn("External CodeRabbit review", agent)
        self.assertNotIn("_review/coderabbit", agent)
        self.assertEqual([], re.findall(r"(?im)^#+[ \t].*commit", agent))

    def test_readme_lists_cleanup_in_refactoring_table(self) -> None:
        readme = text(README)
        start = readme.index("### Refactoring")
        table = readme[start : readme.index("\n###", start)]
        row = next(line for line in table.splitlines() if "`/cleanup`" in line)
        self.assertIn("standards", row.lower())
        self.assertIn("gauntlet", row.lower())


if __name__ == "__main__":
    unittest.main()
