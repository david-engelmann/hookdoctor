//! The pins of slice 9, the eventless unconfigured runs,
//! over synthetic settings and session files the suite
//! writes itself. The list of slice 8 gains the hook run
//! records that name no event and identify no configured
//! hook, standing in the list with the event of their
//! entry as null.

use hookdoctor::examine::{build_report, validate_report};
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::report::render_markdown;

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const SHARED_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "shared.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "shared.sh"}]}]}}"#;

const CALL: &str = r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#;
const OUTCOME: &str = r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#;
const EVENTLESS_ROGUE: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","command":"rogue-hook.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const ROGUE_RUN: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","hookEvent":"PreToolUse","command":"rogue-hook.sh","content":"rogue context","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;

fn report_with(settings: &str, session_text: &str, window_start: Option<&str>) -> JsonValue {
    let document = parse_json(settings).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(session_text, 0).expect("the session reads");
    build_report(
        &hooks,
        &records,
        "settings.json",
        &["session.jsonl".to_string()],
        window_start,
        None,
    )
    .expect("the report builds")
}

fn report(session_text: &str, window_start: Option<&str>) -> JsonValue {
    report_with(SETTINGS, session_text, window_start)
}

fn session(lines: &[&str]) -> String {
    lines.join("\n")
}

#[test]
fn test_an_eventless_unconfigured_run_record_stands_in_the_report() {
    let report = report(&session(&[CALL, OUTCOME, EVENTLESS_ROGUE]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::Null));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(
        entry.get("command"),
        Some(&JsonValue::text("rogue-hook.sh"))
    );
    assert_eq!(entry.get("name"), Some(&JsonValue::text("rogue-hook.sh")));
    assert_eq!(
        entry.get("timestamp"),
        Some(&JsonValue::text("2026-01-01T00:00:04Z"))
    );
    assert_eq!(entry.get("source"), Some(&JsonValue::text("session.jsonl")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_an_eventless_record_naming_a_bare_event_word_stands_in_the_report_by_its_name() {
    let bare_word = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, bare_word]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::Null));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::Null));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_an_eventless_unconfigured_run_and_an_evented_one_stand_in_trace_order() {
    let report = report(&session(&[CALL, OUTCOME, EVENTLESS_ROGUE, ROGUE_RUN]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].get("event"), Some(&JsonValue::Null));
    assert_eq!(runs[0].get("line"), Some(&JsonValue::Int(3)));
    assert_eq!(runs[1].get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(runs[1].get("line"), Some(&JsonValue::Int(4)));
    let report = report(&session(&[CALL, OUTCOME, ROGUE_RUN, EVENTLESS_ROGUE]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(runs[0].get("line"), Some(&JsonValue::Int(3)));
    assert_eq!(runs[1].get("event"), Some(&JsonValue::Null));
    assert_eq!(runs[1].get("line"), Some(&JsonValue::Int(4)));
}

#[test]
fn test_a_tampered_eventless_unconfigured_entry_refuses_the_report_validation() {
    let text = session(&[CALL, OUTCOME, EVENTLESS_ROGUE]);
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&text, 0).expect("the session reads");
    let genuine = build_report(
        &hooks,
        &records,
        "settings.json",
        &["session.jsonl".to_string()],
        None,
        None,
    )
    .expect("the report builds");
    let spelled = hookdoctor::render::render_json(&genuine);
    let tampered_text = spelled.replace("rogue-hook.sh", "other-hook.sh");
    let tampered = parse_json(&tampered_text).expect("the tampered report parses");
    let refused = validate_report(
        &tampered,
        &hooks,
        &records,
        "settings.json",
        &["session.jsonl".to_string()],
        None,
        None,
    );
    assert!(
        refused.is_err(),
        "a tampered eventless entry refuses validation"
    );
}

#[test]
fn test_the_markdown_renders_an_eventless_unconfigured_run() {
    let report = report(&session(&[CALL, OUTCOME, EVENTLESS_ROGUE]), None);
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
}

#[test]
fn test_an_eventless_run_record_identifying_a_configured_hook_stands_nowhere_in_the_unconfigured_list(
) {
    let identified = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"guard.sh","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, identified]), None);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "a configured hook's eventless run names no unconfigured run"
    );
}

#[test]
fn test_an_eventless_run_record_identifying_hooks_of_two_events_stands_nowhere_in_the_unconfigured_list(
) {
    let shared = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"shared.sh","command":"shared.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report_with(SHARED_SETTINGS, &session(&[CALL, OUTCOME, shared]), None);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "a run identifying hooks of two events names no unconfigured run"
    );
}

#[test]
fn test_an_eventless_run_record_with_disagreeing_identifiers_stands_nowhere_in_the_unconfigured_list(
) {
    let disagreeing = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"wrap.sh","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, disagreeing]), None);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "each identifier names a configured hook, so the record is no unconfigured run"
    );
}

#[test]
fn test_an_eventless_run_record_carrying_no_identifier_stands_nowhere_in_the_unconfigured_list() {
    let anonymous = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, anonymous]), None);
    assert!(report.get("unconfigured_runs").is_none());
}

#[test]
fn test_an_eventless_unconfigured_run_outside_the_window_stands_nowhere_in_the_unconfigured_list() {
    let report = report(
        &session(&[CALL, OUTCOME, EVENTLESS_ROGUE]),
        Some("2026-01-02T00:00:00Z"),
    );
    assert!(report.get("unconfigured_runs").is_none());
}
