"""Synthetic fixtures for hookdoctor, written for this repo only.

Every configuration and every session line here is invented. No real
settings file and no real session file stands behind any of it.
"""

import json

SESSION = "synthetic-session-1"

CONFIG = {
    "hooks": {
        "PreToolUse": [
            {
                "matcher": "Bash",
                "hooks": [{"type": "command", "command": "guard-bash.sh"}],
            },
            {
                "matcher": "Edit|Write",
                "hooks": [{"type": "command", "command": "lint-edit.py"}],
            },
            {
                "matcher": "",
                "hooks": [{"type": "command", "command": "audit-all.sh"}],
            },
        ],
        "PostToolUse": [
            {
                "matcher": "Bash",
                "hooks": [{"type": "command", "command": "log-bash.sh"}],
            },
        ],
        "UserPromptSubmit": [
            {"hooks": [{"type": "command", "command": "note-prompt.sh"}]},
        ],
        "Stop": [
            {"hooks": [{"type": "command", "command": "wrap-up.sh"}]},
        ],
    }
}


def config_with(event, groups):
    return {"hooks": {event: groups}}


def group(matcher, *commands, include_matcher=True):
    built = {"hooks": [{"type": "command", "command": c} for c in commands]}
    if include_matcher:
        built["matcher"] = matcher
    return built


def _line(obj):
    return json.dumps(obj)


def prompt_line(text="please run the tests", ts="2026-01-01T09:00:00Z"):
    return _line(
        {
            "type": "user",
            "timestamp": ts,
            "sessionId": SESSION,
            "message": {"role": "user", "content": text},
        }
    )


def tool_use_line(name, call_id, ts):
    return _line(
        {
            "type": "assistant",
            "timestamp": ts,
            "sessionId": SESSION,
            "message": {
                "role": "assistant",
                "content": [
                    {
                        "type": "tool_use",
                        "id": call_id,
                        "name": name,
                        "input": {"command": "synthetic"},
                    }
                ],
            },
        }
    )


def tool_result_line(call_id, ts, text="ok", is_error=False):
    return _line(
        {
            "type": "user",
            "timestamp": ts,
            "sessionId": SESSION,
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": call_id,
                        "content": text,
                        "is_error": is_error,
                    }
                ],
            },
        }
    )


def hook_run_line(
    hook_name,
    event,
    ts,
    attachment_type="hook_success",
    content="",
    stdout="",
    call_id=None,
    exit_code=0,
):
    attachment = {
        "type": attachment_type,
        "hookName": hook_name,
        "hookEvent": event,
        "content": content,
        "stdout": stdout,
        "exitCode": exit_code,
    }
    if call_id is not None:
        attachment["toolUseID"] = call_id
    return _line(
        {
            "type": "attachment",
            "timestamp": ts,
            "sessionId": SESSION,
            "attachment": attachment,
        }
    )


def stop_line(ts, commands=("wrap-up.sh",), prevented=False, has_output=False):
    return _line(
        {
            "type": "system",
            "subtype": "stop_hook_summary",
            "timestamp": ts,
            "sessionId": SESSION,
            "hookCount": len(commands),
            "hookInfos": [{"command": c} for c in commands],
            "hookErrors": [],
            "preventedContinuation": prevented,
            "hasOutput": has_output,
        }
    )


def session_text(*lines):
    return "\n".join(lines) + "\n"
