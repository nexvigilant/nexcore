"""NexCore SDK exception hierarchy."""

from __future__ import annotations


class NexCoreError(Exception):
    """Base exception for all NexCore SDK errors."""

    def __init__(self, message: str, code: str | None = None) -> None:
        self.code = code
        super().__init__(message)


class ConnectionError(NexCoreError):
    """Failed to connect to NexCore API."""


class ApiError(NexCoreError):
    """NexCore API returned an error response."""

    def __init__(self, message: str, code: str, status: int) -> None:
        self.status = status
        super().__init__(message, code)


class ValidationError(NexCoreError):
    """Client-side validation failed."""
