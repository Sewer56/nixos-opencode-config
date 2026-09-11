#!/usr/bin/env python3
"""Exercise validator entry points, task graphs and rule imports in temporary repos."""
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

    def rule(self, name, text="Review scoped evidence.\n"):
        path = self.repo / "config/rules" / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def import_rule(self, agent, name):
        path = self.repo / "config/agent" / f"{agent}.md"
        with path.open("a", encoding="utf-8") as destination:
            destination.write(f'\n{{{{ file="./rules/{name}" }}}}\n')

    def test_nested_verifier_ids_and_transitive_rules_are_reachable(self):
        self.agent("Code", targets=("_review/verifiers/quality", "_review/verifiers/correctness"))
        self.config["subagent_depth"] = 3
        for name in ("quality", "correctness"):
            agent = f"_review/verifiers/{name}"
            self.agent(agent, "subagent")
            self.import_rule(agent, "review/verifiers.md")
        self.rule("review/verifiers.md", '{{ file="./rules/review/findings.md" }}\n')
        self.rule("review/findings.md", '{{ file="./rules/approval.md" }}\n')
        self.rule("approval.md")
        self.rule("README.md", "# Rules\n")
        self.rule("review/README.md", "# Review rules\n")
        self.validate(0)

    def test_orphan_rules_are_rejected_at_any_depth(self):
        self.agent("Code")
        for name in ("orphan.md", "review/orphan.md", "review/nested/orphan.md"):
            with self.subTest(name=name):
                self.rule(name)
                self.validate(1, f"unreachable rule module: config/rules/{name}")
                (self.repo / "config/rules" / name).unlink()

    def test_unreachable_importer_does_not_make_rule_reachable(self):
        self.agent("Code")
        self.rule("review/orphan.md", '{{ file="./rules/review/findings.md" }}\n')
        self.rule("review/findings.md")
        self.validate(1, "unreachable rule module: config/rules/review/findings.md")

    def test_missing_rule_import_is_rejected(self):
        self.agent("Code")
        self.import_rule("Code", "review/missing.md")
        self.validate(1, "unresolved import ./rules/review/missing.md")

    def test_rule_import_cycles_are_rejected(self):
        self.agent("Code")
        self.import_rule("Code", "review/findings.md")
        self.rule("review/findings.md", '{{ file="./rules/review/contract.md" }}\n')
        self.rule("review/contract.md", '{{ file="./rules/review/findings.md" }}\n')
        self.validate(1, "prompt import cycle:")

    def test_obsolete_rule_prefix_is_rejected_in_new_tree(self):
        self.agent("Code")
        self.import_rule("Code", "review/target-findings.md")
        self.rule("review/target-findings.md")
        self.validate(1, "rule filenames use obsolete target- prefix:")

    def fragment(self, name, text="Review scoped evidence.\n"):
        path = self.repo / "config/agent" / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def import_fragment(self, agent, name):
        path = self.repo / "config/agent" / f"{agent}.md"
        with path.open("a", encoding="utf-8") as destination:
            destination.write(f'\n{{{{ file="./agent/{name}" }}}}\n')

    def test_transitive_local_fragments_are_not_agents(self):
        self.agent("Code", targets=("_review/verifiers/quality",))
        self.agent("_review/verifiers/quality", "subagent")
        self.config["subagent_depth"] = 3
        self.import_fragment("_review/verifiers/quality", "_review/verifiers/shared/verification.txt")
        self.fragment("_review/verifiers/shared/verification.txt",
                      '{{ file="./agent/_review/shared/findings.txt" }}\n')
        self.fragment("_review/shared/findings.txt", '{{ file="./rules/review/contract.md" }}\n')
        self.rule("review/contract.md")
        self.validate(0, "Custom agents: 2")

    def test_orphan_local_fragment_chain_is_rejected(self):
        self.agent("Code")
        self.fragment("_review/shared/orphan.txt",
                      '{{ file="./agent/_review/shared/findings.txt" }}\n')
        self.fragment("_review/shared/findings.txt")
        self.validate(1, "unreachable agent fragment: config/agent/_review/shared/findings.txt")

    def test_missing_local_fragment_import_is_rejected(self):
        self.agent("Code")
        self.import_fragment("Code", "_review/shared/missing.txt")
        self.validate(1, "unresolved import ./agent/_review/shared/missing.txt")

    def test_local_fragment_cycles_are_rejected(self):
        self.agent("Code")
        self.import_fragment("Code", "_review/shared/a.txt")
        self.fragment("_review/shared/a.txt", '{{ file="./agent/_review/shared/b.txt" }}\n')
        self.fragment("_review/shared/b.txt", '{{ file="./agent/_review/shared/a.txt" }}\n')
        self.validate(1, "prompt import cycle:")

    def test_markdown_local_fragment_is_rejected(self):
        self.agent("Code")
        self.import_fragment("Code", "_review/shared/accidental.md")
        self.fragment("_review/shared/accidental.md")
        self.validate(1, "agent shared fragments must use .txt, not .md:")


if __name__ == "__main__":
    unittest.main()
