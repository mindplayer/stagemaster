from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from local_worker.errors import WorkerError
from local_worker.proposal import build_candidate, parse_operations, write_candidate
from local_worker.task import build_context, validate_task
from support import config, git, make_repo, response, task


class TaskValidationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.case = Path(self.temporary.name)
        self.root, self.commit = make_repo(self.case)
        self.config = config(self.case)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def assert_code(self, code: str, function) -> None:
        with self.assertRaises(WorkerError) as raised:
            function()
        self.assertEqual(raised.exception.code, code)

    def test_accepts_clean_exact_base_and_builds_bounded_context(self) -> None:
        validated = validate_task(task(self.root, self.commit), self.config)
        context = build_context(validated, self.config)
        self.assertIn("src/lib.rs", context)
        self.assertIn("tests/acceptance.rs", context)
        self.assertNotIn(str(self.case / "unrelated"), context)

    def test_rejects_unknown_field_traversal_and_protected_overlap(self) -> None:
        unknown = task(self.root, self.commit)
        unknown["typo"] = True
        self.assert_code("invalid_task", lambda: validate_task(unknown, self.config))
        traversal = task(self.root, self.commit)
        traversal["write_paths"] = ["../outside.rs"]
        self.assert_code("invalid_path", lambda: validate_task(traversal, self.config))
        overlap = task(self.root, self.commit)
        overlap["write_paths"] = ["tests/acceptance.rs"]
        self.assert_code("protected_path", lambda: validate_task(overlap, self.config))

    def test_rejects_symlink_target_and_parent(self) -> None:
        (self.root / "linked.rs").symlink_to(self.root / "src" / "lib.rs")
        git(self.root, "add", "linked.rs")
        git(self.root, "commit", "-m", "add symlink target")
        commit = git(self.root, "rev-parse", "HEAD")
        symlink_target = task(self.root, commit)
        symlink_target["read_paths"].append("linked.rs")
        symlink_target["write_paths"] = ["linked.rs"]
        self.assert_code("symlink_path", lambda: validate_task(symlink_target, self.config))

        (self.root / "linked-dir").symlink_to(self.root / "src", target_is_directory=True)
        git(self.root, "add", "linked-dir")
        git(self.root, "commit", "-m", "add symlink parent")
        commit = git(self.root, "rev-parse", "HEAD")
        symlink_parent = task(self.root, commit)
        symlink_parent["write_paths"] = ["linked-dir/new.rs"]
        self.assert_code("symlink_path", lambda: validate_task(symlink_parent, self.config))

    def test_rejects_dirty_or_wrong_base(self) -> None:
        (self.root / "src" / "lib.rs").write_text("dirty\n", encoding="utf-8")
        self.assert_code("dirty_workspace", lambda: validate_task(task(self.root, self.commit), self.config))
        git(self.root, "restore", "src/lib.rs")
        wrong = task(self.root, "0" * 40)
        self.assert_code("stale_base", lambda: validate_task(wrong, self.config))

    def test_rejects_context_over_limit(self) -> None:
        validated = validate_task(task(self.root, self.commit), self.config)
        self.config["limits"]["max_context_bytes"] = 10
        self.assert_code("context_too_large", lambda: build_context(validated, self.config))


class ProposalTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.case = Path(self.temporary.name)
        self.root, self.commit = make_repo(self.case)
        self.config = config(self.case)
        self.validated = validate_task(task(self.root, self.commit), self.config)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def assert_code(self, code: str, function) -> None:
        with self.assertRaises(WorkerError) as raised:
            function()
        self.assertEqual(raised.exception.code, code)

    def test_exact_replace_yields_hashed_candidate_without_touching_worktree(self) -> None:
        operations = [
            {
                "kind": "replace",
                "path": "src/lib.rs",
                "old_text": "{ 1 }",
                "new_text": "{ 2 }",
            }
        ]
        candidate = build_candidate(operations, self.validated, 1024)
        self.assertIn("{ 2 }", candidate.files["src/lib.rs"].decode())
        self.assertIn("-pub fn value() -> u8 { 1 }", candidate.diff)
        self.assertEqual((self.root / "src" / "lib.rs").read_text(), "pub fn value() -> u8 { 1 }\n")
        write_candidate(self.case / "attempt", candidate)
        self.assertTrue((self.case / "attempt" / "candidate" / "src" / "lib.rs").is_file())

    def test_create_and_sequential_replace_are_atomic(self) -> None:
        data = task(self.root, self.commit)
        data["write_paths"].append("src/new.rs")
        validated = validate_task(data, self.config)
        operations = [
            {"kind": "create", "path": "src/new.rs", "content": "pub const N: u8 = 1;\n"},
            {"kind": "replace", "path": "src/new.rs", "old_text": "= 1", "new_text": "= 2"},
        ]
        candidate = build_candidate(operations, validated, 1024)
        self.assertEqual(candidate.files["src/new.rs"], b"pub const N: u8 = 2;\n")

        invalid = operations + [
            {"kind": "replace", "path": "src/lib.rs", "old_text": "absent", "new_text": "x"}
        ]
        attempt = self.case / "invalid-attempt"
        self.assert_code("ambiguous_match", lambda: write_candidate(attempt, build_candidate(invalid, validated, 1024)))
        self.assertFalse(attempt.exists())

    def test_rejects_ambiguous_scope_empty_and_stale_operations(self) -> None:
        ambiguous = [{"kind": "replace", "path": "src/lib.rs", "old_text": " ", "new_text": "x"}]
        self.assert_code("ambiguous_match", lambda: build_candidate(ambiguous, self.validated, 1024))
        outside = [{"kind": "replace", "path": "tests/acceptance.rs", "old_text": "protected", "new_text": "x"}]
        self.assert_code("scope_violation", lambda: build_candidate(outside, self.validated, 1024))
        empty = [{"kind": "replace", "path": "src/lib.rs", "old_text": "1", "new_text": "1"}]
        self.assert_code("empty_change", lambda: build_candidate(empty, self.validated, 1024))
        (self.root / "src" / "lib.rs").write_text("changed later\n", encoding="utf-8")
        valid = [{"kind": "replace", "path": "src/lib.rs", "old_text": "1", "new_text": "2"}]
        self.assert_code("stale_source", lambda: build_candidate(valid, self.validated, 1024))

    def test_parser_rejects_truncation_extra_tools_and_narration(self) -> None:
        truncated = response([], finish_reason="length")
        self.assert_code("output_truncated", lambda: parse_operations(truncated))
        extra = response([{"kind": "create", "path": "x", "content": "x"}])
        extra["choices"][0]["message"]["tool_calls"].append(extra["choices"][0]["message"]["tool_calls"][0])
        self.assert_code("extra_or_missing_tool", lambda: parse_operations(extra))
        narrated = response([{"kind": "create", "path": "x", "content": "x"}], content="done")
        self.assert_code("malformed_response", lambda: parse_operations(narrated))


if __name__ == "__main__":
    unittest.main()
