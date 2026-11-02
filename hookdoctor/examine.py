"""The examination that classifies configured hooks against a trace."""

from __future__ import annotations

import json
import re
from datetime import datetime, timezone

from .errors import HookdoctorError, ReportValidationError

REPORT_FORMAT = "hookdoctor-report/1"

CLASS_FIRING = "firing"
CLASS_SILENT = "silent"
CLASS_WITHOUT_EFFECT = "firing-without-effect"

_PLAIN_MATCHER = re.compile(r"[A-Za-z0-9_]+\Z")
_PAIRED_EVENTS = {"PreToolUse": "PostToolUse", "PostToolUse": "PreToolUse"}
_SUBAGENT_PREFIX = "Subagent"


def is_plain_matcher(matcher):
    return isinstance(matcher, str) and _PLAIN_MATCHER.fullmatch(matcher) is not None


def matcher_matches(matcher, tool_name):
    """Apply the probed harness matcher rule to one tool name.

    A matcher never matches a missing tool name. The agreement of
    a subject carrying no tool with every matcher is a subject
    level rule and is decided in _subject_matches, not here.
    """
    if tool_name is None:
        return False
    if matcher is None or matcher == "" or matcher == "*":
        return True
    if is_plain_matcher(matcher):
        return matcher == tool_name
    try:
        return re.fullmatch(matcher, tool_name) is not None
    except re.error as exc:
        raise HookdoctorError("a matcher is not a valid pattern") from exc


def _parse_instant(text):
    """Read one timestamp spelling as an instant, or None."""
    if not isinstance(text, str) or not text:
        return None
    candidate = text
    if candidate.endswith(("Z", "z")):
        candidate = candidate[:-1] + "+00:00"
    try:
        value = datetime.fromisoformat(candidate)
    except ValueError:
        return None
    if value.tzinfo is None:
        value = value.replace(tzinfo=timezone.utc)
    return value


def _window_bound(text, label):
    if text is None:
        return None
    instant = _parse_instant(text)
    if instant is None:
        raise HookdoctorError(f"the window {label} is not a timestamp")
    return instant


def _in_window(record, start_instant, end_instant):
    if start_instant is None and end_instant is None:
        return True
    instant = _parse_instant(record.timestamp)
    if instant is None:
        return False
    if start_instant is not None and instant < start_instant:
        return False
    if end_instant is not None and instant >= end_instant:
        return False
    return True


def _parsed_output(text):
    if not text:
        return None
    try:
        value = json.loads(text)
    except (json.JSONDecodeError, ValueError):
        return None
    return value if isinstance(value, dict) else None


_ERROR_ATTACHMENT_TYPES = frozenset(
    {"hook_blocking_error", "hook_non_blocking_error"}
)


def _output_facts(record):
    """Read the effect bearing fields out of one hook run record."""
    facts = {"denial": False, "modification": False, "consumed_output": False}
    error_record = record.attachment_type in _ERROR_ATTACHMENT_TYPES
    if record.attachment_type == "hook_blocking_error":
        facts["denial"] = True
    if record.content and not error_record:
        facts["consumed_output"] = True
    for text in (record.content, record.stdout):
        output = _parsed_output(text)
        if output is None:
            continue
        candidates = [output]
        specific = output.get("hookSpecificOutput")
        if isinstance(specific, dict):
            candidates.append(specific)
        for candidate in candidates:
            if candidate.get("decision") == "block":
                facts["denial"] = True
            if candidate.get("permissionDecision") == "deny":
                facts["denial"] = True
            if "updatedInput" in candidate:
                facts["modification"] = True
            context = candidate.get("additionalContext")
            if isinstance(context, str) and context and not error_record:
                facts["consumed_output"] = True
    return facts


_DENIAL_WORD = re.compile(r"\bdenied\b")
_NEGATED_DENIAL = re.compile(
    r"\bnot\s+denied\b|n['\u2019]t\s+denied\b|\bnever\s+denied\b"
)
_DENIAL_ATTRIBUTION = re.compile(r"\bdenied\s+by\s+([a-z0-9_.\-/]+)")
_NAME_SHAPE_CHARS = "-._/"


def _denial_attribution(lowered_text):
    """Return the name shaped party a denial is attributed to."""
    match = _DENIAL_ATTRIBUTION.search(lowered_text)
    if match is None:
        return None
    party = match.group(1)
    if any(char in party for char in _NAME_SHAPE_CHARS):
        return party
    return None


def _denial_by_outcome_text(hook, text):
    """Apply the denial reading of HD2-5 to one outcome text."""
    lowered = text.lower()
    if _NEGATED_DENIAL.search(lowered):
        return False
    attribution = _denial_attribution(lowered)
    if hook.command in text:
        return attribution is None or attribution == hook.command.lower()
    if _DENIAL_WORD.search(lowered) is None:
        return False
    return attribution is None


