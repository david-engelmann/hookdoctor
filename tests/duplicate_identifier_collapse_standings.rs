//! The pins of slice 58, the duplicate identifier
//! collapse standings, over hooks and records the
//! suite builds in code and hands to the library
//! directly and over one full call of the shipped
//! binary. The order of the near miss list stands
//! stated at NM-1 and the dedup at NM-2 and the
//! gathering and the tie at BO-3 and BO-4 with
//! the correction recorded against BO-4, and this
//! slice states the collapse of two hooks handed
//! under one identifier into the one place of the
//! order map, at DC-1 to DC-6. Every pin stands
//! as a fence on the landed code, this slice
//! moving no production code.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::examine::{
    build_evidence, build_report, examine_hooks, unconfigured_runs, validate_evidence,
    validate_markdown, validate_report,
};
use hookdoctor::ingest::{parse_config, HookConfig, TraceRecord};
use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::report::render_markdown;

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

fn hook(id: &str, event: &str, matcher: &str, command: &str) -> HookConfig {
    HookConfig {
        id: id.to_string(),
        event: event.to_string(),
        matcher: Some(matcher.to_string()),
        hook_type: "command".to_string(),
        command: command.to_string(),
    }
}

fn hook_run(command: &str, stamp: &str, line: i64) -> TraceRecord {
    let mut built = record("hook_run", Some("sess"), 0, line);
    built.command = Some(command.to_string());
    built.hook_event = Some("PreToolUse".to_string());
    built.timestamp = Some(stamp.to_string());
    built
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

fn entries_of(hooks: &[HookConfig], records: &[TraceRecord], sources: &[String]) -> Vec<JsonValue> {
    let (entries, _misses) =
        examine_hooks(hooks, records, None, None, sources).expect("the examination stands");
    entries
}

fn entry_class(entry: &JsonValue) -> String {
    match entry.get("class") {
        Some(JsonValue::Text(text)) => text.clone(),
        _ => panic!("the entry carries its class"),
    }
}

fn entry_ran(entry: &JsonValue) -> i64 {
    let evidence = entry
        .get("evidence")
        .expect("the entry carries its evidence");
    match evidence.get("ran") {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the evidence carries its ran count"),
    }
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
        miss_text(miss, "event").expect("the miss carries its event"),
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
    reason: &str,
    event: &str,
    tool: &str,
    stamp: &str,
    source: &str,
    line: i64,
) -> (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
) {
    (
        hook_id.to_string(),
        reason.to_string(),
        event.to_string(),
        Some(tool.to_string()),
        Some(stamp.to_string()),
        Some(source.to_string()),
        line,
    )
}

const T: &str = "2026-01-01T09:00:00Z";
const T_EARLY: &str = "2026-01-01T08:00:00Z";
const T_LATE: &str = "2026-01-01T10:00:00Z";

fn three_calls() -> Vec<TraceRecord> {
    vec![
        tool_call(Some("sess"), 0, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
        tool_call(Some("sess"), 0, 1, T, "write", "call-3"),
    ]
}

fn separated_pair() -> Vec<HookConfig> {
    vec![
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("mid", "PreToolUse", "Edit", "guard-mid.sh"),
        hook("dup", "PreToolUse", "Write", "guard-two.sh"),
    ]
}

#[test]
fn test_the_displaced_hook_of_a_separated_pair_keys_on_the_later_place_standing_behind_the_between_hook_and_the_pair_flips_with_its_handing_and_the_hook_place_decides_ahead_of_the_instant(
) {
    let sources = labels(&["only.jsonl"]);
    let hooks = separated_pair();
    let misses = near_misses(&hooks, &three_calls(), &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("mid", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let flipped = [
        hook("dup", "PreToolUse", "Write", "guard-two.sh"),
        hook("mid", "PreToolUse", "Edit", "guard-mid.sh"),
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
    ];
    let misses = near_misses(&flipped, &three_calls(), &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("mid", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
        ]
    );

    let records = [
        tool_call(Some("sess"), 0, 1, T_EARLY, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
        tool_call(Some("sess"), 0, 1, T, "write", "call-3"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("mid", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "bash",
                T_EARLY,
                "only.jsonl",
                1
            ),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let hooks = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("mid1", "PreToolUse", "Edit", "guard-mid1.sh"),
        hook("mid2", "PreToolUse", "Read", "guard-mid2.sh"),
        hook("dup", "PreToolUse", "Write", "guard-two.sh"),
    ];
    let records = [
        tool_call(Some("sess"), 0, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
        tool_call(Some("sess"), 0, 1, T, "read", "call-3"),
        tool_call(Some("sess"), 0, 1, T, "write", "call-4"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "mid1",
                "name-form",
                "PreToolUse",
                "edit",
                T,
                "only.jsonl",
                1
            ),
            shape(
                "mid2",
                "name-form",
                "PreToolUse",
                "read",
                T,
                "only.jsonl",
                1
            ),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let hooks = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("mid", "PreToolUse", "Edit", "guard-mid1.sh"),
        hook("mid", "PreToolUse", "Write", "guard-mid2.sh"),
        hook("dup", "PreToolUse", "Read", "guard-two.sh"),
    ];
    let records = [
        tool_call(Some("sess"), 0, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
        tool_call(Some("sess"), 0, 1, T, "write", "call-3"),
        tool_call(Some("sess"), 0, 1, T, "read", "call-4"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("mid", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "mid",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "read", T, "only.jsonl", 1),
        ]
    );

    let hooks = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("mid", "PreToolUse", "Edit", "guard-mid.sh"),
        hook("dup", "PreToolUse", "write", "guard-two.sh"),
    ];
    let records = [
        tool_call(Some("sess"), 0, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
        tool_call(Some("sess"), 0, 1, T, "write", "call-3"),
    ];
    let entries = entries_of(&hooks, &records, &sources);
    assert_eq!(entries.len(), 3);
    assert_eq!(entry_class(&entries[0]), "silent");
    assert_eq!(entry_class(&entries[2]), "firing-without-effect");
    assert_eq!(entry_ran(&entries[2]), 1);
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("mid", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
        ]
    );

    let hooks = [
        hook("dup", "PreToolUse", "bash", "guard-one.sh"),
        hook("mid", "PreToolUse", "Edit", "guard-mid.sh"),
        hook("dup", "PreToolUse", "Write", "guard-two.sh"),
    ];
    let entries = entries_of(&hooks, &records, &sources);
    assert_eq!(entries.len(), 3);
    assert_eq!(entry_class(&entries[0]), "firing-without-effect");
    assert_eq!(entry_ran(&entries[0]), 1);
    assert_eq!(entry_class(&entries[2]), "silent");
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("mid", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_an_adjacent_pair_stands_at_its_shared_place_ahead_of_a_following_hook_and_its_misses_part_on_the_instant_and_on_the_line_against_the_gathering_order(
) {
    let sources = labels(&["only.jsonl"]);
    let hooks = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("dup", "PreToolUse", "Edit", "guard-two.sh"),
        hook("last", "PreToolUse", "Write", "guard-last.sh"),
    ];
    let misses = near_misses(&hooks, &three_calls(), &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "last",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let pair = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("dup", "PreToolUse", "Edit", "guard-two.sh"),
    ];
    let records = [
        tool_call(Some("sess"), 0, 1, T_LATE, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
    ];
    let misses = near_misses(&pair, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "bash",
                T_LATE,
                "only.jsonl",
                1
            ),
        ]
    );

    let records = [
        tool_call(Some("sess"), 0, 2, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
    ];
    let misses = near_misses(&pair, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 2),
        ]
    );

    let sources = labels(&["a.jsonl", "b.jsonl"]);
    let records = [
        tool_call(Some("sess"), 1, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-2"),
    ];
    let misses = near_misses(&pair, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("dup", "name-form", "PreToolUse", "edit", T, "a.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "bash", T, "b.jsonl", 1),
        ]
    );
}

#[test]
fn test_three_hooks_under_one_identifier_share_the_place_of_the_last_of_them_standing_behind_a_hook_handed_between_and_parting_on_the_instant_and_in_gathering_order_at_a_tie(
) {
    let sources = labels(&["only.jsonl"]);
    let hooks = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("mid", "PreToolUse", "Read", "guard-mid.sh"),
        hook("dup", "PreToolUse", "Edit", "guard-two.sh"),
        hook("dup", "PreToolUse", "Write", "guard-three.sh"),
    ];
    let records = [
        tool_call(Some("sess"), 0, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T, "read", "call-2"),
        tool_call(Some("sess"), 0, 1, T, "edit", "call-3"),
        tool_call(Some("sess"), 0, 1, T, "write", "call-4"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("mid", "name-form", "PreToolUse", "read", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let three = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("dup", "PreToolUse", "Edit", "guard-two.sh"),
        hook("dup", "PreToolUse", "Write", "guard-three.sh"),
    ];
    let records = [
        tool_call(Some("sess"), 0, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 1, T_EARLY, "edit", "call-2"),
        tool_call(Some("sess"), 0, 1, T, "write", "call-3"),
    ];
    let misses = near_misses(&three, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "edit",
                T_EARLY,
                "only.jsonl",
                1
            ),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let misses = near_misses(&three, &three_calls(), &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let flipped = [
        hook("dup", "PreToolUse", "Write", "guard-three.sh"),
        hook("dup", "PreToolUse", "Edit", "guard-two.sh"),
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
    ];
    let misses = near_misses(&flipped, &three_calls(), &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                1
            ),
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
        ]
    );

    let records = [
        tool_call(Some("sess"), 0, 1, T, "bash", "call-1"),
        tool_call(Some("sess"), 0, 9, T, "bash", "call-2"),
        tool_call(Some("sess"), 0, 3, T, "edit", "call-3"),
        tool_call(Some("sess"), 0, 7, T, "edit", "call-4"),
        tool_call(Some("sess"), 0, 5, T, "write", "call-5"),
    ];
    let misses = near_misses(&three, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 3),
            shape(
                "dup",
                "name-form",
                "PreToolUse",
                "write",
                T,
                "only.jsonl",
                5
            ),
            shape("dup", "name-form", "PreToolUse", "edit", T, "only.jsonl", 7),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 9),
        ]
    );
}

#[test]
fn test_the_collapse_leaves_the_entries_and_the_miss_content_standing_one_entry_per_handed_hook_each_standing_as_its_solo_entry_and_two_misses_of_one_shape_under_one_matcher(
) {
    let sources = labels(&["only.jsonl"]);
    let hooks = separated_pair();
    let records = three_calls();
    let entries = entries_of(&hooks, &records, &sources);
    assert_eq!(entries.len(), 3);
    let ids: Vec<Option<String>> = entries
        .iter()
        .map(|entry| match entry.get("id") {
            Some(JsonValue::Text(text)) => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        ids,
        vec![
            Some("dup".to_string()),
            Some("mid".to_string()),
            Some("dup".to_string())
        ]
    );
    for (index, handed) in hooks.iter().enumerate() {
        let solo_entries = entries_of(std::slice::from_ref(handed), &records, &sources);
        assert_eq!(solo_entries.len(), 1);
        assert_eq!(entries[index], solo_entries[0]);
    }

    let pair = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("dup", "PreToolUse", "Bash", "guard-two.sh"),
    ];
    let one_call = [tool_call(Some("sess"), 0, 1, T, "bash", "call-1")];
    let misses = near_misses(&pair, &one_call, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
            shape("dup", "name-form", "PreToolUse", "bash", T, "only.jsonl", 1),
        ]
    );

    let mut population = shapes(&near_misses(&hooks, &records, &sources));
    population.sort();
    let mut union: Vec<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
    )> = Vec::new();
    for handed in &hooks {
        let solo_misses = near_misses(std::slice::from_ref(handed), &records, &sources);
        union.extend(shapes(&solo_misses));
    }
    union.sort();
    assert_eq!(population, union);
}

#[test]
fn test_the_evidence_carries_one_verdict_per_handed_hook_in_handing_order_and_a_report_built_over_a_duplicated_population_validates_as_true_in_both_its_readings(
) {
    let sources = labels(&["only.jsonl"]);
    let hooks = separated_pair();
    let records = three_calls();
    let evidence = build_evidence(&hooks, &records, None, None).expect("the evidence builds");
    let verdicts = match evidence.get("verdicts") {
        Some(JsonValue::List(items)) => items.clone(),
        _ => panic!("the evidence carries its verdicts"),
    };
    assert_eq!(verdicts.len(), 3);
    let commands: Vec<Option<String>> = verdicts
        .iter()
        .map(|verdict| match verdict.get("command") {
            Some(JsonValue::Text(text)) => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        commands,
        vec![
            Some("guard-one.sh".to_string()),
            Some("guard-mid.sh".to_string()),
            Some("guard-two.sh".to_string())
        ]
    );

    let report =
        build_report(&hooks, &records, "config", &sources, None, None).expect("the report builds");
    assert_eq!(
        validate_report(&report, &hooks, &records, "config", &sources, None, None),
        Ok(true)
    );
    let rendered = render_markdown(&report);
    assert_eq!(
        validate_markdown(
            rendered.as_bytes(),
            &hooks,
            &records,
            "config",
            &sources,
            None,
            None
        ),
        Ok(true)
    );

    let evidence = build_evidence(&hooks, &records, None, None).expect("the evidence builds");
    let verdicts = match evidence.get("verdicts") {
        Some(JsonValue::List(items)) => items.clone(),
        _ => panic!("the evidence carries its verdicts"),
    };
    assert_eq!(verdicts.len(), 3);
    assert_eq!(
        validate_evidence(&evidence, &hooks, &records, None, None),
        Ok(true)
    );
    let mut swapped = verdicts.clone();
    swapped.swap(0, 2);
    let format = match evidence.get("format") {
        Some(value) => value.clone(),
        _ => panic!("the evidence carries its format"),
    };
    let tampered = JsonValue::object(vec![
        ("format", format),
        ("verdicts", JsonValue::List(swapped)),
    ]);
    assert!(validate_evidence(&tampered, &hooks, &records, None, None).is_err());

    let pair = [
        hook("dup", "PreToolUse", "Bash", "guard-one.sh"),
        hook("dup", "PreToolUse", "Write", "guard-two.sh"),
    ];
    let records = [
        hook_run("guard-one.sh", T, 1),
        hook_run("undeclared.sh", T, 2),
    ];
    let runs = unconfigured_runs(&pair, &records, None, None, &sources)
        .expect("the unconfigured runs stand");
    assert_eq!(runs.len(), 1);
    assert_eq!(
        match runs[0].get("command") {
            Some(JsonValue::Text(text)) => Some(text.clone()),
            _ => None,
        },
        Some("undeclared.sh".to_string())
    );
}

/// The settings document the door cell stands on,
/// one group carrying two entries under one
/// matcher, each entry carrying its own command.
const SETTINGS: &str = "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\", \"hooks\": [{\"type\": \"command\", \"command\": \"guard-one.sh\"}, {\"type\": \"command\", \"command\": \"guard-two.sh\"}]}]}}";

/// A scratch directory that removes itself when it
/// leaves scope, so a panicking assertion leaves no
/// state behind and concurrent test processes never
/// share a path.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn hold(path: &Path) -> ScratchDir {
        ScratchDir(path.to_path_buf())
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The standing of one full call of the door over a
/// cell directory the suite prepares itself.
struct DoorRun {
    _dir: ScratchDir,
    output: PathBuf,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn door_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("hookdoctor-dc-standings-{}", std::process::id()))
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale cell directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a cell directory stands");
    dir
}

fn run_call(dir: &Path, args: Vec<OsString>) -> DoorRun {
    let output = dir.join("output.json");
    let run = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .args(&args)
        .output()
        .expect("the shipped binary runs");
    DoorRun {
        _dir: ScratchDir::hold(dir),
        output,
        code: run.status.code(),
        stdout: run.stdout,
        stderr: run.stderr,
    }
}

fn word(text: &str) -> OsString {
    OsString::from(text)
}

fn session_line(tool: &str, id: &str, stamp: &str) -> String {
    let mut line = String::new();
    line.push_str("{\"type\": \"assistant\", \"timestamp\": \"");
    line.push_str(stamp);
    line.push_str("\", \"sessionId\": \"sess\", \"message\": {\"content\": [{\"type\": \"tool_use\", \"name\": \"");
    line.push_str(tool);
    line.push_str("\", \"id\": \"");
    line.push_str(id);
    line.push_str("\"}]}}\n");
    line
}

#[test]
fn test_the_configuration_reading_builds_its_identifiers_pairwise_distinct_in_document_order_and_at_the_door_two_entries_stand_under_their_own_identifiers_in_configuration_order(
) {
    let text = "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\", \"hooks\": [{\"command\": \"a.sh\"}, {\"command\": \"b.sh\"}]}, {\"matcher\": \"Edit\", \"hooks\": [{\"command\": \"c.sh\"}]}], \"PostToolUse\": [{\"hooks\": [{\"command\": \"d.sh\"}]}]}}";
    let document = parse_json(text).expect("the document parses");
    let built = parse_config(&document).ok().expect("the reading stands");
    let ids: Vec<String> = built
        .iter()
        .map(|hook_built| hook_built.id.clone())
        .collect();
    assert_eq!(
        ids,
        vec![
            "PreToolUse:0:0".to_string(),
            "PreToolUse:0:1".to_string(),
            "PreToolUse:1:0".to_string(),
            "PostToolUse:0:0".to_string()
        ]
    );
    let distinct: std::collections::HashSet<&String> = ids.iter().collect();
    assert_eq!(distinct.len(), ids.len());

    let dir = door_dir("two-entries");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, session_line("bash", "call-1", T)).expect("the session writes");
    let args = vec![
        word("--config"),
        config.into_os_string(),
        word("--session"),
        session.into_os_string(),
        word("--output"),
        dir.join("output.json").into_os_string(),
        word("--report"),
        dir.join("report.md").into_os_string(),
    ];
    let run = run_call(&dir, args);
    assert_eq!(run.code, Some(0));
    assert!(run.stdout.is_empty());
    assert!(run.stderr.is_empty());
    let text = std::fs::read_to_string(&run.output).expect("the report document reads");
    let document = parse_json(&text).expect("the report document parses");
    let hook_entries = match document.get("hooks") {
        Some(JsonValue::List(items)) => items.clone(),
        _ => panic!("the report carries its hooks"),
    };
    let entry_ids: Vec<Option<String>> = hook_entries
        .iter()
        .map(|entry| match entry.get("id") {
            Some(JsonValue::Text(text)) => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        entry_ids,
        vec![
            Some("PreToolUse:0:0".to_string()),
            Some("PreToolUse:0:1".to_string())
        ]
    );
    let misses = match document.get("near_misses") {
        Some(JsonValue::List(items)) => items.clone(),
        _ => panic!("the report carries its near miss list"),
    };
    let miss_hooks: Vec<Option<String>> = misses
        .iter()
        .map(|miss| miss_text(miss, "hook_id"))
        .collect();
    assert_eq!(
        miss_hooks,
        vec![
            Some("PreToolUse:0:0".to_string()),
            Some("PreToolUse:0:1".to_string())
        ]
    );
}
