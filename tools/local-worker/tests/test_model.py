from __future__ import annotations

import json
import sys
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from local_worker.errors import WorkerError
from local_worker.model import ModelReply, generate
from local_worker.runner import Runner
from support import config, make_repo, response, task, write_json


CHANGE = [{"kind": "replace", "path": "src/lib.rs", "old_text": "{ 1 }", "new_text": "{ 2 }"}]


class LoopbackCase(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.case = Path(self.temporary.name)
        self.workspace, self.commit = make_repo(self.case)
        self.config = config(self.case)
        self.task_path = self.case / "task.json"
        write_json(self.task_path, task(self.workspace, self.commit))

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def serve(
        self,
        *,
        body: bytes,
        status: int = 200,
        header_delay: float = 0.0,
        delay: float = 0.0,
        drip: float = 0.0,
    ):
        class Handler(BaseHTTPRequestHandler):
            def do_POST(handler):
                handler.rfile.read(int(handler.headers.get("Content-Length", "0")))
                if header_delay:
                    time.sleep(header_delay)
                handler.send_response(status)
                handler.send_header("Content-Length", str(len(body)))
                handler.end_headers()
                if delay:
                    time.sleep(delay)
                try:
                    for index, value in enumerate(body):
                        if index and drip:
                            time.sleep(drip)
                        handler.wfile.write(bytes([value]))
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
        return server, thread

    def profile(self, server: HTTPServer) -> dict[str, object]:
        return {"api_base": f"http://127.0.0.1:{server.server_port}/v1"}

    def assert_deadline(self, *, status: int) -> None:
        server, thread = self.serve(body=b'{"ok":true}', status=status, drip=0.08)
        started = time.monotonic()
        with self.assertRaises(WorkerError) as raised:
            generate({}, self.profile(server), 0.2, 1024)
        elapsed = time.monotonic() - started
        self.assertEqual(raised.exception.code, "request_timeout")
        self.assertTrue(raised.exception.response_incomplete)
        self.assertIsNotNone(raised.exception.response_bytes)
        self.assertLess(elapsed, 0.5)
        thread.join(timeout=2)
        self.assertFalse(thread.is_alive())
        self.assertFalse(
            any(candidate.name == "local-worker-http-deadline" for candidate in threading.enumerate())
        )

    def test_whole_request_deadline_covers_slow_success_body(self) -> None:
        self.assert_deadline(status=200)

    def test_whole_request_deadline_covers_slow_error_body(self) -> None:
        self.assert_deadline(status=500)

    def test_whole_request_deadline_covers_response_headers(self) -> None:
        server, thread = self.serve(body=b"{}", header_delay=0.5)
        started = time.monotonic()
        with self.assertRaises(WorkerError) as raised:
            generate({}, self.profile(server), 0.2, 1024)
        self.assertEqual(raised.exception.code, "request_timeout")
        self.assertLess(time.monotonic() - started, 0.5)
        thread.join(timeout=2)
        self.assertFalse(thread.is_alive())

    def run_response(self, raw: bytes, *, status: int = 200, delay: float = 0.05):
        server, thread = self.serve(body=raw, status=status, delay=delay)
        self.config["profiles"]["test"]["api_base"] = f"http://127.0.0.1:{server.server_port}/v1"
        result = Runner(self.config).run(self.task_path)
        thread.join(timeout=2)
        self.assertFalse(thread.is_alive())
        return result

    def response_bytes(self, status: dict[str, object]) -> list[bytes]:
        attempt = Path(self.config["_output_root"]) / str(status["job_id"]) / "attempt-1"
        return [path.read_bytes() for path in attempt.glob("response*")]

    def test_invalid_json_preserves_raw_bytes_elapsed_and_null_usage(self) -> None:
        raw = b"{broken-json"
        status = self.run_response(raw)
        attempt = Path(self.config["_output_root"]) / status["job_id"] / "attempt-1" / "attempt.json"
        manifest = json.loads(attempt.read_text())
        self.assertEqual(status["state"], "failed")
        self.assertEqual(status["error"]["class"], "malformed_response")
        self.assertIn(raw, self.response_bytes(status))
        self.assertGreaterEqual(status["generation_seconds_total"], 0.04)
        self.assertIsNone(manifest["usage"])
        self.assertFalse(manifest["response_incomplete"])

    def test_invalid_utf8_has_distinct_error_and_preserves_bytes(self) -> None:
        raw = b"\xff\xfe"
        status = self.run_response(raw)
        self.assertEqual(status["state"], "failed")
        self.assertEqual(status["error"]["class"], "invalid_response_encoding")
        self.assertIn(raw, self.response_bytes(status))
        self.assertGreaterEqual(status["generation_seconds_total"], 0.04)

    def test_non_object_json_is_classified_and_preserved(self) -> None:
        raw = b"[]"
        status = self.run_response(raw)
        self.assertEqual(status["error"]["class"], "malformed_response")
        self.assertIn(raw, self.response_bytes(status))

    def test_complete_http_error_preserves_body_and_marks_backend_finished(self) -> None:
        raw = b'{"error":"synthetic","usage":{"prompt_tokens":7}}'
        status = self.run_response(raw, status=500)
        attempt = Path(self.config["_output_root"]) / status["job_id"] / "attempt-1" / "attempt.json"
        manifest = json.loads(attempt.read_text())
        self.assertEqual(status["error"]["class"], "api_http_error")
        self.assertEqual(status["backend_state"], "finished")
        self.assertIn(raw, self.response_bytes(status))
        self.assertEqual(manifest["usage"], {"prompt_tokens": 7})

    def test_overlong_response_is_capped_and_keeps_backend_unresolved(self) -> None:
        raw = b"0123456789"
        self.config["limits"]["max_response_bytes"] = 5
        status = self.run_response(raw, delay=0.0)
        attempt = Path(self.config["_output_root"]) / status["job_id"] / "attempt-1"
        manifest = json.loads((attempt / "attempt.json").read_text())
        self.assertEqual(status["error"]["class"], "response_too_large")
        self.assertEqual(status["backend_state"], "unknown")
        self.assertEqual((attempt / "response.partial.json").read_bytes(), raw[:5])
        self.assertTrue(manifest["response_incomplete"])

    def test_task_remaining_budget_bounds_request_and_records_partial_timeout(self) -> None:
        server, thread = self.serve(body=b'{"slow":true}', drip=0.08)
        self.config["profiles"]["test"]["api_base"] = f"http://127.0.0.1:{server.server_port}/v1"
        task_data = task(self.workspace, self.commit)
        task_data["request_timeout_seconds"] = 1
        task_data["total_timeout_seconds"] = 1
        write_json(self.task_path, task_data)
        seen_timeouts: list[float] = []

        def generator(payload, profile, timeout, maximum):
            if not seen_timeouts:
                seen_timeouts.append(-1.0)
                parsed = response(CHANGE)
                return ModelReply(raw=json.dumps(parsed).encode(), parsed=parsed, elapsed_seconds=0.75)
            seen_timeouts.append(timeout)
            return generate(payload, profile, timeout, maximum)

        runner = Runner(self.config, generator)
        first = runner.run(self.task_path)
        diagnostics = self.case / "diagnostics.txt"
        diagnostics.write_text("retry", encoding="utf-8")
        failed = runner.repair(first["job_id"], diagnostics)
        thread.join(timeout=2)
        self.assertFalse(thread.is_alive())
        self.assertAlmostEqual(seen_timeouts[1], 0.25, delta=0.02)
        self.assertEqual(failed["error"]["class"], "request_timeout")
        self.assertEqual(failed["backend_state"], "unknown")
        self.assertGreaterEqual(failed["generation_seconds_total"], 0.94)
        self.assertLess(failed["generation_seconds_total"], 1.2)
        attempt = Path(self.config["_output_root"]) / failed["job_id"] / "attempt-2"
        manifest = json.loads((attempt / "attempt.json").read_text())
        self.assertTrue(manifest["response_incomplete"])
        self.assertGreater(manifest["model_seconds"], 0.0)
        self.assertTrue((attempt / "response.partial.json").is_file())


if __name__ == "__main__":
    unittest.main()
