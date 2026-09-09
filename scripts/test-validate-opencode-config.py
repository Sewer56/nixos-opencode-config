#!/usr/bin/env python3
"""Exercise validator entry points and graph safety in temporary repositories."""
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import yaml

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location(
    "validator", Path(__file__).with_name("validate-opencode-config.py")
)
validator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validator)


class EntryPointTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        (self.repo / "config/agent").mkdir(parents=True)
        (self.repo / "config/command").mkdir()
        self.config = {
            "subagent_depth": 2,
            "tool_output": {"max_lines": 10, "max_bytes": 100},
            "permission": {"external_directory": "ask"},
            "agent": {
                name: {"permission": {"external_directory": "allow"}}
                for name in ("build", "plan")
            },
        }

    def agent(self, name, mode="primary", *, targets=(), **extra):
        frontmatter = {
            "mode": mode,
            "description": "Fixture agent",
            "permission": {
                "*": "deny",
                "read": {"*": "allow", "*.env": "deny", "*.env.*": "deny"},
                "task": {"*": "deny", **dict.fromkeys(targets, "allow")},
            },
            **extra,
        }
        path = self.repo / "config/agent" / f"{name}.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("---\n" + yaml.safe_dump(frontmatter, sort_keys=False)
                        + "---\n# Output\nReport findings.\n", encoding="utf-8")

    def command(self, target):
        (self.repo / "config/command/run.md").write_text(
            f"---\ndescription: Fixture command\nagent: {target}\n---\nRun.\n",
            encoding="utf-8",
        )

    def validate(self, expected, diagnostic=None):
        (self.repo / "config/opencode.json").write_text(
            json.dumps(self.config), encoding="utf-8"
        )
        output = io.StringIO()
        with patch.object(sys, "argv", ["validator", "--repo-root", str(self.repo)]):
            with contextlib.redirect_stdout(output):
                result = validator.main()
        self.assertEqual(result, expected, output.getvalue())
        if diagnostic:
            self.assertIn(diagnostic, output.getvalue())

    def test_visible_primary_and_all_need_no_command(self):
        for mode in ("primary", "all"):
            with self.subTest(mode=mode):
                self.agent("Docs", mode)
                self.validate(0)

    def test_hidden_disabled_and_subagent_are_not_selection_roots(self):
        for extra in ({"hidden": True}, {"disable": True}, {"mode": "subagent"}):
            with self.subTest(extra=extra):
                self.agent("Docs", **extra)
                self.validate(1, "unreachable custom agents: Docs")

    def test_primary_reaches_hidden_reviewer(self):
        self.agent("Docs", targets=("_docs/reviewer",))
        self.agent("_docs/reviewer", "subagent", hidden=True)
        self.config["subagent_depth"] = 3
        self.validate(0)

    def test_unreferenced_subagent_is_still_rejected(self):
        self.agent("Docs")
        self.agent("orphan", "subagent")
        self.validate(1, "unreachable custom agents: orphan")

    def test_hidden_primary_remains_reachable_by_command(self):
        self.agent("Docs", hidden=True)
        self.command("Docs")
        self.validate(0)

    def test_builtin_task_grant_remains_an_entry(self):
        self.agent("worker", "subagent")
        self.config["agent"]["build"]["permission"]["task"] = {
            "*": "deny", "worker": "allow"
        }
        self.validate(0)

    def test_command_cannot_target_subagent(self):
        self.agent("worker", "subagent")
        self.command("worker")
        self.validate(1, "targets subagent-only agent worker")

    def test_missing_command_target_is_rejected(self):
        self.command("missing")
        self.validate(1, "targets missing agent missing")

    def test_missing_and_disabled_task_targets_are_rejected(self):
        self.agent("Docs", targets=("missing",))
        self.validate(1, "allows missing task target missing")
        self.agent("missing", "subagent", disable=True)
        self.config["subagent_depth"] = 3
        self.validate(1, "routes to disabled agent missing")

    def test_primary_root_cycles_are_rejected(self):
        self.agent("Docs", targets=("Docs",))
        self.validate(1, "custom task cycle: Docs -> Docs")

    def test_primary_root_depth_is_enforced(self):
        self.agent("Docs", targets=("a",))
        self.agent("a", "subagent", targets=("b",))
        self.agent("b", "subagent", targets=("c",))
        self.agent("c", "subagent", targets=("d",))
        self.agent("d", "subagent")
        self.config["subagent_depth"] = 6
        self.validate(1, "custom task depth 4 exceeds policy maximum 3")

    def test_depth_configuration_still_matches_graph(self):
        self.agent("Docs", targets=("reviewer",))
        self.agent("reviewer", "subagent")
        self.validate(1, "config.subagent_depth must be 3, got 2")


if __name__ == "__main__":
    unittest.main()
