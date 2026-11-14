//! The pins of slice 4, the evidence format, over synthetic
//! settings and session files the suite writes itself as
//! texts and over the binary's command line.

use std::path::PathBuf;
use std::process::Command;

use hookdoctor::examine::{build_evidence, validate_evidence};
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::parse_json;
use hookdoctor::render::render_json;
use hookdoctor::JsonValue;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hookdoctor"))
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Scratch {
        let path =
            std::env::temp_dir().join(format!("hookdoctor-evidence-{}-{tag}", std::process::id()));
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

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

fn session() -> String {
    [
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#,
    ]
    .join("\n")
}

fn evidence_document(settings: &str, session_text: &str) -> JsonValue {
    let document = parse_json(settings).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(session_text, 0).expect("the session reads");
    build_evidence(&hooks, &records, None, None).expect("the evidence document builds")
}

fn verdict(document: &JsonValue, command: &str) -> JsonValue {
    document
        .get("verdicts")
        .and_then(JsonValue::as_list)
        .expect("the verdicts stand as a list")
        .iter()
        .find(|verdict| verdict.get("command").and_then(JsonValue::as_text) == Some(command))
        .expect("the verdict stands")
        .clone()
}

#[test]
fn test_the_evidence_document_carries_one_verdict_per_hook_in_configuration_order() {
    let document = evidence_document(SETTINGS, &session());
    assert_eq!(
        document.get("format").and_then(JsonValue::as_text),
        Some("hookdoctor-evidence/1")
    );
    let verdicts = document
        .get("verdicts")
        .and_then(JsonValue::as_list)
        .expect("the verdicts stand as a list");
    assert_eq!(verdicts.len(), 2);
    let commands: Vec<&str> = verdicts
        .iter()
        .map(|verdict| {
            verdict
                .get("command")
                .and_then(JsonValue::as_text)
                .expect("the command stands")
        })
        .collect();
    assert_eq!(commands, vec!["guard.sh", "wrap.sh"]);
    let guard = &verdicts[0];
    assert_eq!(
        guard.get("event").and_then(JsonValue::as_text),
        Some("PreToolUse")
    );
    assert_eq!(
        guard.get("matcher").and_then(JsonValue::as_text),
        Some("Bash")
    );
    assert_eq!(
        guard.get("class").and_then(JsonValue::as_text),
        Some("firing-without-effect")
    );
    assert_eq!(guard.get("matched"), Some(&JsonValue::Int(1)));
    assert_eq!(guard.get("ran"), Some(&JsonValue::Int(1)));
    assert!(
        guard.get("id").is_none(),
        "no verdict carries the positional hook identifier"
    );
    let wrap = &verdicts[1];
    assert_eq!(
        wrap.get("class").and_then(JsonValue::as_text),
        Some("silent")
    );
    assert_eq!(wrap.get("first_ran_at"), Some(&JsonValue::Null));
    assert_eq!(wrap.get("last_ran_at"), Some(&JsonValue::Null));
}

#[test]
fn test_a_hook_whose_group_names_no_matcher_stands_with_an_empty_matcher_in_its_verdict() {
    let document = evidence_document(SETTINGS, &session());
    let wrap = verdict(&document, "wrap.sh");
    assert_eq!(
        wrap.get("matcher").and_then(JsonValue::as_text),
        Some(""),
        "the missing matcher stands as the empty text in the verdict"
    );
    let guard = verdict(&document, "guard.sh");
    assert_eq!(
        guard.get("matcher").and_then(JsonValue::as_text),
        Some("Bash")
    );
}

#[test]
fn test_the_evidence_document_is_written_only_when_the_flag_stands() {
    let scratch = Scratch::new("flag");
    let config_path = scratch.0.join("settings.json");
    std::fs::write(&config_path, SETTINGS).expect("the settings are written");
    let session_path = scratch.0.join("session.jsonl");
    std::fs::write(&session_path, session()).expect("the session is written");
    let evidence_path = scratch.0.join("evidence.json");
    let output = Command::new(binary())
        .arg("--config")
        .arg(&config_path)
        .arg("--session")
        .arg(&session_path)
        .arg("--output")
        .arg(scratch.0.join("report-a.json"))
        .arg("--report")
        .arg(scratch.0.join("report-a.md"))
        .output()
        .expect("the binary runs");
    assert!(output.status.success(), "the unflagged run stands");
    assert!(
        !evidence_path.exists(),
        "no evidence document stands after a run without the flag"
    );
    let output = Command::new(binary())
        .arg("--config")
        .arg(&config_path)
        .arg("--session")
        .arg(&session_path)
        .arg("--output")
        .arg(scratch.0.join("report-b.json"))
        .arg("--report")
        .arg(scratch.0.join("report-b.md"))
        .arg("--evidence")
        .arg(&evidence_path)
        .output()
        .expect("the binary runs");
    assert!(output.status.success(), "the flagged run stands");
    let written = std::fs::read_to_string(&evidence_path).expect("the evidence document stands");
    let expected = evidence_document(SETTINGS, &session());
    assert_eq!(written, render_json(&expected));
    let report_a =
        std::fs::read_to_string(scratch.0.join("report-a.json")).expect("the first report stands");
    let report_b =
        std::fs::read_to_string(scratch.0.join("report-b.json")).expect("the second report stands");
    assert_eq!(
        report_a, report_b,
        "the flag moves no other output of the run"
    );
}

fn tamper_first_verdict(
    document: &JsonValue,
    change: impl Fn(Vec<(String, JsonValue)>) -> Vec<(String, JsonValue)>,
) -> JsonValue {
    let verdicts = document
        .get("verdicts")
        .and_then(JsonValue::as_list)
        .expect("the verdicts stand as a list")
        .clone();
    let JsonValue::Object(pairs) = verdicts[0].clone() else {
        panic!("the verdict stands as an object");
    };
    let mut all = vec![JsonValue::Object(change(pairs))];
    all.extend(verdicts.into_iter().skip(1));
    JsonValue::object(vec![
        ("format", JsonValue::text("hookdoctor-evidence/1")),
        ("verdicts", JsonValue::List(all)),
    ])
}

fn set_field(
    mut pairs: Vec<(String, JsonValue)>,
    field: &str,
    value: JsonValue,
) -> Vec<(String, JsonValue)> {
    for pair in pairs.iter_mut() {
        if pair.0 == field {
            pair.1 = value;
            return pairs;
        }
    }
    pairs.push((field.to_string(), value));
    pairs
}

#[test]
fn test_evidence_validation_refuses_a_tampered_class() {
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&session(), 0).expect("the session reads");
    let standing = build_evidence(&hooks, &records, None, None).expect("the document builds");
    assert!(
        validate_evidence(&standing, &hooks, &records, None, None).expect("the standing validates"),
        "the untampered document validates"
    );
    let tampered = tamper_first_verdict(&standing, |pairs| {
        set_field(pairs, "class", JsonValue::text("firing"))
    });
    let error = validate_evidence(&tampered, &hooks, &records, None, None)
        .expect_err("the tampered class refuses");
    assert_eq!(
        error.to_string(),
        "the evidence document disagrees with the examination"
    );
}

#[test]
fn test_evidence_validation_refuses_tampered_counts() {
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&session(), 0).expect("the session reads");
    let standing = build_evidence(&hooks, &records, None, None).expect("the document builds");
    let tampered = tamper_first_verdict(&standing, |pairs| {
        set_field(pairs, "ran", JsonValue::Int(9))
    });
    assert!(
        validate_evidence(&tampered, &hooks, &records, None, None).is_err(),
        "a tampered count refuses"
    );
}

#[test]
fn test_evidence_validation_refuses_a_verdict_with_an_unexpected_field() {
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&session(), 0).expect("the session reads");
    let standing = build_evidence(&hooks, &records, None, None).expect("the document builds");
    let tampered = tamper_first_verdict(&standing, |pairs| {
        set_field(pairs, "id", JsonValue::text("PreToolUse:0:0"))
    });
    assert!(
        validate_evidence(&tampered, &hooks, &records, None, None).is_err(),
        "a verdict carrying the positional identifier refuses"
    );
}
