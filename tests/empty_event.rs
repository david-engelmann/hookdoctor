//! The pins of slice 11, the empty event field, over
//! synthetic settings and session files the suite writes
//! itself. An event field standing as empty text stands
//! as no event field, the name derivation runs behind it
//! as it runs behind an absent field, and a derived part
//! stands as the event only where it is nonempty text, so
//! the record stands in every reading where its eventless
//! twin stands. Two rider pins at the foot complete the
//! proof of SI-2 and SI-3 and turn on no word of the slice.

use hookdoctor::examine::{build_report, validate_report};
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::report::render_markdown;

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const MATCHERLESS_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const EMPTY_KEY_SETTINGS: &str = r#"{"hooks": {"": [{"hooks": [{"type": "command", "command": "empty.sh"}]}],
                                     "PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

const TWO_MATCHERLESS_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "alpha.sh"}]},
                                                          {"hooks": [{"type": "command", "command": "beta.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const SEVERAL_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "shared.sh"}]},
                                                          {"matcher": "Edit", "hooks": [{"type": "command", "command": "scribe.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "shared.sh"}]}]}}"#;

const CALL: &str = r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#;
const OUTCOME: &str = r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#;
const EMPTY_FIELD_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse","hookEvent":"","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const EMPTY_FIELD_COLON_NAME: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse:Bash","hookEvent":"","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const EMPTY_FIELD_ROGUE: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","hookEvent":"","command":"rogue-hook.sh","content":"rogue context","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const COLON_LEADING_NAME: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":":Bash","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const WHITESPACE_FIELD_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse","hookEvent":" ","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const EMPTY_FIELD_DISAGREEING: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"wrap.sh","hookEvent":"","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const EMPTY_KEY_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"empty.sh","hookEvent":"","command":"empty.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const NUMERIC_FIELD_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse:Bash","hookEvent":0,"command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const NAME_FORM_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse","command":"none.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const SEVERAL_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse:Edit","command":"shared.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;

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

fn hook_entry(report: &JsonValue, command: &str) -> JsonValue {
    let Some(JsonValue::List(entries)) = report.get("hooks") else {
        panic!("the hooks stand");
    };
    entries
        .iter()
        .find(|entry| entry.get("command").and_then(JsonValue::as_text) == Some(command))
        .expect("the hook stands in the report")
        .clone()
}

fn class_of(entry: &JsonValue) -> String {
    entry
        .get("class")
        .and_then(JsonValue::as_text)
        .expect("the class stands")
        .to_string()
}

fn evidence(entry: &JsonValue, field: &str) -> i64 {
    let evidence = entry.get("evidence").expect("the evidence stands");
    match evidence.get(field) {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the evidence field {field} stands as an integer"),
    }
}

#[test]
fn test_an_empty_event_field_reads_as_no_event_field_and_the_hook_fires() {
    let report = report_with(MATCHERLESS_SETTINGS, &session(&[EMPTY_FIELD_RECORD]), None);
    let hook = hook_entry(&report, "guard.sh");
    assert_eq!(class_of(&hook), "firing-without-effect");
    assert_eq!(evidence(&hook, "matched"), 1);
    assert_eq!(evidence(&hook, "ran"), 1);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "the record stands as a subject of its hook and never as an unconfigured run"
    );
}

#[test]
fn test_an_empty_event_field_lets_the_name_derivation_run() {
    let report = report(&session(&[EMPTY_FIELD_COLON_NAME]), None);
    let hook = hook_entry(&report, "guard.sh");
    assert_eq!(class_of(&hook), "firing-without-effect");
    assert_eq!(evidence(&hook, "matched"), 1);
    assert_eq!(evidence(&hook, "ran"), 1);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "the record stands as a subject of its hook and never as an unconfigured run"
    );
}

#[test]
fn test_an_empty_event_field_seam_record_stands_in_the_list_with_a_null_event() {
    let report = report(&session(&[CALL, OUTCOME, EMPTY_FIELD_RECORD]), None);
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
fn test_an_empty_event_field_rogue_record_stands_in_the_list_with_a_null_event() {
    let report = report(&session(&[CALL, OUTCOME, EMPTY_FIELD_ROGUE]), None);
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
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_a_name_opening_with_a_colon_derives_no_event() {
    let report = report(&session(&[CALL, OUTCOME, COLON_LEADING_NAME]), None);
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::Null));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("guard.sh")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text(":Bash")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_an_empty_event_field_entry_renders_the_null_spelling_in_the_markdown() {
    let report = report(&session(&[CALL, OUTCOME, EMPTY_FIELD_ROGUE]), None);
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
}

#[test]
fn test_a_report_spelling_an_entry_event_as_empty_text_is_refused() {
    let text = session(&[CALL, OUTCOME, EMPTY_FIELD_ROGUE]);
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
    // The entry's event stands first among its fields, so the first
    // null after the member is the event's spelling on the build
    // tree. On the contract tree the entry already spells the empty
    // text and no null stands, and the document proceeds as rendered,
    // standing accepted there exactly as the standing rebuild accepts
    // it, so the refusal assertion below is the one that fails there.
    let mut tampered_text = spelled.clone();
    if let Some(null_at) = spelled[member_at..].find("null") {
        let at = member_at + null_at;
        tampered_text.replace_range(at..at + "null".len(), "\"\"");
    }
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
        "a report spelling an entry event as empty text refuses validation"
    );
}

#[test]
fn test_a_whitespace_event_field_stands_as_an_event() {
    let report = report_with(
        MATCHERLESS_SETTINGS,
        &session(&[WHITESPACE_FIELD_RECORD]),
        None,
    );
    let hook = hook_entry(&report, "guard.sh");
    assert_eq!(class_of(&hook), "silent");
    assert_eq!(evidence(&hook, "matched"), 0);
    assert_eq!(evidence(&hook, "ran"), 0);
    assert!(
        report.get("unconfigured_runs").is_none(),
        "the record's identifiers join on its hook, so it is no unconfigured run"
    );
}

#[test]
fn test_an_empty_event_field_disagreeing_record_stands_nowhere() {
    let with_record = report(&session(&[CALL, OUTCOME, EMPTY_FIELD_DISAGREEING]), None);
    let without_record = report(&session(&[CALL, OUTCOME]), None);
    assert!(
        with_record.get("unconfigured_runs").is_none(),
        "each identifier names a configured hook, so the record is no unconfigured run"
    );
    assert_eq!(with_record.get("hooks"), without_record.get("hooks"));
}

#[test]
fn test_a_hook_under_an_empty_event_key_still_counts_for_its_record() {
    let report = report_with(EMPTY_KEY_SETTINGS, &session(&[EMPTY_KEY_RECORD]), None);
    let hook = hook_entry(&report, "empty.sh");
    assert_eq!(class_of(&hook), "firing-without-effect");
    assert_eq!(evidence(&hook, "matched"), 1);
    assert_eq!(evidence(&hook, "ran"), 1);
}

#[test]
fn test_a_non_text_event_field_still_derives_from_the_name() {
    let report = report(&session(&[NUMERIC_FIELD_RECORD]), None);
    let hook = hook_entry(&report, "guard.sh");
    assert_eq!(class_of(&hook), "firing-without-effect");
    assert_eq!(evidence(&hook, "matched"), 1);
    assert_eq!(evidence(&hook, "ran"), 1);
}

#[test]
fn test_the_name_form_against_two_matcherless_hooks_of_one_event_lists_the_record() {
    let report = report_with(
        TWO_MATCHERLESS_SETTINGS,
        &session(&[CALL, OUTCOME, NAME_FORM_RECORD]),
        None,
    );
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::Null));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("none.sh")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_one_identifier_identifying_several_hooks_against_the_others_one_stands_nowhere() {
    let with_record = report_with(
        SEVERAL_SETTINGS,
        &session(&[CALL, OUTCOME, SEVERAL_RECORD]),
        None,
    );
    let without_record = report_with(SEVERAL_SETTINGS, &session(&[CALL, OUTCOME]), None);
    assert!(
        with_record.get("unconfigured_runs").is_none(),
        "each identifier names a configured hook, so the record is no unconfigured run"
    );
    assert_eq!(with_record.get("hooks"), without_record.get("hooks"));
}
