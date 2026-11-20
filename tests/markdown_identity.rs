//! The pins of slice 12, the markdown identity of an
//! unconfigured run whose command stands as empty text,
//! over synthetic settings and session files the suite
//! writes itself. An entry carrying a name form prints
//! its name in the markdown where its command stands as
//! empty text, and prints its command as carried in
//! every other case, and a stop entry prints its command
//! as carried because it carries no name form.

use std::path::PathBuf;
use std::process::Command;

use hookdoctor::examine::build_report;
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::report::render_markdown;

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const CALL: &str = r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#;
const OUTCOME: &str = r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#;
const EMPTY_COMMAND_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","command":"","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const EVENTED_EMPTY_COMMAND_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","hookEvent":"PreToolUse","command":"","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const ABSENT_COMMAND_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-hook.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const NAMED_COMMAND_RECORD: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"rogue-name.sh","hookEvent":"PreToolUse","command":"rogue-hook.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;
const STOP_EMPTY_COMMAND_LINE: &str = r#"{"type":"system","subtype":"stop_hook_summary","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","hookInfos":[{"command":""}],"preventedContinuation":false,"hasOutput":false}"#;

fn report(session_text: &str) -> JsonValue {
    let document = parse_json(SETTINGS).expect("the settings parse");
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

fn session(lines: &[&str]) -> String {
    lines.join("\n")
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hookdoctor"))
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Scratch {
        let path = std::env::temp_dir().join(format!(
            "hookdoctor-markdown-identity-{}-{tag}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory stands");
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn governed_markdown_text() -> String {
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&session(&[CALL, OUTCOME, EMPTY_COMMAND_RECORD]), 0)
        .expect("the session reads");
    let report = build_report(
        &hooks,
        &records,
        "settings.json",
        &["session.jsonl".to_string()],
        None,
        None,
    )
    .expect("the report builds");
    render_markdown(&report)
}

#[test]
fn test_an_empty_command_entry_prints_its_name_form_in_the_markdown() {
    let report = report(&session(&[CALL, OUTCOME, EMPTY_COMMAND_RECORD]));
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
    assert!(
        !markdown.contains("-  ran at None"),
        "no gap stands in the identity place: {markdown}"
    );
}

#[test]
fn test_an_evented_empty_command_entry_prints_its_name_form_in_the_markdown() {
    let report = report(&session(&[CALL, OUTCOME, EVENTED_EMPTY_COMMAND_RECORD]));
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at PreToolUse in session.jsonl on line 3."),
        "{markdown}"
    );
}

#[test]
fn test_an_absent_command_entry_prints_its_name_form_in_the_markdown() {
    let report = report(&session(&[CALL, OUTCOME, ABSENT_COMMAND_RECORD]));
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
}

#[test]
fn test_a_nonempty_command_entry_prints_its_command_in_the_markdown() {
    let report = report(&session(&[CALL, OUTCOME, NAMED_COMMAND_RECORD]));
    let markdown = render_markdown(&report);
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at PreToolUse in session.jsonl on line 3."),
        "{markdown}"
    );
    assert!(
        !markdown.contains("- rogue-name.sh ran at"),
        "the command stands in the identity place, not the name form: {markdown}"
    );
}

#[test]
fn test_a_stop_entry_with_an_empty_command_prints_its_command_as_carried_in_the_markdown() {
    let report = report(&session(&[STOP_EMPTY_COMMAND_LINE]));
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("stop")));
    assert_eq!(entry.get("event"), Some(&JsonValue::text("Stop")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("")));
    assert_eq!(entry.get("name"), Some(&JsonValue::Null));
    let markdown = render_markdown(&report);
    assert!(
        markdown.contains("-  ran at Stop in session.jsonl on line 1."),
        "the stop entry carries no name form, so its identity stands as carried: {markdown}"
    );
}

#[test]
fn test_an_empty_command_entry_spells_its_command_as_empty_text_in_the_report() {
    let report = report(&session(&[CALL, OUTCOME, EMPTY_COMMAND_RECORD]));
    let Some(JsonValue::List(runs)) = report.get("unconfigured_runs") else {
        panic!("the member stands");
    };
    assert_eq!(runs.len(), 1);
    let entry = &runs[0];
    assert_eq!(entry.get("event"), Some(&JsonValue::Null));
    assert_eq!(entry.get("kind"), Some(&JsonValue::text("hook_run")));
    assert_eq!(entry.get("command"), Some(&JsonValue::text("")));
    assert_eq!(entry.get("name"), Some(&JsonValue::text("rogue-hook.sh")));
    assert_eq!(entry.get("line"), Some(&JsonValue::Int(3)));
}

#[test]
fn test_a_supplied_markdown_spelling_a_governed_entry_with_the_gap_refuses_the_check() {
    let genuine = governed_markdown_text();
    assert!(
        genuine.contains("- rogue-hook.sh ran at None in session.jsonl on line 3."),
        "the genuine rendering prints the name form: {genuine}"
    );
    let supplied = genuine.replace("- rogue-hook.sh ran at", "-  ran at");
    assert_ne!(supplied, genuine, "the respelling lands");
    let scratch = Scratch::new("gap-spelling");
    let settings = scratch.0.join("settings.json");
    let session_path = scratch.0.join("session.jsonl");
    let markdown = scratch.0.join("supplied-report.md");
    std::fs::write(&settings, SETTINGS).expect("the settings stand");
    std::fs::write(
        &session_path,
        session(&[CALL, OUTCOME, EMPTY_COMMAND_RECORD]),
    )
    .expect("the session stands");
    std::fs::write(&markdown, &supplied).expect("the document stands");
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(session_path)
        .arg("--check-markdown")
        .arg(&markdown)
        .output()
        .expect("the check runs");
    assert_eq!(output.status.code(), Some(2), "the check refuses at exit 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the markdown report disagrees with the examination"),
        "the refusal carries the standing words: {stderr}"
    );
}
