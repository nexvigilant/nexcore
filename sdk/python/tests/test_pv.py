"""Tests for PV signal detection methods."""

from __future__ import annotations

from nexcore_sdk import MeasuredValue, SignalResult


def test_signal_detect(httpx_mock, mock_client):
    """signal_detect returns SignalResult with all fields."""
    httpx_mock.add_response(
        url="http://test:3030/api/v1/pv/signal/complete",
        json={
            "prr": 3.96,
            "prr_ci_lower": 3.22,
            "prr_ci_upper": 4.87,
            "prr_signal": True,
            "ror": 4.17,
            "ror_ci_lower": 3.35,
            "ror_ci_upper": 5.19,
            "ror_signal": True,
            "ic": 1.99,
            "ic_ci_lower": 1.65,
            "ic_signal": True,
            "ebgm": 3.85,
            "eb05": 3.12,
            "ebgm_signal": True,
            "chi_square": 187.5,
            "signal_detected": True,
        },
    )
    result = mock_client.signal_detect(a=100, b=500, c=50, d=10000)
    assert isinstance(result, SignalResult)
    assert result.signal_detected is True
    assert result.prr_ci_lower == 3.22  # Measured<T>, not bare float


def test_signal_prr(httpx_mock, mock_client):
    """signal_prr returns MeasuredValue with CI."""
    httpx_mock.add_response(
        url="http://test:3030/api/v1/pv/signal/prr",
        json={"value": 3.96, "ci_lower": 3.22, "ci_upper": 4.87, "signal": True},
    )
    result = mock_client.signal_prr(a=100, b=500, c=50, d=10000)
    assert isinstance(result, MeasuredValue)
    assert result.ci_lower is not None
    assert result.signal is True


def test_naranjo(httpx_mock, mock_client):
    """Naranjo assessment returns scored result."""
    httpx_mock.add_response(
        url="http://test:3030/api/v1/pv/naranjo",
        json={"score": 6, "category": "Probable", "interpretation": "Score 5-8: Probable ADR"},
    )
    result = mock_client.naranjo(temporal=1, dechallenge=1, previous=1)
    assert result.score == 6
    assert result.category == "Probable"
