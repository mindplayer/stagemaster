from __future__ import annotations

import json
import socket
import time
from dataclasses import dataclass
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from .errors import WorkerError


@dataclass(frozen=True)
class ModelReply:
    raw: bytes
    parsed: dict[str, Any]
    elapsed_seconds: float
    over_limit: bool = False


def request_payload(context: str, write_paths: list[str], profile: dict[str, Any], max_tokens: int) -> dict[str, Any]:
    operation_schema = {
        "oneOf": [
            {
                "type": "object",
                "properties": {
                    "kind": {"type": "string", "enum": ["create"]},
                    "path": {"type": "string", "enum": write_paths},
                    "content": {"type": "string"},
                },
                "required": ["kind", "path", "content"],
                "additionalProperties": False,
            },
            {
                "type": "object",
                "properties": {
                    "kind": {"type": "string", "enum": ["replace"]},
                    "path": {"type": "string", "enum": write_paths},
                    "old_text": {"type": "string", "minLength": 1},
                    "new_text": {"type": "string"},
                },
                "required": ["kind", "path", "old_text", "new_text"],
                "additionalProperties": False,
            },
        ]
    }
    return {
        "model": profile["model"],
        "messages": [
            {
                "role": "system",
                "content": "Return one bounded text-edit proposal through the provided tool. Do not narrate.",
            },
            {"role": "user", "content": context},
        ],
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "propose_changes",
                    "description": "Propose atomic create and exact-text replacement operations.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "operations": {"type": "array", "minItems": 1, "items": operation_schema}
                        },
                        "required": ["operations"],
                        "additionalProperties": False,
                    },
                },
            }
        ],
        "tool_choice": {"type": "function", "function": {"name": "propose_changes"}},
        "temperature": profile["temperature"],
        "max_tokens": max_tokens,
        "stream": False,
        "chat_template_kwargs": {"enable_thinking": profile["enable_thinking"]},
    }


def generate(payload: dict[str, Any], profile: dict[str, Any], timeout: float, max_response_bytes: int) -> ModelReply:
    url = profile["api_base"].rstrip("/") + "/chat/completions"
    request = Request(url, data=json.dumps(payload, ensure_ascii=False).encode(), headers={"Content-Type": "application/json"})
    started = time.monotonic()
    try:
        with urlopen(request, timeout=timeout) as response:
            raw = response.read(max_response_bytes + 1)
    except HTTPError as exc:
        body = exc.read(65536).decode(errors="replace")
        raise WorkerError("api_http_error", f"HTTP {exc.code}: {body}") from exc
    except (URLError, TimeoutError, socket.timeout, OSError) as exc:
        code = "request_timeout" if isinstance(exc, (TimeoutError, socket.timeout)) else "api_disconnect"
        raise WorkerError(code, str(exc)) from exc
    elapsed = time.monotonic() - started
    if len(raw) > max_response_bytes:
        return ModelReply(raw=raw, parsed={}, elapsed_seconds=elapsed, over_limit=True)
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise WorkerError("malformed_response", f"response is not JSON: {exc}") from exc
    if not isinstance(parsed, dict):
        raise WorkerError("malformed_response", "response must be a JSON object")
    return ModelReply(raw=raw, parsed=parsed, elapsed_seconds=elapsed)
