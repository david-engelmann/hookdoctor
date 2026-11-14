//! The pins of slice 3, the eventless run record, over
//! synthetic settings and session files the suite writes
//! itself as texts.

use hookdoctor::examine::examine_hooks;
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::parse_json;
use hookdoctor::JsonValue;

fn run(settings: &str, session: &str) -> Vec<JsonValue> {
    let document = parse_json(settings).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(session, 0).expect("the session reads");
    let (entries, _near_misses) =
        examine_hooks(&hooks, &records, None, None, &["session".to_string()])
            .expect("the examination stands");
    entries
}

fn entry(entries: &[JsonValue], command: &str) -> JsonValue {
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

fn effect(entry: &JsonValue, kind: &str) -> i64 {
    let evidence = entry.get("evidence").expect("the evidence stands");
    let effects = evidence.get("effects").expect("the effects stand");
    match effects.get(kind) {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the effect {kind} stands as an integer"),
    }
}

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

fn call_and_outcome() -> String {
    [
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#,
    ]
    .join("\n")
}

#[test]
fn test_an_eventless_run_record_counts_for_the_hook_its_command_names() {
    let session = format!(
        "{}\n{}",
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"attachment","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"guard.sh","command":"guard.sh","toolUseId":"c1","content":"","exitCode":0,"stdout":"{\"hookSpecificOutput\": {\"updatedInput\": {\"command\": \"ls -la\"}}}"}}"#
    );
    let session = format!(
        "{session}\n{}",
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#
    );
    let entries = run(SETTINGS, &session);
    let hook = entry(&entries, "guard.sh");
    assert_eq!(class_of(&hook), "firing");
    assert_eq!(evidence(&hook, "matched"), 2);
    assert_eq!(evidence(&hook, "ran"), 2);
    assert_eq!(effect(&hook, "modification"), 1);
}

#[test]
fn test_an_eventless_blocking_record_credits_its_denial() {
    let session = format!(
        "{}\n{}",
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"attachment","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","attachment":{"type":"hook_blocking_error","hookName":"guard.sh","command":"guard.sh","toolUseId":"c1","content":"blocked","exitCode":2}}"#
    );
    let session = format!(
        "{session}\n{}",
        call_and_outcome()
            .lines()
            .nth(1)
            .expect("the outcome line stands")
    );
    let entries = run(SETTINGS, &session);
    let hook = entry(&entries, "guard.sh");
    assert_eq!(class_of(&hook), "firing");
    assert_eq!(
        effect(&hook, "denial"),
        1,
        "the one denial is the blocking record's own"
    );
}

#[test]
fn test_an_eventless_record_identifying_hooks_of_two_events_stays_outside() {
    let settings = r#"{"hooks": {
        "PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "shared.sh"}]}],
        "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "shared.sh"}]}]
    }}"#;
    let session = [
        r#"{"type":"user","timestamp":"2026-01-01T00:00:01Z","sessionId":"s1","message":{"content":"hello"}}"#,
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"attachment","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"shared.sh","command":"shared.sh","toolUseId":"c1","content":"added context","exitCode":0}}"#,
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#,
    ]
    .join("\n");
    let entries = run(settings, &session);
    assert_eq!(entries.len(), 2);
    for hook in &entries {
        assert_eq!(
            evidence(hook, "matched"),
            1,
            "each hook matches only its own subject"
        );
        assert_eq!(
            effect(hook, "consumed_output"),
            0,
            "the unplaced record feeds no effect"
        );
    }
}

#[test]
fn test_an_eventless_record_identifying_no_hook_stays_outside() {
    let session = format!(
        "{}\n{}",
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"attachment","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"other.sh","command":"other.sh","toolUseId":"c1","content":"","exitCode":0,"stdout":"{\"hookSpecificOutput\": {\"updatedInput\": {\"command\": \"ls -la\"}}}"}}"#
    );
    let session = format!(
        "{session}\n{}",
        call_and_outcome()
            .lines()
            .nth(1)
            .expect("the outcome line stands")
    );
    let entries = run(SETTINGS, &session);
    let hook = entry(&entries, "guard.sh");
    assert_eq!(class_of(&hook), "firing-without-effect");
    assert_eq!(evidence(&hook, "matched"), 1);
    assert_eq!(effect(&hook, "modification"), 0);
}

#[test]
fn test_an_eventless_record_identifying_two_hooks_of_one_event_counts_for_both() {
    let settings = r#"{"hooks": {"PreToolUse": [
        {"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]},
        {"matcher": "Edit|Write", "hooks": [{"type": "command", "command": "guard.sh"}]}
    ]}}"#;
    let session = [
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"attachment","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"guard.sh","command":"guard.sh","toolUseId":"c1","content":"added context","exitCode":0}}"#,
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#,
    ]
    .join("\n");
    let entries = run(settings, &session);
    assert_eq!(entries.len(), 2);
    for hook in &entries {
        assert_eq!(class_of(hook), "firing");
        assert_eq!(effect(hook, "consumed_output"), 1);
    }
    let edit_hook = entries
        .iter()
        .find(|entry| entry.get("matcher").and_then(JsonValue::as_text) == Some("Edit|Write"))
        .expect("the edit hook stands");
    assert_eq!(
        evidence(edit_hook, "ran"),
        1,
        "the record alone runs the hook no call matched"
    );
}

#[test]
fn test_a_disagreeing_eventless_record_stays_outside() {
    let settings = r#"{"hooks": {"PreToolUse": [
        {"matcher": "Bash", "hooks": [{"type": "command", "command": "a.sh"}]},
        {"matcher": "Write", "hooks": [{"type": "command", "command": "b.sh"}]}
    ]}}"#;
    let session = [
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#,
        r#"{"type":"attachment","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","attachment":{"type":"hook_success","hookName":"b.sh","command":"a.sh","toolUseId":"c1","content":"added context","exitCode":0}}"#,
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#,
    ]
    .join("\n");
    let entries = run(settings, &session);
    let a_hook = entry(&entries, "a.sh");
    assert_eq!(
        evidence(&a_hook, "matched"),
        1,
        "the call alone matches the first hook"
    );
    assert_eq!(effect(&a_hook, "consumed_output"), 0);
    let b_hook = entry(&entries, "b.sh");
    assert_eq!(class_of(&b_hook), "silent");
}
