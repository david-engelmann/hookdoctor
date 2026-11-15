//! The pins of slice 5, the evidence check, over synthetic
//! settings and session files the suite writes itself and
//! over the binary's check mode.

use std::path::PathBuf;
use std::process::Command;

use hookdoctor::examine::build_evidence;
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
            "hookdoctor-evidence-check-{}-{tag}",
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

fn genuine_evidence_text() -> String {
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&session(), 0).expect("the session reads");
    let evidence = build_evidence(&hooks, &records, None, None).expect("the evidence builds");
    render_json(&evidence)
}

/// A scratch standing with the settings, the session, and a
/// supplied evidence document written into it, returning the
/// paths the check mode reads.
fn standing(tag: &str, evidence_text: &str) -> (Scratch, PathBuf, PathBuf, PathBuf) {
    let scratch = Scratch::new(tag);
    let settings = scratch.0.join("settings.json");
    let session_path = scratch.0.join("session.jsonl");
    let evidence = scratch.0.join("evidence.json");
    std::fs::write(&settings, SETTINGS).expect("the settings stand");
    std::fs::write(&session_path, session()).expect("the session stands");
    std::fs::write(&evidence, evidence_text).expect("the evidence stands");
    (scratch, settings, session_path, evidence)
}

fn run_check(
    settings: &PathBuf,
    session_path: &PathBuf,
    evidence: &PathBuf,
) -> std::process::Output {
    Command::new(binary())
        .arg("--config")
        .arg(settings)
        .arg("--session")
        .arg(session_path)
        .arg("--check-evidence")
        .arg(evidence)
        .output()
        .expect("the check runs")
}

#[test]
fn test_a_supplied_evidence_document_validates_in_check_mode() {
    let (_scratch, settings, session_path, evidence) =
        standing("genuine", &genuine_evidence_text());
    let output = run_check(&settings, &session_path, &evidence);
    assert!(
        output.status.success(),
        "the check exits 0 on a genuine document"
    );
    assert!(output.stdout.is_empty(), "the check prints nothing");
}

#[test]
fn test_check_mode_writes_no_examination_outputs() {
    let (scratch, settings, session_path, evidence) =
        standing("no-outputs", &genuine_evidence_text());
    let output = run_check(&settings, &session_path, &evidence);
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
fn test_a_tampered_supplied_document_refuses_in_check_mode() {
    let document = parse_json(&genuine_evidence_text()).expect("the evidence parses");
    let JsonValue::Object(mut fields) = document else {
        panic!("the evidence is an object");
    };
    for (key, value) in fields.iter_mut() {
        if key == "verdicts" {
            let JsonValue::List(verdicts) = value else {
                panic!("the verdicts stand as a list");
            };
            let JsonValue::Object(verdict_fields) = &mut verdicts[0] else {
                panic!("the verdict is an object");
            };
            for (verdict_key, verdict_value) in verdict_fields.iter_mut() {
                if verdict_key == "class" {
                    *verdict_value = JsonValue::Text("silent".to_string());
                }
            }
        }
    }
    let tampered = render_json(&JsonValue::Object(fields));
    let (_scratch, settings, session_path, evidence) = standing("tampered", &tampered);
    let output = run_check(&settings, &session_path, &evidence);
    assert_eq!(output.status.code(), Some(2), "the check refuses at exit 2");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the evidence document disagrees with the examination"),
        "the refusal carries the standing words: {stderr}"
    );
}

#[test]
fn test_a_missing_supplied_document_refuses_in_check_mode() {
    let (scratch, settings, session_path, _evidence) =
        standing("missing", &genuine_evidence_text());
    let missing = scratch.0.join("no-evidence.json");
    let output = run_check(&settings, &session_path, &missing);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a missing document refuses at exit 2"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the evidence document could not be read"),
        "the refusal names the missing document: {stderr}"
    );
}

#[test]
fn test_check_mode_refuses_the_output_flags() {
    let (scratch, settings, session_path, evidence) =
        standing("outputs-refused", &genuine_evidence_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-evidence")
        .arg(&evidence)
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
fn test_check_mode_refuses_the_evidence_flag() {
    let (scratch, settings, session_path, evidence) =
        standing("evidence-refused", &genuine_evidence_text());
    let output = Command::new(binary())
        .arg("--config")
        .arg(&settings)
        .arg("--session")
        .arg(&session_path)
        .arg("--check-evidence")
        .arg(&evidence)
        .arg("--evidence")
        .arg(scratch.0.join("other-evidence.json"))
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
