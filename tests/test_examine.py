import json

from hookdoctor.examine import build_report, matcher_matches
from hookdoctor.ingest import parse_config, parse_session_text

from synthetic_data import (
    CONFIG,
    config_with,
    group,
    hook_run_line,
    prompt_line,
    session_text,
    stop_line,
    tool_result_line,
    tool_use_line,
)

T1 = "2026-01-01T09:00:00Z"
T2 = "2026-01-01T09:05:00Z"
T3 = "2026-01-01T09:10:00Z"


def report_for(config, text, **kwargs):
    hooks = parse_config(config)
    records = parse_session_text(text)
    return build_report(hooks, records, "synthetic-settings.json", ["synthetic-session.jsonl"], **kwargs)


def entry(report, command):
    for hook in report["hooks"]:
        if hook["command"] == command:
            return hook
    raise AssertionError(f"no entry for {command}")


def misses_for(report, hook_id):
    return [m for m in report["near_misses"] if m["hook_id"] == hook_id]


def test_plain_matcher_is_exact_and_case_sensitive():
    assert matcher_matches("Bash", "Bash") is True
    assert matcher_matches("Bash", "bash") is False
    assert matcher_matches("Bash", "Bashful") is False
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("bash", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["matched"] == 0
    assert hook["class"] == "silent"


def test_regex_matcher_must_match_the_whole_name():
    config = config_with("PreToolUse", [group("Edit|Write", "lint-edit.py")])
    only_editor = session_text(
        tool_use_line("Editor", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    hook = entry(report_for(config, only_editor), "lint-edit.py")
    assert hook["evidence"]["matched"] == 0
    assert hook["class"] == "silent"
    with_edit = session_text(
        tool_use_line("Edit", "call-1", T1),
        tool_result_line("call-1", T2),
        tool_use_line("Write", "call-2", T3),
        tool_result_line("call-2", T3),
    )
    hook = entry(report_for(config, with_edit), "lint-edit.py")
    assert hook["evidence"]["matched"] == 2
    assert hook["evidence"]["ran"] == 2


def test_empty_and_missing_matchers_match_every_tool():
    config = {
        "hooks": {
            "PreToolUse": [
                group("", "audit-all.sh"),
                group(None, "missing-matcher.sh", include_matcher=False),
            ]
        }
    }
    text = session_text(
        tool_use_line("Read", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    report = report_for(config, text)
    assert entry(report, "audit-all.sh")["evidence"]["ran"] == 1
    assert entry(report, "missing-matcher.sh")["evidence"]["ran"] == 1


def test_non_tool_events_agree_with_every_matcher():
    config = {
        "hooks": {
            "Stop": [group("Bash", "wrap-up.sh")],
            "UserPromptSubmit": [group("Anything", "note-prompt.sh")],
        }
    }
    text = session_text(prompt_line(ts=T1), stop_line(T2))
    report = report_for(config, text)
    assert entry(report, "wrap-up.sh")["evidence"]["ran"] == 1
    assert entry(report, "note-prompt.sh")["evidence"]["ran"] == 1


def test_pretool_hook_runs_on_a_matched_call():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(tool_use_line("Bash", "call-1", T1))
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["matched"] == 1
    assert hook["evidence"]["ran"] == 1
    assert hook["class"] == "firing-without-effect"


def test_posttool_hook_needs_an_outcome_to_have_run():
    config = config_with("PostToolUse", [group("Bash", "log-bash.sh")])
    no_outcome = session_text(tool_use_line("Bash", "call-1", T1))
    hook = entry(report_for(config, no_outcome), "log-bash.sh")
    assert hook["evidence"]["matched"] == 0
    assert hook["evidence"]["ran"] == 0
    assert hook["class"] == "silent"
    with_outcome = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    hook = entry(report_for(config, with_outcome), "log-bash.sh")
    assert hook["evidence"]["matched"] == 1
    assert hook["evidence"]["ran"] == 1


def test_stop_hook_runs_when_the_summary_names_its_command():
    config = config_with("Stop", [group(None, "wrap-up.sh", include_matcher=False)])
    named = session_text(stop_line(T1, commands=("wrap-up.sh",)))
    hook = entry(report_for(config, named), "wrap-up.sh")
    assert hook["evidence"]["ran"] == 1
    unnamed = session_text(stop_line(T1, commands=("other.sh",)))
    hook = entry(report_for(config, unnamed), "wrap-up.sh")
    assert hook["evidence"]["matched"] == 1
    assert hook["evidence"]["ran"] == 0
    assert hook["class"] == "silent"


def test_hook_run_record_counts_as_a_run():
    config = config_with("PostToolUse", [group("Bash", "log-bash.sh")])
    text = session_text(
        hook_run_line("PostToolUse:Bash", "PostToolUse", T1, call_id="call-9")
    )
    hook = entry(report_for(config, text), "log-bash.sh")
    assert hook["evidence"]["ran"] == 1
    assert hook["class"] == "firing-without-effect"


def test_denial_by_outcome_text_and_by_blocking_record():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    denied = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line(
            "call-1", T2, text="Permission denied by the guard hook", is_error=True
        ),
    )
    hook = entry(report_for(config, denied), "guard-bash.sh")
    assert hook["evidence"]["effects"]["denial"] == 1
    assert hook["class"] == "firing"
    blocked = session_text(
        hook_run_line(
            "PreToolUse:Bash",
            "PreToolUse",
            T1,
            attachment_type="hook_blocking_error",
            call_id="call-1",
        )
    )
    hook = entry(report_for(config, blocked), "guard-bash.sh")
    assert hook["evidence"]["effects"]["denial"] == 1
    assert hook["class"] == "firing"


def test_denial_by_output_decision_and_by_prevented_continuation():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    stdout = json.dumps(
        {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": "synthetic refusal",
            }
        }
    )
    text = session_text(
        hook_run_line("PreToolUse:Bash", "PreToolUse", T1, stdout=stdout, call_id="call-1")
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["effects"]["denial"] == 1
    stop_config = config_with("Stop", [group(None, "wrap-up.sh", include_matcher=False)])
    prevented = session_text(stop_line(T1, prevented=True))
    hook = entry(report_for(stop_config, prevented), "wrap-up.sh")
    assert hook["evidence"]["effects"]["denial"] == 1
    assert hook["class"] == "firing"


def test_modification_by_replacement_input():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    stdout = json.dumps(
        {"hookSpecificOutput": {"updatedInput": {"command": "synthetic-safe"}}}
    )
    text = session_text(
        hook_run_line("PreToolUse:Bash", "PreToolUse", T1, stdout=stdout, call_id="call-1")
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["effects"]["modification"] == 1
    assert hook["evidence"]["effects"]["denial"] == 0
    assert hook["class"] == "firing"


def test_consumed_output_by_context_and_additional_context():
    prompt_config = config_with(
        "UserPromptSubmit", [group(None, "note-prompt.sh", include_matcher=False)]
    )
    text = session_text(
        hook_run_line(
            "UserPromptSubmit", "UserPromptSubmit", T1, content="remember the synthetic plan"
        )
    )
    hook = entry(report_for(prompt_config, text), "note-prompt.sh")
    assert hook["evidence"]["effects"]["consumed_output"] == 1
    assert hook["class"] == "firing"
    bash_config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    stdout = json.dumps(
        {"hookSpecificOutput": {"additionalContext": "synthetic context"}}
    )
    text = session_text(
        hook_run_line("PreToolUse:Bash", "PreToolUse", T1, stdout=stdout, call_id="call-1")
    )
    hook = entry(report_for(bash_config, text), "guard-bash.sh")
    assert hook["evidence"]["effects"]["consumed_output"] == 1
    stop_config = config_with("Stop", [group(None, "wrap-up.sh", include_matcher=False)])
    with_output = session_text(stop_line(T1, has_output=True))
    hook = entry(report_for(stop_config, with_output), "wrap-up.sh")
    assert hook["evidence"]["effects"]["consumed_output"] == 1


def test_plain_success_is_no_effect():
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2, text="3 passed"),
    )
    report = report_for(CONFIG, text)
    guard = entry(report, "guard-bash.sh")
    assert guard["evidence"]["effects"] == {
        "denial": 0,
        "modification": 0,
        "consumed_output": 0,
    }
    assert guard["class"] == "firing-without-effect"
    assert entry(report, "log-bash.sh")["class"] == "firing-without-effect"


def test_firing_hook_with_a_denial():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2, text="the hook denied this call", is_error=True),
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["class"] == "firing"


def test_hook_that_ran_on_every_event_and_changed_nothing_is_without_effect():
    config = config_with("PreToolUse", [group("", "audit-all.sh")])
    lines = []
    for index, tool in enumerate(["Bash", "Read", "Edit"]):
        lines.append(tool_use_line(tool, f"call-{index}", T1))
        lines.append(tool_result_line(f"call-{index}", T2))
    hook = entry(report_for(config, session_text(*lines)), "audit-all.sh")
    assert hook["evidence"]["ran"] == 3
    assert hook["class"] == "firing-without-effect"


def test_silent_hook_matched_nothing():
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2),
        tool_use_line("Read", "call-2", T3),
        tool_result_line("call-2", T3),
    )
    hook = entry(report_for(CONFIG, text), "lint-edit.py")
    assert hook["evidence"]["matched"] == 0
    assert hook["evidence"]["ran"] == 0
    assert hook["class"] == "silent"


