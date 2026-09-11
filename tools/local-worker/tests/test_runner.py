from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from local_worker.errors import WorkerError
from local_worker.model import ModelReply
from local_worker.proposal import build_candidate as original_build_candidate
from local_worker.runner import Runner
from local_worker.task import validate_task
from local_worker.util import process_identity
from support import config, make_repo, reply, task, write_json


CHANGE = [{"kind": "replace", "path": "src/lib.rs", "old_text": "{ 1 }", "new_text": "{ 2 }"}]


class SequenceGenerator:
    def __init__(self, replies):
        self.replies = list(replies)
        self.calls = 0

    def __call__(self, payload, profile, timeout, maximum):
        self.calls += 1
        value = self.replies.pop(0)
        if isinstance(value, Exception):
            raise value
        return value


class RunnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.case = Path(self.temporary.name)
        self.root, self.commit = make_repo(self.case)
        self.config = config(self.case)
        self.task_data = task(self.root, self.commit)
        self.task_path = self.case / "task.json"
        write_json(self.task_path, self.task_data)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def assert_code(self, code: str, function) -> None:
        with self.assertRaises(WorkerError) as raised:
            function()
        self.assertEqual(raised.exception.code, code)

    def test_success_persists_candidate_and_duplicate_does_not_regenerate(self) -> None:
        generator = SequenceGenerator([reply(CHANGE)])
        runner = Runner(self.config, generator)
        status = runner.run(self.task_path)
        self.assertEqual(status["state"], "candidate_ready")
        self.assertEqual(status["attempt"], 1)
        self.assertEqual(generator.calls, 1)
        job = runner.store.job_dir(status["job_id"])
        self.assertTrue((job / "attempt-1" / "response.raw.json").is_file())
        self.assertTrue((job / "attempt-1" / "candidate.diff").is_file())
        self.assertEqual((self.root / "src" / "lib.rs").read_text(), "pub fn value() -> u8 { 1 }\n")
        duplicate = runner.run(self.task_path)
        self.assertTrue(duplicate["duplicate"])
        self.assertEqual(generator.calls, 1)

    def test_same_task_id_with_changed_input_is_conflict(self) -> None:
        runner = Runner(self.config, SequenceGenerator([reply(CHANGE)]))
        runner.run(self.task_path)
        changed = dict(self.task_data)
        changed["goal"] = "Different goal"
        changed_path = self.case / "changed.json"
        write_json(changed_path, changed)
        self.assert_code("identity_conflict", lambda: runner.run(changed_path))

    def test_busy_is_returned_without_creating_a_job(self) -> None:
        runner = Runner(self.config, SequenceGenerator([reply(CHANGE)]))
        lock = runner.store.root / ".generation.lock"
        ready = self.case / "ready"
        child = subprocess.Popen(
            [
                sys.executable,
                "-c",
                "import fcntl,sys,time,pathlib; p=pathlib.Path(sys.argv[1]); p.parent.mkdir(parents=True,exist_ok=True); f=p.open('a+b'); fcntl.flock(f,fcntl.LOCK_EX); pathlib.Path(sys.argv[2]).touch(); time.sleep(10)",
                str(lock),
                str(ready),
            ]
        )
        try:
            for _ in range(100):
                if ready.exists():
                    break
                threading.Event().wait(0.01)
            self.assertTrue(ready.exists())
            self.assert_code("busy", lambda: runner.run(self.task_path))
            self.assertEqual(list(runner.store.root.glob("TEST-001-*")), [])
        finally:
            child.terminate()
            child.wait(timeout=3)

    def test_oversized_response_is_saved_as_partial_and_source_is_untouched(self) -> None:
        oversized = ModelReply(raw=b"x" * 20, parsed={}, elapsed_seconds=0.01, over_limit=True)
        runner = Runner(self.config, SequenceGenerator([oversized]))
        status = runner.run(self.task_path)
        self.assertEqual(status["state"], "failed")
        self.assertEqual(status["error"]["class"], "response_too_large")
        self.assertEqual(status["backend_state"], "unknown")
        attempt = runner.store.job_dir(status["job_id"]) / "attempt-1"
        self.assertTrue((attempt / "response.partial.json").is_file())
        manifest = json.loads((attempt / "attempt.json").read_text())
        self.assertTrue(manifest["response_incomplete"])
        self.assertEqual(manifest["model_seconds"], 0.01)
        self.assertEqual(runner.store.backend()["state"], "unknown")
        self.assertEqual((self.root / "src" / "lib.rs").read_text(), "pub fn value() -> u8 { 1 }\n")

    def test_timeout_blocks_generation_until_explicit_backend_recovery(self) -> None:
        timeout = WorkerError("request_timeout", "synthetic timeout")
        runner = Runner(self.config, SequenceGenerator([timeout]))
        failed = runner.run(self.task_path)
        self.assertEqual(failed["backend_state"], "unknown")

        second_data = task(self.root, self.commit, task_id="TEST-002")
        second_path = self.case / "second.json"
        write_json(second_path, second_data)
        second_runner = Runner(self.config, SequenceGenerator([reply(CHANGE)]))
        self.assert_code("backend_unresolved", lambda: second_runner.run(second_path))
        second_runner.recover(failed["job_id"], "finished")
        self.assertEqual(second_runner.run(second_path)["state"], "candidate_ready")

    def test_cancel_discards_late_response(self) -> None:
        entered = threading.Event()
        release = threading.Event()

        def slow_generator(payload, profile, timeout, maximum):
            entered.set()
            self.assertTrue(release.wait(3))
            return reply(CHANGE)

        runner = Runner(self.config, slow_generator)
        holder: dict[str, object] = {}

        def run_job() -> None:
            holder["status"] = runner.run(self.task_path)

        thread = threading.Thread(target=run_job)
        thread.start()
        self.assertTrue(entered.wait(3))
        jobs = list(runner.store.root.glob("TEST-001-*"))
        self.assertEqual(len(jobs), 1)
        job_id = jobs[0].name
        requested = runner.cancel(job_id)
        self.assertEqual(requested["state"], "cancel_requested")
        self.assertEqual(requested["backend_state"], "still_running")
        release.set()
        thread.join(timeout=3)
        self.assertFalse(thread.is_alive())
        status = holder["status"]
        self.assertEqual(status["state"], "cancelled")
        self.assertFalse((jobs[0] / "attempt-1" / "candidate").exists())
        attempt = json.loads((jobs[0] / "attempt-1" / "attempt.json").read_text())
        self.assertTrue(attempt["late_response_discarded"])

    def test_cancel_during_candidate_build_wins_atomic_publication(self) -> None:
        runner = Runner(self.config, SequenceGenerator([reply(CHANGE)]))
        accepted: list[str] = []

        def cancel_at_boundary(*args, **kwargs):
            job = next(runner.store.root.glob("TEST-001-*"))
            accepted.append(runner.cancel(job.name)["state"])
            return original_build_candidate(*args, **kwargs)

        with patch("local_worker.runner.build_candidate", side_effect=cancel_at_boundary):
            status = runner.run(self.task_path)

        job = runner.store.job_dir(status["job_id"])
        self.assertEqual(accepted, ["cancel_requested"])
        self.assertEqual(status["state"], "cancelled")
        self.assertEqual(status["backend_state"], "finished")
        self.assertEqual(runner.store.backend()["state"], "finished")
        self.assertFalse((job / "attempt-1" / "candidate").exists())
        self.assertEqual(json.loads((job / "attempt-1" / "attempt.json").read_text())["state"], "cancelled")
        self.assert_code("invalid_state", lambda: runner.verify_candidate_source(status["job_id"]))

    def test_cancel_before_request_and_after_terminal_have_consistent_results(self) -> None:
        generator = SequenceGenerator([reply(CHANGE)])
        runner = Runner(self.config, generator)

        cancelled = runner.run(self.task_path, on_started=lambda status: runner.cancel(status["job_id"]))
        self.assertEqual(cancelled["state"], "cancelled")
        self.assertEqual(cancelled["backend_state"], "finished")
        self.assertEqual(generator.calls, 0)

        second_data = task(self.root, self.commit, task_id="TEST-002")
        second_path = self.case / "second.json"
        write_json(second_path, second_data)
        ready = runner.run(second_path)
        after_terminal = runner.cancel(ready["job_id"])
        self.assertEqual(after_terminal["state"], "candidate_ready")
        self.assertFalse(after_terminal["cancel_requested"])

    def test_context_rejection_finishes_record_and_does_not_block_next_task(self) -> None:
        self.config["limits"]["max_context_bytes"] = 1
        generator = SequenceGenerator([reply(CHANGE)])
        runner = Runner(self.config, generator)

        failed = runner.run(self.task_path)
        self.assertEqual(failed["state"], "failed")
        self.assertEqual(failed["backend_state"], "finished")
        self.assertEqual(failed["error"]["class"], "context_too_large")
        self.assertIsNone(failed["owner_pid"])
        self.assertEqual(generator.calls, 0)
        attempt = runner.store.job_dir(failed["job_id"]) / "attempt-1" / "attempt.json"
        self.assertEqual(json.loads(attempt.read_text())["state"], "failed")

        self.config["limits"]["max_context_bytes"] = 65536
        second_data = task(self.root, self.commit, task_id="TEST-002")
        second_path = self.case / "second.json"
        write_json(second_path, second_data)
        self.assertEqual(runner.run(second_path)["state"], "candidate_ready")

    def test_payload_preparation_failure_is_terminal_without_model_call(self) -> None:
        generator = SequenceGenerator([reply(CHANGE)])
        runner = Runner(self.config, generator)
        with patch("local_worker.runner.request_payload", side_effect=ValueError("synthetic payload failure")):
            failed = runner.run(self.task_path)
        self.assertEqual(failed["state"], "failed")
        self.assertEqual(failed["backend_state"], "finished")
        self.assertEqual(failed["error"]["class"], "request_preparation_failed")
        self.assertEqual(generator.calls, 0)

    def test_repair_preparation_failure_is_terminal_and_preserves_prior_candidate(self) -> None:
        generator = SequenceGenerator([reply(CHANGE)])
        runner = Runner(self.config, generator)
        first = runner.run(self.task_path)
        job = runner.store.job_dir(first["job_id"])
        original_candidate = (job / "attempt-1" / "candidate" / "src" / "lib.rs").read_bytes()
        diagnostics = self.case / "diagnostics.txt"
        diagnostics.write_text("retry with host diagnostics", encoding="utf-8")
        self.config["limits"]["max_context_bytes"] = 1

        failed = runner.repair(first["job_id"], diagnostics)
        self.assertEqual(failed["state"], "failed")
        self.assertEqual(failed["attempt"], 2)
        self.assertEqual(failed["backend_state"], "finished")
        self.assertEqual(failed["error"]["class"], "context_too_large")
        self.assertEqual(generator.calls, 1)
        self.assertEqual((job / "attempt-1" / "candidate" / "src" / "lib.rs").read_bytes(), original_candidate)
        self.assertEqual(json.loads((job / "attempt-2" / "attempt.json").read_text())["state"], "failed")

    def test_competing_repairs_cannot_exceed_attempt_limit(self) -> None:
        entered = threading.Event()
        release = threading.Event()

        def generator(payload, profile, timeout, maximum):
            generator.calls += 1
            if generator.calls == 2:
                entered.set()
                self.assertTrue(release.wait(3))
            return reply(CHANGE)

        generator.calls = 0
        self.task_data["max_attempts"] = 2
        write_json(self.task_path, self.task_data)
        runner = Runner(self.config, generator)
        first = runner.run(self.task_path)
        diagnostics = self.case / "diagnostics.txt"
        diagnostics.write_text("retry", encoding="utf-8")
        holder: dict[str, object] = {}

        thread = threading.Thread(
            target=lambda: holder.update(status=runner.repair(first["job_id"], diagnostics))
        )
        thread.start()
        self.assertTrue(entered.wait(3))
        with self.assertRaises(WorkerError) as raised:
            runner.repair(first["job_id"], diagnostics)
        self.assertIn(raised.exception.code, {"busy", "invalid_state"})
        release.set()
        thread.join(timeout=3)
        self.assertFalse(thread.is_alive())
        self.assertEqual(holder["status"]["attempt"], 2)
        self.assertEqual(generator.calls, 2)
        self.assert_code("repair_limit", lambda: runner.repair(first["job_id"], diagnostics))
        self.assertFalse((runner.store.job_dir(first["job_id"]) / "attempt-3").exists())

    def test_explicit_repair_uses_next_attempt_and_enforces_limit(self) -> None:
        malformed = ModelReply(raw=b"{}", parsed={}, elapsed_seconds=0.01)
        generator = SequenceGenerator([malformed, reply(CHANGE), reply(CHANGE)])
        runner = Runner(self.config, generator)
        first = runner.run(self.task_path)
        self.assertEqual(first["error"]["class"], "malformed_response")
        self.assertEqual(first["generation_seconds_total"], 0.01)
        first_manifest = json.loads(
            (runner.store.job_dir(first["job_id"]) / "attempt-1" / "attempt.json").read_text()
        )
        self.assertEqual(first_manifest["model_seconds"], 0.01)
        self.assertIsNone(first_manifest["usage"])
        diagnostics = self.case / "diagnostics.txt"
        diagnostics.write_text("compiler expected a valid candidate", encoding="utf-8")
        second = runner.repair(first["job_id"], diagnostics)
        self.assertEqual(second["state"], "candidate_ready")
        self.assertEqual(second["attempt"], 2)
        third = runner.repair(first["job_id"], diagnostics)
        self.assertEqual(third["attempt"], 3)
        self.assert_code("repair_limit", lambda: runner.repair(first["job_id"], diagnostics))

    def test_recover_checks_owner_identity_and_does_not_replay(self) -> None:
        runner = Runner(self.config, SequenceGenerator([reply(CHANGE)]))
        validated = validate_task(self.task_data, self.config)
        status, _ = runner.store.create(validated)
        runner.store.update(status["job_id"], owner_pid=999999, owner_identity="not-real", state="running")
        recovered = runner.recover(status["job_id"], "finished")
        self.assertEqual(recovered["state"], "interrupted")
        self.assertEqual(recovered["error"]["class"], "process_restart")
        self.assertEqual(recovered["attempt"], 0)

        runner.store.update(
            status["job_id"],
            state="running",
            owner_pid=os.getpid(),
            owner_identity=process_identity(os.getpid()),
        )
        self.assert_code("owner_still_running", lambda: runner.recover(status["job_id"], "finished"))

    def test_verify_source_rejects_expired_candidate(self) -> None:
        runner = Runner(self.config, SequenceGenerator([reply(CHANGE)]))
        status = runner.run(self.task_path)
        (self.root / "src" / "lib.rs").write_text("later edit\n", encoding="utf-8")
        self.assert_code("stale_source", lambda: runner.verify_candidate_source(status["job_id"]))


if __name__ == "__main__":
    unittest.main()
