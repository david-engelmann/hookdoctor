//! The pins of slice 27, the near miss order
//! and dedup standings, over records the suite
//! builds in code and hands to the examination
//! directly. The order of the near miss list
//! stands stated at HD2-11 in the grain of hook
//! and timestamp and line, with the amendment
//! beside that obligation reading the order
//! whole in prose, and the dedup stands stated
//! nowhere in its whole key. The pins below
//! walk the whole key of the order and the
//! whole key of the dedup, the instants against
//! the spellings, and the miss whose timestamp
//! reads as no instant standing at the earliest
//! instant of the grammar. Every pin stands as
//! a fence on the landed code, this slice
//! moving no production code.

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
    stamp: &str,
    tool: &str,
    id: &str,
) -> TraceRecord {
    let mut built = record("tool_call", session, file_index, line);
    built.tool_name = Some(tool.to_string());
    built.call_id = Some(id.to_string());
    built.timestamp = Some(stamp.to_string());
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

fn near_misses(
    hooks: &[HookConfig],
    records: &[TraceRecord],
    sources: &[String],
) -> Vec<JsonValue> {
    let (_entries, misses) =
        examine_hooks(hooks, records, None, None, sources).expect("the examination stands");
    misses
}

fn miss_text(miss: &JsonValue, field: &str) -> Option<String> {
    match miss.get(field) {
        Some(JsonValue::Text(text)) => Some(text.clone()),
        Some(JsonValue::Null) => None,
        _ => panic!("the miss carries its {field}"),
    }
}

fn miss_shape(
    miss: &JsonValue,
) -> (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
) {
    let line = match miss.get("line") {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the miss carries its line"),
    };
    (
        miss_text(miss, "hook_id").expect("the miss carries its hook"),
        miss_text(miss, "reason").expect("the miss carries its reason"),
        miss_text(miss, "tool"),
        miss_text(miss, "timestamp"),
        miss_text(miss, "source"),
        line,
    )
}

fn shapes(
    misses: &[JsonValue],
) -> Vec<(
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
)> {
    misses.iter().map(miss_shape).collect()
}

fn shape(
    hook_id: &str,
    tool: &str,
    stamp: &str,
    source: &str,
    line: i64,
) -> (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
) {
    (
        hook_id.to_string(),
        "name-form".to_string(),
        Some(tool.to_string()),
        Some(stamp.to_string()),
        Some(source.to_string()),
        line,
    )
}

#[test]
fn test_near_misses_stand_in_hook_order_then_instant_then_source_then_line() {
    let hooks = [
        hook("PreToolUse:0:0", "Bash", "guard-bash.sh"),
        hook("PreToolUse:1:0", "Edit", "guard-edit.sh"),
    ];
    let sources = labels(&["second.jsonl", "first.jsonl"]);
    let records = [
        tool_call(
            Some("sess-a"),
            0,
            9,
            "2026-01-01T10:00:00Z",
            "bash",
            "call-a",
        ),
        tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T09:00:00Z",
            "bash",
            "call-b",
        ),
        tool_call(
            Some("sess-b"),
            1,
            8,
            "2026-01-01T09:00:00Z",
            "bash",
            "call-d",
        ),
        tool_call(
            Some("sess-b"),
            1,
            2,
            "2026-01-01T09:00:00Z",
            "bash",
            "call-c",
        ),
        tool_call(
            Some("sess-a"),
            0,
            5,
            "2026-01-01T08:00:00Z",
            "edit",
            "call-e",
        ),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "first.jsonl",
                2
            ),
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "first.jsonl",
                8
            ),
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "second.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T10:00:00Z",
                "second.jsonl",
                9
            ),
            shape(
                "PreToolUse:1:0",
                "edit",
                "2026-01-01T08:00:00Z",
                "second.jsonl",
                5
            ),
        ]
    );
}

#[test]
fn test_near_misses_sort_by_instant_not_by_timestamp_spelling() {
    let hooks = [hook("PreToolUse:0:0", "Bash", "guard-bash.sh")];
    let sources = labels(&["only.jsonl"]);
    let records = [
        tool_call(Some("sess"), 0, 1, "2026-01-01T09:30:00Z", "bash", "call-y"),
        tool_call(
            Some("sess"),
            0,
            2,
            "2026-01-01T10:00:00+01:00",
            "bash",
            "call-x",
        ),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T10:00:00+01:00",
                "only.jsonl",
                2
            ),
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:30:00Z",
                "only.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_a_near_miss_whose_timestamp_reads_as_no_instant_stands_first_among_its_hook() {
    let hooks = [hook("PreToolUse:0:0", "Bash", "guard-bash.sh")];
    let sources = labels(&["only.jsonl"]);
    let records = [
        tool_call(Some("sess"), 0, 1, "2026-01-01T00:00:00Z", "bash", "call-p"),
        tool_call(Some("sess"), 0, 2, "not-a-timestamp", "bash", "call-q"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("PreToolUse:0:0", "bash", "not-a-timestamp", "only.jsonl", 2),
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T00:00:00Z",
                "only.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_one_near_miss_stands_per_reason_event_tool_file_and_line() {
    let hooks = [hook("PreToolUse:0:0", "Bash", "guard-bash.sh")];
    let sources = labels(&["same.jsonl", "same.jsonl"]);
    let records = [
        tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T09:00:00Z",
            "bash",
            "call-1",
        ),
        tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T09:00:00Z",
            "bash",
            "call-2",
        ),
        tool_call(
            Some("sess-b"),
            1,
            1,
            "2026-01-01T09:00:00Z",
            "bash",
            "call-3",
        ),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "same.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_a_near_miss_stands_for_each_silent_hook_where_two_hooks_share_a_matcher_over_one_call() {
    let hooks = [
        hook("PreToolUse:0:0", "Bash", "guard-one.sh"),
        hook("PreToolUse:1:0", "Bash", "guard-two.sh"),
    ];
    let sources = labels(&["only.jsonl"]);
    let records = [tool_call(
        Some("sess"),
        0,
        1,
        "2026-01-01T09:00:00Z",
        "bash",
        "call-1",
    )];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "only.jsonl",
                1
            ),
            shape(
                "PreToolUse:1:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "only.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_two_tools_standing_on_one_line_of_one_file_yield_two_near_misses_for_one_hook() {
    let hooks = [hook("PreToolUse:0:0", "Bash", "guard-bash.sh")];
    let sources = labels(&["only.jsonl"]);
    let records = [
        tool_call(Some("sess"), 0, 1, "2026-01-01T09:00:00Z", "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, "2026-01-01T09:00:00Z", "BASH", "call-2"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "only.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "BASH",
                "2026-01-01T09:00:00Z",
                "only.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_the_first_subject_in_record_order_survives_where_two_subjects_share_the_dedup_key_and_part_on_timestamp(
) {
    let hooks = [hook("PreToolUse:0:0", "Bash", "guard-bash.sh")];
    let sources = labels(&["only.jsonl"]);
    let records = [
        tool_call(Some("sess"), 0, 1, "2026-01-01T10:00:00Z", "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, "2026-01-01T08:00:00Z", "bash", "call-2"),
        tool_call(Some("sess"), 0, 2, "2026-01-01T09:00:00Z", "bash", "call-3"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T09:00:00Z",
                "only.jsonl",
                2
            ),
            shape(
                "PreToolUse:0:0",
                "bash",
                "2026-01-01T10:00:00Z",
                "only.jsonl",
                1
            ),
        ]
    );
}
