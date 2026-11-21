//! The pins of slice 21, the session source resolution,
//! over records the suite builds in code and hands to
//! the examination directly. The resolution stands
//! stated at rule level for the door in PORT2-3, and
//! the pins below walk the standings the obligations
//! of the slice state at the families the executed
//! cells cover, the label standing and the fallback
//! and the record carrying no session name and the
//! empty texts and the stop entries and the two seats
//! standing as one rule and the near miss order the
//! amendment beside HD2-11 completes. Every pin stands
//! as a fence on the landed code, this slice moving
//! no production code.

use hookdoctor::examine::{build_evidence, examine_hooks, unconfigured_runs};
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

fn hook_run(session: Option<&str>, file_index: i64, line: i64) -> TraceRecord {
    let mut built = record("hook_run", session, file_index, line);
    built.command = Some("mystery".to_string());
    built.hook_event = Some("PreToolUse".to_string());
    built
}

fn stop(session: Option<&str>, file_index: i64, line: i64) -> TraceRecord {
    let mut built = record("stop", session, file_index, line);
    built.commands = vec!["mystery-stop".to_string()];
    built
}

fn tool_call(
    session: Option<&str>,
    file_index: i64,
    line: i64,
    stamp: &str,
    id: &str,
) -> TraceRecord {
    let mut built = record("tool_call", session, file_index, line);
    built.tool_name = Some("bash".to_string());
    built.call_id = Some(id.to_string());
    built.timestamp = Some(stamp.to_string());
    built
}

fn bash_hook() -> HookConfig {
    HookConfig {
        id: "PreToolUse:0:0".to_string(),
        event: "PreToolUse".to_string(),
        matcher: Some("Bash".to_string()),
        hook_type: "command".to_string(),
        command: "guard".to_string(),
    }
}

fn labels(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

fn entry_source(entry: &JsonValue) -> Option<String> {
    match entry.get("source") {
        Some(JsonValue::Text(text)) => Some(text.clone()),
        Some(JsonValue::Null) => None,
        _ => panic!("the entry carries its source"),
    }
}

fn entry_line(entry: &JsonValue) -> i64 {
    match entry.get("line") {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the entry carries its line"),
    }
}

#[test]
fn test_the_label_standing_at_a_records_index_stands_as_its_source_at_the_unconfigured_seat_and_at_the_examination_seat(
) {
    let runs = unconfigured_runs(
        &[],
        &[
            hook_run(Some("sess-a"), 0, 1),
            hook_run(Some("sess-b"), 1, 2),
        ],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(entry_source(&runs[0]).as_deref(), Some("alpha"));
    let runs = unconfigured_runs(
        &[],
        &[
            hook_run(Some("sess-a"), 0, 1),
            hook_run(Some("sess-b"), 1, 2),
        ],
        None,
        None,
        &labels(&["alpha", "beta"]),
    )
    .unwrap();
    assert_eq!(entry_source(&runs[0]).as_deref(), Some("alpha"));
    assert_eq!(entry_source(&runs[1]).as_deref(), Some("beta"));
    let hooks = vec![bash_hook()];
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T00:00:00Z",
            "c1",
        )],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(misses.len(), 1);
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("alpha"));
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(
            Some("sess-b"),
            1,
            1,
            "2026-01-01T00:00:00Z",
            "c1",
        )],
        None,
        None,
        &labels(&["alpha", "beta"]),
    )
    .unwrap();
    assert_eq!(misses.len(), 1);
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("beta"));
}

#[test]
fn test_a_record_past_the_supplied_sources_reads_as_its_own_session_name_at_both_seats_and_a_record_at_a_negative_index_reads_the_same(
) {
    let runs = unconfigured_runs(
        &[],
        &[hook_run(Some("sess-b"), 1, 2)],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&runs[0]).as_deref(), Some("sess-b"));
    let runs = unconfigured_runs(
        &[],
        &[hook_run(Some("sess-neg"), -1, 4)],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&runs[0]).as_deref(), Some("sess-neg"));
    let runs = unconfigured_runs(
        &[],
        &[hook_run(Some("sess-a"), 0, 1)],
        None,
        None,
        &labels(&[]),
    )
    .unwrap();
    assert_eq!(entry_source(&runs[0]).as_deref(), Some("sess-a"));
    let hooks = vec![bash_hook()];
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(
            Some("sess-b"),
            1,
            1,
            "2026-01-01T00:00:00Z",
            "c1",
        )],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("sess-b"));
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(
            Some("sess-neg"),
            -1,
            1,
            "2026-01-01T00:00:00Z",
            "c1",
        )],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("sess-neg"));
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T00:00:00Z",
            "c1",
        )],
        None,
        None,
        &labels(&[]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("sess-a"));
}

