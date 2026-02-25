"""Tests for NexCoreClient core functionality."""

from __future__ import annotations

import pytest
from nexcore_sdk import NexCoreClient, ConnectionError, ApiError


def test_client_context_manager():
    """Client works as context manager."""
    with NexCoreClient("http://localhost:9999") as client:
        assert client is not None


def test_health_success(httpx_mock, mock_client):
    """Health endpoint returns parsed response."""
    httpx_mock.add_response(
        url="http://test:3030/health",
        json={"status": "ok", "version": "1.0.0", "uptime_seconds": 42.5},
    )
    result = mock_client.health()
    assert result.status == "ok"
    assert result.version == "1.0.0"


def test_api_error_raised(httpx_mock, mock_client):
    """Non-2xx responses raise ApiError."""
    httpx_mock.add_response(
        url="http://test:3030/health",
        status_code=500,
        json={"code": "INTERNAL", "message": "Something broke"},
    )
    with pytest.raises(ApiError) as exc_info:
        mock_client.health()
    assert exc_info.value.status == 500
    assert exc_info.value.code == "INTERNAL"
