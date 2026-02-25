"""Pydantic v2 models mirroring NexCore Rust DTOs.

Each model maps 1:1 to a Rust response struct in nexcore-api.
"""

from __future__ import annotations

from pydantic import BaseModel, Field


# ── PV Signal Detection (pv.rs) ──────────────────────────


class MeasuredValue(BaseModel):
    """A measured metric with confidence interval — mirrors SignalMetricResponse."""

    value: float
    ci_lower: float
    ci_upper: float
    signal: bool


class SignalResult(BaseModel):
    """Complete multi-method signal detection — mirrors SignalCompleteResponse."""

    prr: float
    prr_ci_lower: float
    prr_ci_upper: float
    prr_signal: bool
    ror: float
    ror_ci_lower: float
    ror_ci_upper: float
    ror_signal: bool
    ic: float
    ic_ci_lower: float
    ic_signal: bool
    ebgm: float
    eb05: float
    ebgm_signal: bool
    chi_square: float
    signal_detected: bool


class NaranjoResult(BaseModel):
    """Naranjo causality assessment — mirrors NaranjoResponse."""

    score: int
    category: str
    interpretation: str


# ── FAERS (faers.rs) ─────────────────────────────────────


class FaersDrug(BaseModel):
    """Drug entry from a FAERS report."""

    medicinalproduct: str
    drugcharacterization: str


class FaersReaction(BaseModel):
    """Reaction entry from a FAERS report."""

    reactionmeddrapt: str
    reactionoutcome: str


class FaersPatient(BaseModel):
    """Patient record with drugs and reactions."""

    drug: list[FaersDrug] = Field(default_factory=list)
    reaction: list[FaersReaction] = Field(default_factory=list)


class FaersSearchResult(BaseModel):
    """Single FAERS adverse event report — mirrors FaersResult."""

    safetyreportid: str
    receivedate: str
    serious: int
    patient: FaersPatient


class FaersSearchResponse(BaseModel):
    """FAERS search response with results and total."""

    results: list[FaersSearchResult] = Field(default_factory=list)
    total: int
    query: str


class FaersEventCount(BaseModel):
    """Event frequency for a drug."""

    event: str
    count: int
    percentage: float


class FaersDrugEventsResult(BaseModel):
    """Top adverse events for a drug — mirrors FaersDrugEventsResponse."""

    drug: str
    events: list[FaersEventCount] = Field(default_factory=list)
    total_reports: int


class FaersSignalCheckResult(BaseModel):
    """Signal check for a drug-event pair — mirrors FaersSignalCheckResponse."""

    drug: str
    event: str
    signal_detected: bool
    prr: float
    ror: float
    case_count: int


# ── Signal Pipeline (signal.rs) ──────────────────────────


class SignalDetectResult(BaseModel):
    """Single signal detection result — mirrors SignalDetectResponse."""

    drug: str
    event: str
    prr: float | None = None
    ror: float | None = None
    ic: float = 0.0
    ebgm: float = 0.0
    chi_square: float = 0.0
    strength: str = ""
    signal: bool = False


class SignalBatchResult(BaseModel):
    """Batch signal detection — mirrors SignalBatchResponse."""

    results: list[SignalDetectResult] = Field(default_factory=list)
    signals_found: int = 0


# ── Health ───────────────────────────────────────────────


class HealthResponse(BaseModel):
    """API health check response."""

    status: str
    version: str = ""
    uptime_seconds: float = 0.0
