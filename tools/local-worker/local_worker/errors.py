from __future__ import annotations

from typing import Any


class WorkerError(Exception):
    """A classified error safe to persist in a job manifest."""

    def __init__(
        self,
        code: str,
        message: str,
        details: dict[str, Any] | None = None,
        *,
        response_bytes: bytes | None = None,
        response_incomplete: bool = False,
        backend_state: str | None = None,
    ):
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details or {}
        self.response_bytes = response_bytes
        self.response_incomplete = response_incomplete
        self.backend_state = backend_state

    def as_dict(self) -> dict[str, Any]:
        return {"class": self.code, "message": self.message, "details": self.details}
