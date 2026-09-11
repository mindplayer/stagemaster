from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from local_worker.errors import WorkerError
from local_worker.model import ModelReply
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
        self.assertTrue((runner.store.job_dir(status["job_id"]) / "attempt-1" / "response.partial.json").is_file())
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
        release.set()
        thread.join(timeout=3)
        self.assertFalse(thread.is_alive())
        status = holder["status"]
        self.assertEqual(status["state"], "cancelled")
        self.assertFalse((jobs[0] / "attempt-1" / "candidate").exists())
        attempt = json.loads((jobs[0] / "attempt-1" / "attempt.json").read_text())
        self.assertTrue(attempt["late_response_discarded"])

    def test_explicit_repair_uses_next_attempt_and_enforces_limit(self) -> None:
        malformed = ModelReply(raw=b"{}", parsed={}, elapsed_seconds=0.01)
        generator = SequenceGenerator([malformed, reply(CHANGE), reply(CHANGE)])
        runner = Runner(self.config, generator)
        first = runner.run(self.task_path)
        self.assertEqual(first["error"]["class"], "malformed_response")
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