#[test]
fn test_a_record_carrying_no_session_name_reads_as_no_source_at_both_seats_where_no_supplied_source_stands_for_it(
) {
    let runs = unconfigured_runs(
        &[],
        &[hook_run(None, 2, 3)],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(entry_source(&runs[0]), None);
    let runs = unconfigured_runs(
        &[],
        &[hook_run(None, -1, 5)],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&runs[0]), None);
    let runs = unconfigured_runs(&[], &[hook_run(None, 0, 2)], None, None, &labels(&[])).unwrap();
    assert_eq!(entry_source(&runs[0]), None);
    let hooks = vec![bash_hook()];
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(None, 2, 1, "2026-01-01T00:00:00Z", "c1")],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(misses.len(), 1);
    assert_eq!(entry_source(&misses[0]), None);
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(None, 0, 1, "2026-01-01T00:00:00Z", "c1")],
        None,
        None,
        &labels(&[]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]), None);
}

#[test]
fn test_a_supplied_source_standing_as_the_empty_text_is_taken_as_it_stands_and_a_records_own_empty_session_name_reads_as_an_empty_source_and_not_as_no_source(
) {
    let runs = unconfigured_runs(
        &[],
        &[
            hook_run(Some("sess-a"), 0, 1),
            hook_run(Some("sess-b"), 1, 2),
        ],
        None,
        None,
        &labels(&["", "beta"]),
    )
    .unwrap();
    assert_eq!(entry_source(&runs[0]).as_deref(), Some(""));
    assert_eq!(entry_source(&runs[1]).as_deref(), Some("beta"));
    let runs =
        unconfigured_runs(&[], &[hook_run(Some(""), 3, 3)], None, None, &labels(&[])).unwrap();
    assert_eq!(entry_source(&runs[0]).as_deref(), Some(""));
    let runs = unconfigured_runs(
        &[],
        &[hook_run(Some(""), 5, 3)],
        None,
        None,
        &labels(&["", "beta"]),
    )
    .unwrap();
    assert_eq!(entry_source(&runs[0]).as_deref(), Some(""));
    let hooks = vec![bash_hook()];
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T00:00:00Z",
            "c1",
        )],
        None,
        None,
        &labels(&[""]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]).as_deref(), Some(""));
}

#[test]
fn test_the_stop_entries_resolve_by_the_same_rule_as_the_hook_run_entries_at_the_unconfigured_seat()
{
    let runs = unconfigured_runs(
        &[],
        &[stop(Some("sess-b"), 1, 6), stop(Some("sess-a"), 0, 7)],
        None,
        None,
        &labels(&["alpha"]),
    )
    .unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(entry_source(&runs[0]).as_deref(), Some("sess-b"));
    assert_eq!(entry_source(&runs[1]).as_deref(), Some("alpha"));
}

