"""Astra G0 review probes; expected to fail on 97e9366, without calling Qwen.

Run from any directory with Python 3.12. Temporary repositories and a loopback
HTTP stub are isolated from the project and the configured model gateway.
These are review evidence, not replacement production acceptance tests.
"""

from __future__ import annotations

import io
import json
import sys
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/local-worker"))
sys.path.insert(0, str(ROOT / "tools/local-worker/tests"))

from local_worker.errors import WorkerError
from local_worker.model import generate
from local_worker.proposal import build_candidate as original_build_candidate
from local_worker.runner import Runner
from support import config, make_repo, reply, task, write_json

CHANGE = [{"kind": "replace", "path": "src/lib.rs", "old_text": "{ 1 }", "new_text": "{ 2 }"}]


def observation(case: str, **values: object) -> None:
    print(json.dumps({"case": case, **values}, sort_keys=True), flush=True)


class G0ReviewRegressions(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="stagemaster-g0-review-")
        self.addCleanup(self.temporary.cleanup)
        self.case = Path(self.temporary.name)
        self.workspace, commit = make_repo(self.case)
        self.config = config(self.case)
        self.task_path = self.case / "task.json"
        write_json(self.task_path, task(self.workspace, commit))

    def test_accepted_cancel_during_candidate_build_cannot_publish(self) -> None:
        runner = Runner(self.config, lambda *args: reply(CHANGE))
        accepted: list[str] = []

        def cancel_at_boundary(*args, **kwargs):
            job = next(runner.store.root.glob("TEST-001-*"))
            accepted.append(runner.cancel(job.name)["state"])
            return original_build_candidate(*args, **kwargs)

        # Deterministically interleave cancel after the post-response flag check.
        with patch("local_worker.runner.build_candidate", side_effect=cancel_at_boundary):
            status = runner.run(self.task_path)
        observation("cancel_at_publish", accepted=accepted, final_state=status["state"],
                    cancel_flag=status["cancel_requested"], global_backend=runner.store.backend()["state"])
        self.assertEqual(accepted, ["cancel_requested"])
        self.assertEqual(status["state"], "cancelled")

    def test_context_rejection_leaves_terminal_record_without_inference(self) -> None:
        self.config["limits"]["max_context_bytes"] = 1
        calls: list[int] = []

        def generator(*args):
            calls.append(1)
            return reply(CHANGE)

        runner = Runner(self.config, generator)
        try:
            runner.run(self.task_path)
        except WorkerError as exc:
            self.assertEqual(exc.code, "context_too_large")
        job = next(runner.store.root.glob("TEST-001-*"))
        status = runner.store.load(job.name)
        observation("context_rejection", final_state=status["state"], backend=status["backend_state"],
                    calls=len(calls), error=status["error"])
        self.assertEqual(calls, [])
        self.assertEqual(status["state"], "failed")
        self.assertEqual(status["backend_state"], "finished")

    def test_invalid_json_keeps_received_bytes_and_elapsed_budget(self) -> None:
        runner = Runner(self.config)
        raw = b"{broken-json"
        # Two seconds elapsed at the transport boundary; no real network call.
        with patch("local_worker.model.urlopen", return_value=io.BytesIO(raw)), patch(
            "local_worker.model.time.monotonic", side_effect=[100.0, 102.0]
        ):
            status = runner.run(self.task_path)
        attempt = runner.store.job_dir(status["job_id"]) / "attempt-1"
        response_files = list(attempt.glob("response*"))
        observation("malformed_json", final_state=status["state"], error=status["error"]["class"],
                    response_files=[p.name for p in response_files], recorded_seconds=status["generation_seconds_total"])
        self.assertEqual(status["state"], "failed")
        problems = []
        if not any(p.read_bytes() == raw for p in response_files):
            problems.append("received response bytes not preserved")
        if status["generation_seconds_total"] < 2.0:
            problems.append("elapsed request time missing from budget")
        self.assertEqual(problems, [])

    def test_dripping_http_body_obeys_whole_request_deadline(self) -> None:
        class DrippingResponse(BaseHTTPRequestHandler):
            def do_POST(self):
                self.rfile.read(int(self.headers.get("Content-Length", "0")))
                body = b'{"ok":true}'
                self.send_response(200)
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                try:
                    for index, value in enumerate(body):
                        if index:
                            time.sleep(0.08)
                        self.wfile.write(bytes([value]))
                        self.wfile.flush()
                except (BrokenPipeError, ConnectionResetError):
                    pass

            def log_message(self, *args):
                pass

        server = HTTPServer(("127.0.0.1", 0), DrippingResponse)
        server.timeout = 2
        thread = threading.Thread(target=server.handle_request, daemon=True)
        thread.start()
        started = time.monotonic()
        timeout = 0.2
        error = None
        try:
            generate({}, {"api_base": f"http://127.0.0.1:{server.server_port}/v1"}, timeout, 1024)
        except WorkerError as exc:
            error = exc.code
        finally:
            elapsed = time.monotonic() - started
            thread.join(timeout=2)
            server.server_close()
        observation("request_deadline", limit_seconds=timeout, elapsed_seconds=round(elapsed, 3), error=error)
        self.assertEqual(error, "request_timeout")
        self.assertLess(elapsed, timeout + 0.3)


if __name__ == "__main__":
    unittest.main(verbosity=2)
