//! The pins of slice 10, the identification seam of the
//! unconfigured runs, over synthetic settings and session
//! files the suite writes itself. The list of slices 8
//! and 9 gains the hook run records whose two identifiers
//! the joint reading cannot join, one identifier
//! identifying a configured hook and the other identifying
//! none, standing in the list beside the standing entries.

use hookdoctor::examine::{build_report, validate_report};
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::report::render_markdown;

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const SHARED_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "shared.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "shared.sh"}]}]}}"#;

const SIBLING_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]},
                                                          {"matcher": "Edit", "hooks": [{"type": "command", "command": "scribe.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const CALL: &str = r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#;
const OUTCOME: &str = r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#;
const ROGUE_RUN: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","hookEvent":"PreToolUse","command":"rogue-hook.sh","content":"rogue context","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const SEAM_EVENTLESS: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const SEAM_MIRROR: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse:Bash","command":"other.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const SEAM_EVENTED: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse","hookEvent":"PreToolUse","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const SEAM_EVENTED_MIRROR: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse:Bash","hookEvent":"PreToolUse","command":"other.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const SEAM_SHARED: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"shared-hook","command":"shared.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;

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
fn test_an_eventless_record_whose_command_identifies_a_hook_its_name_cannot_stands_in_the_unconfigured_list(
) {
    let report = report(&session(&[CALL, OUTCOME, SEAM_EVENTLESS]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::Null));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("guard.sh")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(
        entry.get("timestamp"),
        Some(&JsonValue::text("2026-01-01T00:00:04Z"))
    );
    assert_eq!(entry.get("source"), Some(&JsonValue::text("session.jsonl")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_an_eventless_record_whose_name_identifies_a_hook_its_command_cannot_stands_in_the_unconfigured_list(
) {
    let report = report(&session(&[CALL, OUTCOME, SEAM_MIRROR]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("other.sh")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("PreToolUse:Bash")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_an_evented_record_whose_command_identifies_a_hook_its_name_cannot_stands_in_the_unconfigured_list(
) {
    let report = report(&session(&[CALL, OUTCOME, SEAM_EVENTED]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("guard.sh")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_an_evented_record_whose_name_identifies_a_hook_its_command_cannot_stands_in_the_unconfigured_list(
) {
    let report = report(&session(&[CALL, OUTCOME, SEAM_EVENTED_MIRROR]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("other.sh")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("PreToolUse:Bash")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_an_eventless_record_whose_command_stands_under_two_events_and_whose_name_identifies_none_stands_in_the_unconfigured_list(
) {
    let report = report_with(
        SHARED_SETTINGS,
        &session(&[CALL, OUTCOME, SEAM_SHARED]),
        None,
    );
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::Null));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("shared.sh")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("shared-hook")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_a_seam_run_stands_in_trace_order_among_the_unconfigured_runs() {
    let first = report(&session(&[CALL, OUTCOME, SEAM_EVENTLESS, ROGUE_RUN]), None);
    let Some(JsonValue::List(runs)) = first.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].get("event"), Some(&JsonValue::Null));
    assert_eq!(runs[0].get("line"), Some(&JsonValue::Int(3)));
    assert_eq!(runs[1].get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(runs[1].get("line"), Some(&JsonValue::Int(4)));
    let second = report(&session(&[CALL, OUTCOME, ROGUE_RUN, SEAM_EVENTLESS]), None);
    let Some(JsonValue::List(runs)) = second.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(runs[0].get("line"), Some(&JsonValue::Int(3)));
    assert_eq!(runs[1].get("event"), Some(&JsonValue::Null));
    assert_eq!(runs[1].get("line"), Some(&JsonValue::Int(4)));
}

#[test]
fn test_the_markdown_renders_a_seam_run() {
    let report = report(&session(&[CALL, OUTCOME, SEAM_EVENTLESS]), None);
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- guard.sh ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
}

#[test]
fn test_a_tampered_seam_entry_refuses_the_report_validation() {
    let text = session(&[CALL, OUTCOME, SEAM_EVENTLESS]);
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
    let Some(JsonValue::List(runs)) = genuine.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let spelled = hookdoctor::render::render_json(&genuine);
    let member_at = spelled
        .find("unconfigured_runs")
        .expect("the member stands in the rendering");
    let entry_at = member_at
        + spelled[member_at..]
            .find("guard.sh")
            .expect("the seam entry stands in the rendering");
    let mut tampered_text = spelled.clone();
    tampered_text.replace_range(entry_at..entry_at + "guard.sh".len(), "other-hook.sh");
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
    assert!(refused.is_err(), "a tampered seam entry refuses validation");
}

#[test]
fn test_a_record_whose_identifiers_point_at_different_hooks_still_stands_nowhere_in_the_unconfigured_list(
) {
    let disagreeing = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"wrap.sh","hookEvent":"PreToolUse","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report(&session(&[CALL, OUTCOME, disagreeing]), None);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "each identifier names a configured hook, so the record is no unconfigured run"
    );
}

#[test]
fn test_a_record_whose_identifiers_point_at_sibling_hooks_still_stands_nowhere_in_the_unconfigured_list(
) {
    let disagreeing = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse:Bash","hookEvent":"PreToolUse","command":"scribe.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
    let report = report_with(
        SIBLING_SETTINGS,
        &session(&[CALL, OUTCOME, disagreeing]),
        None,
    );
    assert!(
        report.get("unconfigured_runs").is_none(),
        "each identifier names a sibling hook of the one event, so the record is no unconfigured run"
    );
}

#[test]
fn test_a_seam_record_counts_in_no_hooks_matched_or_ran_evidence() {
    let with_seam = report(&session(&[CALL, OUTCOME, SEAM_EVENTLESS]), None);
    let without_seam = report(&session(&[CALL, OUTCOME]), None);
    assert_eq!(with_seam.get("hooks"), without_seam.get("hooks"));
}
