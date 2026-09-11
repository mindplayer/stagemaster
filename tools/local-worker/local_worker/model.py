from __future__ import annotations

import contextlib
import http.client
import json
import socket
import threading
import time
from dataclasses import dataclass
from typing import Any
from urllib.parse import urlsplit

from .errors import WorkerError


@dataclass(frozen=True)
class ModelReply:
    raw: bytes
    parsed: dict[str, Any]
    elapsed_seconds: float
    over_limit: bool = False
    incomplete: bool = False


@dataclass(frozen=True)
class TransportResponse:
    status: int
    reason: str
    raw: bytes
    over_limit: bool = False
    incomplete: bool = False


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


def _close_connection(connection: http.client.HTTPConnection) -> None:
    sock = connection.sock
    if sock is not None:
        with contextlib.suppress(OSError):
            sock.shutdown(socket.SHUT_RDWR)
    with contextlib.suppress(OSError):
        connection.close()


def _perform_http(
    body: bytes,
    profile: dict[str, Any],
    deadline: float,
    max_response_bytes: int,
) -> TransportResponse:
    parsed = urlsplit(profile["api_base"])
    if parsed.scheme != "http" or parsed.hostname is None:
        raise WorkerError("invalid_model_profile", "model endpoint must use HTTP")
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise WorkerError("request_timeout", "request deadline expired", backend_state="unknown")
    connection = http.client.HTTPConnection(parsed.hostname, parsed.port, timeout=remaining)
    finished = threading.Event()
    expired = threading.Event()
    raw = bytearray()

    def close_at_deadline() -> None:
        if not finished.wait(max(0.0, deadline - time.monotonic())):
            expired.set()
            _close_connection(connection)

    watchdog = threading.Thread(target=close_at_deadline, name="local-worker-http-deadline")
    watchdog.start()
    response: http.client.HTTPResponse | None = None
    failure: WorkerError | None = None
    result: TransportResponse | None = None
    try:
        path = parsed.path.rstrip("/") + "/chat/completions"
        connection.request("POST", path, body=body, headers={"Content-Type": "application/json"})
        response = connection.getresponse()
        read = getattr(response, "read1", response.read)
        while True:
            if expired.is_set() or time.monotonic() >= deadline:
                raise TimeoutError("request deadline expired while reading response")
            chunk = read(min(65536, max_response_bytes + 1 - len(raw)))
            if not chunk:
                break
            raw.extend(chunk)
            if len(raw) > max_response_bytes:
                result = TransportResponse(
                    status=response.status,
                    reason=response.reason,
                    raw=bytes(raw[:max_response_bytes]),
                    over_limit=True,
                    incomplete=True,
                )
                break
        if result is None:
            result = TransportResponse(status=response.status, reason=response.reason, raw=bytes(raw))
    except (TimeoutError, socket.timeout) as exc:
        failure = WorkerError(
            "request_timeout",
            str(exc),
            response_bytes=bytes(raw),
            response_incomplete=True,
            backend_state="unknown",
        )
    except (OSError, http.client.HTTPException) as exc:
        if expired.is_set() or time.monotonic() >= deadline:
            failure = WorkerError(
                "request_timeout",
                "request deadline expired",
                response_bytes=bytes(raw),
                response_incomplete=True,
                backend_state="unknown",
            )
        else:
            failure = WorkerError(
                "api_disconnect",
                str(exc),
                response_bytes=bytes(raw) if raw else None,
                response_incomplete=bool(raw),
                backend_state="unknown",
            )
    finally:
        finished.set()
        if response is not None:
            with contextlib.suppress(OSError):
                response.close()
        _close_connection(connection)
        watchdog.join()
    if failure is not None:
        raise failure
    if expired.is_set():
        raise WorkerError(
            "request_timeout",
            "request deadline expired",
            response_bytes=bytes(raw),
            response_incomplete=True,
            backend_state="unknown",
        )
    if result is None:
        raise WorkerError("api_disconnect", "request ended without a response", backend_state="unknown")
    return result


def generate(payload: dict[str, Any], profile: dict[str, Any], timeout: float, max_response_bytes: int) -> ModelReply:
    started = time.monotonic()
    deadline = started + timeout
    body = json.dumps(payload, ensure_ascii=False).encode()
    try:
        transport = _perform_http(body, profile, deadline, max_response_bytes)
    except WorkerError as exc:
        exc.details["elapsed_seconds"] = round(time.monotonic() - started, 6)
        raise
    elapsed = time.monotonic() - started
    if transport.over_limit:
        return ModelReply(
            raw=transport.raw,
            parsed={},
            elapsed_seconds=elapsed,
            over_limit=True,
            incomplete=transport.incomplete,
        )
    if not 200 <= transport.status < 300:
        summary = transport.raw.decode("utf-8", errors="replace")[:512]
        raise WorkerError(
            "api_http_error",
            f"HTTP {transport.status} {transport.reason}: {summary}",
            {"elapsed_seconds": round(elapsed, 6)},
            response_bytes=transport.raw,
            response_incomplete=transport.incomplete,
            backend_state="finished" if not transport.incomplete else "unknown",
        )
    try:
        text = transport.raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise WorkerError(
            "invalid_response_encoding",
            "response is not valid UTF-8",
            {"elapsed_seconds": round(elapsed, 6)},
            response_bytes=transport.raw,
            backend_state="finished",
        ) from exc
    try:
        parsed = json.loads(text)
    except json.JSONDecodeError as exc:
        raise WorkerError(
            "malformed_response",
            f"response is not JSON: {exc}",
            {"elapsed_seconds": round(elapsed, 6)},
            response_bytes=transport.raw,
            backend_state="finished",
        ) from exc
    if not isinstance(parsed, dict):
        raise WorkerError(
            "malformed_response",
            "response must be a JSON object",
            {"elapsed_seconds": round(elapsed, 6)},
            response_bytes=transport.raw,
            backend_state="finished",
        )
    return ModelReply(raw=transport.raw, parsed=parsed, elapsed_seconds=elapsed)
