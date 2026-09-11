from __future__ import annotations

import difflib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .errors import WorkerError
from .task import ValidatedTask
from .util import MISSING_HASH, atomic_json, atomic_write, canonical_json, safe_path, sha256_bytes, sha256_file


@dataclass(frozen=True)
class Candidate:
    operations: list[dict[str, str]]
    files: dict[str, bytes]
    diff: str
    proposal_hash: str
    candidate_hashes: dict[str, str]


def parse_operations(response: dict[str, Any]) -> tuple[list[dict[str, str]], Any, str | None]:
    choices = response.get("choices")
    if not isinstance(choices, list) or len(choices) != 1 or not isinstance(choices[0], dict):
        raise WorkerError("malformed_response", "expected exactly one response choice")
    choice = choices[0]
    if choice.get("finish_reason") == "length":
        raise WorkerError("output_truncated", "model output reached max_output_tokens")
    message = choice.get("message")
    if not isinstance(message, dict):
        raise WorkerError("malformed_response", "choice has no message object")
    if message.get("content") not in {None, ""}:
        raise WorkerError("malformed_response", "model narrated instead of returning only a proposal")
    calls = message.get("tool_calls")
    if not isinstance(calls, list) or len(calls) != 1 or not isinstance(calls[0], dict):
        raise WorkerError("extra_or_missing_tool", "expected exactly one propose_changes tool call")
    function = calls[0].get("function")
    if not isinstance(function, dict) or function.get("name") != "propose_changes":
        raise WorkerError("unexpected_tool", "model called an unsupported tool")
    arguments = function.get("arguments")
    try:
        decoded = json.loads(arguments) if isinstance(arguments, str) else arguments
    except json.JSONDecodeError as exc:
        raise WorkerError("malformed_proposal", f"tool arguments are invalid JSON: {exc}") from exc
    if not isinstance(decoded, dict) or set(decoded) != {"operations"}:
        raise WorkerError("malformed_proposal", "proposal must contain only operations")
    operations = decoded["operations"]
    if not isinstance(operations, list) or not operations:
        raise WorkerError("empty_proposal", "proposal operations must be non-empty")
    normalized: list[dict[str, str]] = []
    for operation in operations:
        if not isinstance(operation, dict):
            raise WorkerError("malformed_proposal", "each operation must be an object")
        kind = operation.get("kind")
        expected = {"kind", "path", "content"} if kind == "create" else {"kind", "path", "old_text", "new_text"}
        if kind not in {"create", "replace"} or set(operation) != expected:
            raise WorkerError("malformed_proposal", f"invalid {kind!r} operation fields")
        if not all(isinstance(value, str) for value in operation.values()):
            raise WorkerError("malformed_proposal", "operation fields must be strings")
        normalized.append(dict(operation))
    usage = response.get("usage") if isinstance(response.get("usage"), dict) else None
    returned_model = response.get("model") if isinstance(response.get("model"), str) else None
    return normalized, usage, returned_model


def verify_source(task: ValidatedTask) -> None:
    mismatches: list[dict[str, str]] = []
    for relative, expected in task.source_hashes.items():
        path = safe_path(task.workspace, relative, must_exist=expected != MISSING_HASH)
        actual = sha256_file(path)
        if actual != expected:
            mismatches.append({"path": relative, "expected": expected, "actual": actual})
    if mismatches:
        raise WorkerError("stale_source", "task source changed after validation", {"mismatches": mismatches})


def build_candidate(operations: list[dict[str, str]], task: ValidatedTask, max_file_bytes: int) -> Candidate:
    verify_source(task)
    allowed = set(task.data["write_paths"])
    contents: dict[str, str | None] = {}
    originals: dict[str, str | None] = {}
    for relative in allowed:
        path = safe_path(task.workspace, relative, must_exist=False)
        value = path.read_text(encoding="utf-8") if path.exists() else None
        contents[relative] = value
        originals[relative] = value

    for operation in operations:
        relative = operation["path"]
        if relative not in allowed:
            raise WorkerError("scope_violation", f"proposal path is not writable: {relative}")
        kind = operation["kind"]
        current = contents[relative]
        if kind == "create":
            if originals[relative] is not None or current is not None:
                raise WorkerError("create_existing", f"create target already exists: {relative}")
            current = operation["content"]
        else:
            if current is None:
                raise WorkerError("replace_missing", f"replace target does not exist: {relative}")
            old_text = operation["old_text"]
            if not old_text:
                raise WorkerError("empty_match", f"replace match must not be empty: {relative}")
            count = current.count(old_text)
            if count != 1:
                raise WorkerError(
                    "ambiguous_match",
                    f"replace text must occur exactly once: {relative}",
                    {"matches": count},
                )
            current = current.replace(old_text, operation["new_text"], 1)
        if len(current.encode()) > max_file_bytes:
            raise WorkerError("candidate_too_large", f"candidate file exceeds byte limit: {relative}")
        contents[relative] = current

    changed: dict[str, bytes] = {}
    candidate_hashes: dict[str, str] = {}
    patch_lines: list[str] = []
    for relative in task.data["write_paths"]:
        before = originals[relative]
        after = contents[relative]
        if after == before:
            continue
        if after is None:
            raise WorkerError("unsupported_delete", "v1 does not support deletion")
        encoded = after.encode()
        changed[relative] = encoded
        candidate_hashes[relative] = sha256_bytes(encoded)
        before_lines = [] if before is None else before.splitlines()
        after_lines = after.splitlines()
        patch_lines.extend(
            difflib.unified_diff(
                before_lines,
                after_lines,
                fromfile="/dev/null" if before is None else f"a/{relative}",
                tofile=f"b/{relative}",
                lineterm="",
            )
        )
    if not changed:
        raise WorkerError("empty_change", "proposal produces no file changes")
    diff = "\n".join(patch_lines) + "\n"
    return Candidate(
        operations=operations,
        files=changed,
        diff=diff,
        proposal_hash=sha256_bytes(canonical_json(operations)),
        candidate_hashes=candidate_hashes,
    )


def write_candidate(attempt_dir: Path, candidate: Candidate) -> None:
    candidate_root = attempt_dir / "candidate"
    if candidate_root.exists():
        raise WorkerError("artifact_conflict", f"candidate directory already exists: {candidate_root}")
    candidate_root.mkdir(parents=True, mode=0o700)
    for relative, content in candidate.files.items():
        target = candidate_root.joinpath(*Path(relative).parts)
        target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        target.resolve(strict=False).relative_to(candidate_root.resolve(strict=True))
        atomic_write(target, content)
    atomic_write(attempt_dir / "candidate.diff", candidate.diff.encode())
    atomic_json(
        attempt_dir / "proposal.json",
        {
            "operations": candidate.operations,
            "proposal_hash": candidate.proposal_hash,
            "candidate_hashes": candidate.candidate_hashes,
        },
    )
