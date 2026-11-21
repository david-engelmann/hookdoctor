//! The pins of slice 22, the evidence dates spelling,
//! over records the suite builds in code and hands to
//! the examination directly. HD2-7 states the dates of
//! a hook entry as the earliest and latest timestamp
//! among the subjects it ran on, read as instants, and
//! the pins below walk the choice the slice states
//! where two or more spellings of one extreme instant
//! stand among the ran subjects, at the call seat and
//! at the outcome seat and in the verdicts of the
//! evidence document. Every pin stands as a fence on
//! the landed code, this slice moving no production
//! code.

use hookdoctor::examine::{build_evidence, examine_hooks};
use hookdoctor::ingest::{HookConfig, TraceRecord};
use hookdoctor::jsonvalue::JsonValue;

fn record(kind: &str, timestamp: Option<&str>, line: i64) -> TraceRecord {
    TraceRecord {
        kind: kind.to_string(),
        timestamp: timestamp.map(str::to_string),
        session: Some("sess".to_string()),
        file_index: 0,
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

fn call(id: &str, timestamp: Option<&str>, line: i64) -> TraceRecord {
    let mut built = record("tool_call", timestamp, line);
    built.tool_name = Some("Bash".to_string());
    built.call_id = Some(id.to_string());
    built
}

fn outcome(id: &str, timestamp: Option<&str>, line: i64) -> TraceRecord {
    let mut built = record("tool_outcome", timestamp, line);
    built.call_id = Some(id.to_string());
    built.text = "ok".to_string();
    built
}

fn hook(event: &str) -> HookConfig {
    HookConfig {
        id: format!("{event}:0:0"),
        event: event.to_string(),
        matcher: Some("Bash".to_string()),
        hook_type: "command".to_string(),
        command: "guard-bash.sh".to_string(),
    }
}

fn date_text(evidence: &JsonValue, key: &str) -> Option<String> {
    match evidence.get(key) {
        Some(JsonValue::Text(text)) => Some(text.clone()),
        Some(JsonValue::Null) => None,
        _ => panic!("the evidence carries its {key}"),
    }
}

fn dates(event: &str, records: &[TraceRecord]) -> (Option<String>, Option<String>, i64) {
    let hooks = vec![hook(event)];
    let (entries, _) = examine_hooks(&hooks, records, None, None, &[]).unwrap();
    let evidence = entries[0].get("evidence").expect("the evidence stands");
    let ran = match evidence.get("ran") {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the evidence carries its ran count"),
    };
    (
        date_text(evidence, "first_ran_at"),
        date_text(evidence, "last_ran_at"),
        ran,
    )
}

fn assert_dates(
    event: &str,
    records: &[TraceRecord],
    want_first: Option<&str>,
    want_last: Option<&str>,
    want_ran: i64,
) {
    let (first, last, ran) = dates(event, records);
    assert_eq!(first.as_deref(), want_first, "the first date stands");
    assert_eq!(last.as_deref(), want_last, "the last date stands");
    assert_eq!(ran, want_ran, "the ran count stands");
}

#[test]
fn test_two_spellings_of_one_instant_stand_as_the_first_and_last_dates_by_the_order_of_the_text_in_both_directions(
) {
    // One instant in the universal spelling and under a plus
    // offset, the universal spelling being the smaller text.
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-01-01T10:00:00+01:00"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("2026-01-01T10:00:00+01:00"),
        2,
    );
    // The same instant under a minus offset, where the offset
    // spelling is the smaller text and the dates follow it.
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-01-01T04:00:00-05:00"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T04:00:00-05:00"),
        Some("2026-01-01T09:00:00Z"),
        2,
    );
}

#[test]
fn test_the_dates_of_a_hook_stand_unmoved_under_the_reversed_order_of_the_subjects_in_the_trace() {
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-01-01T04:00:00-05:00"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T04:00:00-05:00"),
        Some("2026-01-01T09:00:00Z"),
        2,
    );
    let records = vec![
        call("c1", Some("2026-01-01T04:00:00-05:00"), 1),
        call("c2", Some("2026-01-01T09:00:00Z"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T04:00:00-05:00"),
        Some("2026-01-01T09:00:00Z"),
        2,
    );
}

#[test]
fn test_three_spellings_of_one_instant_leave_the_middle_spelling_as_neither_date_and_one_spelling_alone_stands_as_both_dates(
) {
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-01-01T10:00:00+01:00"), 2),
        call("c3", Some("2026-01-01T04:00:00-05:00"), 3),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T04:00:00-05:00"),
        Some("2026-01-01T10:00:00+01:00"),
        3,
    );
    let records = vec![call("c1", Some("2026-01-01T09:00:00Z"), 1)];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("2026-01-01T09:00:00Z"),
        1,
    );
}

