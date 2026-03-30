"""NexCore Python SDK — typed client for NexVigilant's pharmacovigilance platform.

Provides access to 780+ MCP tools, PV signal detection, FAERS queries,
and causality assessment through a clean Python API backed by Rust computation.
"""

from .client import NexCoreClient
from .exceptions import ApiError, ConnectionError, NexCoreError, ValidationError
from .models import (
    FaersDrugEventsResult,
    FaersEventCount,
    FaersSearchResponse,
    FaersSearchResult,
    FaersSignalCheckResult,
    HealthResponse,
    MeasuredValue,
    NaranjoResult,
    SignalBatchResult,
    SignalDetectResult,
    SignalResult,
)
from ._version import __version__

__all__ = [
    "NexCoreClient",
    "NexCoreError",
    "ApiError",
    "ConnectionError",
    "ValidationError",
    "MeasuredValue",
    "SignalResult",
    "NaranjoResult",
    "FaersSearchResponse",
    "FaersSearchResult",
    "FaersDrugEventsResult",
    "FaersEventCount",
    "FaersSignalCheckResult",
    "SignalDetectResult",
    "SignalBatchResult",
    "HealthResponse",
    "__version__",
]
