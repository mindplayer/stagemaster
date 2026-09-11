from __future__ import annotations

import os
from pathlib import Path
from typing import Any

from .errors import WorkerError
from .task import ValidatedTask, task_as_json
from .util import TASK_ID_RE, atomic_json, canonical_json, exclusive_lock, process_identity, read_json, sha256_bytes, utc_now


TERMINAL_STATES = {"candidate_ready", "failed", "cancelled", "interrupted"}


class Store:
    def __init__(self, config: dict[str, Any]):
        self.config = config
        self.root = Path(config["_output_root"])
        self.root.mkdir(parents=True, exist_ok=True, mode=0o700)

    def job_dir(self, job_id: str) -> Path:
        if not TASK_ID_RE.fullmatch(job_id):
            raise WorkerError("invalid_job_id", f"invalid job id: {job_id}")
        return self.root / job_id

    def load(self, job_id: str) -> dict[str, Any]:
        path = self.job_dir(job_id) / "status.json"
        if not path.is_file():
            raise WorkerError("unknown_job", f"unknown job: {job_id}")
        return read_json(path)

    def update(self, job_id: str, **changes: Any) -> dict[str, Any]:
        directory = self.job_dir(job_id)
        with exclusive_lock(directory / ".lock", blocking=True):
            status = read_json(directory / "status.json")
            status.update(changes)
            status["updated_at"] = utc_now()
            atomic_json(directory / "status.json", status)
        return status

    def _lookup_unlocked(self, task: ValidatedTask) -> dict[str, Any] | None:
        for directory in self.root.iterdir():
            status_path = directory / "status.json"
            if not directory.is_dir() or not status_path.is_file():
                continue
            status = read_json(status_path)
            if status.get("task_id") != task.data["task_id"]:
                continue
            identity = (
                status.get("base_commit"),
                status.get("contract_revision"),
                status.get("input_digest"),
            )
            expected = (task.data["base_commit"], task.data["contract_revision"], task.input_digest)
            if identity == expected:
                return status
            raise WorkerError(
                "identity_conflict",
                "task_id was reused with different base, contract, or input",
                {"existing_job_id": status.get("job_id")},
            )
        return None

    def lookup(self, task: ValidatedTask) -> dict[str, Any] | None:
        with exclusive_lock(self.root / ".registry.lock", blocking=True):
            return self._lookup_unlocked(task)

    def create(self, task: ValidatedTask) -> tuple[dict[str, Any], bool]:
        with exclusive_lock(self.root / ".registry.lock", blocking=True):
            existing = self._lookup_unlocked(task)
            if existing is not None:
                return existing, False
            job_id = f"{task.data['task_id']}-{task.input_digest[:12]}"
            directory = self.job_dir(job_id)
            directory.mkdir(mode=0o700)
            now = utc_now()
            status = {
                "schema_version": 1,
                "job_id": job_id,
                "task_id": task.data["task_id"],
                "contract_revision": task.data["contract_revision"],
                "base_commit": task.data["base_commit"],
                "input_digest": task.input_digest,
                "state": "running",
                "attempt": 0,
                "progress": "record created",
                "created_at": now,
                "updated_at": now,
                "owner_pid": os.getpid(),
                "owner_identity": process_identity(os.getpid()),
                "cancel_requested": False,
                "backend_state": "unknown",
                "generation_seconds_total": 0.0,
                "error": None,
                "unresolved": [],
            }
            atomic_json(directory / "task.json", task_as_json(task))
            atomic_json(directory / "status.json", status)
            return status, True

    def backend(self) -> dict[str, Any]:
        path = self.root / "backend.json"
        return read_json(path) if path.exists() else {"state": "finished", "updated_at": None, "reason": None}

    def set_backend(self, state: str, reason: str | None) -> None:
        if state not in {"finished", "still_running", "unknown"}:
            raise WorkerError("invalid_backend_state", f"unsupported backend state: {state}")
        atomic_json(self.root / "backend.json", {"state": state, "updated_at": utc_now(), "reason": reason})

    def ensure_backend_available(self) -> None:
        backend = self.backend()
        if backend["state"] != "finished":
            raise WorkerError("backend_unresolved", "previous backend activity is not confirmed finished", backend)


def response_hash(response: bytes) -> str:
    return sha256_bytes(response)


def manifest_hash(value: dict[str, Any]) -> str:
    return sha256_bytes(canonical_json(value))