#[test]
fn test_two_spellings_at_the_earliest_instant_and_two_at_the_latest_stand_as_the_dates_by_the_text_at_each_extreme(
) {
    let records = vec![
        call("c1", Some("2026-01-01T10:00:00+01:00"), 1),
        call("c2", Some("2026-01-01T09:00:00Z"), 2),
        call("c3", Some("2026-01-01T07:00:00-05:00"), 3),
        call("c4", Some("2026-01-01T12:00:00Z"), 4),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("2026-01-01T12:00:00Z"),
        4,
    );
}

#[test]
fn test_two_fraction_widths_of_one_instant_stand_as_the_first_and_last_dates_by_the_order_of_the_text(
) {
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00.5Z"), 1),
        call("c2", Some("2026-01-01T09:00:00.500Z"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00.500Z"),
        Some("2026-01-01T09:00:00.5Z"),
        2,
    );
}

#[test]
fn test_the_forms_and_the_letter_case_of_one_instant_stand_as_pure_text_at_the_dates() {
    // The extended form and the basic form of one instant.
    let records = vec![
        call("c1", Some("20260101T090000Z"), 1),
        call("c2", Some("2026-01-01T09:00:00Z"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("20260101T090000Z"),
        2,
    );
    // The calendar form and the week date form of one
    // instant, the first of January 2026 standing as the
    // Thursday of week one.
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-W01-4T09:00:00Z"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("2026-W01-4T09:00:00Z"),
        2,
    );
    // The uppercase zed and the lowercase zed of one instant.
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-01-01T09:00:00z"), 2),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("2026-01-01T09:00:00z"),
        2,
    );
}

#[test]
fn test_a_stamp_that_does_not_read_as_an_instant_and_a_subject_carrying_no_stamp_take_no_part_in_the_dates(
) {
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-01-01T10:00:00+01:00"), 2),
        call("c3", Some("not a timestamp"), 3),
        call("c4", None, 4),
    ];
    assert_dates(
        "PreToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("2026-01-01T10:00:00+01:00"),
        4,
    );
}

#[test]
fn test_the_dates_of_a_posttooluse_hook_stand_by_the_same_tiebreak_at_the_outcome_seat() {
    // The calls carry one earlier instant between them, and
    // the outcomes carry the tied instant under two
    // spellings, so the dates can only have come from the
    // outcomes.
    let records = vec![
        call("c1", Some("2026-01-01T08:00:00Z"), 1),
        outcome("c1", Some("2026-01-01T09:00:00Z"), 2),
        call("c2", Some("2026-01-01T08:30:00Z"), 3),
        outcome("c2", Some("2026-01-01T10:00:00+01:00"), 4),
    ];
    assert_dates(
        "PostToolUse",
        &records,
        Some("2026-01-01T09:00:00Z"),
        Some("2026-01-01T10:00:00+01:00"),
        2,
    );
}

#[test]
fn test_the_verdicts_of_the_evidence_document_carry_the_dates_as_the_report_entry_spells_them_at_a_tied_instant(
) {
    let records = vec![
        call("c1", Some("2026-01-01T09:00:00Z"), 1),
        call("c2", Some("2026-01-01T10:00:00+01:00"), 2),
    ];
    let hooks = vec![hook("PreToolUse")];
    let (entries, _) = examine_hooks(&hooks, &records, None, None, &[]).unwrap();
    let evidence = entries[0].get("evidence").expect("the evidence stands");
    let document = build_evidence(&hooks, &records, None, None).unwrap();
    let verdicts = document
        .get("verdicts")
        .and_then(JsonValue::as_list)
        .expect("the verdicts stand as a list");
    assert_eq!(verdicts.len(), 1);
    assert_eq!(
        date_text(&verdicts[0], "first_ran_at"),
        date_text(evidence, "first_ran_at")
    );
    assert_eq!(
        date_text(&verdicts[0], "last_ran_at"),
        date_text(evidence, "last_ran_at")
    );
    assert_eq!(
        date_text(&verdicts[0], "first_ran_at").as_deref(),
        Some("2026-01-01T09:00:00Z")
    );
    assert_eq!(
        date_text(&verdicts[0], "last_ran_at").as_deref(),
        Some("2026-01-01T10:00:00+01:00")
    );
}
