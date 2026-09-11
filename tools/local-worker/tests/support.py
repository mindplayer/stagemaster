from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

from local_worker.model import ModelReply


def git(root: Path, *arguments: str) -> str:
    return subprocess.run(
        ["git", "-C", str(root), *arguments],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()


def make_repo(case: Path) -> tuple[Path, str]:
    root = case / "repo"
    (root / "src").mkdir(parents=True)
    (root / "tests").mkdir()
    (root / "src" / "lib.rs").write_text("pub fn value() -> u8 { 1 }\n", encoding="utf-8")
    (root / "tests" / "acceptance.rs").write_text("// protected acceptance\n", encoding="utf-8")
    git(root, "init", "-b", "main")
    git(root, "config", "user.name", "Local Worker Test")
    git(root, "config", "user.email", "local-worker@example.invalid")
    git(root, "add", "--all")
    git(root, "commit", "-m", "acceptance base")
    return root, git(root, "rev-parse", "HEAD")


def config(case: Path) -> dict[str, Any]:
    ai_root = case / "ai"
    ai_root.mkdir()
    return {
        "schema_version": 1,
        "ai_root": str(ai_root),
        "profiles": {
            "test": {
                "api_base": "http://127.0.0.1:18100/v1",
                "model": "default_model",
                "temperature": 0,
                "enable_thinking": False,
            }
        },
        "limits": {
            "max_context_bytes": 65536,
            "max_response_bytes": 1048576,
            "max_paths": 6,
            "max_file_bytes": 1048576,
            "max_output_tokens": 4096,
            "max_attempts": 3,
            "max_request_timeout_seconds": 360,
            "max_total_timeout_seconds": 900,
            "max_diagnostics_bytes": 24000,
        },
        "runtime": {"name": "synthetic", "framework": "unittest"},
        "_path": str(case / "config.json"),
        "_output_root": str(ai_root / "outputs" / "local-worker"),
    }


def task(root: Path, commit: str, *, task_id: str = "TEST-001") -> dict[str, Any]:
    return {
        "schema_version": 1,
        "task_id": task_id,
        "contract_revision": "1",
        "workspace_root": str(root),
        "base_commit": commit,
        "read_paths": ["src/lib.rs", "tests/acceptance.rs"],
        "write_paths": ["src/lib.rs"],
        "protected_paths": ["tests/acceptance.rs"],
        "goal": "Change value to two.",
        "contract": "Keep the function signature.",
        "acceptance": "Protected tests remain unchanged.",
        "model_profile": "test",
        "max_output_tokens": 4096,
        "max_attempts": 3,
        "request_timeout_seconds": 360,
        "total_timeout_seconds": 900,
    }


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value), encoding="utf-8")


def response(operations: list[dict[str, str]], *, finish_reason: str = "tool_calls", content: str | None = None) -> dict[str, Any]:
    return {
        "model": "synthetic-model",
        "choices": [
            {
                "finish_reason": finish_reason,
                "message": {
                    "content": content,
                    "tool_calls": [
                        {
                            "id": "call-1",
                            "type": "function",
                            "function": {
                                "name": "propose_changes",
                                "arguments": json.dumps({"operations": operations}),
                            },
                        }
                    ],
                },
            }
        ],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
    }


def reply(operations: list[dict[str, str]], **kwargs: Any) -> ModelReply:
    parsed = response(operations, **kwargs)
    raw = json.dumps(parsed).encode()
    return ModelReply(raw=raw, parsed=parsed, elapsed_seconds=0.01)