def _name_form_agrees(hook, name):
    """Decide whether a run record name is this hook own name form.

    The forms are the command standing alone, the event alone for
    a hook whose matcher is empty or missing, and the event joined
    to the hook matcher or to the hook command. Returns None when
    the record carries no name at all.
    """
    if not name:
        return None
    if name == hook.command:
        return True
    if name == hook.event:
        return hook.matcher is None or hook.matcher == ""
    prefix = hook.event + ":"
    if name.startswith(prefix):
        suffix = name[len(prefix):]
        return suffix == (hook.matcher or "") or suffix == hook.command
    return False


def _run_agrees(hook, record):
    """Decide whether a hook run record identifies this hook.

    A record carrying both a command and a name form agrees only
    when both identify the same hook, so a record whose two
    identifiers point at different hooks agrees with no hook. A
    record carrying one identifier is decided by that one.
    """
    if record.hook_event != hook.event:
        return False
    verdicts = []
    if record.command:
        verdicts.append(record.command == hook.command)
    name_verdict = _name_form_agrees(hook, record.hook_name)
    if name_verdict is not None:
        verdicts.append(name_verdict)
    return bool(verdicts) and all(verdicts)


def _levenshtein(left, right):
    if left == right:
        return 0
    previous = list(range(len(right) + 1))
    for i, a in enumerate(left, start=1):
        current = [i]
        for j, b in enumerate(right, start=1):
            current.append(
                min(
                    previous[j] + 1,
                    current[j - 1] + 1,
                    previous[j - 1] + (0 if a == b else 1),
                )
            )
        previous = current
    return previous[-1]


def _name_form_key(name):
    return name.lower().replace("-", "").replace("_", "")


class _Subject:
    """One traced subject a hook can match, run on, or nearly match."""

    def __init__(self, event, tool_name, record, outcome=None):
        self.event = event
        self.tool_name = tool_name
        self.record = record
        self.outcome = outcome


def _subjects(records):
    outcome_queues = {}
    for record in records:
        if record.kind == "tool_outcome":
            outcome_queues.setdefault(record.call_id, []).append(record)
    consumed = {}
    subjects = []
    for record in records:
        if record.kind == "tool_call":
            queue = outcome_queues.get(record.call_id, [])
            position = consumed.get(record.call_id, 0)
            outcome = queue[position] if position < len(queue) else None
            if outcome is not None:
                consumed[record.call_id] = position + 1
            subjects.append(
                _Subject("PreToolUse", record.tool_name, record, outcome=outcome)
            )
            if outcome is not None:
                subjects.append(
                    _Subject(
                        "PostToolUse", record.tool_name, outcome, outcome=outcome
                    )
                )
        elif record.kind == "prompt":
            subjects.append(_Subject("UserPromptSubmit", None, record))
        elif record.kind == "stop":
            subjects.append(_Subject("Stop", None, record))
        elif record.kind == "hook_run":
            event = record.hook_event
            if event is not None:
                subjects.append(_Subject(event, None, record))
    return subjects


def _subject_matches(hook, subject):
    if subject.event != hook.event:
        return False
    if subject.record.kind == "hook_run":
        return _run_agrees(hook, subject.record)
    if subject.tool_name is None:
        # An event with no tool subject agrees with every matcher.
        return True
    return matcher_matches(hook.matcher, subject.tool_name)


def _subject_ran(hook, subject):
    if subject.record.kind == "hook_run":
        return _run_agrees(hook, subject.record)
    if subject.event == "Stop":
        return hook.command in subject.record.commands
    return True


def _subject_effects(hook, subject):
    effects = {"denial": 0, "modification": 0, "consumed_output": 0}
    record = subject.record
    if record.kind == "hook_run":
        facts = _output_facts(record)
        for kind, stands in facts.items():
            if stands:
                effects[kind] = 1
        return effects
    if record.kind == "stop":
        if record.prevented_continuation:
            effects["denial"] = 1
        if record.has_output:
            effects["consumed_output"] = 1
        return effects
    if subject.event == "PreToolUse" and record.kind == "tool_call":
        outcome = subject.outcome
        if outcome is not None and outcome.is_error:
            if _denial_by_outcome_text(hook, outcome.text or ""):
                effects["denial"] = 1
        return effects
    return effects


