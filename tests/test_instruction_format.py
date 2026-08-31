"""Static contract tests for instruction-format validation.

Input: the config validator module and fixture strings for its
``instruction_format_issues`` function. Output: unittest PASS/FAIL;
repository remains unchanged.

Run: ``python3 -m unittest discover -s tests -p 'test_*.py'``.
"""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
VALIDATOR_PATH = ROOT / "scripts/validate-opencode-config.py"


def load_validator():
    spec = importlib.util.spec_from_file_location("validate_opencode_config", VALIDATOR_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class InstructionFormatTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.issues = staticmethod(load_validator().instruction_format_issues)

    def errors(self, text: str) -> list[str]:
        return [message for severity, message in self.issues(text) if severity == "error"]

    def warnings(self, text: str) -> list[str]:
        return [message for severity, message in self.issues(text) if severity == "warning"]

    def test_conforming_text_has_no_issues(self) -> None:
        text = (
            "---\n"
            "description: demo\n"
            "---\n"
            "\n"
            "# Head\n"
            "\n"
            "One short paragraph.\n"
            "\n"
            "- One item.\n"
            "- Two items.\n"
        )
        self.assertEqual([], self.issues(text))

    def test_over_cap_single_line_is_an_error(self) -> None:
        text = "word " * 49 + "word.\n"
        self.assertEqual(1, len(self.errors(text)))
        self.assertIn("statement", self.errors(text)[0])

    def test_over_cap_list_item_excludes_the_marker(self) -> None:
        self.assertEqual([], self.errors("- " + "c" * 240 + "\n"))
        self.assertEqual(1, len(self.errors("- " + "c" * 241 + "\n")))

    def test_wrapped_multi_line_prose_is_not_joined(self) -> None:
        text = ("word " * 30 + "word.\n") * 2
        self.assertEqual([], self.errors(text))

    def test_stacked_short_statements_have_no_issues(self) -> None:
        text = "# Head\n" + "One standalone statement.\n" * 9
        self.assertEqual([], self.issues(text))

    def test_em_dash_outside_vs_inside_fence(self) -> None:
        self.assertEqual(1, len(self.errors("Bad — dash.\n")))
        self.assertIn("em dash", self.errors("Bad — dash.\n")[0])
        self.assertEqual([], self.issues("```text\na — b\n```\n"))

    def test_frontmatter_is_exempt(self) -> None:
        text = "---\ndescription: has — one dash\n---\n\nClean body.\n"
        self.assertEqual([], self.issues(text))

    def test_table_row_and_url_only_line_are_exempt(self) -> None:
        table = "| see https://example.com/" + "a" * 100 + " — ok |\n"
        url = "https://example.com/" + "a" * 100 + "\n"
        self.assertEqual([], self.issues(table))
        self.assertEqual([], self.issues(url))

    def test_long_plain_line_is_a_split_suggestion_warning(self) -> None:
        text = "b" * 100 + "\n"
        self.assertEqual([], self.errors(text))
        self.assertEqual(1, len(self.warnings(text)))
        self.assertIn("split", self.warnings(text)[0])

    def test_statement_cap_boundary_is_exact(self) -> None:
        self.assertEqual([], self.errors("c" * 240 + "\n"))
        self.assertEqual(1, len(self.errors("c" * 241 + "\n")))


if __name__ == "__main__":
    unittest.main()
