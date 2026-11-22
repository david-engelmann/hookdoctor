//! The pins of slice 36, the name form prefix
//! standings of the identification reading, over
//! records the suite builds in code and hands to
//! the examination directly. HD2-3 states the name
//! form in summary words, the event alone or the
//! event joined to the hook matcher or the command
//! standing alone as the name, and no sentence
//! states the prefix arm the reading carries, a
//! name standing as the event joined by a colon to
//! a suffix agreeing where the suffix stands as
//! the matcher text whole or as the command, the
//! matcher standing absent reading as the empty
//! text at that seat. The pins below walk that arm
//! seat by seat, the command suffix and the
//! trailing colon and the disagreement a joined
//! name carrying neither suffix brings and the
//! literal grain of the suffix itself. Every pin
//! stands as a fence on the landed code, this
//! slice moving no production code.

use hookdoctor::examine::{examine_hooks, unconfigured_runs};
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

fn run_record(
    line: i64,
    event: &str,
    name: &str,
    command: Option<&str>,
    content: &str,
) -> TraceRecord {
    let mut built = record("hook_run", Some("sess"), 0, line);
    built.hook_event = Some(event.to_string());
    built.hook_name = Some(name.to_string());
    built.command = command.map(str::to_string);
    built.content = content.to_string();
    built
}

fn hook(id: &str, event: &str, matcher: Option<&str>, command: &str) -> HookConfig {
    HookConfig {
        id: id.to_string(),
        event: event.to_string(),
        matcher: matcher.map(str::to_string),
        hook_type: "command".to_string(),
        command: command.to_string(),
    }
}

fn labels(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

fn int_at(entry: &JsonValue, key: &str) -> i64 {
    match entry.get(key) {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the entry carries {key}"),
    }
}

fn standing_of(entry: &JsonValue) -> (String, i64, i64, i64, String) {
    let id = match entry.get("id") {
        Some(JsonValue::Text(text)) => text.clone(),
        _ => panic!("the entry carries its id"),
    };
    let class = match entry.get("class") {
        Some(JsonValue::Text(text)) => text.clone(),
        _ => panic!("the entry carries its class"),
    };
    let evidence = entry.get("evidence").expect("the entry carries evidence");
    let effects = evidence
        .get("effects")
        .expect("the evidence carries effects");
    let total = int_at(effects, "denial")
        + int_at(effects, "modification")
        + int_at(effects, "consumed_output");
    (
        id,
        int_at(evidence, "matched"),
        int_at(evidence, "ran"),
        total,
        class,
    )
}

fn entry_standings(
    hooks: &[HookConfig],
    records: &[TraceRecord],
) -> Vec<(String, i64, i64, i64, String)> {
    let sources = labels(&["only.jsonl"]);
    let (entries, _misses) =
        examine_hooks(hooks, records, None, None, &sources).expect("the examination stands");
    entries.iter().map(standing_of).collect()
}

fn unconfigured_list(hooks: &[HookConfig], records: &[TraceRecord]) -> Vec<JsonValue> {
    let sources = labels(&["only.jsonl"]);
    unconfigured_runs(hooks, records, None, None, &sources).expect("the list stands")
}

fn standing(
    id: &str,
    matched: i64,
    ran: i64,
    effects: i64,
    class: &str,
) -> (String, i64, i64, i64, String) {
    (id.to_string(), matched, ran, effects, class.to_string())
}

#[test]
fn test_a_run_record_named_with_its_event_joined_to_its_hooks_command_identifies_that_hook_and_no_sibling(
) {
    let hooks = [
        hook("Stop:0:0", "Stop", None, "wrap-up.sh"),
        hook("PreToolUse:0:0", "PreToolUse", Some("Bash"), "guard.sh"),
        hook("PreToolUse:1:0", "PreToolUse", Some("Edit"), "scribe.sh"),
    ];
    let records = [
        run_record(1, "Stop", "Stop:wrap-up.sh", None, "wrapped"),
        run_record(2, "PreToolUse", "PreToolUse:guard.sh", None, "guarded"),
    ];
    assert_eq!(
        entry_standings(&hooks, &records),
        vec![
            standing("Stop:0:0", 1, 1, 1, "firing"),
            standing("PreToolUse:0:0", 1, 1, 1, "firing"),
            standing("PreToolUse:1:0", 0, 0, 0, "silent"),
        ]
    );
    assert_eq!(unconfigured_list(&hooks, &records).len(), 0);
}

#[test]
fn test_a_run_record_named_with_its_event_and_a_trailing_colon_identifies_a_hook_carrying_no_matcher_or_an_empty_matcher_and_no_hook_carrying_a_matcher(
) {
    let matcherless = [
        hook("PreToolUse:0:0", "PreToolUse", None, "alpha.sh"),
        hook("PreToolUse:1:0", "PreToolUse", Some(""), "beta.sh"),
    ];
    let records = [run_record(1, "PreToolUse", "PreToolUse:", None, "ran")];
    assert_eq!(
        entry_standings(&matcherless, &records),
        vec![
            standing("PreToolUse:0:0", 1, 1, 1, "firing"),
            standing("PreToolUse:1:0", 1, 1, 1, "firing"),
        ]
    );
    assert_eq!(unconfigured_list(&matcherless, &records).len(), 0);
    let standing_matcher = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        Some("Bash"),
        "gamma.sh",
    )];
    assert_eq!(
        entry_standings(&standing_matcher, &records),
        vec![standing("PreToolUse:0:0", 0, 0, 0, "silent")]
    );
    let runs = unconfigured_list(&standing_matcher, &records);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].get("name"), Some(&JsonValue::text("PreToolUse:")));
    assert_eq!(runs[0].get("command"), Some(&JsonValue::Null));
    let pattern_matcher = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        Some("B.*"),
        "delta.sh",
    )];
    assert_eq!(
        entry_standings(&pattern_matcher, &records),
        vec![standing("PreToolUse:0:0", 0, 0, 0, "silent")]
    );
    assert_eq!(unconfigured_list(&pattern_matcher, &records).len(), 1);
}

