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

#[test]
fn test_two_subjects_parting_on_the_event_alone_yield_two_near_misses_for_one_hook() {
    let mut subagent_hook = hook("SubagentStop:0:0", "", "guard-subagent.sh");
    subagent_hook.event = "SubagentStop".to_string();
    subagent_hook.matcher = None;
    let hooks = [subagent_hook];
    let sources = labels(&["only.jsonl"]);
    let mut plain = record("hook_run", None, 0, 1);
    plain.hook_event = Some("Stop".to_string());
    plain.timestamp = Some("2026-01-01T09:00:00Z".to_string());
    let mut doubled = record("hook_run", None, 0, 1);
    doubled.hook_event = Some("SubagentSubagentStop".to_string());
    doubled.timestamp = Some("2026-01-01T09:01:00Z".to_string());
    let records = [plain, doubled];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            (
                "SubagentStop:0:0".to_string(),
                "subagent-prefix".to_string(),
                None,
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                1,
            ),
            (
                "SubagentStop:0:0".to_string(),
                "subagent-prefix".to_string(),
                None,
                Some("2026-01-01T09:01:00Z".to_string()),
                Some("only.jsonl".to_string()),
                1,
            ),
        ]
    );
    assert_eq!(
        misses
            .iter()
            .map(|miss| (miss_text(miss, "event"), miss_text(miss, "detail")))
            .collect::<Vec<_>>(),
        vec![
            (
                Some("Stop".to_string()),
                Some("event Stop differs from SubagentStop by the Subagent prefix".to_string(),),
            ),
            (
                Some("SubagentSubagentStop".to_string()),
                Some(
                    "event SubagentSubagentStop differs from SubagentStop by the Subagent prefix"
                        .to_string(),
                ),
            ),
        ]
    );
}

