"""Generic MCP tool proxy for NexCoreClient.

Mixin class providing access to any of the 780+ MCP tools
via the REST gateway at /api/v1/mcp/{tool}.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    pass


class McpMixin:
    """Generic MCP tool invocation via REST gateway."""

    def _post(self, path: str, json: dict[str, Any]) -> dict[str, Any]: ...

    def mcp(self, tool: str, params: dict[str, Any] | None = None) -> Any:
        """Call any MCP tool by name through the REST gateway.

        Args:
            tool: MCP tool name (e.g. "pv_core_fdr_adjust").
            params: Tool-specific parameters as a dict.

        Returns:
            The tool's response payload (varies by tool).

        Example:
            >>> result = client.mcp("pv_core_fdr_adjust", {
            ...     "p_values": [0.01, 0.04, 0.03],
            ...     "method": "benjamini_hochberg"
            ... })
        """
        data = self._post(f"/api/v1/mcp/{tool}", params or {})
        return data
