"""Typed FAERS methods for NexCoreClient.

Mixin class providing FDA Adverse Event Reporting System query methods.
Routes through /api/v1/faers/*.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from .models import (
    FaersDrugEventsResult,
    FaersSearchResponse,
    FaersSignalCheckResult,
)

if TYPE_CHECKING:
    pass


class FaersMixin:
    """FAERS query and signal check methods."""

    def _get(self, path: str, params: dict[str, Any] | None = None) -> dict[str, Any]: ...
    def _post(self, path: str, json: dict[str, Any]) -> dict[str, Any]: ...

    def faers_search(
        self, query: str, *, limit: int = 25
    ) -> FaersSearchResponse:
        """Search FAERS adverse event reports.

        Args:
            query: Drug name, reaction term, or free-text query.
            limit: Max results (default 25, max 100).
        """
        data = self._get("/api/v1/faers/search", {"query": query, "limit": limit})
        return FaersSearchResponse.model_validate(data)

    def faers_drug_events(
        self, drug: str, *, limit: int = 20
    ) -> FaersDrugEventsResult:
        """Get top adverse events for a drug.

        Args:
            drug: Drug name (generic or brand).
            limit: Top N events (default 20, max 100).
        """
        data = self._get("/api/v1/faers/drug-events", {"drug": drug, "limit": limit})
        return FaersDrugEventsResult.model_validate(data)

    def faers_signal_check(
        self, drug: str, event: str
    ) -> FaersSignalCheckResult:
        """Check for disproportionality signal between a drug and event.

        Args:
            drug: Drug name.
            event: Adverse event (MedDRA preferred term).
        """
        data = self._post(
            "/api/v1/faers/signal-check", {"drug": drug, "event": event}
        )
        return FaersSignalCheckResult.model_validate(data)
