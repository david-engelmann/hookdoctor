//! The pins of slice 13, the markdown identity standings
//! beyond the name form, over documents the suite hands
//! to the rendering directly. A built entry carries its
//! name as text or as null alone, so the entries below
//! stand crafted in the shape the examination builds,
//! with the name respelled in a kind other than text.
//! An empty command beside such a name prints the gap,
//! and a command standing in another kind falls through
//! to the name, and a name standing in another kind
//! beside a null or absent command spells as the
//! rendering spells any value.

use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::report::render_markdown;

fn run_entry(command: Option<JsonValue>, name: JsonValue) -> JsonValue {
    let mut fields = vec![
        ("event", JsonValue::Null),
        ("kind", JsonValue::text("hook_run")),
    ];
    if let Some(command) = command {
        fields.push(("command", command));
    }
    fields.push(("name", name));
    fields.push(("timestamp", JsonValue::Null));
    fields.push(("source", JsonValue::text("session.jsonl")));
    fields.push(("line", JsonValue::Int(3)));
    JsonValue::object(fields)
}

fn markdown(entry: JsonValue) -> String {
    let document = JsonValue::object(vec![("unconfigured_runs", JsonValue::List(vec![entry]))]);
    render_markdown(&document)
}

#[test]
fn test_an_empty_command_entry_with_a_number_name_spells_the_name_in_the_markdown() {
    let markdown = markdown(run_entry(Some(JsonValue::text("")), JsonValue::Int(5)));
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- 5 ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
    assert!(
        !markdown.contains("-  ran at"),
        "the gap is no identity, so it stands nowhere in the identity place: {markdown}"
    );
}

#[test]
fn test_an_empty_command_entry_with_a_boolean_name_spells_the_name_in_the_markdown() {
    let markdown = markdown(run_entry(Some(JsonValue::text("")), JsonValue::Bool(false)));
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- False ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
    assert!(
        !markdown.contains("-  ran at"),
        "the gap is no identity, so it stands nowhere in the identity place: {markdown}"
    );
}

#[test]
fn test_an_empty_command_entry_with_a_list_name_spells_the_name_in_the_markdown() {
    let markdown = markdown(run_entry(
        Some(JsonValue::text("")),
        JsonValue::List(Vec::new()),
    ));
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- []\n ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
    assert!(
        !markdown.contains("-  ran at"),
        "the gap is no identity, so it stands nowhere in the identity place: {markdown}"
    );
}

#[test]
fn test_a_number_command_entry_prints_its_text_name_in_the_markdown() {
    let markdown = markdown(run_entry(
        Some(JsonValue::Int(5)),
        JsonValue::text("rogue-hook.sh"),
    ));
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- rogue-hook.sh ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
    assert!(
        !markdown.contains("- 5 ran at"),
        "the command standing in another kind falls through, so its digits stand nowhere in the identity place: {markdown}"
    );
}

#[test]
fn test_a_number_name_beside_a_null_command_spells_its_digits_in_the_markdown() {
    let markdown = markdown(run_entry(Some(JsonValue::Null), JsonValue::Int(5)));
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- 5 ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
}

#[test]
fn test_a_number_name_beside_an_absent_command_spells_its_digits_in_the_markdown() {
    let markdown = markdown(run_entry(None, JsonValue::Int(5)));
    assert!(markdown.contains("## Unconfigured runs"), "{markdown}");
    assert!(
        markdown.contains("- 5 ran at None in session.jsonl on line 3."),
        "{markdown}"
    );
}
