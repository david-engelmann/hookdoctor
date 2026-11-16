//! The pins of slice 7, the markdown check, over synthetic
//! settings and session files the suite writes itself and
//! over the binary's check mode. The genuine document is the
//! markdown the examination renders under the source labels
//! the door resolves from the file names standing in the
//! scratch, so the labels the check resolves stand in the
//! comparison exactly as they stand in an examination. The
//! comparison stands over the bytes themselves, so a
//! document one byte short of the rendering refuses beside
//! a document whose words were changed.

use std::path::PathBuf;
use std::process::Command;

use hookdoctor::examine::build_report;
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::parse_json;
use hookdoctor::report::render_markdown;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hookdoctor"))
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Scratch {
        let path = std::env::temp_dir().join(format!(
            "hookdoctor-markdown-check-{}-{tag}",
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

fn genuine_markdown_text() -> String {
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
    render_markdown(&report)
}

/// A scratch standing with the settings, the session, and a
/// supplied markdown document written into it, returning the
/// paths the check mode reads.
fn standing(tag: &str, markdown_text: &str) -> (Scratch, PathBuf, PathBuf, PathBuf) {
    let scratch = Scratch::new(tag);
    let settings = scratch.0.join("settings.json");
    let session_path = scratch.0.join("session.jsonl");
    let markdown = scratch.0.join("supplied-report.md");
    std::fs::write(&settings, SETTINGS).expect("the settings stand");
    std::fs::write(&session_path, session()).expect("the session stands");
    std::fs::write(&markdown, markdown_text).expect("the document stands");
    (scratch, settings, session_path, markdown)
}

fn run_check(
    settings: &PathBuf,
    session_path: &PathBuf,
    markdown: &PathBuf,
) -> std::process::Output {
    Command::new(binary())
        .arg("--config")
        .arg(settings)
        .arg("--session")
        .arg(session_path)
        .arg("--check-markdown")
        .arg(markdown)
        .output()
        .expect("the check runs")
}

#[test]
fn test_a_supplied_markdown_report_validates_in_check_mode() {
    let (_scratch, settings, session_path, markdown) =
        standing("genuine", &genuine_markdown_text());
    let output = run_check(&settings, &session_path, &markdown);
    assert!(
        output.status.success(),
        "the check exits 0 on a genuine document"
    );
    assert!(output.stdout.is_empty(), "the check prints nothing");
}

#[test]
fn test_markdown_check_mode_writes_no_examination_outputs() {
    let (scratch, settings, session_path, markdown) =
        standing("no-outputs", &genuine_markdown_text());
    let output = run_check(&settings, &session_path, &markdown);
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
fn test_a_tampered_supplied_markdown_report_refuses_in_check_mode() {
    let tampered = genuine_markdown_text().replace("Class firing-without-effect", "Class silent");
    assert_ne!(tampered, genuine_markdown_text(), "the tamper lands");
    let (_scratch, settings, session_path, markdown) = standing("tampered", &tampered);
    let output = run_check(&settings, &session_path, &markdown);
    assert_eq!(output.status.code(), Some(2), "the check refuses at exit 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the markdown report disagrees with the examination"),
        "the refusal carries the standing words: {stderr}"
    );
}

#[test]
fn test_a_missing_supplied_markdown_report_refuses_in_check_mode() {
    let (scratch, settings, session_path, _markdown) =
        standing("missing", &genuine_markdown_text());
    let missing = scratch.0.join("no-report.md");
    let output = run_check(&settings, &session_path, &missing);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a missing document refuses at exit 2"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the markdown report could not be read"),
        "the refusal names the missing document: {stderr}"
    );
}

#[test]
fn test_markdown_check_mode_refuses_the_output_flags() {
    let (scratch, settings, session_path, markdown) =
        standing("outputs-refused", &genuine_markdown_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-markdown")
        .arg(&markdown)
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
fn test_markdown_check_mode_refuses_the_evidence_flag() {
    let (scratch, settings, session_path, markdown) =
        standing("evidence-refused", &genuine_markdown_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-markdown")
        .arg(&markdown)
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
fn test_markdown_check_mode_refuses_the_check_evidence_flag() {
    let (scratch, settings, session_path, markdown) =
        standing("evidence-check-refused", &genuine_markdown_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-markdown")
        .arg(&markdown)
        .arg("--check-evidence")
        .arg(scratch.0.join("evidence.json"))
        .output()
        .expect("the run stands");
    assert_eq!(
        output.status.code(),
        Some(2),
        "the two checks refuse standing together"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("usage: hookdoctor"),
        "the refusal is the usage refusal: {stderr}"
    );
}

#[test]
fn test_markdown_check_mode_refuses_the_check_report_flag() {
    let (scratch, settings, session_path, markdown) =
        standing("report-check-refused", &genuine_markdown_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-markdown")
        .arg(&markdown)
        .arg("--check-report")
        .arg(scratch.0.join("report.json"))
        .output()
        .expect("the run stands");
    assert_eq!(
        output.status.code(),
        Some(2),
        "the two checks refuse standing together"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("usage: hookdoctor"),
        "the refusal is the usage refusal: {stderr}"
    );
}

#[test]
fn test_a_markdown_report_under_other_source_labels_refuses_in_check_mode() {
    let (_scratch, settings, session_path, markdown) =
        standing("other-labels", &genuine_markdown_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--config-label")
        .arg("other-settings.json")
        .arg("--check-markdown")
        .arg(&markdown)
        .output()
        .expect("the check runs");
    assert_eq!(output.status.code(), Some(2), "the check refuses at exit 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the markdown report disagrees with the examination"),
        "the labels stand in the comparison: {stderr}"
    );
}

#[test]
fn test_a_markdown_report_missing_its_final_newline_refuses_in_check_mode() {
    let genuine = genuine_markdown_text();
    assert!(genuine.ends_with('\n'), "the rendering ends in a newline");
    let shortened = &genuine[..genuine.len() - 1];
    let (_scratch, settings, session_path, markdown) = standing("shortened", shortened);
    let output = run_check(&settings, &session_path, &markdown);
    assert_eq!(output.status.code(), Some(2), "the check refuses at exit 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the markdown report disagrees with the examination"),
        "one missing byte is a different document: {stderr}"
    );
}
