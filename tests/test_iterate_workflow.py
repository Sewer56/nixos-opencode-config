"""Static instruction regressions, not live agent execution."""

from pathlib import Path
import re
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[1]


def agent(name):
    source = (ROOT / f".opencode/agent/_iterate/{name}.md").read_text()
    _, metadata, body = source.split("---", 2)
    return yaml.safe_load(metadata), body


class IterateWorkflowTests(unittest.TestCase):
    def test_routing_and_write_ownership(self):
        command = (ROOT / ".opencode/command/iterate/edit.md").read_text()
        _, metadata, prompt = command.split("---", 2)
        self.assertEqual("_iterate/edit", yaml.safe_load(metadata)["agent"])
        self.assertEqual("Request:\n$ARGUMENTS", prompt.strip())
        parent, _ = agent("edit")
        self.assertEqual({"*": "deny", "artifacts/iterate/**": "allow"},
                         parent["permission"]["edit"])
        self.assertEqual("allow", parent["permission"]["task"]["_iterate/editor"])
        writer, _ = agent("editor")
        self.assertNotIn("task", writer["permission"])
        self.assertEqual("allow", writer["permission"]["edit"]["tests/**"])
        self.assertNotIn("artifacts/iterate/**", writer["permission"]["edit"])

    def test_editor_envelope_and_status_protocol(self):
        _, parent = agent("edit")
        envelopes = re.findall(r"<editor-inputs>\n(.*?)\n</editor-inputs>",
                               parent, re.S)
        self.assertEqual([
            "Request Path: [[absolute request_path]]\n"
            "Contract Path: [[absolute contract_path]]\n"
            "Repair Notes: [[failed checks or verified target blockers, otherwise None]]"
        ], envelopes)
        _, writer = agent("editor")
        output = re.search(r"```text\n(.*?)\n```", writer, re.S).group(1)
        self.assertEqual(["Status", "Changed Paths", "Question", "Summary"],
                         [line.split(":", 1)[0] for line in output.splitlines()])
        self.assertEqual({"DONE", "NO_CHANGE", "INCOMPLETE", "NEEDS_INPUT", "FAIL"},
                         set(output.splitlines()[0].split(": ")[1].split(" | ")))

    def test_autonomy_and_frozen_scope_guards(self):
        # Text guards protect decision boundaries, not model compliance.
        _, writer = agent("editor")
        for guard in (
            "Choose routine details autonomously within scope",
            "Resolve precedence; stop on real authority conflicts",
            "Contract defects are `INCOMPLETE`, not questions or scope expansion",
            "Accept current target edits; ignore unrelated changes",
            "Reread changed targets; preserve compatible edits",
            "Ask only for material choices or incompatible target edits, not locks",
            "Orchestrator owns staging; staging-only issues never block writing",
            "`VERIFY` is no-edit",
            "Never edit request, contract, run artifacts, or unlisted consumers",
        ):
            with self.subTest(guard=guard):
                self.assertIn(guard, writer)
        _, parent = agent("edit")
        self.assertLess(parent.index("mark assertions needing changes `UPDATE`"),
                        parent.index("Write `contract.md`"))
        self.assertIn("never widen frozen scope or ask to unlock it", parent)
        self.assertIn("Allow two repair turns, never widening targets", parent)

    def test_continuation_is_run_bound_and_outside_prompt(self):
        _, parent = agent("edit")
        for guard in (
            "Save returned `task_id` in `editor-task.md` with run and authority identity",
            "Reuse it as a tool argument for repairs and same-run continuation only",
            "Keep `task_id` outside the prompt envelope",
            "Revalidate request, contract, and target state before continuation",
            "Changed authority needs fresh preflight and a new task, never cross-run reuse",
            "For stale/unavailable identity, record the fallback before starting a new task",
        ):
            with self.subTest(guard=guard):
                self.assertIn(guard, parent)
        _, writer = agent("editor")
        self.assertIn("Revalidate inputs and targets on continuation", writer)


if __name__ == "__main__":
    unittest.main()
