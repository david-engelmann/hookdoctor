"""Pins for the Round 1 review fixes recorded in CONTRACT.md.

Every fixture here is synthetic and written for this file. The
shapes follow the probes the red team and the two axis review
ran against slice 2.
"""

import json

import pytest

from hookdoctor.errors import HookdoctorError
from hookdoctor.examine import build_report, matcher_matches
from hookdoctor.ingest import parse_config, parse_session_text

from synthetic_data import (
    SESSION,
    config_with,
    group,
    hook_run_line,
    session_text,
    tool_result_line,
    tool_use_line,
)

T1 = "2026-01-01T09:00:00Z"
T2 = "2026-01-01T09:05:00Z"
T3 = "2026-01-01T09:10:00Z"


def report_for(config, text, **kwargs):
    hooks = parse_config(config)
    records = parse_session_text(text)
    return build_report(
        hooks, records, "synthetic-settings.json", ["synthetic-session.jsonl"], **kwargs
    )


def entry(report, command):
    for hook in report["hooks"]:
        if hook["command"] == command:
            return hook
    raise AssertionError(f"no entry for {command}")


def two_hook_config():
    return {
        "hooks": {
            "PreToolUse": [
                group("Bash", "guard-bash.sh"),
                group("", "audit-all.sh"),
            ]
        }
    }


def raw_hook_run_line(name, event, ts, command=None, attachment_type="hook_blocking_error"):
    attachment = {
        "type": attachment_type,
        "hookEvent": event,
        "content": "",
        "stdout": "",
        "exitCode": 1,
    }
    if name is not None:
        attachment["hookName"] = name
    if command is not None:
        attachment["command"] = command
    return json.dumps(
        {
            "type": "attachment",
            "timestamp": ts,
            "sessionId": SESSION,
            "attachment": attachment,
        }
    )


# FIX-1, the denial reading of outcome text.


def test_negated_denial_text_credits_no_hook():
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line(
            "call-1",
            T2,
            text="the request was not denied; the sandbox failed to start",
            is_error=True,
        ),
        tool_use_line("Bash", "call-2", T3),
        tool_result_line(
            "call-2", T3, text="the call wasn't denied by anyone", is_error=True
        ),
    )
    report = report_for(two_hook_config(), text)
    for command in ("guard-bash.sh", "audit-all.sh"):
        hook = entry(report, command)
        assert hook["evidence"]["effects"]["denial"] == 0
        assert hook["class"] == "firing-without-effect"


def test_denial_attributed_to_an_unconfigured_party_credits_no_hook():
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line(
            "call-1",
            T2,
            text="access was denied by external-tool xyz-guard",
            is_error=True,
        ),
    )
    report = report_for(two_hook_config(), text)
    for command in ("guard-bash.sh", "audit-all.sh"):
        hook = entry(report, command)
        assert hook["evidence"]["effects"]["denial"] == 0
        assert hook["class"] == "firing-without-effect"


def test_ambient_denial_text_credits_every_matching_hook():
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line(
            "call-1",
            T2,
            text="cat: /restricted/file: Permission denied",
            is_error=True,
        ),
    )
    report = report_for(two_hook_config(), text)
    for command in ("guard-bash.sh", "audit-all.sh"):
        hook = entry(report, command)
        assert hook["evidence"]["effects"]["denial"] == 1
        assert hook["class"] == "firing"


def test_denial_attributed_to_a_configured_hook_credits_only_that_hook():
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2, text="denied by guard-bash.sh", is_error=True),
    )
    report = report_for(two_hook_config(), text)
    guard = entry(report, "guard-bash.sh")
    assert guard["evidence"]["effects"]["denial"] == 1
    assert guard["class"] == "firing"
    audit = entry(report, "audit-all.sh")
    assert audit["evidence"]["effects"]["denial"] == 0
    assert audit["class"] == "firing-without-effect"


# FIX-2, error record content is not consumed output.


def test_non_blocking_error_content_is_not_consumed_output():
    config = config_with("PostToolUse", [group("Bash", "log-bash.sh")])
    text = session_text(
        hook_run_line(
            "PostToolUse:Bash",
            "PostToolUse",
            T1,
            attachment_type="hook_non_blocking_error",
            content="warning: the hook timed out and changed nothing",
            call_id="call-1",
        )
    )
    hook = entry(report_for(config, text), "log-bash.sh")
    assert hook["evidence"]["ran"] == 1
    assert hook["evidence"]["effects"]["consumed_output"] == 0
    assert hook["class"] == "firing-without-effect"


def test_blocking_error_content_is_not_consumed_output():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        hook_run_line(
            "PreToolUse:Bash",
            "PreToolUse",
            T1,
            attachment_type="hook_blocking_error",
            content="the guard refused in prose",
            call_id="call-1",
        )
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["effects"]["denial"] == 1
    assert hook["evidence"]["effects"]["consumed_output"] == 0
    assert hook["class"] == "firing"


# FIX-3, run record agreement is the hook own identity.


def test_sibling_hook_is_not_credited_by_another_hooks_run_record():
    text = session_text(
        hook_run_line(
            "PreToolUse:Bash",
            "PreToolUse",
            T1,
            attachment_type="hook_blocking_error",
            call_id="call-1",
        )
    )
    report = report_for(two_hook_config(), text)
    guard = entry(report, "guard-bash.sh")
    assert guard["evidence"]["effects"]["denial"] == 1
    assert guard["class"] == "firing"
    audit = entry(report, "audit-all.sh")
    assert audit["evidence"]["matched"] == 0
    assert audit["evidence"]["ran"] == 0
    assert audit["class"] == "silent"


