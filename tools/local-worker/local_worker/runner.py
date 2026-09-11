from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Callable

from .errors import WorkerError
from .model import ModelReply, generate, request_payload
from .proposal import build_candidate, parse_operations, write_candidate
from .records import Store, response_hash
from .task import ValidatedTask, build_context, validate_task, validate_task_file
from .util import atomic_json, atomic_write, exclusive_lock, process_identity, read_json, run_git, safe_path, sha256_file


Generator = Callable[[dict[str, Any], dict[str, Any], float, int], ModelReply]
StartedCallback = Callable[[dict[str, Any]], None]


class Runner:
    def __init__(self, config: dict[str, Any], generator: Generator = generate):
        self.config = config
        self.store = Store(config)
        self.generator = generator

    def _active_recovery_required(self) -> None:
        for directory in self.store.root.iterdir():
            status_path = directory / "status.json"
            if not directory.is_dir() or not status_path.is_file():
                continue
            status = read_json(status_path)
            if status.get("state") not in {"running", "cancel_requested"}:
                continue
            owner_pid = status.get("owner_pid")
            owner_identity = status.get("owner_identity")
            if not isinstance(owner_pid, int) or process_identity(owner_pid) != owner_identity:
                raise WorkerError(
                    "recovery_required",
                    "an interrupted job must be recovered explicitly before new generation",
                    {"job_id": status.get("job_id")},
                )

    def run(self, task_path: Path, on_started: StartedCallback | None = None) -> dict[str, Any]:
        task = validate_task_file(task_path, self.config)
        existing = self.store.lookup(task)
        if existing is not None:
            return {**existing, "duplicate": True}
        with exclusive_lock(self.store.root / ".generation.lock", blocking=False):
            self._active_recovery_required()
            self.store.ensure_backend_available()
            status, created = self.store.create(task)
            if not created:
                return {**status, "duplicate": True}
            if on_started:
                on_started(status)
            return self._attempt(task, status["job_id"], previous_diff=None, diagnostics=None)

    def repair(self, job_id: str, diagnostics_path: Path, on_started: StartedCallback | None = None) -> dict[str, Any]:
        status = self.store.load(job_id)
        if status["state"] not in {"candidate_ready", "failed"}:
            raise WorkerError("invalid_state", f"job cannot be repaired from {status['state']}")
        saved = read_json(self.store.job_dir(job_id) / "task.json")
        task = validate_task(saved["task"], self.config)
        if task.input_digest != saved.get("input_digest"):
            raise WorkerError("artifact_integrity", "stored task identity no longer matches")
        if status["attempt"] >= task.data["max_attempts"]:
            raise WorkerError("repair_limit", "max_attempts has been reached")
        if not diagnostics_path.is_file() or diagnostics_path.is_symlink():
            raise WorkerError("invalid_diagnostics", "diagnostics must be a regular UTF-8 file")
        raw_diagnostics = diagnostics_path.read_bytes()
        maximum = int(self.config["limits"]["max_diagnostics_bytes"])
        if len(raw_diagnostics) > maximum:
            raise WorkerError("diagnostics_too_large", f"diagnostics exceed {maximum} bytes")
        try:
            diagnostics = raw_diagnostics.decode("utf-8")
        except UnicodeDecodeError as exc:
            raise WorkerError("invalid_diagnostics", "diagnostics must be UTF-8") from exc
        previous_diff_path = self.store.job_dir(job_id) / f"attempt-{status['attempt']}" / "candidate.diff"
        previous_diff = previous_diff_path.read_text(encoding="utf-8") if previous_diff_path.is_file() else "(no valid prior candidate)"
        with exclusive_lock(self.store.root / ".generation.lock", blocking=False):
            self._active_recovery_required_for(job_id)
            self.store.ensure_backend_available()
            reset = self.store.update(
                job_id,
                state="running",
                progress="explicit repair started",
                cancel_requested=False,
                owner_pid=os.getpid(),
                owner_identity=process_identity(os.getpid()),
                error=None,
            )
            if on_started:
                on_started(reset)
            return self._attempt(task, job_id, previous_diff=previous_diff, diagnostics=diagnostics)

    def _active_recovery_required_for(self, current_job_id: str) -> None:
        for directory in self.store.root.iterdir():
            status_path = directory / "status.json"
            if directory.name == current_job_id or not directory.is_dir() or not status_path.is_file():
                continue
            status = read_json(status_path)
            if status.get("state") in {"running", "cancel_requested"}:
                owner_pid = status.get("owner_pid")
                if not isinstance(owner_pid, int) or process_identity(owner_pid) != status.get("owner_identity"):
                    raise WorkerError("recovery_required", "another interrupted job requires explicit recovery")

    def _attempt(
        self,
        task: ValidatedTask,
        job_id: str,
        *,
        previous_diff: str | None,
        diagnostics: str | None,
    ) -> dict[str, Any]:
        status = self.store.load(job_id)
        attempt = int(status["attempt"]) + 1
        remaining = float(task.data["total_timeout_seconds"]) - float(status["generation_seconds_total"])
        if remaining <= 0:
            return self._fail(job_id, WorkerError("total_timeout", "generation time budget is exhausted"), "finished")
        attempt_dir = self.store.job_dir(job_id) / f"attempt-{attempt}"
        attempt_dir.mkdir(mode=0o700)
        context = build_context(task, self.config, previous_diff=previous_diff, diagnostics=diagnostics)
        profile = self.config["profiles"][task.data["model_profile"]]
        payload = request_payload(context, task.data["write_paths"], profile, task.data["max_output_tokens"])
        atomic_json(attempt_dir / "request.json", payload)
        self.store.update(
            job_id,
            state="running",
            attempt=attempt,
            progress=f"model request {attempt} in progress",
            owner_pid=os.getpid(),
            owner_identity=process_identity(os.getpid()),
            backend_state="unknown",
        )
        if self.store.load(job_id)["cancel_requested"]:
            return self._cancelled(job_id, "finished", "cancelled before model request")
        try:
            reply = self.generator(
                payload,
                profile,
                min(float(task.data["request_timeout_seconds"]), remaining),
                int(self.config["limits"]["max_response_bytes"]),
            )
        except WorkerError as exc:
            elapsed = float(exc.details.get("elapsed_seconds", 0.0))
            if elapsed:
                self.store.update(
                    job_id,
                    generation_seconds_total=round(float(status["generation_seconds_total"]) + elapsed, 3),
                )
            backend_state = "unknown" if exc.code in {"request_timeout", "api_disconnect", "api_http_error"} else "finished"
            atomic_json(attempt_dir / "attempt.json", {"attempt": attempt, "state": "failed", "error": exc.as_dict()})
            return self._fail(job_id, exc, backend_state)

        atomic_write(attempt_dir / ("response.partial.json" if reply.over_limit else "response.raw.json"), reply.raw)
        total_seconds = float(status["generation_seconds_total"]) + reply.elapsed_seconds
        self.store.update(job_id, generation_seconds_total=round(total_seconds, 3))
        self.store.set_backend("finished", None)
        if self.store.load(job_id)["cancel_requested"]:
            atomic_json(
                attempt_dir / "attempt.json",
                {
                    "attempt": attempt,
                    "state": "cancelled",
                    "late_response_discarded": True,
                    "response_sha256": response_hash(reply.raw),
                    "model_seconds": round(reply.elapsed_seconds, 3),
                },
            )
            return self._cancelled(job_id, "finished", "late model response was persisted but not accepted")
        if reply.over_limit:
            error = WorkerError(
                "response_too_large",
                f"response exceeded {self.config['limits']['max_response_bytes']} bytes; partial bytes saved",
            )
            atomic_json(attempt_dir / "attempt.json", {"attempt": attempt, "state": "failed", "error": error.as_dict()})
            return self._fail(job_id, error, "finished")
        try:
            operations, usage, returned_model = parse_operations(reply.parsed)
            candidate = build_candidate(operations, task, int(self.config["limits"]["max_file_bytes"]))
            write_candidate(attempt_dir, candidate)
        except WorkerError as exc:
            atomic_json(
                attempt_dir / "attempt.json",
                {
                    "attempt": attempt,
                    "state": "failed",
                    "response_sha256": response_hash(reply.raw),
                    "model_seconds": round(reply.elapsed_seconds, 3),
                    "error": exc.as_dict(),
                },
            )
            return self._fail(job_id, exc, "finished")

        attempt_manifest = {
            "attempt": attempt,
            "state": "candidate_ready",
            "response_sha256": response_hash(reply.raw),
            "proposal_hash": candidate.proposal_hash,
            "candidate_hashes": candidate.candidate_hashes,
            "model_seconds": round(reply.elapsed_seconds, 3),
            "requested_model": profile["model"],
            "returned_model": returned_model,
            "runtime": self.config["runtime"],
            "usage": usage,
            "read_paths": task.data["read_paths"],
            "changed_paths": sorted(candidate.files),
            "diff_path": str(attempt_dir / "candidate.diff"),
            "candidate_root": str(attempt_dir / "candidate"),
        }
        atomic_json(attempt_dir / "attempt.json", attempt_manifest)
        return self.store.update(
            job_id,
            state="candidate_ready",
            progress="candidate text and diff validated; host tests not run",
            owner_pid=None,
            owner_identity=None,
            backend_state="finished",
            proposal_hash=candidate.proposal_hash,
            candidate_hashes=candidate.candidate_hashes,
            result_manifest=str(attempt_dir / "attempt.json"),
            error=None,
            unresolved=["candidate_ready does not mean compiled, tested, accepted, or integrated"],
        )

    def _fail(self, job_id: str, error: WorkerError, backend_state: str) -> dict[str, Any]:
        self.store.set_backend(backend_state, error.code if backend_state != "finished" else None)
        return self.store.update(
            job_id,
            state="failed",
            progress=error.message,
            owner_pid=None,
            owner_identity=None,
            backend_state=backend_state,
            error=error.as_dict(),
            unresolved=[error.message],
        )

    def _cancelled(self, job_id: str, backend_state: str, progress: str) -> dict[str, Any]:
        self.store.set_backend(backend_state, None)
        return self.store.update(
            job_id,
            state="cancelled",
            progress=progress,
            owner_pid=None,
            owner_identity=None,
            backend_state=backend_state,
            error=None,
            unresolved=[],
        )

    def cancel(self, job_id: str) -> dict[str, Any]:
        status = self.store.load(job_id)
        if status["state"] != "running":
            return status
        updated = self.store.update(
            job_id,
            state="cancel_requested",
            progress="cancellation accepted; waiting for bounded request to return",
            cancel_requested=True,
            backend_state="still_running",
        )
        self.store.set_backend("still_running", "cancel requested while bounded request is active")
        return updated

    def recover(self, job_id: str | None, backend_state: str) -> dict[str, Any]:
        if job_id is not None:
            status = self.store.load(job_id)
            if status["state"] in {"running", "cancel_requested"}:
                owner_pid = status.get("owner_pid")
                if isinstance(owner_pid, int) and process_identity(owner_pid) == status.get("owner_identity"):
                    raise WorkerError("owner_still_running", "recorded worker process identity is still active")
                status = self.store.update(
                    job_id,
                    state="interrupted",
                    progress="explicitly recovered after worker process loss; request was not replayed",
                    owner_pid=None,
                    owner_identity=None,
                    backend_state=backend_state,
                    error={
                        "class": "process_restart",
                        "message": "owner process identity no longer matches",
                        "details": {},
                    },
                    unresolved=[] if backend_state == "finished" else ["backend completion remains unconfirmed"],
                )
            else:
                status = self.store.update(job_id, backend_state=backend_state)
        else:
            status = {"job_id": None, "state": "backend_recovered"}
        self.store.set_backend(backend_state, None if backend_state == "finished" else "explicit recovery")
        return {**status, "global_backend_state": backend_state}

    def verify_candidate_source(self, job_id: str) -> dict[str, Any]:
        status = self.store.load(job_id)
        saved = read_json(self.store.job_dir(job_id) / "task.json")
        if saved.get("input_digest") != status.get("input_digest"):
            raise WorkerError("artifact_integrity", "stored task digest does not match status")
        workspace = Path(saved["task"]["workspace_root"])
        mismatches = []
        head = run_git(workspace, ["rev-parse", "HEAD"]).decode().strip()
        if head != saved["task"]["base_commit"]:
            mismatches.append({"path": "HEAD", "expected": saved["task"]["base_commit"], "actual": head})
        for relative, expected in saved["source_hashes"].items():
            actual = sha256_file(safe_path(workspace, relative, must_exist=expected != "missing"))
            if actual != expected:
                mismatches.append({"path": relative, "expected": expected, "actual": actual})
        if mismatches:
            raise WorkerError("stale_source", "candidate source is stale", {"mismatches": mismatches})
        return {"job_id": job_id, "state": status["state"], "source_current": True}
