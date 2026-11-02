import copy
import json

import pytest

from hookdoctor.errors import ReportValidationError
from hookdoctor.examine import REPORT_FORMAT, build_report, validate_report
from hookdoctor.ingest import parse_config, parse_session_text

from synthetic_data import (
    CONFIG,
    prompt_line,
    session_text,
    stop_line,
    tool_result_line,
    tool_use_line,
)

TEXT = session_text(
    prompt_line(ts="2026-01-01T09:00:00Z"),
    tool_use_line("Bash", "call-1", "2026-01-01T09:00:05Z"),
    tool_result_line("call-1", "2026-01-01T09:00:07Z", text="denied by policy", is_error=True),
    stop_line("2026-01-01T09:01:00Z"),
)


def built():
    hooks = parse_config(CONFIG)
    records = parse_session_text(TEXT)
    report = build_report(hooks, records, "synthetic-settings.json", ["synthetic-session.jsonl"])
    return hooks, records, report


def test_report_carries_the_format_and_one_entry_per_hook():
    hooks, _records, report = built()
    assert report["format"] == REPORT_FORMAT == "hookdoctor-report/1"
    assert report["config_source"] == "synthetic-settings.json"
    assert report["session_sources"] == ["synthetic-session.jsonl"]
    assert report["window"] == {"start": None, "end": None}
    assert len(report["hooks"]) == len(hooks)
    for entry in report["hooks"]:
        assert set(entry) == {
            "id",
            "event",
            "matcher",
            "command",
            "hook_type",
            "class",
            "evidence",
        }
        assert set(entry["evidence"]) == {
            "matched",
            "ran",
            "effects",
            "first_ran_at",
            "last_ran_at",
        }
        assert set(entry["evidence"]["effects"]) == {
            "denial",
            "modification",
            "consumed_output",
        }
    json.dumps(report)


def test_report_validation_refuses_a_tampered_class():
    hooks, records, report = built()
    assert validate_report(
        report, hooks, records, "synthetic-settings.json", ["synthetic-session.jsonl"]
    )
    tampered = copy.deepcopy(report)
    tampered["hooks"][0]["class"] = "silent"
    with pytest.raises(ReportValidationError):
        validate_report(
            tampered, hooks, records, "synthetic-settings.json", ["synthetic-session.jsonl"]
        )


def test_report_validation_refuses_tampered_evidence_and_metadata():
    hooks, records, report = built()
    args = (hooks, records, "synthetic-settings.json", ["synthetic-session.jsonl"])
    tampered = copy.deepcopy(report)
    tampered["hooks"][0]["evidence"]["matched"] += 1
    with pytest.raises(ReportValidationError):
        validate_report(tampered, *args)
    tampered = copy.deepcopy(report)
    tampered["config_source"] = "another-settings.json"
    with pytest.raises(ReportValidationError):
        validate_report(tampered, *args)
    tampered = copy.deepcopy(report)
    tampered["unexpected"] = True
    with pytest.raises(ReportValidationError):
        validate_report(tampered, *args)
    tampered = copy.deepcopy(report)
    del tampered["near_misses"]
    with pytest.raises(ReportValidationError):
        validate_report(tampered, *args)