#[test]
fn test_the_name_form_reason_stands_ahead_of_the_one_letter_reason_where_both_edits_apply_and_the_fold_takes_case_and_both_separators_together_over_the_whole_name(
) {
    let hooks = [
        hook("PreToolUse:0:0", "Bash", "guard-bash.sh"),
        hook("PreToolUse:1:0", "My_Tool", "guard-my-tool.sh"),
        hook("PreToolUse:2:0", "my_tool", "guard-my-tool-lower.sh"),
        hook("PreToolUse:3:0", "My-Tool", "guard-my-tool-hyphen.sh"),
    ];
    let sources = labels(&["only.jsonl"]);
    let records = [
        tool_call(Some("sess"), 0, 1, "2026-01-01T09:00:00Z", "bash", "call-1"),
        tool_call(Some("sess"), 0, 2, "2026-01-01T09:00:00Z", "Bosh", "call-2"),
        tool_call(Some("sess"), 0, 3, "2026-01-01T09:00:00Z", "Bsah", "call-3"),
        tool_call(
            Some("sess"),
            0,
            4,
            "2026-01-01T09:00:00Z",
            "mytool",
            "call-4",
        ),
        tool_call(
            Some("sess"),
            0,
            5,
            "2026-01-01T09:00:00Z",
            "MY_TOOL",
            "call-5",
        ),
        tool_call(
            Some("sess"),
            0,
            6,
            "2026-01-01T09:00:00Z",
            "my-tool",
            "call-6",
        ),
        tool_call(
            Some("sess"),
            0,
            7,
            "2026-01-01T09:00:00Z",
            "my_wool",
            "call-7",
        ),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            (
                "PreToolUse:0:0".to_string(),
                "name-form".to_string(),
                Some("bash".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                1
            ),
            (
                "PreToolUse:0:0".to_string(),
                "one-letter".to_string(),
                Some("Bosh".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                2
            ),
            (
                "PreToolUse:1:0".to_string(),
                "name-form".to_string(),
                Some("mytool".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                4
            ),
            (
                "PreToolUse:1:0".to_string(),
                "name-form".to_string(),
                Some("MY_TOOL".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                5
            ),
            (
                "PreToolUse:1:0".to_string(),
                "name-form".to_string(),
                Some("my-tool".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                6
            ),
            (
                "PreToolUse:2:0".to_string(),
                "name-form".to_string(),
                Some("mytool".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                4
            ),
            (
                "PreToolUse:2:0".to_string(),
                "name-form".to_string(),
                Some("MY_TOOL".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                5
            ),
            (
                "PreToolUse:2:0".to_string(),
                "name-form".to_string(),
                Some("my-tool".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                6
            ),
            (
                "PreToolUse:2:0".to_string(),
                "one-letter".to_string(),
                Some("my_wool".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                7
            ),
        ]
    );
}

#[test]
fn test_the_subagent_prefix_reason_stands_for_a_tool_less_subject_under_a_hook_of_any_matcher_standing_and_in_either_direction_with_the_prefix_in_its_exact_case(
) {
    let mut stop_hook = hook("Stop:0:0", "", "guard-stop.sh");
    stop_hook.event = "Stop".to_string();
    stop_hook.matcher = None;
    let mut subagent_hook = hook("SubagentStop:0:0", "", "guard-subagent-stop.sh");
    subagent_hook.event = "SubagentStop".to_string();
    subagent_hook.matcher = None;
    let pattern_hook = hook("PreToolUse:0:0", "B.*", "guard-pattern.sh");
    let hooks = [stop_hook, subagent_hook, pattern_hook];
    let sources = labels(&["only.jsonl"]);
    let mut first = record("hook_run", None, 0, 1);
    first.hook_event = Some("SubagentStop".to_string());
    first.timestamp = Some("2026-01-01T09:00:00Z".to_string());
    let mut second = record("hook_run", None, 0, 2);
    second.hook_event = Some("Stop".to_string());
    second.timestamp = Some("2026-01-01T09:01:00Z".to_string());
    let mut third = record("hook_run", None, 0, 3);
    third.hook_event = Some("SubagentPreToolUse".to_string());
    third.timestamp = Some("2026-01-01T09:02:00Z".to_string());
    let mut fourth = record("hook_run", None, 0, 4);
    fourth.hook_event = Some("subagentStop".to_string());
    fourth.timestamp = Some("2026-01-01T09:03:00Z".to_string());
    let records = [first, second, third, fourth];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            (
                "Stop:0:0".to_string(),
                "subagent-prefix".to_string(),
                None,
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                1
            ),
            (
                "SubagentStop:0:0".to_string(),
                "subagent-prefix".to_string(),
                None,
                Some("2026-01-01T09:01:00Z".to_string()),
                Some("only.jsonl".to_string()),
                2
            ),
            (
                "PreToolUse:0:0".to_string(),
                "subagent-prefix".to_string(),
                None,
                Some("2026-01-01T09:02:00Z".to_string()),
                Some("only.jsonl".to_string()),
                3
            ),
        ]
    );
    assert_eq!(
        misses
            .iter()
            .map(|miss| (miss_text(miss, "event"), miss_text(miss, "detail")))
            .collect::<Vec<_>>(),
        vec![
            (
                Some("SubagentStop".to_string()),
                Some("event SubagentStop differs from Stop by the Subagent prefix".to_string(),)
            ),
            (
                Some("Stop".to_string()),
                Some("event Stop differs from SubagentStop by the Subagent prefix".to_string(),)
            ),
            (
                Some("SubagentPreToolUse".to_string()),
                Some(
                    "event SubagentPreToolUse differs from PreToolUse by the Subagent prefix"
                        .to_string(),
                )
            ),
        ]
    );
}

#[test]
fn test_the_paired_event_reason_stands_for_a_tool_the_matcher_reading_matches_with_no_matcher_standing_or_a_plain_name_or_a_pattern_and_for_no_other_tool(
) {
    let mut matcherless = hook("PostToolUse:0:0", "", "log-any.sh");
    matcherless.event = "PostToolUse".to_string();
    matcherless.matcher = None;
    let mut plain = hook("PostToolUse:1:0", "Bash", "log-bash.sh");
    plain.event = "PostToolUse".to_string();
    let mut other_plain = hook("PostToolUse:2:0", "Edit", "log-edit.sh");
    other_plain.event = "PostToolUse".to_string();
    let mut pattern = hook("PostToolUse:3:0", "Ba.h", "log-pattern.sh");
    pattern.event = "PostToolUse".to_string();
    let mut other_pattern = hook("PostToolUse:4:0", "Ed.t", "log-other-pattern.sh");
    other_pattern.event = "PostToolUse".to_string();
    let same_event = hook("PreToolUse:0:0", "Bash+", "guard-bash-plus.sh");
    let hooks = [
        matcherless,
        plain,
        other_plain,
        pattern,
        other_pattern,
        same_event,
    ];
    let sources = labels(&["only.jsonl"]);
    let records = [tool_call(
        Some("sess"),
        0,
        1,
        "2026-01-01T09:00:00Z",
        "Bash",
        "call-1",
    )];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            (
                "PostToolUse:0:0".to_string(),
                "paired-event".to_string(),
                Some("Bash".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                1
            ),
            (
                "PostToolUse:1:0".to_string(),
                "paired-event".to_string(),
                Some("Bash".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                1
            ),
            (
                "PostToolUse:3:0".to_string(),
                "paired-event".to_string(),
                Some("Bash".to_string()),
                Some("2026-01-01T09:00:00Z".to_string()),
                Some("only.jsonl".to_string()),
                1
            ),
        ]
    );
    assert_eq!(
        misses
            .iter()
            .map(|miss| miss_text(miss, "detail"))
            .collect::<Vec<_>>(),
        vec![
            Some("event PreToolUse is the paired event of PostToolUse for tool Bash the matcher matches".to_string(),),
            Some("event PreToolUse is the paired event of PostToolUse for tool Bash the matcher matches".to_string(),),
            Some("event PreToolUse is the paired event of PostToolUse for tool Bash the matcher matches".to_string(),),
        ]
    );
}

#[test]
fn test_a_paired_subject_under_a_hook_whose_matcher_is_not_a_valid_pattern_refuses_the_examination_whole(
) {
    let mut broken_hook = hook("PostToolUse:0:0", "[a-", "log-broken.sh");
    broken_hook.event = "PostToolUse".to_string();
    let hooks = [broken_hook];
    let sources = labels(&["only.jsonl"]);
    let records = [tool_call(
        Some("sess"),
        0,
        1,
        "2026-01-01T09:00:00Z",
        "Bash",
        "call-1",
    )];
    let outcome = examine_hooks(&hooks, &records, None, None, &sources);
    assert!(outcome.is_err());
}