#[test]
fn test_one_record_reads_one_source_in_its_near_miss_and_in_its_unconfigured_run_and_the_evidence_verdicts_carry_no_source(
) {
    let hooks = vec![bash_hook()];
    let standing = labels(&["alpha"]);
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[
            tool_call(Some("sess-a"), 0, 1, "2026-01-01T00:00:00Z", "c1"),
            tool_call(Some("sess-b"), 1, 2, "2026-01-01T00:01:00Z", "c2"),
        ],
        None,
        None,
        &standing,
    )
    .unwrap();
    assert_eq!(misses.len(), 2);
    let runs = unconfigured_runs(
        &[],
        &[
            hook_run(Some("sess-a"), 0, 1),
            hook_run(Some("sess-b"), 1, 2),
        ],
        None,
        None,
        &standing,
    )
    .unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("alpha"));
    assert_eq!(entry_source(&runs[0]).as_deref(), Some("alpha"));
    assert_eq!(entry_source(&misses[0]), entry_source(&runs[0]));
    assert_eq!(entry_source(&misses[1]).as_deref(), Some("sess-b"));
    assert_eq!(entry_source(&runs[1]).as_deref(), Some("sess-b"));
    assert_eq!(entry_source(&misses[1]), entry_source(&runs[1]));
    let document = build_evidence(
        &hooks,
        &[tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T00:00:00Z",
            "c1",
        )],
        None,
        None,
    )
    .unwrap();
    let verdicts = document
        .get("verdicts")
        .and_then(JsonValue::as_list)
        .expect("the verdicts stand as a list");
    assert_eq!(verdicts.len(), 1);
    assert!(
        verdicts[0].get("source").is_none(),
        "a verdict carries no source"
    );
}

#[test]
fn test_the_near_misses_stand_in_the_order_of_hook_and_instant_and_source_and_line_with_an_unreadable_timestamp_at_the_earliest_instant_and_no_source_at_the_empty_text(
) {
    let hooks = vec![bash_hook()];
    let stamp = "2026-01-01T00:00:00Z";
    // The source decides on a full tie of hook and instant and
    // line, against the order the records stand in.
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[
            tool_call(Some("sess-z"), 0, 7, stamp, "c1"),
            tool_call(Some("sess-a"), 1, 7, stamp, "c2"),
        ],
        None,
        None,
        &labels(&["zeta", "alpha"]),
    )
    .unwrap();
    assert_eq!(misses.len(), 2);
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("alpha"));
    assert_eq!(entry_source(&misses[1]).as_deref(), Some("zeta"));
    // The source outranks the line.
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[
            tool_call(Some("sess-z"), 0, 3, stamp, "c1"),
            tool_call(Some("sess-a"), 1, 9, stamp, "c2"),
        ],
        None,
        None,
        &labels(&["zeta", "alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("alpha"));
    assert_eq!(entry_line(&misses[0]), 9);
    assert_eq!(entry_source(&misses[1]).as_deref(), Some("zeta"));
    // No source reads as the empty text in the key and stands
    // first on a full tie.
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[
            tool_call(Some("sess-b"), 1, 5, stamp, "c1"),
            tool_call(None, 0, 5, stamp, "c2"),
        ],
        None,
        None,
        &labels(&[]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]), None);
    assert_eq!(entry_source(&misses[1]).as_deref(), Some("sess-b"));
    // Where the sources agree the line decides, as the control.
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[
            tool_call(Some("sess-a"), 0, 9, stamp, "c1"),
            tool_call(Some("sess-b"), 1, 3, stamp, "c2"),
        ],
        None,
        None,
        &labels(&["same", "same"]),
    )
    .unwrap();
    assert_eq!(entry_line(&misses[0]), 3);
    assert_eq!(entry_line(&misses[1]), 9);
    // A miss carrying no timestamp reads as the earliest
    // instant and stands ahead of a miss carrying a real one,
    // against both its line and its source.
    let mut bare = tool_call(Some("sess-z"), 0, 9, stamp, "c1");
    bare.timestamp = None;
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[tool_call(Some("sess-a"), 1, 1, stamp, "c2"), bare],
        None,
        None,
        &labels(&["zeta", "alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("zeta"));
    assert_eq!(entry_source(&misses[1]).as_deref(), Some("alpha"));
    // Two misses whose timestamp texts do not read as instants
    // tie at the earliest instant and the source decides.
    let (_entries, misses) = examine_hooks(
        &hooks,
        &[
            tool_call(Some("sess-z"), 0, 5, "not-a-time", "c1"),
            tool_call(Some("sess-a"), 1, 5, "also-not-a-time", "c2"),
        ],
        None,
        None,
        &labels(&["zeta", "alpha"]),
    )
    .unwrap();
    assert_eq!(entry_source(&misses[0]).as_deref(), Some("alpha"));
    assert_eq!(entry_source(&misses[1]).as_deref(), Some("zeta"));
}
