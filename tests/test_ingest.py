import pytest

from hookdoctor.errors import HookdoctorError
from hookdoctor.ingest import parse_config, parse_session_text

from synthetic_data import (
    CONFIG,
    hook_run_line,
    prompt_line,
    session_text,
    stop_line,
    tool_result_line,
    tool_use_line,
)


def test_config_flattens_groups_in_document_order():
    hooks = parse_config(CONFIG)
    assert [h.id for h in hooks] == [
        "PreToolUse:0:0",
        "PreToolUse:1:0",
        "PreToolUse:2:0",
        "PostToolUse:0:0",
        "UserPromptSubmit:0:0",
        "Stop:0:0",
    ]
    assert [h.command for h in hooks] == [
        "guard-bash.sh",
        "lint-edit.py",
        "audit-all.sh",
        "log-bash.sh",
        "note-prompt.sh",
        "wrap-up.sh",
    ]
    assert hooks[0].matcher == "Bash"
    assert hooks[1].matcher == "Edit|Write"
    assert hooks[2].matcher == ""
    assert hooks[4].matcher is None
    assert hooks[5].matcher is None
    assert all(h.hook_type == "command" for h in hooks)


def test_config_without_hooks_yields_no_hooks():
    assert parse_config({}) == []
    assert parse_config({"hooks": {}}) == []
    assert parse_config({"unrelated": 1}) == []


def test_malformed_config_is_refused():
    with pytest.raises(HookdoctorError):
        parse_config(["not", "an", "object"])
    with pytest.raises(HookdoctorError):
        parse_config({"hooks": {"PreToolUse": {"matcher": "Bash"}}})
    with pytest.raises(HookdoctorError):
        parse_config({"hooks": {"PreToolUse": [{"matcher": "Bash"}]}})
    with pytest.raises(HookdoctorError):
        parse_config(
            {"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command"}]}]}}
        )
    with pytest.raises(HookdoctorError):
        parse_config({"hooks": {"PreToolUse": [{"matcher": 7, "hooks": []}]}})


def test_trace_builds_dated_records_from_each_line_kind():
    text = session_text(
        prompt_line(ts="2026-01-01T09:00:00Z"),
        tool_use_line("Bash", "call-1", "2026-01-01T09:00:05Z"),
        hook_run_line(
            "PreToolUse:Bash", "PreToolUse", "2026-01-01T09:00:06Z", call_id="call-1"
        ),
        tool_result_line("call-1", "2026-01-01T09:00:07Z", text="3 passed"),
        stop_line("2026-01-01T09:01:00Z"),
        '{"type": "summary", "timestamp": "2026-01-01T09:02:00Z", "sessionId": "synthetic-session-1"}',
    )
    records = parse_session_text(text)
    assert [r.kind for r in records] == [
        "prompt",
        "tool_call",
        "hook_run",
        "tool_outcome",
        "stop",
        "other",
    ]
    assert [r.line for r in records] == [1, 2, 3, 4, 5, 6]
    call = records[1]
    assert call.tool_name == "Bash"
    assert call.call_id == "call-1"
    assert call.timestamp == "2026-01-01T09:00:05Z"
    run = records[2]
    assert run.hook_name == "PreToolUse:Bash"
    assert run.hook_event == "PreToolUse"
    assert run.call_id == "call-1"
    assert run.attachment_type == "hook_success"
    outcome = records[3]
    assert outcome.call_id == "call-1"
    assert outcome.is_error is False
    assert outcome.text == "3 passed"
    stop = records[4]
    assert stop.commands == ("wrap-up.sh",)
    assert stop.prevented_continuation is False
    assert stop.has_output is False


def test_trace_refuses_a_line_that_is_not_json():
    with pytest.raises(HookdoctorError):
        parse_session_text("this is not json\n")
    with pytest.raises(HookdoctorError):
        parse_session_text(session_text(prompt_line(), "{broken"))


def test_trace_refuses_a_line_with_no_type():
    with pytest.raises(HookdoctorError):
        parse_session_text('{"timestamp": "2026-01-01T09:00:00Z"}\n')
    with pytest.raises(HookdoctorError):
        parse_session_text("42\n")
