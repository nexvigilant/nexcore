"""Shared test fixtures for NexCore SDK tests."""

from __future__ import annotations

import pytest

from nexcore_sdk import NexCoreClient


@pytest.fixture
def mock_client(httpx_mock) -> NexCoreClient:
    """Client wired to pytest-httpx mock transport."""
    return NexCoreClient("http://test:3030")
