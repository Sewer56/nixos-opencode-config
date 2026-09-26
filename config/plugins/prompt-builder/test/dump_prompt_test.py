"""Check the report script through its command-line interface."""

import json
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "dump-prompt.py"


class DumpPromptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="pb-report-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.prefix = self.directory / "probe with spaces"

    def capture(self, hook="context"):
        contents = {
            "raw.txt": "Private instructions\n````markdown\nexample\n````\n",
            "final.txt": "# Environment\nWorking directory: /project\nUnicode: café",
            "tools.raw.json": json.dumps({"read": {"description": "Original text", "input": {}}}),
            "tools.final.json": json.dumps({"read": {"description": "Short text", "input": {}}}),
            "tools.txt": "read\t12\t(desc 10, input 2)",
        }
        for suffix, content in contents.items():
            Path(f"{self.prefix}.{hook}.{suffix}").write_text(content, encoding="utf-8")
        return contents

    def run_script(self, *arguments):
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(self.prefix), *arguments],
            capture_output=True, text=True, check=False,
        )

    def test_default_prefix_is_in_plugin_directory(self):
        # Run a copy from another directory so the test never writes into the repo.
        scripts = self.directory / "plugin" / "scripts"
        scripts.mkdir(parents=True)
        script = scripts / SCRIPT.name
        shutil.copyfile(SCRIPT, script)
        prefix = scripts.parent / "probe"
        self.prefix = prefix
        self.capture()
        result = subprocess.run(
            [sys.executable, str(script), str(prefix)], cwd=self.directory,
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(Path(f"{prefix}.context.report.md").is_file())

    # Core behavior
    def test_report_should_preserve_all_captures(self):
        for hook in ("context", "compaction", "generate"):
            with self.subTest(hook=hook):
                # Arrange
                contents = self.capture(hook)
                output = Path(f"{self.prefix}.{hook}.report.md")

                # Act
                result = self.run_script("--hook", hook)

                # Assert
                self.assertEqual(result.returncode, 0, result.stderr)
                report = output.read_text(encoding="utf-8")
                for suffix, content in contents.items():
                    self.assertIn(content, report, suffix)
                    source = Path(f"{self.prefix}.{hook}.{suffix}")
                    self.assertEqual(source.read_text(encoding="utf-8"), content)
                self.assertIn("`````text\nPrivate instructions", report)
                self.assertIn("not the final network request", report)
                self.assertIn("not tokens", report)
                self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o600)
                self.assertNotIn("Private instructions", result.stdout + result.stderr)

    def test_report_should_accept_new_hook_when_capture_exists(self):
        # Arrange
        self.capture("new_hook")

        # Act
        result = self.run_script("--hook", "new_hook")

        # Assert
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(Path(f"{self.prefix}.new_hook.report.md").is_file())

    # Edge cases
    def test_report_should_explain_missing_capture(self):
        result = self.run_script()

        self.assertEqual(result.returncode, 1)
        self.assertIn(f"Missing capture: {self.prefix}.context.raw.txt", result.stderr)
        self.assertIn("Set PB_DUMP=1 in the OpenCode server's environment", result.stderr)
        self.assertIn("make a request", result.stderr)
        self.assertFalse(Path(f"{self.prefix}.context.report.md").exists())

    def test_report_should_refuse_existing_outputs(self):
        for kind in ("file", "symlink"):
            with self.subTest(kind=kind):
                # Arrange
                self.capture()
                protected = self.directory / f"protected-{kind}"
                protected.write_text("keep this", encoding="utf-8")
                output = self.directory / f"report-{kind}.md"
                if kind == "symlink":
                    output.symlink_to(protected)
                else:
                    output.write_text("keep this", encoding="utf-8")

                # Act
                result = self.run_script("--output", str(output))

                # Assert
                self.assertEqual(result.returncode, 1)
                self.assertIn("Choose a new --output path", result.stderr)
                self.assertEqual(output.read_text(encoding="utf-8"), "keep this")
                self.assertEqual(protected.read_text(encoding="utf-8"), "keep this")

    def test_report_should_leave_no_output_when_capture_is_unreadable(self):
        for kind in ("missing", "invalid_utf8"):
            with self.subTest(kind=kind):
                # Arrange
                self.capture()
                source = Path(f"{self.prefix}.context.tools.txt")
                output = self.directory / f"{kind}.md"
                if kind == "missing":
                    source.unlink()
                else:
                    source.write_bytes(b"\xff")

                # Act
                result = self.run_script("--output", str(output))

                # Assert
                self.assertEqual(result.returncode, 1)
                if kind == "missing":
                    self.assertIn("Missing capture:", result.stderr)
                else:
                    self.assertIn("Check the capture prefix", result.stderr)
                self.assertFalse(output.exists())

    # Convenience
    def test_report_should_use_custom_output_path(self):
        # Arrange
        self.capture()
        output = self.directory / "custom report.md"

        # Act
        result = self.run_script("--output", str(output))

        # Assert
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(output.is_file())
        self.assertIn(str(output), result.stdout)


if __name__ == "__main__":
    unittest.main()
