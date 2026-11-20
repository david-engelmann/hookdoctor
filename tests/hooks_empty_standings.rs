//! The pins of slice 14, the hooks section's two empty
//! standings, over synthetic settings and session files
//! the suite writes itself. A configuration carrying
//! the empty text as a hooks key builds its entries
//! with the event standing as the empty text, and the
//! Event line prints the gap. A group carrying the
//! empty text as its matcher builds its entries with
//! the matcher standing as the empty text, and the
//! Matcher line prints the gap, where a group carrying
//! no matcher or carrying null builds its entries with
//! the matcher standing as null and the line prints
//! the word None. Every pin stands on the landed code
//! as a fence, this slice moving no production code.

use hookdoctor::examine::build_report;
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::report::render_markdown;

const EMPTY_KEY_SETTINGS: &str =
    r#"{"hooks": {"": [{"hooks": [{"type": "command", "command": "empty.sh"}]}]}}"#;

const EMPTY_MATCHER_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "", "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

const NULL_MATCHER_SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": null, "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

const NO_MATCHER_SETTINGS: &str =
    r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

fn report_with(settings: &str, session_text: &str) -> JsonValue {
    let document = parse_json(settings).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(session_text, 0).expect("the session reads");
    build_report(
        &hooks,
        &records,
        "settings.json",
        &["session.jsonl".to_string()],
        None,
        None,
    )
    .expect("the report builds")
}

fn report(settings: &str) -> JsonValue {
    report_with(settings, "")
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

fn has_line(markdown: &str, line: &str) -> bool {
    markdown.lines().any(|standing| standing == line)
}

#[test]
fn test_a_hook_under_an_empty_event_key_prints_the_gap_on_the_event_line() {
    let report = report(EMPTY_KEY_SETTINGS);
    let entry = hook_entry(&report, "empty.sh");
    assert_eq!(entry.get("event"), Some(&JsonValue::text("")));
    assert_eq!(entry.get("id"), Some(&JsonValue::text(":0:0")));
    assert_eq!(entry.get("matcher"), Some(&JsonValue::Null));
    let markdown = render_markdown(&report);
    assert!(has_line(&markdown, "Event "), "{markdown}");
    assert!(
        !markdown.contains("Event None"),
        "the empty event never spells as the word None: {markdown}"
    );
}

#[test]
fn test_a_group_with_an_empty_matcher_prints_the_gap_on_the_matcher_line() {
    let report = report(EMPTY_MATCHER_SETTINGS);
    let entry = hook_entry(&report, "guard.sh");
    assert_eq!(entry.get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("matcher"), Some(&JsonValue::text("")));
    let markdown = render_markdown(&report);
    assert!(has_line(&markdown, "Matcher "), "{markdown}");
    assert!(
        !markdown.contains("Matcher None"),
        "the empty matcher prints the gap and never the word None: {markdown}"
    );
}

#[test]
fn test_a_group_with_a_null_matcher_prints_none_on_the_matcher_line() {
    let report = report(NULL_MATCHER_SETTINGS);
    let entry = hook_entry(&report, "guard.sh");
    assert_eq!(entry.get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("matcher"), Some(&JsonValue::Null));
    let markdown = render_markdown(&report);
    assert!(has_line(&markdown, "Matcher None"), "{markdown}");
}

#[test]
fn test_a_group_with_no_matcher_prints_none_on_the_matcher_line() {
    let absent_report = report(NO_MATCHER_SETTINGS);
    let entry = hook_entry(&absent_report, "guard.sh");
    assert_eq!(entry.get("event"), Some(&JsonValue::text("PreToolUse")));
    assert_eq!(entry.get("matcher"), Some(&JsonValue::Null));
    let null_report = report(NULL_MATCHER_SETTINGS);
    assert_eq!(
        absent_report.get("hooks"),
        null_report.get("hooks"),
        "the absent standing and the null standing build the same entries"
    );
    let markdown = render_markdown(&absent_report);
    assert!(has_line(&markdown, "Matcher None"), "{markdown}");
}
