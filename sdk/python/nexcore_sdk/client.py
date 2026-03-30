"""NexCoreClient — synchronous HTTP client for the NexCore REST API.

Uses httpx for HTTP transport and Pydantic v2 for response validation.
Composes PV, FAERS, MCP, and Signal methods via mixins.
"""

from __future__ import annotations

from typing import Any

import httpx

from .exceptions import ApiError, ConnectionError, NexCoreError
from .faers import FaersMixin
from .mcp import McpMixin
from .models import HealthResponse, SignalBatchResult, SignalDetectResult
from .pv import PvMixin


class NexCoreClient(PvMixin, FaersMixin, McpMixin):
    """Synchronous client for NexCore REST API.

    Args:
        base_url: NexCore API base URL (default: http://localhost:3030).
        timeout: Request timeout in seconds (default: 30).
        api_key: Optional API key for authenticated mode.

    Example:
        >>> client = NexCoreClient()
        >>> result = client.signal_detect(a=100, b=500, c=50, d=10000)
        >>> print(f"PRR: {result.prr} [{result.prr_ci_lower}, {result.prr_ci_upper}]")
    """

    def __init__(
        self,
        base_url: str = "http://localhost:3030",
        *,
        timeout: float = 30.0,
        api_key: str | None = None,
    ) -> None:
        headers: dict[str, str] = {}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"

        self._client = httpx.Client(
            base_url=base_url,
            timeout=timeout,
            headers=headers,
        )

    def close(self) -> None:
        """Close the underlying HTTP connection pool."""
        self._client.close()

    def __enter__(self) -> NexCoreClient:
        return self

    def __exit__(self, *args: object) -> None:
        self.close()

    # ── HTTP helpers ─────────────────────────────────────

    def _post(self, path: str, json: dict[str, Any]) -> dict[str, Any]:
        """POST request with error handling."""
        try:
            resp = self._client.post(path, json=json)
        except httpx.ConnectError as e:
            raise ConnectionError(
                f"Cannot connect to NexCore API: {e}"
            ) from e
        except httpx.TimeoutException as e:
            raise NexCoreError(f"Request timed out: {e}") from e

        return self._handle_response(resp)

    def _get(self, path: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        """GET request with error handling."""
        try:
            resp = self._client.get(path, params=params)
        except httpx.ConnectError as e:
            raise ConnectionError(
                f"Cannot connect to NexCore API: {e}"
            ) from e
        except httpx.TimeoutException as e:
            raise NexCoreError(f"Request timed out: {e}") from e

        return self._handle_response(resp)

    def _handle_response(self, resp: httpx.Response) -> dict[str, Any]:
        """Parse response, raising ApiError on non-2xx."""
        if resp.status_code >= 400:
            try:
                body = resp.json()
                code = body.get("code", "UNKNOWN")
                message = body.get("message", resp.text)
            except Exception:
                code = "HTTP_ERROR"
                message = resp.text or f"HTTP {resp.status_code}"
            raise ApiError(message, code=code, status=resp.status_code)

        return resp.json()

    # ── Health ───────────────────────────────────────────

    def health(self) -> HealthResponse:
        """Check API health."""
        data = self._get("/health")
        return HealthResponse.model_validate(data)

    # ── Signal Pipeline (batch) ──────────────────────────

    def signal_batch(
        self,
        items: list[dict[str, Any]],
    ) -> SignalBatchResult:
        """Batch signal detection for multiple drug-event pairs.

        Each item should have: drug, event, a, b, c, d.
        Processed in parallel on the Rust side.

        Args:
            items: List of dicts with drug, event, a, b, c, d keys.

        Returns:
            SignalBatchResult with all results and signal count.
        """
        data = self._post("/api/v1/signal/batch", {"items": items})
        return SignalBatchResult.model_validate(data)

    def signal_detect_pipeline(
        self,
        *,
        drug: str,
        event: str,
        a: int,
        b: int,
        c: int,
        d: int,
    ) -> SignalDetectResult:
        """Single signal detection via the signal pipeline (signal-stats + signal-core).

        Uses the /api/v1/signal/detect endpoint which provides strength classification.
        """
        data = self._post(
            "/api/v1/signal/detect",
            {"drug": drug, "event": event, "a": a, "b": b, "c": c, "d": d},
        )
        return SignalDetectResult.model_validate(data)