def test_bare_event_run_record_credits_only_matcherless_hooks():
    text = session_text(
        hook_run_line(
            "PreToolUse",
            "PreToolUse",
            T1,
            attachment_type="hook_blocking_error",
            call_id="call-1",
        )
    )
    report = report_for(two_hook_config(), text)
    audit = entry(report, "audit-all.sh")
    assert audit["evidence"]["effects"]["denial"] == 1
    assert audit["class"] == "firing"
    guard = entry(report, "guard-bash.sh")
    assert guard["evidence"]["matched"] == 0
    assert guard["class"] == "silent"


def test_run_record_with_disagreeing_identifiers_credits_no_hook():
    config = {
        "hooks": {
            "PreToolUse": [
                group("Bash", "guard-bash.sh"),
                group("Edit|Write", "lint-edit.py"),
            ]
        }
    }
    text = session_text(
        raw_hook_run_line("PreToolUse:Edit", "PreToolUse", T1, command="guard-bash.sh")
    )
    report = report_for(config, text)
    for command in ("guard-bash.sh", "lint-edit.py"):
        hook = entry(report, command)
        assert hook["evidence"]["matched"] == 0
        assert hook["evidence"]["ran"] == 0
        assert hook["class"] == "silent"


def test_run_record_command_alone_identifies_its_hook():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        raw_hook_run_line(None, "PreToolUse", T1, command="guard-bash.sh")
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["effects"]["denial"] == 1
    assert hook["class"] == "firing"


# FIX-4, subjects and near misses carry their true sources.


def test_posttool_evidence_dates_come_from_the_outcome():
    config = config_with("PostToolUse", [group("Bash", "log-bash.sh")])
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T3),
    )
    hook = entry(report_for(config, text), "log-bash.sh")
    assert hook["evidence"]["first_ran_at"] == T3
    assert hook["evidence"]["last_ran_at"] == T3


def test_near_misses_name_their_session_source():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    hooks = parse_config(config)
    first = parse_session_text(
        session_text(
            tool_use_line("bash", "call-1", T1),
            tool_result_line("call-1", T2),
        ),
        file_index=0,
    )
    second = parse_session_text(
        session_text(
            tool_use_line("bash", "call-2", T1),
            tool_result_line("call-2", T2),
        ),
        file_index=1,
    )
    report = build_report(
        hooks,
        first + second,
        "synthetic-settings.json",
        ["first-session.jsonl", "second-session.jsonl"],
    )
    hook = entry(report, "guard-bash.sh")
    assert hook["class"] == "silent"
    misses = [m for m in report["near_misses"] if m["hook_id"] == hook["id"]]
    assert len(misses) == 2
    assert [m["source"] for m in misses] == [
        "first-session.jsonl",
        "second-session.jsonl",
    ]
    assert all(m["line"] == 1 for m in misses)
    assert all(m["reason"] == "name-form" for m in misses)


# FIX-5, uniform configuration and instant truth.


def test_invalid_matcher_pattern_is_refused_at_ingestion():
    with pytest.raises(HookdoctorError):
        parse_config(config_with("PreToolUse", [group("Edit(", "broken.sh")]))
    with pytest.raises(HookdoctorError):
        parse_config(config_with("Stop", [group("Edit(", "broken.sh")]))


def test_window_bounds_compare_as_instants():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("Bash", "call-1", "2026-01-01T04:00:00-05:00"),
        tool_result_line("call-1", "2026-01-01T04:01:00-05:00"),
    )
    report = report_for(
        config,
        text,
        window_start="2026-01-01T09:00:00Z",
        window_end="2026-01-01T10:00:00Z",
    )
    hook = entry(report, "guard-bash.sh")
    assert hook["evidence"]["ran"] == 1
    assert hook["class"] == "firing-without-effect"


def test_evidence_dates_are_ordered_as_instants():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("Bash", "call-1", "2026-01-01T08:00:00-05:00"),
        tool_result_line("call-1", "2026-01-01T08:01:00-05:00"),
        tool_use_line("Bash", "call-2", "2026-01-01T09:00:00Z"),
        tool_result_line("call-2", "2026-01-01T09:01:00Z"),
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["first_ran_at"] == "2026-01-01T09:00:00Z"
    assert hook["evidence"]["last_ran_at"] == "2026-01-01T08:00:00-05:00"


# FIX-6, outcomes pair to calls in order.


def test_outcomes_pair_to_calls_in_order():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    success_first = session_text(
        tool_use_line("Bash", "dup", T1),
        tool_result_line("dup", T2, text="ok"),
        tool_use_line("Bash", "dup", T3),
        tool_result_line("dup", T3, text="the hook denied this call", is_error=True),
    )
    hook = entry(report_for(config, success_first), "guard-bash.sh")
    assert hook["evidence"]["ran"] == 2
    assert hook["evidence"]["effects"]["denial"] == 1
    denial_first = session_text(
        tool_use_line("Bash", "dup", T1),
        tool_result_line("dup", T1, text="the hook denied this call", is_error=True),
        tool_use_line("Bash", "dup", T2),
        tool_result_line("dup", T3, text="ok"),
    )
    hook = entry(report_for(config, denial_first), "guard-bash.sh")
    assert hook["evidence"]["ran"] == 2
    assert hook["evidence"]["effects"]["denial"] == 1


# FIX-7, the matcher never matches a missing tool name.


def test_matcher_never_matches_a_missing_tool_name():
    assert matcher_matches("Bash", None) is False
    assert matcher_matches("", None) is False
    assert matcher_matches(None, None) is False
