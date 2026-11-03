"""Round 2 pins for the duplicate JSON key refusal."""

from pathlib import Path

import pytest

from hookdoctor import HookdoctorError, parse_session_text
from hookdoctor.cli import _read_json, main
from hookdoctor.examine import _parsed_output


ROOT = Path(__file__).resolve().parents[1]


def _write(tmp_path, name, text):
    path = tmp_path / name
    path.write_text(text, encoding="utf-8")
    return path


def test_duplicate_keys_in_the_configuration_are_refused(tmp_path, capsys):
    config = _write(
        tmp_path,
        "settings.json",
        '{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": '
        '[{"type": "command", "command": "guard-bash.sh"}]}], '
        '"PreToolUse": []}}',
    )
    session = _write(
        tmp_path,
        "session.jsonl",
        '{"type": "user", "timestamp": "2026-01-01T09:00:00Z", '
        '"sessionId": "synthetic-session-1", '
        '"message": {"role": "user", "content": "hello"}}\n',
    )
    result = main(
        [
            "--config",
            str(config),
            "--session",
            str(session),
            "--output",
            str(tmp_path / "out.json"),
            "--report",
            str(tmp_path / "out.md"),
        ]
    )
    assert result == 2
    assert (
        "hookdoctor error: the configuration carries the key PreToolUse"
        " more than once"
        in capsys.readouterr().err
    )

    entry = _write(
        tmp_path,
        "entry.json",
        '{"hooks": {"Stop": [{"hooks": [{"type": "command", '
        '"command": "first.sh", "command": "second.sh"}]}]}}',
    )
    with pytest.raises(HookdoctorError) as caught:
        _read_json(entry, "configuration")
    assert (
        str(caught.value)
        == "the configuration carries the key command more than once"
    )


def test_duplicate_keys_refuse_at_any_depth_and_name_the_first_repeat():
    with pytest.raises(HookdoctorError) as caught:
        parse_session_text(
            '{"type": "user", "message": {"content": '
            '[{"type": "text", "text": "a", "text": "b"}]}, '
            '"type": "assistant"}\n'
        )
    assert (
        str(caught.value)
        == "a session line carries the key text more than once"
    )

    with pytest.raises(HookdoctorError) as caught:
        parse_session_text(
            '{"b": 1, "a": 2, "b": 3, "a": 4, "type": "user"}\n'
        )
    assert (
        str(caught.value)
        == "a session line carries the key b more than once"
    )

    with pytest.raises(HookdoctorError) as caught:
        parse_session_text(
            '{"type": "user", "message": {"content": '
            '[{"type": "tool_result", "tool_use_id": "call-1", '
            '"tool_use_id": "call-2"}]}}\n'
        )
    assert (
        str(caught.value)
        == "a session line carries the key tool_use_id more than once"
    )


def test_duplicate_keys_in_a_session_line_are_refused(tmp_path, capsys):
    config = _write(
        tmp_path,
        "settings.json",
        '{"hooks": {"Stop": [{"hooks": '
        '[{"type": "command", "command": "wrap-up.sh"}]}]}}',
    )
    session = _write(
        tmp_path,
        "session.jsonl",
        '{"type": "user", "type": "assistant", "timestamp": '
        '"2026-01-01T09:00:00Z", "sessionId": "synthetic-session-1", '
        '"message": {"role": "user", "content": "hello"}}\n',
    )
    result = main(
        [
            "--config",
            str(config),
            "--session",
            str(session),
            "--output",
            str(tmp_path / "out.json"),
            "--report",
            str(tmp_path / "out.md"),
        ]
    )
    assert result == 2
    assert (
        "hookdoctor error: a session line carries the key type"
        " more than once"
        in capsys.readouterr().err
    )


def test_the_same_key_in_different_objects_is_accepted(tmp_path):
    config = _write(
        tmp_path,
        "settings.json",
        '{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": '
        '[{"type": "command", "command": "guard-bash.sh"}]}], '
        '"PostToolUse": [{"matcher": "Bash", "hooks": '
        '[{"type": "command", "command": "log-bash.sh"}]}]}}',
    )
    document = _read_json(config, "configuration")
    assert set(document["hooks"]) == {"PreToolUse", "PostToolUse"}

    records = parse_session_text(
        '{"type": "user", "message": {"content": '
        '[{"type": "text", "text": "one"}, '
        '{"type": "text", "text": "two"}]}}\n'
    )
    assert [record.kind for record in records] == ["prompt"]
    assert records[0].text == "one\ntwo"


def test_output_text_keeps_its_standing_sampling_reading():
    assert _parsed_output('{"decision": "allow", "decision": "block"}') == {
        "decision": "block"
    }
    assert _parsed_output('{"decision": "block", "decision": "allow"}') == {
        "decision": "allow"
    }
