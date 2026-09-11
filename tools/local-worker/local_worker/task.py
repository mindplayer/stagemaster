from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

from .errors import WorkerError
from .util import (
    FULL_COMMIT_RE,
    MISSING_HASH,
    TASK_ID_RE,
    canonical_json,
    normalize_relative_path,
    read_json,
    run_git,
    safe_path,
    sha256_bytes,
    sha256_file,
)


TASK_FIELDS = {
    "schema_version",
    "task_id",
    "contract_revision",
    "workspace_root",
    "base_commit",
    "read_paths",
    "write_paths",
    "protected_paths",
    "goal",
    "contract",
    "acceptance",
    "model_profile",
    "max_output_tokens",
    "max_attempts",
    "request_timeout_seconds",
    "total_timeout_seconds",
}


@dataclass(frozen=True)
class ValidatedTask:
    data: dict[str, Any]
    workspace: Path
    source_hashes: dict[str, str]
    protected_hashes: dict[str, str]
    input_digest: str


def load_config(path: Path) -> dict[str, Any]:
    config = read_json(path.resolve(strict=True))
    expected = {"schema_version", "ai_root", "profiles", "limits", "runtime"}
    if set(config) != expected or config.get("schema_version") != 1:
        raise WorkerError("invalid_config", "config fields or schema_version are invalid")
    ai_root = Path(config["ai_root"])
    if not ai_root.is_absolute() or not ai_root.resolve(strict=True).is_dir():
        raise WorkerError("invalid_config", "ai_root must be an existing absolute directory")
    profiles = config["profiles"]
    if not isinstance(profiles, dict) or not profiles:
        raise WorkerError("invalid_config", "profiles must be a non-empty object")
    for name, profile in profiles.items():
        if not isinstance(name, str) or not isinstance(profile, dict):
            raise WorkerError("invalid_config", "model profile is invalid")
        if set(profile) != {"api_base", "model", "temperature", "enable_thinking"}:
            raise WorkerError("invalid_config", f"model profile fields are invalid: {name}")
        parsed = urlsplit(profile["api_base"])
        if parsed.scheme != "http" or parsed.hostname != "127.0.0.1" or parsed.path.rstrip("/") != "/v1":
            raise WorkerError("invalid_config", "v1 only permits a 127.0.0.1 HTTP /v1 endpoint")
    limits = config["limits"]
    if set(limits) != {"max_context_bytes", "max_response_bytes", "max_paths", "max_file_bytes", "max_output_tokens", "max_attempts", "max_request_timeout_seconds", "max_total_timeout_seconds", "max_diagnostics_bytes"}:
        raise WorkerError("invalid_config", "config limits fields are invalid")
    config["_path"] = str(path.resolve(strict=True))
    config["_output_root"] = str(ai_root / "outputs" / "local-worker")
    return config


def _string(task: dict[str, Any], name: str) -> str:
    value = task[name]
    if not isinstance(value, str) or not value.strip():
        raise WorkerError("invalid_task", f"{name} must be a non-empty string")
    return value


def _paths(task: dict[str, Any], name: str, maximum: int, *, required: bool) -> list[str]:
    value = task[name]
    if not isinstance(value, list) or (required and not value) or len(value) > maximum:
        raise WorkerError("invalid_task", f"{name} must contain {'1..' if required else '0..'}{maximum} paths")
    normalized = [normalize_relative_path(item) for item in value]
    if len(normalized) != len(set(normalized)):
        raise WorkerError("invalid_task", f"{name} contains duplicate paths")
    return normalized