def _near_miss_reason(hook, subject):
    if subject.event == hook.event and subject.tool_name and is_plain_matcher(
        hook.matcher
    ):
        tool = subject.tool_name
        if tool != hook.matcher and _name_form_key(tool) == _name_form_key(
            hook.matcher
        ):
            return "name-form"
        if _levenshtein(hook.matcher, tool) == 1:
            return "one-letter"
        return None
    paired = _PAIRED_EVENTS.get(hook.event)
    if (
        paired is not None
        and subject.event == paired
        and subject.tool_name
        and matcher_matches(hook.matcher, subject.tool_name)
    ):
        return "paired-event"
    if subject.event == _SUBAGENT_PREFIX + hook.event or hook.event == (
        _SUBAGENT_PREFIX + subject.event
    ):
        return "subagent-prefix"
    return None


def _near_miss_detail(reason, hook, subject):
    tool = subject.tool_name or "no tool"
    if reason == "name-form":
        return (
            f"tool {tool} differs from matcher {hook.matcher} "
            "by letter case or separator form only"
        )
    if reason == "one-letter":
        return f"tool {tool} is one letter away from matcher {hook.matcher}"
    if reason == "paired-event":
        return (
            f"event {subject.event} is the paired event of {hook.event} "
            f"for tool {tool} the matcher matches"
        )
    return f"event {subject.event} differs from {hook.event} by the Subagent prefix"


def examine_hooks(
    hooks, records, window_start=None, window_end=None, session_sources=()
):
    """Return one entry per hook and the near miss list for silent hooks."""
    start_instant = _window_bound(window_start, "start")
    end_instant = _window_bound(window_end, "end")
    windowed = [
        record for record in records if _in_window(record, start_instant, end_instant)
    ]
    subjects = _subjects(windowed)

    def source_of(record):
        if record.file_index < len(session_sources):
            return session_sources[record.file_index]
        return record.session
    entries = []
    near_misses = []
    for hook in hooks:
        matched = 0
        ran_subjects = []
        effect_counts = {"denial": 0, "modification": 0, "consumed_output": 0}
        for subject in subjects:
            if not _subject_matches(hook, subject):
                continue
            matched += 1
            if not _subject_ran(hook, subject):
                continue
            ran_subjects.append(subject)
            effects = _subject_effects(hook, subject)
            for kind in effect_counts:
                effect_counts[kind] += effects[kind]
        total_effects = sum(effect_counts.values())
        if not ran_subjects:
            hook_class = CLASS_SILENT
        elif total_effects > 0:
            hook_class = CLASS_FIRING
        else:
            hook_class = CLASS_WITHOUT_EFFECT
        stamps = []
        for subject in ran_subjects:
            instant = _parse_instant(subject.record.timestamp)
            if instant is not None:
                stamps.append((instant, subject.record.timestamp))
        entries.append(
            {
                "id": hook.id,
                "event": hook.event,
                "matcher": hook.matcher,
                "command": hook.command,
                "hook_type": hook.hook_type,
                "class": hook_class,
                "evidence": {
                    "matched": matched,
                    "ran": len(ran_subjects),
                    "effects": effect_counts,
                    "first_ran_at": min(stamps)[1] if stamps else None,
                    "last_ran_at": max(stamps)[1] if stamps else None,
                },
            }
        )
        if hook_class == CLASS_SILENT:
            seen = set()
            for subject in subjects:
                reason = _near_miss_reason(hook, subject)
                if reason is None:
                    continue
                record = subject.record
                key = (
                    reason,
                    subject.event,
                    subject.tool_name,
                    record.file_index,
                    record.line,
                )
                if key in seen:
                    continue
                seen.add(key)
                near_misses.append(
                    {
                        "hook_id": hook.id,
                        "reason": reason,
                        "event": subject.event,
                        "tool": subject.tool_name,
                        "timestamp": record.timestamp,
                        "source": source_of(record),
                        "line": record.line,
                        "detail": _near_miss_detail(reason, hook, subject),
                    }
                )
    order = {hook.id: index for index, hook in enumerate(hooks)}
    earliest = datetime.min.replace(tzinfo=timezone.utc)
    near_misses.sort(
        key=lambda miss: (
            order[miss["hook_id"]],
            _parse_instant(miss["timestamp"]) or earliest,
            miss["source"] or "",
            miss["line"],
        )
    )
    return entries, near_misses


def build_report(
    hooks,
    records,
    config_source,
    session_sources,
    window_start=None,
    window_end=None,
):
    entries, near_misses = examine_hooks(
        hooks, records, window_start, window_end, session_sources=session_sources
    )
    return {
        "format": REPORT_FORMAT,
        "config_source": config_source,
        "session_sources": list(session_sources),
        "window": {"start": window_start, "end": window_end},
        "hooks": entries,
        "near_misses": near_misses,
    }


def validate_report(
    report,
    hooks,
    records,
    config_source,
    session_sources,
    window_start=None,
    window_end=None,
):
    expected = build_report(
        hooks, records, config_source, session_sources, window_start, window_end
    )
    if report != expected:
        raise ReportValidationError("the report disagrees with the examination")
    return True
