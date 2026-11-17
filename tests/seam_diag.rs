//! Diagnostic for the slice 9 red team's principal note, the seam
//! between the two identification readings. Not a contract pin.
//! The case stands specified in the red team report. A hook run
//! record with no event field, its hook name the bare word
//! PreToolUse, and its command guard.sh, where guard.sh's hook
//! carries the matcher Bash. The expected standing under the code
//! as pushed is that the report carries no unconfigured runs
//! member for the record and the guard hook's entry stands
//! exactly as it stands without the record.

use hookdoctor::examine::build_report;
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::{parse_json, JsonValue};

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}],
                                     "Stop": [{"hooks": [{"type": "command", "command": "wrap.sh"}]}]}}"#;

const CALL: &str = r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#;
const OUTCOME: &str = r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#;
const SEAM: &str = r#"{"type":"attachment","timestamp":"2026-01-01T00:00:04Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"PreToolUse","command":"guard.sh","content":"","stdout":"","exitCode":0,"toolUseID":"c1"}}"#;

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

#[test]
fn test_the_seam_case_stands_nowhere() {
    let with = report(&session(&[CALL, OUTCOME, SEAM]));
    let without = report(&session(&[CALL, OUTCOME]));
    assert_eq!(
        with.get("unconfigured_runs"),
        None,
        "the seam record stands outside the list"
    );
    assert_eq!(
        with.get("hooks"),
        without.get("hooks"),
        "the guard hook's entry stands exactly as without the record"
    );
    assert_eq!(without.get("unconfigured_runs"), None);
}
