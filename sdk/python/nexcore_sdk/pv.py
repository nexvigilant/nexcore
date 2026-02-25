"""Typed PV (Pharmacovigilance) methods for NexCoreClient.

Mixin class providing signal detection and causality assessment methods.
Routes through /api/v1/pv/*.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from .models import MeasuredValue, NaranjoResult, SignalResult

if TYPE_CHECKING:
    pass


class PvMixin:
    """PV signal detection and causality assessment methods."""

    def _post(self, path: str, json: dict[str, Any]) -> dict[str, Any]: ...

    def signal_detect(
        self, *, a: int, b: int, c: int, d: int
    ) -> SignalResult:
        """Run complete multi-method signal detection (PRR, ROR, IC, EBGM, chi-square).

        Args:
            a: Drug + Event count
            b: Drug + No Event count
            c: No Drug + Event count
            d: No Drug + No Event count

        Returns:
            SignalResult with all metrics and confidence intervals.
        """
        data = self._post("/api/v1/pv/signal/complete", {"a": a, "b": b, "c": c, "d": d})
        return SignalResult.model_validate(data)

    def signal_prr(self, *, a: int, b: int, c: int, d: int) -> MeasuredValue:
        """Calculate Proportional Reporting Ratio with 95% CI."""
        data = self._post("/api/v1/pv/signal/prr", {"a": a, "b": b, "c": c, "d": d})
        return MeasuredValue.model_validate(data)

    def signal_ror(self, *, a: int, b: int, c: int, d: int) -> MeasuredValue:
        """Calculate Reporting Odds Ratio with 95% CI."""
        data = self._post("/api/v1/pv/signal/ror", {"a": a, "b": b, "c": c, "d": d})
        return MeasuredValue.model_validate(data)

    def naranjo(
        self,
        *,
        temporal: int = 0,
        dechallenge: int = 0,
        rechallenge: int = 0,
        alternatives: int = 0,
        previous: int = 0,
    ) -> NaranjoResult:
        """Naranjo causality assessment (simplified 5-question).

        Args:
            temporal: Temporal relationship (1=yes, 0=unknown, -1=no)
            dechallenge: Improved after withdrawal
            rechallenge: Recurred on re-exposure
            alternatives: Alternative causes (-1=exist, 1=none, 0=unknown)
            previous: Previously reported in literature
        """
        data = self._post(
            "/api/v1/pv/naranjo",
            {
                "temporal": temporal,
                "dechallenge": dechallenge,
                "rechallenge": rechallenge,
                "alternatives": alternatives,
                "previous": previous,
            },
        )
        return NaranjoResult.model_validate(data)
