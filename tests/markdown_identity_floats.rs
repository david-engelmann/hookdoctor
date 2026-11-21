//! The pins of slice 15, the float completion of the
//! markdown identity, over documents the suite hands
//! to the rendering directly. A built entry carries its
//! name as text or as null alone, so the entries below
//! stand crafted in the shape the examination builds,
//! with the name respelled as a float. An empty command
//! beside a float name prints the gap, and a float name
//! beside a null or absent command spells in the
//! f-string forms, its finite values in the layout the
//! spelling draws and its three non-finite values in
//! lowercase words.

use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::report::render_markdown;

fn float_entry(command: Option<JsonValue>, name: JsonValue) -> JsonValue {
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

fn rendered(entry: JsonValue) -> String {
    let document = JsonValue::object(vec![("unconfigured_runs", JsonValue::List(vec![entry]))]);
    render_markdown(&document)
}

#[test]
fn test_a_float_name_beside_an_empty_command_spells_the_name_in_the_markdown() {
    let page = rendered(float_entry(
        Some(JsonValue::text("")),
        JsonValue::Float(1.5),
    ));
    assert!(page.contains("## Unconfigured runs"), "{page}");
    assert!(
        page.contains("- 1.5 ran at None in session.jsonl on line 3."),
        "{page}"
    );
    assert!(
        !page.contains("-  ran at"),
        "the gap is no identity, so it stands nowhere in the identity place {page}"
    );
}

#[test]
fn test_a_float_name_beside_a_null_command_prints_its_fixed_spelling_in_the_markdown() {
    let page = rendered(float_entry(Some(JsonValue::Null), JsonValue::Float(1e15)));
    assert!(page.contains("## Unconfigured runs"), "{page}");
    assert!(
        page.contains("- 1000000000000000.0 ran at None in session.jsonl on line 3."),
        "{page}"
    );
    assert!(
        !page.contains("1e+15"),
        "ten to the fifteenth stands inside the fixed range, so the scientific spelling stands nowhere in this page {page}"
    );
}

#[test]
fn test_a_float_name_beside_an_absent_command_prints_its_scientific_spelling_in_the_markdown() {
    let page = rendered(float_entry(None, JsonValue::Float(1e16)));
    assert!(page.contains("## Unconfigured runs"), "{page}");
    assert!(
        page.contains("- 1e+16 ran at None in session.jsonl on line 3."),
        "{page}"
    );
    assert!(
        !page.contains("10000000000000000.0"),
        "ten to the sixteenth stands past the fixed range, so the fixed spelling stands nowhere in this page {page}"
    );
}

#[test]
fn test_a_nan_name_beside_a_null_command_prints_nan_in_the_markdown() {
    let page = rendered(float_entry(
        Some(JsonValue::Null),
        JsonValue::Float(f64::NAN),
    ));
    assert!(page.contains("## Unconfigured runs"), "{page}");
    assert!(
        page.contains("- nan ran at None in session.jsonl on line 3."),
        "{page}"
    );
    assert!(
        !page.contains("NaN"),
        "the encoder word for the value stands nowhere on the markdown surface {page}"
    );
}

#[test]
fn test_an_infinity_name_beside_an_absent_command_prints_inf_in_the_markdown() {
    let page = rendered(float_entry(None, JsonValue::Float(f64::INFINITY)));
    assert!(page.contains("## Unconfigured runs"), "{page}");
    assert!(
        page.contains("- inf ran at None in session.jsonl on line 3."),
        "{page}"
    );
    assert!(
        !page.contains("Infinity"),
        "the encoder word for the value stands nowhere on the markdown surface {page}"
    );
}

#[test]
fn test_a_minus_infinity_name_beside_a_null_command_prints_minus_inf_in_the_markdown() {
    let page = rendered(float_entry(
        Some(JsonValue::Null),
        JsonValue::Float(f64::NEG_INFINITY),
    ));
    assert!(page.contains("## Unconfigured runs"), "{page}");
    assert!(
        page.contains("- -inf ran at None in session.jsonl on line 3."),
        "{page}"
    );
    assert!(
        !page.contains("Infinity"),
        "the encoder word for the value stands nowhere on the markdown surface {page}"
    );
}

#[test]
fn test_a_float_name_beside_a_number_command_prints_its_spelling_in_the_markdown() {
    let page = rendered(float_entry(Some(JsonValue::Int(7)), JsonValue::Float(1.5)));
    assert!(page.contains("## Unconfigured runs"), "{page}");
    assert!(
        page.contains("- 1.5 ran at None in session.jsonl on line 3."),
        "{page}"
    );
    assert!(
        !page.contains("- 7 ran at"),
        "the command standing in another kind falls through, so its digits stand nowhere in the identity place {page}"
    );
}
