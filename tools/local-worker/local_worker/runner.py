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
        with exclusive_lock(self.store.root / ".generation.lock", blocking=False):
            self._active_recovery_required_for(job_id)
            self.store.ensure_backend_available()
            fresh = self.store.load(job_id)
            if fresh["state"] not in {"candidate_ready", "failed"}:
                raise WorkerError("invalid_state", f"job cannot be repaired from {fresh['state']}")
            if fresh["attempt"] >= task.data["max_attempts"]:
                raise WorkerError("repair_limit", "max_attempts has been reached")
            remaining = float(task.data["total_timeout_seconds"]) - float(fresh["generation_seconds_total"])
            if remaining <= 0:
                raise WorkerError("total_timeout", "generation time budget is exhausted")
            previous_diff_path = self.store.job_dir(job_id) / f"attempt-{fresh['attempt']}" / "candidate.diff"
            previous_diff = (
                previous_diff_path.read_text(encoding="utf-8")
                if previous_diff_path.is_file()
                else "(no valid prior candidate)"
            )
            with self.store.locked_status(job_id) as current:
                if current["state"] not in {"candidate_ready", "failed"}:
                    raise WorkerError("invalid_state", f"job cannot be repaired from {current['state']}")
                if current["attempt"] != fresh["attempt"] or current["attempt"] >= task.data["max_attempts"]:
                    raise WorkerError("repair_limit", "max_attempts has been reached")
                current.update(
                    state="running",
                    progress="explicit repair started",
                    cancel_requested=False,
                    owner_pid=os.getpid(),
                    owner_identity=process_identity(os.getpid()),
                    backend_state="finished",
                    proposal_hash=None,
                    candidate_hashes={},
                    result_manifest=None,
                    error=None,
                    unresolved=[],
                )
                reset = dict(current)
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
        preparation_error: WorkerError | None = None
        with self.store.locked_status(job_id) as current:
            if current["state"] == "cancel_requested" or current["cancel_requested"]:
                current.update(
                    state="cancelled",
                    progress="cancelled before model request",
                    owner_pid=None,
                    owner_identity=None,
                    backend_state="finished",
                    proposal_hash=None,
                    candidate_hashes={},
                    result_manifest=None,
                    error=None,
                    unresolved=[],
                )
                cancelled = dict(current)
                attempt = None
                remaining = 0.0
            elif current["state"] != "running":
                raise WorkerError("invalid_state", f"attempt cannot start from {current['state']}")
            else:
                remaining = float(task.data["total_timeout_seconds"]) - float(
                    current["generation_seconds_total"]
                )
                if remaining <= 0:
                    preparation_error = WorkerError("total_timeout", "generation time budget is exhausted")
                    current.update(
                        state="failed",
                        progress=preparation_error.message,
                        owner_pid=None,
                        owner_identity=None,
                        backend_state="finished",
                        proposal_hash=None,
                        candidate_hashes={},
                        result_manifest=None,
                        error=preparation_error.as_dict(),
                        unresolved=[preparation_error.message],
                    )
                    cancelled = dict(current)
                    attempt = None
                else:
                    attempt = int(current["attempt"]) + 1
                    attempt_dir = self.store.job_dir(job_id) / f"attempt-{attempt}"
                    try:
                        attempt_dir.mkdir(mode=0o700)
                    except OSError as exc:
                        preparation_error = WorkerError("artifact_write", f"cannot create attempt directory: {exc}")
                        current.update(
                            state="failed",
                            attempt=attempt,
                            progress=preparation_error.message,
                            owner_pid=None,
                            owner_identity=None,
                            backend_state="finished",
                            proposal_hash=None,
                            candidate_hashes={},
                            result_manifest=None,
                            error=preparation_error.as_dict(),
                            unresolved=[preparation_error.message],
                        )
                    else:
                        current.update(
                            state="running",
                            attempt=attempt,
                            progress=f"preparing model request {attempt}",
                            owner_pid=os.getpid(),
                            owner_identity=process_identity(os.getpid()),
                            backend_state="finished",
                            error=None,
                            unresolved=[],
                        )
                    cancelled = dict(current)
        if attempt is None or preparation_error is not None:
            self.store.set_backend("finished", None)
            return cancelled

        attempt_dir = self.store.job_dir(job_id) / f"attempt-{attempt}"
        try:
            context = build_context(task, self.config, previous_diff=previous_diff, diagnostics=diagnostics)
            profile = self.config["profiles"][task.data["model_profile"]]
            payload = request_payload(context, task.data["write_paths"], profile, task.data["max_output_tokens"])
            atomic_json(attempt_dir / "request.json", payload)
        except WorkerError as exc:
            return self._finish_error(job_id, attempt, attempt_dir, exc, "finished", elapsed_seconds=0.0)
        except (OSError, TypeError, ValueError) as exc:
            error = WorkerError("request_preparation_failed", f"cannot prepare model request: {exc}")
            return self._finish_error(job_id, attempt, attempt_dir, error, "finished", elapsed_seconds=0.0)

        with self.store.locked_status(job_id) as current:
            if current["attempt"] != attempt:
                raise WorkerError("attempt_conflict", "job attempt changed during preparation")
            if current["state"] == "cancel_requested" or current["cancel_requested"]:
                atomic_json(
                    attempt_dir / "attempt.json",
                    self._attempt_manifest(attempt, "cancelled", response_path=None, response_raw=None),
                )
                current.update(
                    state="cancelled",
                    progress="cancelled before model request",
                    owner_pid=None,
                    owner_identity=None,
                    backend_state="finished",
                    proposal_hash=None,
                    candidate_hashes={},
                    result_manifest=str(attempt_dir / "attempt.json"),
                    error=None,
                    unresolved=[],
                )
                before_request = dict(current)
            elif current["state"] != "running":
                raise WorkerError("invalid_state", f"request cannot start from {current['state']}")
            else:
                current.update(
                    progress=f"model request {attempt} in progress",
                    backend_state="still_running",
                )
                before_request = dict(current)
        if before_request["state"] == "cancelled":
            self.store.set_backend("finished", None)
            return before_request

        self.store.set_backend("still_running", f"model request {attempt} is active")
        timeout = min(float(task.data["request_timeout_seconds"]), remaining)
        try:
            reply = self.generator(
                payload,
                profile,
                timeout,
                int(self.config["limits"]["max_response_bytes"]),
            )
        except WorkerError as exc:
            elapsed = float(exc.details.get("elapsed_seconds", 0.0))
            response_path = self._save_response(attempt_dir, exc.response_bytes, exc.response_incomplete)
            backend_state = exc.backend_state or (
                "unknown" if exc.code in {"request_timeout", "api_disconnect"} else "finished"
            )
            return self._finish_error(
                job_id,
                attempt,
                attempt_dir,
                exc,
                backend_state,
                elapsed_seconds=elapsed,
                response_path=response_path,
                response_raw=exc.response_bytes,
                response_incomplete=exc.response_incomplete,
            )

        response_path = self._save_response(attempt_dir, reply.raw, reply.over_limit or reply.incomplete)
        backend_state = "unknown" if reply.over_limit or reply.incomplete else "finished"
        with self.store.locked_status(job_id) as current:
            if current["attempt"] != attempt:
                raise WorkerError("attempt_conflict", "job attempt changed while request was active")
            current.update(
                generation_seconds_total=round(
                    float(current["generation_seconds_total"]) + reply.elapsed_seconds, 6
                ),
                progress="model response persisted; validating candidate",
                backend_state=backend_state,
            )
            after_response = dict(current)
        self.store.set_backend(backend_state, None if backend_state == "finished" else "response was incomplete")
        if after_response["state"] == "cancel_requested" or after_response["cancel_requested"]:
            return self._finish_cancelled(
                job_id,
                attempt,
                attempt_dir,
                backend_state,
                response_path,
                reply.raw,
                reply.elapsed_seconds,
                self._usage(reply.parsed),
            )
        if reply.over_limit:
            error = WorkerError(
                "response_too_large",
                f"response exceeded {self.config['limits']['max_response_bytes']} bytes; partial bytes saved",
            )
            return self._finish_error(
                job_id,
                attempt,
                attempt_dir,
                error,
                "unknown",
                elapsed_seconds=reply.elapsed_seconds,
                response_path=response_path,
                response_raw=reply.raw,
                response_incomplete=True,
                elapsed_already_recorded=True,
                usage=self._usage(reply.parsed),
            )
        try:
            operations, usage, returned_model = parse_operations(reply.parsed)
            candidate = build_candidate(operations, task, int(self.config["limits"]["max_file_bytes"]))
        except WorkerError as exc:
            return self._finish_error(
                job_id,
                attempt,
                attempt_dir,
                exc,
                "finished",
                elapsed_seconds=reply.elapsed_seconds,
                response_path=response_path,
                response_raw=reply.raw,
                elapsed_already_recorded=True,
                usage=self._usage(reply.parsed),
            )

        attempt_manifest = {
            **self._attempt_manifest(
                attempt,
                "candidate_ready",
                response_path=response_path,
                response_raw=reply.raw,
                model_seconds=reply.elapsed_seconds,
                usage=usage,
            ),
            "proposal_hash": candidate.proposal_hash,
            "candidate_hashes": candidate.candidate_hashes,
            "requested_model": profile["model"],
            "returned_model": returned_model,
            "runtime": self.config["runtime"],
            "read_paths": task.data["read_paths"],
            "changed_paths": sorted(candidate.files),
            "diff_path": str(attempt_dir / "candidate.diff"),
            "candidate_root": str(attempt_dir / "candidate"),
        }
        try:
            with self.store.locked_status(job_id) as current:
                if current["attempt"] != attempt:
                    raise WorkerError("attempt_conflict", "job attempt changed before candidate publication")
                if current["state"] == "cancel_requested" or current["cancel_requested"]:
                    atomic_json(
                        attempt_dir / "attempt.json",
                        {
                            **self._attempt_manifest(
                                attempt,
                                "cancelled",
                                response_path=response_path,
                                response_raw=reply.raw,
                                model_seconds=reply.elapsed_seconds,
                                usage=usage,
                            ),
                            "late_response_discarded": True,
                        },
                    )
                    current.update(
                        state="cancelled",
                        progress="cancellation won candidate publication; response retained",
                        owner_pid=None,
                        owner_identity=None,
                        backend_state="finished",
                        proposal_hash=None,
                        candidate_hashes={},
                        result_manifest=str(attempt_dir / "attempt.json"),
                        error=None,
                        unresolved=[],
                    )
                elif current["state"] != "running":
                    raise WorkerError("invalid_state", f"candidate cannot publish from {current['state']}")
                else:
                    write_candidate(attempt_dir, candidate)
                    atomic_json(attempt_dir / "attempt.json", attempt_manifest)
                    current.update(
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
                published = dict(current)
        except OSError as exc:
            error = WorkerError("artifact_write", f"cannot persist candidate artifacts: {exc}")
            return self._finish_error(
                job_id,
                attempt,
                attempt_dir,
                error,
                "finished",
                elapsed_seconds=reply.elapsed_seconds,
                response_path=response_path,
                response_raw=reply.raw,
                elapsed_already_recorded=True,
                usage=usage,
            )
        return published

    @staticmethod
    def _usage(parsed: dict[str, Any] | None) -> Any:
        return parsed.get("usage") if isinstance(parsed, dict) and isinstance(parsed.get("usage"), dict) else None

    @staticmethod
    def _usage_from_raw(raw: bytes | None) -> Any:
        if raw is None:
            return None
        try:
            parsed = json.loads(raw.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError):
            return None
        return parsed.get("usage") if isinstance(parsed, dict) and isinstance(parsed.get("usage"), dict) else None

    @staticmethod
    def _save_response(attempt_dir: Path, raw: bytes | None, incomplete: bool) -> str | None:
        if raw is None:
            return None
        path = attempt_dir / ("response.partial.json" if incomplete else "response.raw.json")
        atomic_write(path, raw)
        return str(path)

    @staticmethod
    def _attempt_manifest(
        attempt: int,
        state: str,
        *,
        response_path: str | None,
        response_raw: bytes | None,
        response_incomplete: bool = False,
        model_seconds: float = 0.0,
        usage: Any = None,
    ) -> dict[str, Any]:
        return {
            "attempt": attempt,
            "state": state,
            "response_path": response_path,
            "response_sha256": response_hash(response_raw) if response_raw is not None else None,
            "response_incomplete": response_incomplete,
            "model_seconds": round(model_seconds, 6),
            "usage": usage,
        }

    def _finish_error(
        self,
        job_id: str,
        attempt: int,
        attempt_dir: Path,
        error: WorkerError,
        backend_state: str,
        *,
        elapsed_seconds: float,
        response_path: str | None = None,
        response_raw: bytes | None = None,
        response_incomplete: bool = False,
        elapsed_already_recorded: bool = False,
        usage: Any = None,
    ) -> dict[str, Any]:
        usage = self._usage_from_raw(response_raw) if usage is None else usage
        with self.store.locked_status(job_id) as current:
            if current["attempt"] != attempt:
                raise WorkerError("attempt_conflict", "job attempt changed before failure publication")
            cancelled = current["state"] == "cancel_requested" or current["cancel_requested"]
            manifest = {
                **self._attempt_manifest(
                    attempt,
                    "cancelled" if cancelled else "failed",
                    response_path=response_path,
                    response_raw=response_raw,
                    response_incomplete=response_incomplete,
                    model_seconds=elapsed_seconds,
                    usage=usage,
                ),
                "error": error.as_dict(),
            }
            if cancelled:
                manifest["late_response_discarded"] = response_raw is not None
            atomic_json(attempt_dir / "attempt.json", manifest)
            if not elapsed_already_recorded:
                current["generation_seconds_total"] = round(
                    float(current["generation_seconds_total"]) + elapsed_seconds, 6
                )
            current.update(
                state="cancelled" if cancelled else "failed",
                progress=("cancellation accepted; response retained but not accepted" if cancelled else error.message),
                owner_pid=None,
                owner_identity=None,
                backend_state=backend_state,
                error=None if cancelled else error.as_dict(),
                unresolved=([] if cancelled and backend_state == "finished" else [error.message]),
                result_manifest=str(attempt_dir / "attempt.json"),
                proposal_hash=None,
                candidate_hashes={},
            )
            finished = dict(current)
        self.store.set_backend(backend_state, None if backend_state == "finished" else error.code)
        return finished

    def _finish_cancelled(
        self,
        job_id: str,
        attempt: int,
        attempt_dir: Path,
        backend_state: str,
        response_path: str | None,
        response_raw: bytes,
        elapsed_seconds: float,
        usage: Any,
    ) -> dict[str, Any]:
        with self.store.locked_status(job_id) as current:
            atomic_json(
                attempt_dir / "attempt.json",
                {
                    **self._attempt_manifest(
                        attempt,
                        "cancelled",
                        response_path=response_path,
                        response_raw=response_raw,
                        response_incomplete=backend_state != "finished",
                        model_seconds=elapsed_seconds,
                        usage=usage,
                    ),
                    "late_response_discarded": True,
                },
            )
            current.update(
                state="cancelled",
                progress="late model response was persisted but not accepted",
                owner_pid=None,
                owner_identity=None,
                backend_state=backend_state,
                error=None,
                unresolved=[] if backend_state == "finished" else ["backend completion remains unconfirmed"],
                result_manifest=str(attempt_dir / "attempt.json"),
                proposal_hash=None,
                candidate_hashes={},
            )
            cancelled = dict(current)
        self.store.set_backend(backend_state, None if backend_state == "finished" else "cancelled response incomplete")
        return cancelled

    def cancel(self, job_id: str) -> dict[str, Any]:
        with self.store.locked_status(job_id) as status:
            if status["state"] != "running":
                return dict(status)
            backend_state = status["backend_state"]
            status.update(
                state="cancel_requested",
                progress=(
                    "cancellation accepted; waiting for bounded request to return"
                    if backend_state == "still_running"
                    else "cancellation accepted before candidate publication"
                ),
                cancel_requested=True,
            )
            updated = dict(status)
        self.store.set_backend(
            backend_state,
            "cancel requested while bounded request is active" if backend_state != "finished" else None,
        )
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
        if status["state"] != "candidate_ready":
            raise WorkerError("invalid_state", f"job has no acceptable candidate in state {status['state']}")
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
