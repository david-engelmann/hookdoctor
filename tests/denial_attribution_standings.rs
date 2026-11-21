//! The pins of slice 28, the denial attribution
//! first occurrence standings, over records the
//! suite builds in code and hands to the
//! examination directly. The family text states
//! the attribution shape and the credited and
//! uncredited standings, and no sentence states
//! the rule of the first attribution shaped
//! occurrence, the shaped first party governing
//! whole and an unshaped first token governing
//! as no attribution and a denial word standing
//! short of the whole shape deciding nothing.
//! The pins below walk that rule occurrence by
//! occurrence, each handing the examination two
//! hooks matching one call and one error outcome
//! carrying its crafted text, and reading both
//! entries their denial counts and classes.
//! Every pin stands as a fence on the landed
//! code, this slice moving no production code.

use hookdoctor::examine::examine_hooks;
use hookdoctor::ingest::{HookConfig, TraceRecord};
use hookdoctor::jsonvalue::JsonValue;

fn record(kind: &str, session: Option<&str>, file_index: i64, line: i64) -> TraceRecord {
    TraceRecord {
        kind: kind.to_string(),
        timestamp: None,
        session: session.map(str::to_string),
        file_index,
        line,
        tool_name: None,
        call_id: None,
        is_error: false,
        text: String::new(),
        hook_name: None,
        hook_event: None,
        command: None,
        content: String::new(),
        stdout: String::new(),
        exit_code: None,
        attachment_type: None,
        commands: Vec::new(),
        prevented_continuation: false,
        has_output: false,
    }
}

fn tool_call(
    session: Option<&str>,
    file_index: i64,
    line: i64,
    tool: &str,
    id: &str,
) -> TraceRecord {
    let mut built = record("tool_call", session, file_index, line);
    built.tool_name = Some(tool.to_string());
    built.call_id = Some(id.to_string());
    built
}

fn error_outcome(
    session: Option<&str>,
    file_index: i64,
    line: i64,
    id: &str,
    text: &str,
) -> TraceRecord {
    let mut built = record("tool_outcome", session, file_index, line);
    built.call_id = Some(id.to_string());
    built.is_error = true;
    built.text = text.to_string();
    built
}

fn hook(id: &str, matcher: &str, command: &str) -> HookConfig {
    HookConfig {
        id: id.to_string(),
        event: "PreToolUse".to_string(),
        matcher: Some(matcher.to_string()),
        hook_type: "command".to_string(),
        command: command.to_string(),
    }
}

fn labels(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

fn denial_and_class(entry: &JsonValue) -> (i64, String) {
    let denial = match entry
        .get("evidence")
        .and_then(|evidence| evidence.get("effects"))
        .and_then(|effects| effects.get("denial"))
    {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the entry carries its denial count"),
    };
    let class = match entry.get("class") {
        Some(JsonValue::Text(text)) => text.clone(),
        _ => panic!("the entry carries its class"),
    };
    (denial, class)
}

fn standings(text: &str) -> Vec<(i64, String)> {
    let hooks = [
        hook("PreToolUse:0:0", "Bash", "first-hook.sh"),
        hook("PreToolUse:1:0", "Bash", "second-hook.sh"),
    ];
    let sources = labels(&["only.jsonl"]);
    let records = [
        tool_call(Some("sess"), 0, 1, "Bash", "call-1"),
        error_outcome(Some("sess"), 0, 2, "call-1", text),
    ];
    let (entries, _misses) =
        examine_hooks(&hooks, &records, None, None, &sources).expect("the examination stands");
    entries.iter().map(denial_and_class).collect()
}

fn standing(denial: i64, class: &str) -> (i64, String) {
    (denial, class.to_string())
}

#[test]
fn test_the_first_attribution_decides_over_a_later_attribution() {
    assert_eq!(
        standings(
            "the call was denied by first-hook.sh and the retry was denied by second-hook.sh"
        ),
        vec![standing(1, "firing"), standing(0, "firing-without-effect")]
    );
}

#[test]
fn test_an_unshaped_first_party_blocks_a_later_attribution() {
    assert_eq!(
        standings("the call was denied by policy and the retry was denied by second-hook.sh"),
        vec![standing(1, "firing"), standing(1, "firing")]
    );
}

#[test]
fn test_a_first_attribution_to_an_unconfigured_party_credits_no_hook_despite_a_later_configured_party(
) {
    assert_eq!(
        standings(
            "the call was denied by outside-party.sh and the retry was denied by second-hook.sh"
        ),
        vec![
            standing(0, "firing-without-effect"),
            standing(0, "firing-without-effect")
        ]
    );
}

#[test]
fn test_a_bare_denial_word_does_not_decide_the_attribution() {
    assert_eq!(
        standings("the request was denied. the call was denied by second-hook.sh"),
        vec![standing(0, "firing-without-effect"), standing(1, "firing")]
    );
}

#[test]
fn test_denied_by_with_no_token_standing_does_not_decide_the_attribution() {
    assert_eq!(
        standings(
            "the call was denied by (see the log) and the retry was denied by second-hook.sh"
        ),
        vec![standing(0, "firing-without-effect"), standing(1, "firing")]
    );
}

#[test]
fn test_an_attribution_behind_a_word_character_does_not_decide() {
    assert_eq!(
        standings(
            "the log spelled the outcome undenied by first-hook.sh, and the retry stands denied by second-hook.sh"
        ),
        vec![standing(0, "firing-without-effect"), standing(1, "firing")]
    );
}
