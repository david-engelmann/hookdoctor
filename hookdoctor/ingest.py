"""Ingestion of local hook configurations and session trace files.

Both ingesters work on data already read from local files. Nothing in
this module touches a network or a path.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass

from .errors import HookdoctorError

HOOK_ATTACHMENT_TYPES = frozenset(
    {"hook_success", "hook_blocking_error", "hook_non_blocking_error"}
)


def _object_from_pairs(pairs, document):
    """Build one JSON object, refusing a key that stands twice."""
    result = {}
    for key, value in pairs:
        if key in result:
            raise HookdoctorError(
                f"{document} carries the key {key} more than once"
            )
        result[key] = value
    return result


def pairs_hook(document):
    """Return a decoder hook that refuses repeated keys in one object."""

    def hook(pairs):
        return _object_from_pairs(pairs, document)

    return hook


@dataclass(frozen=True)
class HookConfig:
    """One configured hook entry flattened from its matcher group."""

    id: str
    event: str
    matcher: str | None
    hook_type: str
    command: str


@dataclass(frozen=True)
class TraceRecord:
    """One dated record built from one piece of a session line."""

    kind: str
    timestamp: str | None
    session: str | None
    file_index: int
    line: int
    tool_name: str | None = None
    call_id: str | None = None
    is_error: bool = False
    text: str = ""
    hook_name: str | None = None
    hook_event: str | None = None
    command: str | None = None
    content: str = ""
    stdout: str = ""
    exit_code: int | None = None
    attachment_type: str | None = None
    commands: tuple = ()
    prevented_continuation: bool = False
    has_output: bool = False


def parse_config(document):
    """Flatten a settings document into hook records in document order."""
    if not isinstance(document, dict):
        raise HookdoctorError("the configuration is not a JSON object")
    hooks_object = document.get("hooks")
    if hooks_object is None:
        return []
    if not isinstance(hooks_object, dict):
        raise HookdoctorError("the hooks value is not an object")
    hooks = []
    for event, groups in hooks_object.items():
        if not isinstance(groups, list):
            raise HookdoctorError("a hooks event does not carry a list of groups")
        for group_index, group in enumerate(groups):
            if not isinstance(group, dict):
                raise HookdoctorError("a matcher group is not an object")
            entries = group.get("hooks")
            if not isinstance(entries, list):
                raise HookdoctorError("a matcher group carries no hooks list")
            matcher = group.get("matcher")
            if matcher is not None and not isinstance(matcher, str):
                raise HookdoctorError("a matcher is not a string")
            if matcher:
                try:
                    re.compile(matcher)
                except re.error as exc:
                    raise HookdoctorError(
                        "a matcher is not a valid pattern"
                    ) from exc
            for entry_index, entry in enumerate(entries):
                if not isinstance(entry, dict):
                    raise HookdoctorError("a hook entry is not an object")
                command = entry.get("command")
                if not isinstance(command, str) or not command:
                    raise HookdoctorError("a hook entry carries no command")
                hook_type = entry.get("type", "command")
                if not isinstance(hook_type, str) or not hook_type:
                    raise HookdoctorError("a hook entry carries no type")
                hooks.append(
                    HookConfig(
                        id=f"{event}:{group_index}:{entry_index}",
                        event=event,
                        matcher=matcher,
                        hook_type=hook_type,
                        command=command,
                    )
                )
    return hooks


def _text_from_result_content(content):
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for block in content:
            if isinstance(block, dict) and block.get("type") == "text":
                text = block.get("text")
                if isinstance(text, str):
                    parts.append(text)
        return "\n".join(parts)
    return ""


def parse_session_text(text, file_index=0):
    """Build dated trace records from the text of one session file."""
    records = []

    def add(kind, timestamp, session, line, **kwargs):
        records.append(
            TraceRecord(
                kind=kind,
                timestamp=timestamp,
                session=session,
                file_index=file_index,
                line=line,
                **kwargs,
            )
        )

    for line_number, raw in enumerate(text.splitlines(), start=1):
        if not raw.strip():
            continue
        try:
            line_object = json.loads(
                raw, object_pairs_hook=pairs_hook("a session line")
            )
        except json.JSONDecodeError as exc:
            raise HookdoctorError("a session line is not valid JSON") from exc
        if not isinstance(line_object, dict) or "type" not in line_object:
            raise HookdoctorError("a session line carries no type")
        timestamp = line_object.get("timestamp")
        if not isinstance(timestamp, str):
            timestamp = None
        session = line_object.get("sessionId")
        if not isinstance(session, str):
            session = None
        line_type = line_object["type"]

        if line_type == "assistant":
            message = line_object.get("message")
            content = message.get("content") if isinstance(message, dict) else None
            if isinstance(content, list):
                placed = False
                for block in content:
                    if (
                        isinstance(block, dict)
                        and block.get("type") == "tool_use"
                        and isinstance(block.get("name"), str)
                        and isinstance(block.get("id"), str)
                    ):
                        add(
                            "tool_call",
                            timestamp,
                            session,
                            line_number,
                            tool_name=block["name"],
                            call_id=block["id"],
                        )
                        placed = True
                if placed:
                    continue
            add("other", timestamp, session, line_number)
        elif line_type == "user":
            message = line_object.get("message")
            content = message.get("content") if isinstance(message, dict) else None
            if isinstance(content, str):
                add("prompt", timestamp, session, line_number, text=content)
                continue
            placed = False
            if isinstance(content, list):
                prompt_texts = []
                for block in content:
                    if not isinstance(block, dict):
                        continue
                    if block.get("type") == "tool_result" and isinstance(
                        block.get("tool_use_id"), str
                    ):
                        add(
                            "tool_outcome",
                            timestamp,
                            session,
                            line_number,
                            call_id=block["tool_use_id"],
                            is_error=bool(block.get("is_error", False)),
                            text=_text_from_result_content(block.get("content")),
                        )
                        placed = True
                    elif block.get("type") == "text" and isinstance(
                        block.get("text"), str
                    ):
                        prompt_texts.append(block["text"])
                if prompt_texts:
                    add(
                        "prompt",
                        timestamp,
                        session,
                        line_number,
                        text="\n".join(prompt_texts),
                    )
                    placed = True
            if not placed:
                add("other", timestamp, session, line_number)
        elif line_type == "attachment":
            attachment = line_object.get("attachment")
            if (
                isinstance(attachment, dict)
                and attachment.get("type") in HOOK_ATTACHMENT_TYPES
            ):
                hook_name = attachment.get("hookName")
                if not isinstance(hook_name, str):
                    hook_name = None
                hook_event = attachment.get("hookEvent")
                if not isinstance(hook_event, str):
                    hook_event = None
                if hook_event is None and hook_name and ":" in hook_name:
                    hook_event = hook_name.split(":", 1)[0]
                tool_use_id = attachment.get("toolUseID")
                if not isinstance(tool_use_id, str):
                    tool_use_id = None
                command = attachment.get("command")
                if not isinstance(command, str):
                    command = None
                content_text = attachment.get("content")
                if not isinstance(content_text, str):
                    content_text = ""
                stdout_text = attachment.get("stdout")
                if not isinstance(stdout_text, str):
                    stdout_text = ""
                exit_code = attachment.get("exitCode")
                if not isinstance(exit_code, int) or isinstance(exit_code, bool):
                    exit_code = None
                add(
                    "hook_run",
                    timestamp,
                    session,
                    line_number,
                    call_id=tool_use_id,
                    hook_name=hook_name,
                    hook_event=hook_event,
                    command=command,
                    content=content_text,
                    stdout=stdout_text,
                    exit_code=exit_code,
                    attachment_type=attachment["type"],
                )
            else:
                add("other", timestamp, session, line_number)
        elif line_type == "system" and line_object.get("subtype") == "stop_hook_summary":
            commands = []
            infos = line_object.get("hookInfos")
            if isinstance(infos, list):
                for info in infos:
                    if isinstance(info, dict) and isinstance(info.get("command"), str):
                        commands.append(info["command"])
            add(
                "stop",
                timestamp,
                session,
                line_number,
                commands=tuple(commands),
                prevented_continuation=bool(
                    line_object.get("preventedContinuation", False)
                ),
                has_output=bool(line_object.get("hasOutput", False)),
            )
        else:
            add("other", timestamp, session, line_number)
    return records
