//! The pins of slice 23, the identity selection stated
//! whole, over documents the suite hands to the
//! rendering directly and over one session text the
//! suite hands to the ingester. The identity of an
//! unconfigured run stands as the command where the
//! command stands as nonempty text and as the name
//! form in every other standing of the command, the
//! name form spelling the name as the rendering spells
//! every field, and an empty stop command taken from
//! a session line prints the word None in the identity
//! place through the shipped door. The pins at the
//! empty text row beside a name standing as null or
//! absent or in another kind fail on the landed guard,
//! which prints the gap at that row, and every other
//! pin stands as a fence on both guards.

use hookdoctor::examine::build_report;
use hookdoctor::ingest::parse_session_text;
use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::report::render_markdown;

fn run_entry(command: Option<JsonValue>, name: Option<JsonValue>) -> JsonValue {
    let mut fields = vec![
        ("event", JsonValue::Null),
        ("kind", JsonValue::text("hook_run")),
    ];
    if let Some(command) = command {
        fields.push(("command", command));
    }
    if let Some(name) = name {
        fields.push(("name", name));
    }
    fields.push(("timestamp", JsonValue::Null));
    fields.push(("source", JsonValue::text("session.jsonl")));
    fields.push(("line", JsonValue::Int(3)));
    JsonValue::object(fields)
}

fn page(entry: JsonValue) -> String {
    let document = JsonValue::object(vec![("unconfigured_runs", JsonValue::List(vec![entry]))]);
    render_markdown(&document)
}

fn identity_line(command: Option<JsonValue>, name: Option<JsonValue>) -> String {
    let rendered = page(run_entry(command, name));
    let lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains(" ran at "))
        .collect();
    assert_eq!(lines.len(), 1, "{rendered}");
    lines[0].to_string()
}

#[test]
fn test_a_nonempty_command_stands_as_the_identity_beside_every_name_standing() {
    let expected = "- run.sh ran at None in session.jsonl on line 3.";
    assert_eq!(
        identity_line(
            Some(JsonValue::text("run.sh")),
            Some(JsonValue::text("named"))
        ),
        expected
    );
    assert_eq!(
        identity_line(Some(JsonValue::text("run.sh")), Some(JsonValue::Null)),
        expected
    );
    assert_eq!(
        identity_line(Some(JsonValue::text("run.sh")), None),
        expected
    );
    assert_eq!(
        identity_line(Some(JsonValue::text("run.sh")), Some(JsonValue::Int(5))),
        expected
    );
}

#[test]
fn test_an_empty_command_beside_a_text_name_stands_as_the_name_form() {
    assert_eq!(
        identity_line(Some(JsonValue::text("")), Some(JsonValue::text("named"))),
        "- named ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(Some(JsonValue::text("")), Some(JsonValue::text(""))),
        "-  ran at None in session.jsonl on line 3."
    );
}

#[test]
fn test_an_empty_command_beside_a_null_name_stands_as_none_in_the_identity_place() {
    assert_eq!(
        identity_line(Some(JsonValue::text("")), Some(JsonValue::Null)),
        "- None ran at None in session.jsonl on line 3."
    );
}

#[test]
fn test_an_empty_command_beside_an_absent_name_stands_as_none_in_the_identity_place() {
    assert_eq!(
        identity_line(Some(JsonValue::text("")), None),
        "- None ran at None in session.jsonl on line 3."
    );
}

#[test]
fn test_an_empty_command_beside_a_name_in_another_kind_spells_that_name_in_the_identity_place() {
    assert_eq!(
        identity_line(Some(JsonValue::text("")), Some(JsonValue::Int(5))),
        "- 5 ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(Some(JsonValue::text("")), Some(JsonValue::Bool(true))),
        "- True ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(Some(JsonValue::text("")), Some(JsonValue::Float(2.5))),
        "- 2.5 ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(
            Some(JsonValue::text("")),
            Some(JsonValue::BigInt(
                "123456789012345678901234567890".to_string()
            ))
        ),
        "- 123456789012345678901234567890 ran at None in session.jsonl on line 3."
    );
    let list_page = page(run_entry(
        Some(JsonValue::text("")),
        Some(JsonValue::List(vec![JsonValue::Int(1)])),
    ));
    assert!(
        list_page.contains("- [\n  1\n]\n ran at None in session.jsonl on line 3."),
        "{list_page}"
    );
    let object_page = page(run_entry(
        Some(JsonValue::text("")),
        Some(JsonValue::object(vec![("a", JsonValue::Int(1))])),
    ));
    assert!(
        object_page.contains("- {\n  \"a\": 1\n}\n ran at None in session.jsonl on line 3."),
        "{object_page}"
    );
}

#[test]
fn test_a_null_command_falls_through_to_the_name_form_at_every_name_standing() {
    assert_eq!(
        identity_line(Some(JsonValue::Null), Some(JsonValue::Null)),
        "- None ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(Some(JsonValue::Null), Some(JsonValue::text("named"))),
        "- named ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(Some(JsonValue::Null), Some(JsonValue::Int(5))),
        "- 5 ran at None in session.jsonl on line 3."
    );
}

#[test]
fn test_an_absent_command_falls_through_to_the_name_form_at_every_name_standing() {
    assert_eq!(
        identity_line(None, Some(JsonValue::Null)),
        "- None ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(None, Some(JsonValue::text("named"))),
        "- named ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(None, Some(JsonValue::Int(5))),
        "- 5 ran at None in session.jsonl on line 3."
    );
}

#[test]
fn test_a_command_in_another_kind_falls_through_to_the_name_form() {
    assert_eq!(
        identity_line(Some(JsonValue::Int(7)), Some(JsonValue::text("named"))),
        "- named ran at None in session.jsonl on line 3."
    );
    assert_eq!(
        identity_line(Some(JsonValue::Int(7)), Some(JsonValue::Null)),
        "- None ran at None in session.jsonl on line 3."
    );
}

#[test]
fn test_an_empty_stop_command_prints_none_in_the_identity_place_through_the_door() {
    let text = "{\"type\":\"system\",\"subtype\":\"stop_hook_summary\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"sessionId\":\"s1\",\"hookInfos\":[{\"command\":\"\"},{\"command\":\"real.sh\"}]}";
    let records = parse_session_text(text, 0).expect("the session line parses");
    let report = build_report(&[], &records, "cfg", &[], None, None).expect("the report builds");
    let rendered = render_markdown(&report);
    let lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains(" ran at "))
        .collect();
    assert_eq!(
        lines,
        vec![
            "- None ran at Stop in s1 on line 1.",
            "- real.sh ran at Stop in s1 on line 1.",
        ]
    );
}
