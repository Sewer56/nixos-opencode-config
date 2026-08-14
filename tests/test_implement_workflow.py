"""Static contract tests for simplified /implement workflow.

Input: active command, orchestrator, cohort creator, cohort loop, and reviewer
prompt files. Output: unittest PASS/FAIL; repository remains unchanged.

Run: ``python3 -m unittest discover -s tests -p 'test_*.py'``.
"""

from __future__ import annotations

import ast
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FLAKE = ROOT / "flake.nix"
ORCHESTRATOR = ROOT / "config/agent/_implement.md"
CREATE_COHORTS = ROOT / "config/agent/_implement/create-cohorts.md"
COHORT = ROOT / "config/agent/_implement/cohort.md"
CODE_WRITING = ROOT / "config/rules/groups/implementation/code-writing.md"
INTEGRATION_REPAIR = ROOT / "config/agent/_implement/integration-repair.md"
ITERATE_EDIT = ROOT / ".opencode/agent/_iterate/edit.md"
ITERATE_EDITOR = ROOT / ".opencode/agent/_iterate/editor.md"
COMMAND = ROOT / "config/command/implement.md"
REVIEW_FINDINGS = ROOT / "config/rules/groups/implementation/review-findings.md"
IMPLEMENT_REVIEWERS = (
    ROOT / "config/agent/_implement/cohort/review/correctness.md",
    ROOT / "config/agent/_implement/cohort/review/quality.md",
    ROOT / "config/agent/_implement/cohort/review/optional/tests.md",
    ROOT / "config/agent/_implement/cohort/review/optional/security.md",
    ROOT / "config/agent/_implement/cohort/review/optional/performance.md",
    ROOT / "config/agent/_implement/review/integration.md",
)
SHELL_OWNING_AGENTS = (
    ROOT / ".opencode/agent/_iterate/edit.md",
    ROOT / "config/agent/_audit/public-api.md",
    ROOT / "config/agent/_docs.md",
    ORCHESTRATOR,
    ROOT / "config/agent/_implement/integration-repair.md",
    ROOT / "config/agent/_refactor/document.md",
    ROOT / "config/agent/_refactor/errors.md",
    ROOT / "config/agent/_refactor/reorder.md",
    ROOT / "config/agent/_review/coderabbit.md",
    ROOT / "config/agent/commit.md",
)
SHARED_BASH_PERMISSION = """  bash:
    "*": allow
    "sudo *": deny
    "git push *": deny
    "git reset --hard *": deny
    "git clean *": deny
    "git commit --no-verify *": deny
"""
COHORT_BASH_PERMISSION = SHARED_BASH_PERMISSION + '    "git commit *": deny\n'
READ_ONLY_BASH_PERMISSION = """  bash:
    "*": allow
    "sudo *": deny
    "git push *": deny
    "git commit *": deny
    "git add *": deny
    "git reset *": deny
    "git clean *": deny
    "git rebase *": deny
    "git merge *": deny
    "git checkout *": deny
    "git switch *": deny
    "git restore *": deny
    "git stash *": deny
    "git rm *": deny
    "git mv *": deny
    "git apply *": deny
    "git cherry-pick *": deny
    "git revert *": deny
    "rm *": deny
    "mv *": deny
    "cp *": deny
    "touch *": deny
    "mkdir *": deny
    "rmdir *": deny
    "tee *": deny
    "dd *": deny
    "ln *": deny
    "chmod *": deny
    "chown *": deny
    "patch *": deny
"""
VERIFIER = ROOT / "config/agent/_review/verifier.md"
ARTIFACT_PATHS_CARD = ROOT / "config/rules/cards/implementation/artifact-paths.md"
ARTIFACT_WRITERS = (ORCHESTRATOR, *IMPLEMENT_REVIEWERS, VERIFIER, CREATE_COHORTS)
READ_ONLY_BASH_AGENTS = (*IMPLEMENT_REVIEWERS, VERIFIER, CREATE_COHORTS)
WRITABLE_SURFACE_AGENTS = (*IMPLEMENT_REVIEWERS, VERIFIER)
PLAN_REVIEWER = ROOT / "config/agent/_plan/draft/reviewer.md"
WRITABLE_SURFACE = """# Writable surface
Create or overwrite files only under `artifact/` with the write/edit tools (both share one permission); `edit` cannot fill an existing empty file. Bash is read-only inspection: never create or modify tracked files or git state with it. If writing the assigned path fails, return only the `# Output` envelope with `Status: INCOMPLETE` — never probe, relocate, write any other artifact, or write via bash. Env/secret files (`*.env*`, except `*.env.example`) are off-limits via bash too.
"""
WRITABLE_SURFACE_ITERATE = """# Writable surface
Create or overwrite files only under `artifacts/iterate/` with the write/edit tools (both share one permission); `edit` cannot fill an existing empty file. Bash is read-only inspection: never create or modify tracked files or git state with it. If writing the assigned path fails, return only the `# Output` envelope with `Status: INCOMPLETE` — never probe, relocate, write any other artifact, or write via bash. Env/secret files (`*.env*`, except `*.env.example`) are off-limits via bash too.
"""
CROSS_WORKFLOW_READ_ONLY_BASH = (
    ROOT / ".opencode/agent/_iterate/review.md",
    ROOT / ".opencode/agent/_iterate/verifier.md",
    ROOT / "config/agent/_plan/draft/explorer.md",
    PLAN_REVIEWER,
    ROOT / "config/agent/_plan/draft/verifier.md",
    ROOT / "config/agent/_docs/reviewers/accuracy.md",
    ROOT / "config/agent/_docs/reviewers/usability.md",
    ROOT / "config/agent/_refactor/document/reviewers/documentation.md",
    ROOT / "config/agent/_refactor/document/reviewers/errors.md",
    ROOT / "config/agent/_refactor/errors/collector.md",
    ROOT / "config/agent/_audit/public-api/collector.md",
    ROOT / "config/agent/_write/pr.md",
)
WRITABLE_SURFACE_CROSS_ARTIFACT = (
    ROOT / "config/agent/_docs/reviewers/accuracy.md",
    ROOT / "config/agent/_docs/reviewers/usability.md",
    ROOT / "config/agent/_refactor/document/reviewers/documentation.md",
    ROOT / "config/agent/_refactor/document/reviewers/errors.md",
    ROOT / "config/agent/_refactor/errors/collector.md",
)
WRITABLE_SURFACE_CROSS_ITERATE = (
    ROOT / ".opencode/agent/_iterate/review.md",
    ROOT / ".opencode/agent/_iterate/verifier.md",
)
OVERWRITE_WORDING_AGENTS = (
    ROOT / "config/agent/_docs.md",
    ROOT / "config/agent/_refactor/document.md",
    ROOT / "config/agent/_refactor/errors.md",
    ROOT / "config/agent/_audit/public-api.md",
)


