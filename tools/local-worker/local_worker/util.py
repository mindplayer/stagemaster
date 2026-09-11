from __future__ import annotations

import contextlib
import fcntl
import hashlib
import json
import os
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any, Iterator

from .errors import WorkerError


MISSING_HASH = "missing"
TASK_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
JOB_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,159}$")
FULL_COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds")


def canonical_json(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    if not path.exists():
        return MISSING_HASH
    if path.is_symlink() or not path.is_file():
        raise WorkerError("unsafe_path", f"expected a regular file: {path}")
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise WorkerError("invalid_json", f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise WorkerError("invalid_json", f"expected a JSON object: {path}")
    return value


def atomic_write(path: Path, data: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        with contextlib.suppress(FileNotFoundError):
            temporary.unlink()


def atomic_json(path: Path, value: Any) -> None:
    data = json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True).encode() + b"\n"
    atomic_write(path, data)


@contextlib.contextmanager
def exclusive_lock(path: Path, *, blocking: bool) -> Iterator[None]:
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with path.open("a+b") as handle:
        flags = fcntl.LOCK_EX | (0 if blocking else fcntl.LOCK_NB)
        try:
            fcntl.flock(handle.fileno(), flags)
        except BlockingIOError as exc:
            raise WorkerError("busy", "another local generation is active") from exc
        try:
            yield
        finally:
            fcntl.flock(handle.fileno(), fcntl.LOCK_UN)


def normalize_relative_path(raw: Any) -> str:
    if not isinstance(raw, str) or not raw or "\x00" in raw or "\\" in raw:
        raise WorkerError("invalid_path", f"invalid relative path: {raw!r}")
    path = PurePosixPath(raw)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        raise WorkerError("invalid_path", f"absolute or traversing path rejected: {raw}")
    normalized = path.as_posix()
    if normalized == ".git" or normalized.startswith(".git/"):
        raise WorkerError("protected_path", "Git metadata is never task-visible")
    return normalized


def safe_path(root: Path, relative: str, *, must_exist: bool) -> Path:
    resolved_root = root.resolve(strict=True)
    current = resolved_root
    parts = PurePosixPath(relative).parts
    for index, part in enumerate(parts):
        current = current / part
        if current.is_symlink():
            raise WorkerError("symlink_path", f"symlink component rejected: {relative}")
        if index < len(parts) - 1 and not current.is_dir():
            raise WorkerError("missing_path", f"missing directory in path: {relative}")
    if must_exist and not current.exists():
        raise WorkerError("missing_path", f"required file is missing: {relative}")
    try:
        current.resolve(strict=False).relative_to(resolved_root)
    except ValueError as exc:
        raise WorkerError("path_escape", f"path escapes workspace: {relative}") from exc
    return current


def run_git(root: Path, arguments: list[str], *, allow_failure: bool = False) -> bytes:
    completed = subprocess.run(["git", "-C", str(root), *arguments], capture_output=True, check=False)
    if completed.returncode and not allow_failure:
        message = completed.stderr.decode(errors="replace").strip()
        raise WorkerError("git_error", message or f"git {' '.join(arguments)} failed")
    return completed.stdout


def process_identity(pid: int) -> str | None:
    if pid <= 0:
        return None
    completed = subprocess.run(
        ["ps", "-p", str(pid), "-o", "pid=,lstart=,command="],
        capture_output=True,
        text=True,
        check=False,
    )
    identity = " ".join(completed.stdout.split())
    return identity or None