def validate_task(raw: dict[str, Any], config: dict[str, Any]) -> ValidatedTask:
    if set(raw) != TASK_FIELDS:
        raise WorkerError(
            "invalid_task",
            "task contains missing or unknown fields",
            {"missing": sorted(TASK_FIELDS - raw.keys()), "unknown": sorted(raw.keys() - TASK_FIELDS)},
        )
    if raw["schema_version"] != 1:
        raise WorkerError("invalid_task", "schema_version must be integer 1")
    task_id = _string(raw, "task_id")
    if not TASK_ID_RE.fullmatch(task_id):
        raise WorkerError("invalid_task", "task_id contains unsupported characters")
    for name in ("contract_revision", "goal", "contract", "acceptance", "model_profile"):
        _string(raw, name)

    workspace_input = Path(raw["workspace_root"])
    if not workspace_input.is_absolute() or workspace_input.is_symlink():
        raise WorkerError("invalid_workspace", "workspace_root must be absolute and not a symlink")
    workspace = workspace_input.resolve(strict=True)
    top = Path(run_git(workspace, ["rev-parse", "--show-toplevel"]).decode().strip()).resolve(strict=True)
    if top != workspace:
        raise WorkerError("invalid_workspace", "workspace_root must be a Git worktree root")
    base_commit = raw["base_commit"]
    if not isinstance(base_commit, str) or not FULL_COMMIT_RE.fullmatch(base_commit):
        raise WorkerError("invalid_base", "base_commit must be a full lowercase commit id")
    head = run_git(workspace, ["rev-parse", "HEAD"]).decode().strip()
    if head != base_commit:
        raise WorkerError("stale_base", "worktree HEAD does not equal base_commit", {"head": head, "base": base_commit})
    if run_git(workspace, ["status", "--porcelain"]).strip():
        raise WorkerError("dirty_workspace", "task worktree must be clean at the acceptance commit")

    max_paths = int(config["limits"]["max_paths"])
    read_paths = _paths(raw, "read_paths", max_paths, required=False)
    write_paths = _paths(raw, "write_paths", max_paths, required=True)
    protected_paths = _paths(raw, "protected_paths", max_paths, required=False)
    overlap = sorted(set(write_paths) & set(protected_paths))
    if overlap:
        raise WorkerError("protected_path", "protected paths take priority over writes", {"paths": overlap})

    source_hashes: dict[str, str] = {}
    protected_hashes: dict[str, str] = {}
    max_file_bytes = int(config["limits"]["max_file_bytes"])
    for relative in sorted(set(read_paths + protected_paths)):
        path = safe_path(workspace, relative, must_exist=True)
        if not path.is_file():
            raise WorkerError("invalid_path", f"task path is not a regular file: {relative}")
        if path.stat().st_size > max_file_bytes:
            raise WorkerError("file_too_large", f"task file exceeds byte limit: {relative}")
        try:
            path.read_text(encoding="utf-8")
        except UnicodeDecodeError as exc:
            raise WorkerError("binary_file", f"only UTF-8 files are supported: {relative}") from exc
        digest = sha256_file(path)
        source_hashes[relative] = digest
        if relative in protected_paths:
            protected_hashes[relative] = digest
    for relative in write_paths:
        path = safe_path(workspace, relative, must_exist=False)
        if path.exists():
            if relative not in read_paths:
                raise WorkerError("write_not_readable", f"existing write path must also be listed in read_paths: {relative}")
            source_hashes[relative] = sha256_file(path)
        else:
            source_hashes[relative] = MISSING_HASH
            if not path.parent.is_dir():
                raise WorkerError("missing_path", f"new file parent directory does not exist: {relative}")

    limits = config["limits"]
    bounded = {
        "max_output_tokens": (1, int(limits["max_output_tokens"])),
        "max_attempts": (1, int(limits["max_attempts"])),
        "request_timeout_seconds": (1, int(limits["max_request_timeout_seconds"])),
        "total_timeout_seconds": (1, int(limits["max_total_timeout_seconds"])),
    }
    for name, (minimum, maximum) in bounded.items():
        value = raw[name]
        if not isinstance(value, int) or isinstance(value, bool) or not minimum <= value <= maximum:
            raise WorkerError("invalid_limit", f"{name} must be between {minimum} and {maximum}")
    if raw["request_timeout_seconds"] > raw["total_timeout_seconds"]:
        raise WorkerError("invalid_limit", "request timeout cannot exceed total timeout")
    if raw["model_profile"] not in config["profiles"]:
        raise WorkerError("invalid_model_profile", f"unknown model profile: {raw['model_profile']}")

    normalized = dict(raw)
    normalized.update(
        workspace_root=str(workspace),
        read_paths=read_paths,
        write_paths=write_paths,
        protected_paths=protected_paths,
    )
    digest_data = {
        "task": normalized,
        "source_hashes": source_hashes,
        "protected_hashes": protected_hashes,
    }
    return ValidatedTask(
        data=normalized,
        workspace=workspace,
        source_hashes=source_hashes,
        protected_hashes=protected_hashes,
        input_digest=sha256_bytes(canonical_json(digest_data)),
    )


def validate_task_file(path: Path, config: dict[str, Any]) -> ValidatedTask:
    return validate_task(read_json(path), config)


def build_context(
    task: ValidatedTask,
    config: dict[str, Any],
    *,
    previous_diff: str | None = None,
    diagnostics: str | None = None,
) -> str:
    data = task.data
    sections = [
        "You are a bounded code-editing worker. Return exactly one propose_changes tool call and no shell commands.",
        f"TASK {data['task_id']} CONTRACT REVISION {data['contract_revision']}",
        f"GOAL\n{data['goal']}",
        f"CONTRACT\n{data['contract']}",
        f"ACCEPTANCE\n{data['acceptance']}",
        "WRITE PATHS\n" + "\n".join(f"- {path}" for path in data["write_paths"]),
        "PROTECTED PATHS\n" + ("\n".join(f"- {path}" for path in data["protected_paths"]) or "(none)"),
    ]
    for relative in dict.fromkeys(data["read_paths"] + data["write_paths"]):
        path = safe_path(task.workspace, relative, must_exist=False)
        if not path.exists():
            sections.append(f"FILE {relative} (new; absent at base)")
        else:
            content = path.read_text(encoding="utf-8")
            sections.append(f"FILE {relative} sha256={task.source_hashes[relative]}\n```text\n{content}\n```")
    if previous_diff is not None:
        sections.append(
            "PREVIOUS CANDIDATE DIFF (do not edit it incrementally; return operations against the original base)\n"
            + previous_diff
        )
    if diagnostics is not None:
        sections.append("HOST DIAGNOSTICS\n" + diagnostics)
    context = "\n\n".join(sections)
    size = len(context.encode())
    maximum = int(config["limits"]["max_context_bytes"])
    if size > maximum:
        raise WorkerError("context_too_large", f"context is {size} bytes; configured maximum is {maximum}")
    return context


def task_as_json(task: ValidatedTask) -> dict[str, Any]:
    return {
        "task": task.data,
        "source_hashes": task.source_hashes,
        "protected_hashes": task.protected_hashes,
        "input_digest": task.input_digest,
    }