def test_every_hook_gets_exactly_one_class():
    text = session_text(
        prompt_line(ts=T1),
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2, text="denied by policy", is_error=True),
        stop_line(T3),
    )
    report = report_for(CONFIG, text)
    hooks = parse_config(CONFIG)
    assert len(report["hooks"]) == len(hooks)
    assert [h["id"] for h in report["hooks"]] == [h.id for h in hooks]
    assert {h["class"] for h in report["hooks"]} <= {
        "firing",
        "silent",
        "firing-without-effect",
    }


def test_evidence_counts_and_dates_come_from_the_trace():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2, text="denied once", is_error=True),
        tool_use_line("Read", "call-2", T2),
        tool_result_line("call-2", T2),
        tool_use_line("Bash", "call-3", T3),
        tool_result_line("call-3", T3),
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    evidence = hook["evidence"]
    assert evidence["matched"] == 2
    assert evidence["ran"] == 2
    assert evidence["effects"]["denial"] == 1
    assert evidence["first_ran_at"] == T1
    assert evidence["last_ran_at"] == T3


def test_hook_that_never_ran_has_no_dates():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(tool_use_line("Read", "call-1", T1))
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["first_ran_at"] is None
    assert hook["evidence"]["last_ran_at"] is None


def test_near_miss_by_name_form():
    config = config_with("PreToolUse", [group("web_fetch", "fetch-guard.sh")])
    text = session_text(
        tool_use_line("web-fetch", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    report = report_for(config, text)
    hook = entry(report, "fetch-guard.sh")
    assert hook["class"] == "silent"
    misses = misses_for(report, hook["id"])
    assert len(misses) == 1
    assert misses[0]["reason"] == "name-form"
    assert misses[0]["tool"] == "web-fetch"
    assert misses[0]["timestamp"] == T1
    case_config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    case_text = session_text(
        tool_use_line("bash", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    report = report_for(case_config, case_text)
    misses = misses_for(report, "PreToolUse:0:0")
    assert len(misses) == 1
    assert misses[0]["reason"] == "name-form"


def test_near_miss_by_one_letter():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("Bsh", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    report = report_for(config, text)
    misses = misses_for(report, "PreToolUse:0:0")
    assert len(misses) == 1
    assert misses[0]["reason"] == "one-letter"
    assert misses[0]["tool"] == "Bsh"


def test_near_miss_by_paired_event():
    config = config_with("PostToolUse", [group("Bash", "log-bash.sh")])
    text = session_text(tool_use_line("Bash", "call-1", T1))
    report = report_for(config, text)
    hook = entry(report, "log-bash.sh")
    assert hook["class"] == "silent"
    misses = misses_for(report, hook["id"])
    assert len(misses) == 1
    assert misses[0]["reason"] == "paired-event"
    assert misses[0]["event"] == "PreToolUse"


def test_near_miss_by_subagent_prefix():
    config = config_with("Stop", [group(None, "wrap-up.sh", include_matcher=False)])
    text = session_text(
        hook_run_line("SubagentStop", "SubagentStop", T1, content="")
    )
    report = report_for(config, text)
    hook = entry(report, "wrap-up.sh")
    assert hook["class"] == "silent"
    misses = misses_for(report, hook["id"])
    assert len(misses) == 1
    assert misses[0]["reason"] == "subagent-prefix"
    reverse_config = config_with(
        "SubagentStop", [group(None, "sub-wrap.sh", include_matcher=False)]
    )
    reverse_text = session_text(stop_line(T1, commands=("other.sh",)))
    report = report_for(reverse_config, reverse_text)
    hook = entry(report, "sub-wrap.sh")
    assert hook["class"] == "silent"
    misses = misses_for(report, hook["id"])
    assert len(misses) == 1
    assert misses[0]["reason"] == "subagent-prefix"


def test_a_match_is_never_a_near_miss_and_near_misses_change_no_class():
    config = {
        "hooks": {
            "PreToolUse": [
                group("Bash", "guard-bash.sh"),
                group("Bashh", "typo-guard.sh"),
            ]
        }
    }
    text = session_text(
        tool_use_line("Bash", "call-1", T1),
        tool_result_line("call-1", T2),
    )
    report = report_for(config, text)
    guard = entry(report, "guard-bash.sh")
    assert guard["class"] == "firing-without-effect"
    assert misses_for(report, guard["id"]) == []
    typo = entry(report, "typo-guard.sh")
    assert typo["class"] == "silent"
    misses = misses_for(report, typo["id"])
    assert len(misses) == 1
    assert misses[0]["reason"] == "one-letter"
    assert typo["evidence"]["matched"] == 0


def test_match_outside_the_window_leaves_the_hook_silent():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("Bash", "call-1", "2026-01-03T09:00:00Z"),
        tool_result_line("call-1", "2026-01-03T09:05:00Z"),
    )
    report = report_for(
        config,
        text,
        window_start="2026-01-01T00:00:00Z",
        window_end="2026-01-02T00:00:00Z",
    )
    hook = entry(report, "guard-bash.sh")
    assert hook["class"] == "silent"
    assert hook["evidence"]["matched"] == 0
    assert report["near_misses"] == []


def test_near_miss_inside_the_window_is_shown_for_an_outside_match():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("bash", "call-1", "2026-01-01T10:00:00Z"),
        tool_result_line("call-1", "2026-01-01T10:05:00Z"),
        tool_use_line("Bash", "call-2", "2026-01-03T09:00:00Z"),
        tool_result_line("call-2", "2026-01-03T09:05:00Z"),
    )
    report = report_for(
        config,
        text,
        window_start="2026-01-01T00:00:00Z",
        window_end="2026-01-02T00:00:00Z",
    )
    hook = entry(report, "guard-bash.sh")
    assert hook["class"] == "silent"
    misses = misses_for(report, hook["id"])
    assert len(misses) == 1
    assert misses[0]["reason"] == "name-form"
    assert misses[0]["timestamp"] == "2026-01-01T10:00:00Z"


def test_unbounded_window_counts_everything():
    config = config_with("PreToolUse", [group("Bash", "guard-bash.sh")])
    text = session_text(
        tool_use_line("Bash", "call-1", "2026-01-03T09:00:00Z"),
        tool_result_line("call-1", "2026-01-03T09:05:00Z"),
    )
    hook = entry(report_for(config, text), "guard-bash.sh")
    assert hook["evidence"]["ran"] == 1
    windowed = report_for(config, text, window_start="2026-01-01T00:00:00Z")
    hook = entry(windowed, "guard-bash.sh")
    assert hook["evidence"]["ran"] == 1
