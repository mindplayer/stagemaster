"""DEV-004 residual contract probes; no Qwen calls or product source writes.

Expected to fail on f7fafb5. Temporary repositories and loopback HTTP only.
"""

from __future__ import annotations

import json
import sys
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path[:0] = [str(ROOT / "tools/local-worker"), str(ROOT / "tools/local-worker/tests")]

from local_worker.errors import WorkerError
from local_worker.model import generate
from local_worker.runner import Runner
from support import config, make_repo, reply, response, task, write_json

CHANGE = [{"kind": "replace", "path": "src/lib.rs", "old_text": "{ 1 }", "new_text": "{ 2 }"}]


class Dev004Rereview(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="stagemaster-dev004-review-")
        self.addCleanup(temporary.cleanup)
        self.case = Path(temporary.name)
        self.workspace, commit = make_repo(self.case)
        self.config = config(self.case)
        self.task_path = self.case / "task.json"
        write_json(self.task_path, task(self.workspace, commit))

    def serve(self, *, delayed_headers=False, short_body=False):
        body = json.dumps(response(CHANGE)).encode()

        class Handler(BaseHTTPRequestHandler):
            # HTTP/1.0 closes the connection; Content-Length still frames the body.
            protocol_version = "HTTP/1.0"

            def do_POST(handler):
                handler.rfile.read(int(handler.headers.get("Content-Length", "0")))
                if delayed_headers:
                    time.sleep(0.35)
                handler.send_response(200)
                handler.send_header("Content-Length", str(len(body) + (20 if short_body else 0)))
                handler.end_headers()
                try:
                    if delayed_headers:
                        handler.wfile.write(body[:1])
                        handler.wfile.flush()
                        time.sleep(0.8)
                        handler.wfile.write(body[1:])
                    else:
                        handler.wfile.write(body)
                    handler.wfile.flush()
                except (BrokenPipeError, ConnectionResetError):
                    pass

            def log_message(self, *args):
                pass

        server = HTTPServer(("127.0.0.1", 0), Handler)
        server.timeout = 2
        thread = threading.Thread(target=server.handle_request)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(lambda: thread.join(timeout=2))
        return {"api_base": f"http://127.0.0.1:{server.server_port}/v1"}, thread, body

    def test_late_cancel_cannot_rewrite_finished_backend(self):
        request_started, release_reply = threading.Event(), threading.Event()
        cancel_write, release_cancel = threading.Event(), threading.Event()
        results, failures = {}, []

        def generator(*args):
            request_started.set()
            if not release_reply.wait(3):
                raise AssertionError("review response barrier timed out")
            return reply(CHANGE)

        runner = Runner(self.config, generator)
        original_set_backend = runner.store.set_backend

        def delayed_set_backend(state, reason):
            if threading.current_thread().name == "review-cancel":
                cancel_write.set()
                if not release_cancel.wait(3):
                    raise AssertionError("review cancel barrier timed out")
            original_set_backend(state, reason)

        # Force cancellation to pause after its job update, before its backend write.
        runner.store.set_backend = delayed_set_backend

        def capture(name, function):
            try:
                results[name] = function()
            except Exception as exc:
                failures.append(repr(exc))

        worker = threading.Thread(target=lambda: capture("run", lambda: runner.run(self.task_path)))
        cancel_thread = None
        try:
            worker.start()
            self.assertTrue(request_started.wait(3))
            job_id = next(runner.store.root.glob("TEST-001-*")).name
            cancel_thread = threading.Thread(
                target=lambda: capture("cancel", lambda: runner.cancel(job_id)), name="review-cancel"
            )
            cancel_thread.start()
            self.assertTrue(cancel_write.wait(3))
            release_reply.set()
            worker.join(timeout=2)
            self.assertFalse(worker.is_alive())
            release_cancel.set()
            cancel_thread.join(timeout=2)
            self.assertFalse(cancel_thread.is_alive())
        finally:
            release_reply.set()
            release_cancel.set()
            worker.join(timeout=3)
            if cancel_thread is not None:
                cancel_thread.join(timeout=3)
        self.assertEqual(failures, [])
        status = runner.store.load(job_id)
        backend = runner.store.backend()["state"]
        print(json.dumps({"case": "late_cancel_backend", "job_state": status["state"],
                          "job_backend": status["backend_state"], "global_backend": backend}), flush=True)
        self.assertEqual(status["state"], "cancelled")
        self.assertEqual(status["backend_state"], "finished")
        self.assertEqual(backend, "finished")

    def test_delayed_headers_then_stalled_close_connection_obeys_deadline(self):
        profile, _, _ = self.serve(delayed_headers=True)
        started = time.monotonic()
        with self.assertRaises(WorkerError) as raised:
            generate({}, profile, 0.5, 4096)
        elapsed = time.monotonic() - started
        print(json.dumps({"case": "close_connection_deadline", "limit": 0.5,
                          "elapsed": round(elapsed, 3), "error": raised.exception.code}), flush=True)
        self.assertEqual(raised.exception.code, "request_timeout")
        self.assertLess(elapsed, 0.7)

    def test_short_content_length_response_cannot_publish_candidate(self):
        profile, _, body = self.serve(short_body=True)
        self.config["profiles"]["test"]["api_base"] = profile["api_base"]
        runner = Runner(self.config)
        status = runner.run(self.task_path)
        attempt = runner.store.job_dir(status["job_id"]) / "attempt-1"
        manifest = json.loads((attempt / "attempt.json").read_text())
        print(json.dumps({"case": "short_content_length", "actual_bytes": len(body),
                          "declared_bytes": len(body) + 20, "state": status["state"],
                          "incomplete": manifest["response_incomplete"],
                          "backend": status["backend_state"]}), flush=True)
        self.assertEqual(status["state"], "failed")
        self.assertEqual(status["backend_state"], "unknown")
        self.assertTrue(manifest["response_incomplete"])
        self.assertEqual((attempt / "response.partial.json").read_bytes(), body)
        self.assertFalse((attempt / "candidate").exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
