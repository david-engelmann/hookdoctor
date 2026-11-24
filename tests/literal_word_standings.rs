//! The pins of slice 45, the literal word
//! standings, over values the pins read from
//! literal tokens through the public reader and
//! hand to the public rendering and over values
//! the pins build at the builder seats of the
//! value type. PORT-7 states the JSON rendering
//! as byte for byte the Python rendering, naming
//! none of the words the writer prints at its
//! own grain, and GR-3 states the three words
//! from the reader its decoding side alone, and
//! the pins below state the words at the writer
//! its own grain, the three words standing in
//! lowercase at every seat and no other form
//! standing for a boolean or a null in any
//! rendered document, and the markdown report
//! carrying the words at its container seat.
//! Every pin stands as a fence on the landed
//! code, this slice moving no production code.

use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::render::render_json;
use hookdoctor::report::render_markdown;

fn value_of(token: &str) -> JsonValue {
    match parse_json(token) {
        Ok(value) => value,
        Err(_) => panic!("the token {token} reads as a value"),
    }
}

fn document_of(token: &str) -> String {
    render_json(&value_of(token))
}

#[test]
fn test_the_three_words_print_at_the_whole_document_seat() {
    assert_eq!(document_of("true"), "true\n");
    assert_eq!(document_of("false"), "false\n");
    assert_eq!(document_of("null"), "null\n");
}

#[test]
fn test_the_words_print_the_same_inside_lists_and_objects() {
    assert_eq!(
        document_of("[true, false, null]"),
        "[\n  true,\n  false,\n  null\n]\n"
    );
    assert_eq!(
        document_of("{\"a\": true, \"b\": false, \"c\": null}"),
        "{\n  \"a\": true,\n  \"b\": false,\n  \"c\": null\n}\n"
    );
}

#[test]
fn test_the_words_print_at_deep_seats_and_for_built_values() {
    assert_eq!(
        document_of("{\"outer\": [{\"inner\": [true, {\"deep\": null}]}, false]}"),
        "{\n  \"outer\": [\n    {\n      \"inner\": [\n        true,\n        {\n          \"deep\": null\n        }\n      ]\n    },\n    false\n  ]\n}\n"
    );
    let built = JsonValue::object(vec![
        ("matcher", JsonValue::optional_text(&None)),
        ("command", JsonValue::text("run.sh")),
        ("flag", JsonValue::Bool(true)),
        ("other", JsonValue::Bool(false)),
    ]);
    assert_eq!(
        render_json(&built),
        "{\n  \"command\": \"run.sh\",\n  \"flag\": true,\n  \"matcher\": null,\n  \"other\": false\n}\n"
    );
}

#[test]
fn test_no_other_spelling_of_a_boolean_or_a_null_stands_in_a_document() {
    let rendered = document_of(
        "{\"t\": \"x\", \"i\": -3, \"f\": 1.5, \"b\": true, \"n\": null, \"l\": [false], \"o\": {\"k\": null}}",
    );
    assert_eq!(
        rendered,
        "{\n  \"b\": true,\n  \"f\": 1.5,\n  \"i\": -3,\n  \"l\": [\n    false\n  ],\n  \"n\": null,\n  \"o\": {\n    \"k\": null\n  },\n  \"t\": \"x\"\n}\n"
    );
    for forbidden in ["True", "False", "None", "\"true\"", "\"false\"", "\"null\""] {
        assert!(
            !rendered.contains(forbidden),
            "the forbidden form {forbidden} stands in the document"
        );
    }
}

#[test]
fn test_the_markdown_container_seat_carries_the_json_words() {
    let report = JsonValue::object(vec![
        ("format", JsonValue::Bool(true)),
        ("config_source", JsonValue::Null),
        (
            "session_sources",
            JsonValue::List(vec![JsonValue::Bool(false), JsonValue::Null]),
        ),
        (
            "window",
            JsonValue::object(vec![
                (
                    "start",
                    JsonValue::List(vec![JsonValue::Null, JsonValue::Bool(true)]),
                ),
                ("end", JsonValue::Null),
            ]),
        ),
        ("hooks", JsonValue::List(vec![])),
        ("near_misses", JsonValue::List(vec![])),
        ("unconfigured_runs", JsonValue::List(vec![])),
    ]);
    let markdown = render_markdown(&report);
    assert!(markdown.contains("[\n  null,\n  true\n]"));
    assert_eq!(
        markdown,
        "# hookdoctor examination\n\nFormat True\nConfiguration source None\nSession sources False, None\nWindow start [\n  null,\n  true\n]\n end None\n\n## Hooks\n\nNo hooks are configured.\n\n## Near misses\n\nNone.\n"
    );
}
