//! The pins of slice 46, the container layout
//! standings, over values the pins read from
//! literal tokens through the public reader and
//! hand to the public rendering and over values
//! the pins build at the builder seats of the
//! value type. PORT-7 states the JSON rendering
//! as byte for byte the Python rendering, naming
//! the layout only at its two space width, and
//! HD2-11 states the layout summary at the
//! command grain, and GR-10 states the containers
//! from the reader its grammar side alone, and
//! the pins below state the printed layout of
//! the two container arms at its own grain, the
//! frame and the indentation and the separators
//! and the order and the empty forms and the one
//! trailing newline of the rendering whole.
//! Every pin stands as a fence on the landed
//! code, this slice moving no production code.

use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::render::render_json;

fn value_of(token: &str) -> JsonValue {
    match parse_json(token) {
        Ok(value) => value,
        Err(_) => panic!("the token {token} reads as a value"),
    }
}

fn document_of(token: &str) -> String {
    render_json(&value_of(token))
}

fn leading_spaces(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

#[test]
fn test_populated_containers_open_and_close_at_the_level_that_opened_them() {
    assert_eq!(
        document_of("{\"a\":[{\"b\":[{\"c\":[1]}]}]}"),
        "{\n  \"a\": [\n    {\n      \"b\": [\n        {\n          \"c\": [\n            1\n          ]\n        }\n      ]\n    }\n  ]\n}\n"
    );
    assert_eq!(
        document_of("[1,[2,[3]]]"),
        "[\n  1,\n  [\n    2,\n    [\n      3\n    ]\n  ]\n]\n"
    );
    assert_eq!(
        document_of("{\"a\":{\"b\":{\"c\":1}}}"),
        "{\n  \"a\": {\n    \"b\": {\n      \"c\": 1\n    }\n  }\n}\n"
    );
}

#[test]
fn test_items_stand_indented_by_two_spaces_for_each_level_of_depth() {
    assert_eq!(
        document_of("{\"a\":[{\"b\":[{\"c\":[1]}]}]}"),
        "{\n  \"a\": [\n    {\n      \"b\": [\n        {\n          \"c\": [\n            1\n          ]\n        }\n      ]\n    }\n  ]\n}\n"
    );
    assert_eq!(
        document_of("[1,[2,[3]]]"),
        "[\n  1,\n  [\n    2,\n    [\n      3\n    ]\n  ]\n]\n"
    );
    let mut deep_text = String::new();
    for _ in 0..1000 {
        deep_text.push('[');
    }
    deep_text.push_str("null");
    for _ in 0..1000 {
        deep_text.push(']');
    }
    let rendered = render_json(&value_of(&deep_text));
    assert_eq!(rendered.len(), 2_004_005);
    assert_eq!(rendered.matches('\n').count(), 2_001);
    let lines: Vec<&str> = rendered.split('\n').collect();
    assert_eq!(leading_spaces(lines[1]), 2);
    assert_eq!(leading_spaces(lines[999]), 1_998);
    assert_eq!(lines[1000].trim(), "null");
    assert_eq!(leading_spaces(lines[1000]), 2_000);
    assert_eq!(leading_spaces(lines[1001]), 1_998);
    assert_eq!(leading_spaces(lines[1999]), 2);
    assert_eq!(lines[2000], "]");
}

#[test]
fn test_a_comma_stands_after_every_list_item_but_the_last_and_a_newline_after_every_item() {
    assert_eq!(
        document_of("[1,2,3,4,5]"),
        "[\n  1,\n  2,\n  3,\n  4,\n  5\n]\n"
    );
    assert_eq!(document_of("[7]"), "[\n  7\n]\n");
    assert_eq!(
        document_of("[1,\"t\",1.5,true,null,[9],{\"k\":9}]"),
        "[\n  1,\n  \"t\",\n  1.5,\n  true,\n  null,\n  [\n    9\n  ],\n  {\n    \"k\": 9\n  }\n]\n"
    );
}

#[test]
fn test_a_colon_and_one_space_stand_after_every_key_and_a_comma_after_every_pair_but_the_last() {
    assert_eq!(
        document_of("{\"\":1,\"x\":2,\"a key\":3,\"a\\\"b\":4,\"aaaaaaaaaa\":5}"),
        "{\n  \"\": 1,\n  \"a key\": 3,\n  \"a\\\"b\": 4,\n  \"aaaaaaaaaa\": 5,\n  \"x\": 2\n}\n"
    );
    assert_eq!(
        document_of("{\"i\":1,\"t\":\"x\",\"f\":1.5,\"b\":false,\"n\":null,\"l\":[9],\"o\":{\"k\":9}}"),
        "{\n  \"b\": false,\n  \"f\": 1.5,\n  \"i\": 1,\n  \"l\": [\n    9\n  ],\n  \"n\": null,\n  \"o\": {\n    \"k\": 9\n  },\n  \"t\": \"x\"\n}\n"
    );
    assert_eq!(document_of("{\"a\":7}"), "{\n  \"a\": 7\n}\n");
}

#[test]
fn test_object_pairs_print_in_sorted_order_and_a_repeated_key_prints_both_its_pairs_in_their_stored_order(
) {
    assert_eq!(
        document_of("{\"bb\":1,\"a\":2,\"ccc\":3}"),
        "{\n  \"a\": 2,\n  \"bb\": 1,\n  \"ccc\": 3\n}\n"
    );
    let repeated = JsonValue::object(vec![
        ("b", JsonValue::Int(1)),
        ("a", JsonValue::Int(2)),
        ("b", JsonValue::Int(3)),
    ]);
    assert_eq!(
        render_json(&repeated),
        "{\n  \"a\": 2,\n  \"b\": 1,\n  \"b\": 3\n}\n"
    );
    let wide = JsonValue::object(vec![
        ("m", JsonValue::Int(0)),
        ("b", JsonValue::Int(1)),
        ("m", JsonValue::Int(2)),
        ("a", JsonValue::Int(3)),
        ("m", JsonValue::Int(4)),
        ("z", JsonValue::Int(5)),
        ("m", JsonValue::Int(6)),
        ("b", JsonValue::Int(7)),
        ("m", JsonValue::Int(8)),
        ("a", JsonValue::Int(9)),
        ("m", JsonValue::Int(10)),
        ("q", JsonValue::Int(11)),
        ("m", JsonValue::Int(12)),
        ("b", JsonValue::Int(13)),
        ("m", JsonValue::Int(14)),
        ("a", JsonValue::Int(15)),
        ("m", JsonValue::Int(16)),
        ("z", JsonValue::Int(17)),
        ("m", JsonValue::Int(18)),
        ("b", JsonValue::Int(19)),
        ("m", JsonValue::Int(20)),
        ("a", JsonValue::Int(21)),
        ("m", JsonValue::Int(22)),
        ("q", JsonValue::Int(23)),
    ]);
    assert_eq!(
        render_json(&wide),
        "{\n  \"a\": 3,\n  \"a\": 9,\n  \"a\": 15,\n  \"a\": 21,\n  \"b\": 1,\n  \"b\": 7,\n  \"b\": 13,\n  \"b\": 19,\n  \"m\": 0,\n  \"m\": 2,\n  \"m\": 4,\n  \"m\": 6,\n  \"m\": 8,\n  \"m\": 10,\n  \"m\": 12,\n  \"m\": 14,\n  \"m\": 16,\n  \"m\": 18,\n  \"m\": 20,\n  \"m\": 22,\n  \"q\": 11,\n  \"q\": 23,\n  \"z\": 5,\n  \"z\": 17\n}\n"
    );
}

#[test]
fn test_empty_containers_print_as_their_bare_pairs_at_every_seat_and_the_document_ends_with_exactly_one_newline(
) {
    assert_eq!(document_of("[]"), "[]\n");
    assert_eq!(document_of("{}"), "{}\n");
    assert_eq!(
        document_of("[[],{},[[]]]"),
        "[\n  [],\n  {},\n  [\n    []\n  ]\n]\n"
    );
    assert_eq!(
        document_of("{\"l\":[],\"o\":{},\"both\":[[],{}]}"),
        "{\n  \"both\": [\n    [],\n    {}\n  ],\n  \"l\": [],\n  \"o\": {}\n}\n"
    );
    assert_eq!(document_of("[{}]"), "[\n  {}\n]\n");
    let rendered = document_of(
        "{\"t\": \"x\", \"i\": -3, \"f\": 1.5, \"b\": true, \"n\": null, \"l\": [false], \"o\": {\"k\": null}}",
    );
    assert_eq!(
        rendered,
        "{\n  \"b\": true,\n  \"f\": 1.5,\n  \"i\": -3,\n  \"l\": [\n    false\n  ],\n  \"n\": null,\n  \"o\": {\n    \"k\": null\n  },\n  \"t\": \"x\"\n}\n"
    );
    assert_eq!(rendered.matches('\n').count(), 13);
    assert!(rendered.ends_with("}\n") && !rendered.ends_with("}\n\n"));
}