def text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def bash_permission(frontmatter: str) -> str:
    lines = frontmatter.splitlines(keepends=True)
    start = next(i for i, line in enumerate(lines) if line == "  bash:\n")
    end = next(
        (i for i in range(start + 1, len(lines)) if lines[i].startswith("  ") and not lines[i].startswith("    ")),
        len(lines),
    )
    return "".join(lines[start:end])


def expand_config_imports(source: str) -> str:
    pattern = re.compile(r'\{\{ file="(?P<path>\./[^"]+)" \}\}')
    return pattern.sub(
        lambda match: text(ROOT / "config" / match.group("path").removeprefix("./")),
        source,
    )


class ImplementWorkflowTests(unittest.TestCase):
    def test_shell_owners_use_shared_permissive_bash_map(self) -> None:
        for path in SHELL_OWNING_AGENTS:
            with self.subTest(path=path.relative_to(ROOT)):
                frontmatter = text(path).split("---", 2)[1]
                self.assertEqual(SHARED_BASH_PERMISSION, bash_permission(frontmatter))

    def test_validator_documents_scope_in_module_docstring(self) -> None:
        source = text(ROOT / "scripts/validate-opencode-config.py")
        description = ast.get_docstring(ast.parse(source)) or ""
        for heading in (
            "Configuration documents",
            "Agent frontmatter and permissions",
            "Commands and task graph",
            "Prompt structure and imports",
            "Documentation and source syntax",
            "Outside scope",
        ):
            self.assertIn(heading, description)
        self.assertNotIn("--describe", source)

    def test_required_workflow_files_exist(self) -> None:
        required = (
            "config/agent/_implement/create-cohorts.md",
            "config/agent/_implement/cohort.md",
            "config/agent/_implement/integration-repair.md",
            "config/agent/_review/verifier.md",
            "config/rules/groups/implementation/code-writing.md",
            "config/rules/groups/implementation/cohort-planning.md",
            "config/rules/groups/implementation/implementation-review.md",
            "config/rules/groups/implementation/review-findings.md",
        )
        for path in required:
            self.assertTrue((ROOT / path).is_file(), path)
        subprocess.run(
            ["git", "ls-files", "--error-unmatch", "config/agent/_review/verifier.md"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )

    def test_parent_calls_one_cohort_agent_per_cohort(self) -> None:
        body = text(ORCHESTRATOR)
        self.assertIn("call `_implement/cohort` exactly once for each cohort", body)
        self.assertIn('"_implement/cohort": allow', body)
        self.assertNotIn('"_implement/cohort/review/optional/tests": allow', body)

    def test_cohort_owns_complete_loop(self) -> None:
        body = text(COHORT)
        for marker in (
            "sole code writer and loop owner",
            "Stage and run quick checks",
            "Call exact reviewers",
            "Call exact verifier and repair",
            "Allow at most five repair turns total",
            "Commit",
        ):
            self.assertIn(marker, body)

    def test_shared_writer_lint_uses_auto_mode(self) -> None:
        rule = text(CODE_WRITING)
        self.assertTrue(rule.startswith("## RULE GROUP: IMPLEMENTATION / CODE WRITING\n"))
        self.assertIn("\n### Lint gate\n", rule)
        self.assertEqual(
            ["rust-llm-tidy"],
            re.findall(r"`(rust-llm-tidy[^`]*)`", rule),
        )
        self.assertIn("repository-wide tracked staged and unstaged `.rs`/`.md` changes", rule)
        self.assertIn("may include unrelated tracked changes", rule)
        self.assertIn("untracked files are excluded until staged", rule)
        self.assertIn("No eligible tracked changes is a successful skip", rule)
        self.assertIn("Non-zero blocks handoff", rule)
        self.assertIn("bounded writer loop", rule)

        rule_import = '{{ file="./rules/groups/implementation/code-writing.md" }}'
        for path in (COHORT, INTEGRATION_REPAIR):
            with self.subTest(path=path.relative_to(ROOT)):
                self.assertIn(rule_import, text(path))

    def test_rust_llm_tidy_wrapper_preserves_caller_cwd(self) -> None:
        flake = text(FLAKE)
        self.assertIn("manifestPath ? null", flake)
        self.assertIn('manifestPath = "$HOME/opencode/tools/rust-llm-tidy/src/Cargo.toml";', flake)
        self.assertIn('--manifest-path "${manifestPath}"', flake)
        tidy = flake.index('name = "rust-llm-tidy"')
        tidy_block = flake[tidy : flake.index("})", tidy) + 2]
        self.assertIn("manifestPath =", tidy_block)
        self.assertNotIn('cd "${dir}"', tidy_block)
        self.assertIn('cd "${dir}"', flake[:tidy])

    def test_lint_gate_precedes_first_imported_rule_group_after_expansion(self) -> None:
        rule = text(CODE_WRITING)
        self.assertEqual(1, len(re.findall(r"(?m)^## RULE GROUP:", rule)))

        expanded = expand_config_imports(rule)
        rule_groups = [match.start() for match in re.finditer(r"(?m)^## RULE GROUP:", expanded)]
        self.assertGreaterEqual(len(rule_groups), 2)
        self.assertLess(expanded.index("\n### Lint gate\n"), rule_groups[1])

    def test_cohort_lints_before_staging_validation_and_review(self) -> None:
        body = text(COHORT)
        lint = body.index("Run the shared code-writing lint gate")
        staging = body.index("stage only paths changed by cohort writer")
        validation = body.index("Run quick validation")
        review = body.index("Review only after quick checks PASS")
        self.assertLess(lint, staging)
        self.assertLess(staging, validation)
        self.assertLess(validation, review)
        self.assertIn("rerun this all-checks loop from the lint gate before restaging", body)

    def test_exact_reviewer_and_verifier_names(self) -> None:
        body = text(COHORT)
        for name in (
            "_implement/cohort/review/correctness",
            "_implement/cohort/review/quality",
            "_implement/cohort/review/optional/tests",
            "_implement/cohort/review/optional/security",
            "_implement/cohort/review/optional/performance",
            "_review/verifier",
        ):
            self.assertIn(name, body)

    def test_mandatory_and_risk_routed_reviews(self) -> None:
        body = text(COHORT)
        self.assertIn("Always call `_implement/cohort/review/correctness`", body)
        self.assertIn("it owns checking that applicable tests ran after staging", body)
        self.assertIn("Always call `_implement/cohort/review/quality` before commit", body)
        self.assertIn("Call optional tests, security, or performance reviewer only when routed or matching concrete risk", body)

    def test_review_calls_supply_and_validate_explicit_envelopes(self) -> None:
        for path in (COHORT, ORCHESTRATOR):
            with self.subTest(path=path.relative_to(ROOT)):
                body = text(path)
                for marker in (
                    "<review-inputs>",
                    "one explicit envelope with every declared input",
                    "Plan Path:",
                    "Handoff Path:",
                    "Base Commit:",
                    "Changed Paths:",
                    "Validation Path: [[validation_path]]",
                    "Review Path: [[review_path]]",
                    "Verdict Path: [[verdict_path]]",
                    "Prior Verdict Paths:",
                    "write the requested artifact",
                    "return only its exact five-line `# Output` envelope",
                    "readable, schema-conforming artifact at the exact assigned `review_path`, artifact-consistent with the returned envelope",
                    "artifact-consistent decision and count",
                    "Missing or malformed evidence is `INCOMPLETE`, never PASS",
                ):
                    self.assertIn(marker, body)
        self.assertIn("Cohort Path: [[cohort_path]]", text(COHORT))
        orchestrator = text(ORCHESTRATOR)
        self.assertIn("Add `Cohort Path: None` for non-integration reviewers", orchestrator)
        self.assertIn("Use implementation `base_commit` and final changed paths for integration/security/performance", orchestrator)
        self.assertIn("For correctness/quality, use the commit at `HEAD` before the staged final repair and its exact staged repair paths", orchestrator)

    def test_iterate_editor_calls_supply_and_validate_absolute_paths(self) -> None:
        caller = text(ITERATE_EDIT)
        for marker in (
            "every `_iterate/editor` call supplies exactly",
            "Request Path: [[absolute request_path]]",
            "Contract Path: [[absolute contract_path]]",
            "Repair Notes:",
        ):
            self.assertIn(marker, caller)

        editor = text(ITERATE_EDITOR)
        for marker in (
            "Explicit absolute `request_path` and `contract_path`",
            "Before editing, reject missing, non-absolute, unreadable, or non-file `request_path` or `contract_path`",
            "read contract first and request second",
        ):
            self.assertIn(marker, editor)

    def test_implementation_reviewers_fail_closed_via_shared_rule(self) -> None:
        rule = text(REVIEW_FINDINGS)
        for marker in (
            "- Validate one explicit labeled `<review-inputs>` envelope",
            "every declared input",
            "write the requested artifact with `Decision: INCOMPLETE`",
            "exact `# Output` envelope with `Status: INCOMPLETE`",
            "- Valid input: write the requested artifact",
            "return only the declared output envelope",
        ):
            self.assertIn(marker, rule)
        for path in IMPLEMENT_REVIEWERS:
            with self.subTest(path=path.relative_to(ROOT)):
                body = text(path)
                self.assertIn('{{ file="./rules/groups/implementation/review-findings.md" }}', body)
                self.assertIn("validation_path", body)
                self.assertIn("review_path", body)
                self.assertIn("# Artifact", body)
                self.assertIn("# Output", body)

    def test_optional_reviewers_have_distinct_subtree(self) -> None:
        review = ROOT / "config/agent/_implement/cohort/review"
        for name in ("tests.md", "security.md", "performance.md"):
            self.assertTrue((review / "optional" / name).is_file())
            self.assertFalse((review / name).exists())

    def test_quality_gates_every_commit(self) -> None:
        body = text(COHORT)
        self.assertIn("Every selected reviewer must complete", body)
        self.assertIn("re-read staged diff", body)
        self.assertNotIn("staged_diff_hash", body)
        self.assertNotIn("SHA-256", body)
        self.assertIn('"_implement/cohort/review/quality": allow', body)
        quality = ROOT / "config/agent/_implement/cohort/review/quality.md"
        correctness = ROOT / "config/agent/_implement/cohort/review/correctness.md"
        self.assertTrue(quality.is_file())
        self.assertFalse((ROOT / "config/agent/_implement/review/quality.md").exists())
        self.assertIn("every proposed commit", text(quality))
        self.assertIn("Require applicable tests to pass after staging", text(correctness))
        self.assertIn("Missing evidence is `INCOMPLETE`", text(correctness))

        parent = text(ORCHESTRATOR)
        self.assertIn("also call `_implement/cohort/review/correctness` and `_implement/cohort/review/quality`", parent)
        self.assertIn("Every selected reviewer must complete", parent)
        self.assertIn("both source/destination", parent)
        self.assertIn("write fresh ledger before review", parent)
        self.assertIn("validate including tests, then rerun integration, correctness, quality, and affected optional reviews", parent)
        self.assertNotIn("diff_hash", parent)
        self.assertNotIn("plan_hash", parent)

    def test_unrelated_user_changes_are_preserved(self) -> None:
        for path in (ORCHESTRATOR, COHORT, ROOT / "config/agent/commit.md"):
            with self.subTest(path=path.name):
                body = text(path)
                self.assertIn("unrelated", body)
                self.assertIn("preserv", body.lower())

    def test_real_index_staging_is_scoped(self) -> None:
        frontmatter = text(COHORT).split("---", 2)[1]
        self.assertEqual(COHORT_BASH_PERMISSION, bash_permission(frontmatter))
        self.assertIn("stage only paths changed by cohort writer", text(COHORT))
        self.assertIn("git diff --cached", text(COHORT))

    def test_incomplete_and_final_integration_remain(self) -> None:
        body = text(ORCHESTRATOR)
        self.assertIn("Status: SUCCESS | INCOMPLETE | NEEDS_INPUT | FAIL", body)
        self.assertIn("Always call `_implement/review/integration`", body)
        self.assertIn("Send candidates to `_review/verifier`", body)

    def test_orchestrator_is_compact_and_cannot_edit_code(self) -> None:
        body = text(ORCHESTRATOR)
        self.assertLess(len(body.encode()), 12_000)
        frontmatter = body.split("---", 2)[1]
        self.assertIn('edit:\n    "*": deny', frontmatter)
        self.assertNotIn('edit:\n    "*": allow', frontmatter)
        self.assertIn('"artifact/**": allow', frontmatter)

    def test_no_agent_overrides_global_external_directory(self) -> None:
        for root in (ROOT / "config/agent", ROOT / ".opencode/agent"):
            for path in root.rglob("*.md"):
                with self.subTest(path=path.relative_to(ROOT)):
                    self.assertNotIn("external_directory:", text(path).split("---", 2)[1] if text(path).startswith("---") else "")

    def test_coderabbit_uses_no_local_verifier(self) -> None:
        body = text(ROOT / "config/agent/_review/coderabbit.md")
        frontmatter = body.split("---", 2)[1]
        self.assertNotIn("verifier", frontmatter)
        self.assertIn("do not add a second verifier", body)
        self.assertIn("Status: PASS | ADVISORY | INCOMPLETE | NEEDS_INPUT | FAIL", body)
        self.assertNotIn("Diff Hash", body)
        self.assertNotIn("Instruction Hash", body)

    def test_implementation_commit_uses_path_boundary_and_optional_amend(self) -> None:
        body = text(ROOT / "config/agent/commit.md")
        frontmatter = body.split("---", 2)[1]
        self.assertIn(SHARED_BASH_PERMISSION, frontmatter)
        self.assertNotIn('"git add', frontmatter)
        self.assertNotIn('"git commit --amend', frontmatter)
        self.assertIn("For an implementation boundary", body)
        self.assertIn("Amend current `HEAD` only when user explicitly requests it", body)
        self.assertNotIn("expected_staged_diff_hash", body)
        self.assertNotIn("SHA-256", body)
        self.assertIn('git commit --amend', body)
        self.assertIn("Never bypass hooks", body)
        self.assertIn("stage with blanket pathspecs", body)

    def test_artifact_writers_can_write_only_artifacts(self) -> None:
        for path in ARTIFACT_WRITERS:
            with self.subTest(path=path.relative_to(ROOT)):
                frontmatter = text(path).split("---", 2)[1]
                self.assertIn('edit:\n    "*": deny\n    "artifact/**": allow', frontmatter)

    def test_reviewers_and_verifier_use_read_only_bash_blacklist(self) -> None:
        for path in READ_ONLY_BASH_AGENTS:
            with self.subTest(path=path.relative_to(ROOT)):
                frontmatter = text(path).split("---", 2)[1]
                self.assertEqual(READ_ONLY_BASH_PERMISSION, bash_permission(frontmatter))

    def test_cohort_denies_direct_git_commit(self) -> None:
        frontmatter = text(COHORT).split("---", 2)[1]
        self.assertIn('"git commit *": deny', frontmatter)
        self.assertIn('"artifact/**": deny', frontmatter)

    def test_artifact_paths_card_binds_path_variables(self) -> None:
        self.assertTrue(ARTIFACT_PATHS_CARD.is_file())
        card = text(ARTIFACT_PATHS_CARD)
        for marker in (
            "never a directory",
            "`review_path`",
            "`verdict_path`",
            "creates or overwrites that exact file",
            "never write any other path",
            "stub",
        ):
            self.assertIn(marker, card)
        card_import = '{{ file="./rules/cards/implementation/artifact-paths.md" }}'
        for path in (ORCHESTRATOR, COHORT, CREATE_COHORTS):
            with self.subTest(path=path.relative_to(ROOT)):
                self.assertIn(card_import, text(path))

    def test_reviewers_and_verifier_declare_writable_surface(self) -> None:
        for path in WRITABLE_SURFACE_AGENTS:
            with self.subTest(path=path.relative_to(ROOT)):
                body = text(path)
                self.assertIn(WRITABLE_SURFACE, body)

    def test_reviewer_gate_fails_closed_without_writes(self) -> None:
        rule = text(REVIEW_FINDINGS)
        self.assertIn("not writable, write nothing", rule)
        self.assertIn("Never probe, relocate, or write any other artifact", rule)

    def test_committed_content_is_self_contained(self) -> None:
        self.assertIn(
            "never cite plan- or cohort-only identifiers",
            text(ROOT / "config/rules/cards/docs/code-documentation.md"),
        )
        self.assertIn("plan-internal", text(ROOT / "config/rules/cards/structure/plan-artifacts.md"))

    def test_advisory_repair_split(self) -> None:
        card = text(ROOT / "config/rules/cards/implementation/review-findings.md")
        self.assertIn("Accepted BLOCKING findings and accepted advisories enter repair", card)
        self.assertIn("final integration gate", card)
        self.assertIn("without widening scope", card)
        self.assertIn("is not a FAIL", card)
        self.assertIn("Repair accepted blockers and advisories", text(COHORT))
        self.assertIn("accepted blockers and accepted advisories", text(ORCHESTRATOR))
        self.assertNotIn("Never repair advisories", text(ORCHESTRATOR))
        self.assertIn("Accepted advisories", text(INTEGRATION_REPAIR))
        self.assertIn("without widening scope", text(INTEGRATION_REPAIR))

    def test_cohort_delegation_honesty_and_reverify(self) -> None:
        body = text(COHORT)
        self.assertIn("never perform delegated review, verdict, or commit work yourself", body)
        self.assertIn("rerun the verifier when re-reviews emit new candidates", body)

    def test_orchestrator_preflight_and_restart_hygiene(self) -> None:
        body = text(ORCHESTRATOR)
        self.assertIn(".git/info/exclude", body)
        self.assertIn("remove its partial artifacts and stubs", body)

    def test_non_implement_writers_accept_current_target_state(self) -> None:
        for relative in (
            "config/agent/_docs.md",
            "config/agent/_refactor/document.md",
            "config/agent/_refactor/errors.md",
        ):
            body = text(ROOT / relative)
            frontmatter = body.split("---", 2)[1]
            self.assertNotIn('"git add*": allow', frontmatter)
            self.assertIn("Do not stage files", body)

    def test_cross_workflow_read_only_agents_use_bash_blacklist(self) -> None:
        for path in CROSS_WORKFLOW_READ_ONLY_BASH:
            with self.subTest(path=path.relative_to(ROOT)):
                frontmatter = text(path).split("---", 2)[1]
                self.assertEqual(READ_ONLY_BASH_PERMISSION, bash_permission(frontmatter))

    def test_cross_workflow_writers_declare_writable_surface(self) -> None:
        for path in WRITABLE_SURFACE_CROSS_ARTIFACT:
            with self.subTest(path=path.relative_to(ROOT)):
                body = text(path)
                self.assertIn(WRITABLE_SURFACE, body)
        for path in WRITABLE_SURFACE_CROSS_ITERATE:
            with self.subTest(path=path.relative_to(ROOT)):
                body = text(path)
                self.assertIn(WRITABLE_SURFACE_ITERATE, body)

    def test_cross_workflow_writers_use_broad_artifact_glob(self) -> None:
        for path in WRITABLE_SURFACE_CROSS_ARTIFACT:
            with self.subTest(path=path.relative_to(ROOT)):
                frontmatter = text(path).split("---", 2)[1]
                self.assertIn('"artifact/**": allow', frontmatter)
        for path in WRITABLE_SURFACE_CROSS_ITERATE:
            with self.subTest(path=path.relative_to(ROOT)):
                frontmatter = text(path).split("---", 2)[1]
                self.assertIn('"artifacts/iterate/**": allow', frontmatter)

    def test_orchestrators_use_create_or_overwrite_wording(self) -> None:
        for path in OVERWRITE_WORDING_AGENTS:
            with self.subTest(path=path.relative_to(ROOT)):
                body = text(path)
                self.assertNotIn("Never overwrite", body)
                self.assertIn("Create or overwrite", body)


if __name__ == "__main__":
    unittest.main()
