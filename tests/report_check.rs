//! The pins of slice 6, the report check, over synthetic
//! settings and session files the suite writes itself and
//! over the binary's check mode. The genuine report is built
//! under the source labels the door resolves from the file
//! names standing in the scratch, so the labels the check
//! resolves stand in the comparison exactly as they stand in
//! an examination.

use std::path::PathBuf;
use std::process::Command;

use hookdoctor::examine::build_report;
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
        let path = std::env::temp_dir().join(format!(
            "hookdoctor-report-check-{}-{tag}",
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

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

fn session() -> String {
    [
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#,
    ]
    .join("\n")
}

fn genuine_report_text() -> String {
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&session(), 0).expect("the session reads");
    let report = build_report(
        &hooks,
        &records,
        "settings.json",
        &["session.jsonl".to_string()],
        None,
        None,
    )
    .expect("the report builds");
    render_json(&report)
}

/// A scratch standing with the settings, the session, and a
/// supplied report document written into it, returning the
/// paths the check mode reads.
fn standing(tag: &str, report_text: &str) -> (Scratch, PathBuf, PathBuf, PathBuf) {
    let scratch = Scratch::new(tag);
    let settings = scratch.0.join("settings.json");
    let session_path = scratch.0.join("session.jsonl");
    let report = scratch.0.join("supplied-report.json");
    std::fs::write(&settings, SETTINGS).expect("the settings stand");
    std::fs::write(&session_path, session()).expect("the session stands");
    std::fs::write(&report, report_text).expect("the report stands");
    (scratch, settings, session_path, report)
}

fn run_check(settings: &PathBuf, session_path: &PathBuf, report: &PathBuf) -> std::process::Output {
    Command::new(binary())
        .arg("--config")
        .arg(settings)
        .arg("--session")
        .arg(session_path)
        .arg("--check-report")
        .arg(report)
        .output()
        .expect("the check runs")
}

#[test]
fn test_a_supplied_report_validates_in_check_mode() {
    let (_scratch, settings, session_path, report) = standing("genuine", &genuine_report_text());
    let output = run_check(&settings, &session_path, &report);
    assert!(
        output.status.success(),
        "the check exits 0 on a genuine document"
    );
    assert!(output.stdout.is_empty(), "the check prints nothing");
}

#[test]
fn test_report_check_mode_writes_no_examination_outputs() {
    let (scratch, settings, session_path, report) = standing("no-outputs", &genuine_report_text());
    let output = run_check(&settings, &session_path, &report);
    assert!(output.status.success());
    let names: Vec<String> = std::fs::read_dir(&scratch.0)
        .expect("the scratch reads")
        .map(|entry| {
            entry
                .expect("the entry stands")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names.len(), 3, "only the three inputs stand: {names:?}");
}

#[test]
fn test_a_tampered_supplied_report_refuses_in_check_mode() {
    let document = parse_json(&genuine_report_text()).expect("the report parses");
    let JsonValue::Object(mut fields) = document else {
        panic!("the report is an object");
    };
    for (key, value) in fields.iter_mut() {
        if key == "hooks" {
            let JsonValue::List(entries) = value else {
                panic!("the entries stand as a list");
            };
            let JsonValue::Object(entry_fields) = &mut entries[0] else {
                panic!("the entry is an object");
            };
            for (entry_key, entry_value) in entry_fields.iter_mut() {
                if entry_key == "class" {
                    *entry_value = JsonValue::Text("silent".to_string());
                }
            }
        }
    }
    let tampered = render_json(&JsonValue::Object(fields));
    let (_scratch, settings, session_path, report) = standing("tampered", &tampered);
    let output = run_check(&settings, &session_path, &report);
    assert_eq!(output.status.code(), Some(2), "the check refuses at exit 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the report disagrees with the examination"),
        "the refusal carries the standing words: {stderr}"
    );
}

#[test]
fn test_a_missing_supplied_report_refuses_in_check_mode() {
    let (scratch, settings, session_path, _report) = standing("missing", &genuine_report_text());
    let missing = scratch.0.join("no-report.json");
    let output = run_check(&settings, &session_path, &missing);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a missing document refuses at exit 2"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the report could not be read"),
        "the refusal names the missing document: {stderr}"
    );
}

#[test]
fn test_report_check_mode_refuses_the_output_flags() {
    let (scratch, settings, session_path, report) =
        standing("outputs-refused", &genuine_report_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-report")
        .arg(&report)
        .arg("--output")
        .arg(scratch.0.join("report.json"))
        .arg("--report")
        .arg(scratch.0.join("report.md"))
        .output()
        .expect("the run stands");
    assert_eq!(
        output.status.code(),
        Some(2),
        "the output flags refuse in check mode"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("usage: hookdoctor"),
        "the refusal is the usage refusal: {stderr}"
    );
}

#[test]
fn test_report_check_mode_refuses_the_evidence_flag() {
    let (scratch, settings, session_path, report) =
        standing("evidence-refused", &genuine_report_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-report")
        .arg(&report)
        .arg("--evidence")
        .arg(scratch.0.join("evidence.json"))
        .output()
        .expect("the run stands");
    assert_eq!(
        output.status.code(),
        Some(2),
        "the evidence flag refuses in check mode"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("usage: hookdoctor"),
        "the refusal is the usage refusal: {stderr}"
    );
}

#[test]
fn test_report_check_mode_refuses_the_check_evidence_flag() {
    let (scratch, settings, session_path, report) =
        standing("both-checks-refused", &genuine_report_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-report")
        .arg(&report)
        .arg("--check-evidence")
        .arg(scratch.0.join("evidence.json"))
        .output()
        .expect("the run stands");
    assert_eq!(
        output.status.code(),
        Some(2),
        "the two check flags refuse standing together"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("usage: hookdoctor"),
        "the refusal is the usage refusal: {stderr}"
    );
}

#[test]
fn test_a_report_under_other_source_labels_refuses_in_check_mode() {
    let (_scratch, settings, session_path, report) = standing("labels", &genuine_report_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--config-label")
        .arg("audit-settings.json")
        .arg("--check-report")
        .arg(&report)
        .output()
        .expect("the check runs");
    assert_eq!(
        output.status.code(),
        Some(2),
        "a report of another label standing refuses at exit 2"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the report disagrees with the examination"),
        "the refusal carries the standing words: {stderr}"
    );
}
