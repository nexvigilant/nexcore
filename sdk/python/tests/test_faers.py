"""Tests for FAERS query methods."""

from __future__ import annotations

from nexcore_sdk import FaersDrugEventsResult, FaersSignalCheckResult


def test_faers_drug_events(httpx_mock, mock_client):
    """faers_drug_events returns structured event counts."""
    httpx_mock.add_response(
        url="http://test:3030/api/v1/faers/drug-events?drug=ASPIRIN&limit=5",
        json={
            "drug": "ASPIRIN",
            "events": [
                {"event": "NAUSEA", "count": 1500, "percentage": 12.3},
                {"event": "HEADACHE", "count": 1200, "percentage": 9.8},
            ],
            "total_reports": 12200,
        },
    )
    result = mock_client.faers_drug_events("ASPIRIN", limit=5)
    assert isinstance(result, FaersDrugEventsResult)
    assert result.drug == "ASPIRIN"
    assert len(result.events) == 2
    assert result.events[0].event == "NAUSEA"


def test_faers_signal_check(httpx_mock, mock_client):
    """faers_signal_check returns disproportionality result."""
    httpx_mock.add_response(
        url="http://test:3030/api/v1/faers/signal-check",
        json={
            "drug": "ASPIRIN",
            "event": "GI BLEEDING",
            "signal_detected": True,
            "prr": 4.2,
            "ror": 4.5,
            "case_count": 350,
        },
    )
    result = mock_client.faers_signal_check("ASPIRIN", "GI BLEEDING")
    assert isinstance(result, FaersSignalCheckResult)
    assert result.signal_detected is True
    assert result.prr == 4.2