#[test]
fn test_a_run_record_whose_name_stands_as_its_event_joined_to_an_unrelated_word_identifies_no_hook_beside_a_command_that_identifies_one_and_stands_in_the_unconfigured_list(
) {
    let guard = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        Some("Bash"),
        "guard.sh",
    )];
    let records = [run_record(
        1,
        "PreToolUse",
        "PreToolUse:whatever",
        Some("guard.sh"),
        "ran",
    )];
    assert_eq!(
        entry_standings(&guard, &records),
        vec![standing("PreToolUse:0:0", 0, 0, 0, "silent")]
    );
    let runs = unconfigured_list(&guard, &records);
    assert_eq!(runs.len(), 1);
    assert_eq!(
        runs[0].get("name"),
        Some(&JsonValue::text("PreToolUse:whatever"))
    );
    assert_eq!(runs[0].get("command"), Some(&JsonValue::text("guard.sh")));
    assert_eq!(runs[0].get("event"), Some(&JsonValue::text("PreToolUse")));
    let matcherless = [hook("PreToolUse:0:0", "PreToolUse", None, "alpha.sh")];
    let alone = [run_record(
        1,
        "PreToolUse",
        "PreToolUse:whatever",
        None,
        "ran",
    )];
    assert_eq!(
        entry_standings(&matcherless, &alone),
        vec![standing("PreToolUse:0:0", 0, 0, 0, "silent")]
    );
    let alone_runs = unconfigured_list(&matcherless, &alone);
    assert_eq!(alone_runs.len(), 1);
    assert_eq!(alone_runs[0].get("command"), Some(&JsonValue::Null));
}

#[test]
fn test_the_suffix_of_a_joined_name_stands_as_the_matcher_text_whole_or_the_command_and_a_suffix_standing_as_neither_identifies_no_hook(
) {
    let pattern = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        Some("B.*"),
        "delta.sh",
    )];
    let whole = [run_record(1, "PreToolUse", "PreToolUse:B.*", None, "ran")];
    assert_eq!(
        entry_standings(&pattern, &whole),
        vec![standing("PreToolUse:0:0", 1, 1, 1, "firing")]
    );
    let guard = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        Some("Bash"),
        "guard.sh",
    )];
    let lowered = [run_record(1, "PreToolUse", "pretooluse:Bash", None, "ran")];
    assert_eq!(
        entry_standings(&guard, &lowered),
        vec![standing("PreToolUse:0:0", 0, 0, 0, "silent")]
    );
    let further = [run_record(
        1,
        "PreToolUse",
        "PreToolUse:Bash:extra",
        None,
        "ran",
    )];
    assert_eq!(
        entry_standings(&guard, &further),
        vec![standing("PreToolUse:0:0", 0, 0, 0, "silent")]
    );
    assert_eq!(unconfigured_list(&guard, &further).len(), 1);
    let control = [run_record(1, "PreToolUse", "PreToolUse:Bash", None, "ran")];
    assert_eq!(
        entry_standings(&guard, &control),
        vec![standing("PreToolUse:0:0", 1, 1, 1, "firing")]
    );
}
