//! The pins of slice 8, the unconfigured runs, over
//! synthetic settings and session files the suite writes
//! itself. The list names the hooks the traces show running
//! that the configuration does not declare, standing in the
//! report only where such runs stand.

use hookdoctor::examine::{build_report, validate_report};
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::report::render_markdown;

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const CALL: &str = r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#;
const OUTCOME: &str = r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#;
const ROGUE_RUN: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","hookEvent":"PreToolUse","command":"rogue-hook.sh","content":"rogue context","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const STOP_LINE: &str = r#"{"type":"system","subtype":"stop_hook_summary","timestamp":"2026-01-01T00:00:05Z","sessionId":"s1","hookCount":2,"hookInfos":[{"command":"wrap.sh"},{"command":"rogue-stop.sh"}],"hookErrors":[],"preventedContinuation":false,"hasOutput":true}"#;

fn report(session_text: &str, window_start: Option<&str>) -> JsonValue {
    let document = parse_json(SETTINGS).expect("the settings parse");
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

fn session(lines: &[&str]) -> String {
    lines.join("\n")
}

#[test]
fn test_an_unconfigured_run_record_stands_in_the_report() {
    let report = report(&session(&[CALL, OUTCOME, ROGUE_RUN]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::text("PreToolUse")));
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
fn test_a_configured_hooks_run_record_stands_nowhere_in_the_unconfigured_list() {
    let configured_run = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse:Bash","hookEvent":"PreToolUse","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, configured_run]), None);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "a configured hook's run names no unconfigured run"
    );
}

#[test]
fn test_an_unconfigured_stop_command_stands_in_the_report() {
    let report = report(&session(&[CALL, OUTCOME, STOP_LINE]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1, "the configured wrap.sh names no entry");
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::text("Stop")));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("stop")));
    assert_eq!(
        entry.get("command"),
        Some(&JsonValue::text("rogue-stop.sh"))
    );
    assert_eq!(entry.get("name"), Some(&JsonValue::Null));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_a_report_with_no_unconfigured_runs_carries_no_unconfigured_member() {
    let plain_stop = r#"{"type":"system","subtype":"stop_hook_summary","timestamp":"2026-01-01T00:00:05Z","sessionId":"s1","hookCount":1,"hookInfos":[{"command":"wrap.sh"}],"hookErrors":[],"preventedContinuation":false,"hasOutput":false}"#;
    let report = report(&session(&[CALL, OUTCOME, plain_stop]), None);
    assert!(report.get("unconfigured_runs").is_none());
}

#[test]
fn test_a_run_record_with_disagreeing_identifiers_stands_nowhere_in_the_unconfigured_list() {
    let disagreeing = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"wrap.sh","hookEvent":"PreToolUse","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, disagreeing]), None);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "both identifiers name configured hooks, so the record is no unconfigured run"
    );
}

#[test]
fn test_an_unconfigured_run_outside_the_window_stands_nowhere_in_the_unconfigured_list() {
    let report = report(
        &session(&[CALL, OUTCOME, ROGUE_RUN]),
        Some("2026-01-02T00:00:00Z"),
    );
    assert!(report.get("unconfigured_runs").is_none());
}

#[test]
fn test_a_tampered_unconfigured_list_refuses_the_report_validation() {
    let text = session(&[CALL, OUTCOME, ROGUE_RUN]);
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
    assert!(refused.is_err(), "a tampered list refuses validation");
}

#[test]
fn test_the_markdown_renders_the_unconfigured_runs() {
    let report = report(&session(&[CALL, OUTCOME, ROGUE_RUN, STOP_LINE]), None);
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at PreToolUse in session.jsonl on line 3."),
        "{markdown}"
    );
    assert!(
        markdown.contains("- rogue-stop.sh ran at Stop in session.jsonl on line 4."),
        "{markdown}"
    );
}

#[test]
fn test_a_run_record_carrying_no_identifier_stands_nowhere_in_the_unconfigured_list() {
    let anonymous = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookEvent":"PreToolUse","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, anonymous]), None);
    assert!(report.get("unconfigured_runs").is_none());
}
