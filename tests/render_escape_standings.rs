//! The pins of slice 42, the JSON rendering
//! escape standings, over values the suite
//! builds as literals and hands to the public
//! rendering door directly. PORT-7 states the
//! rendering by reference to the Python
//! rendering and names its ASCII escaping by
//! its probe alone, and the pins below state
//! that escaping at its fine grain, the named
//! escapes and the pass through range and the
//! remaining basic plane and the astral form
//! and the two seats under the one table.
//! Every pin stands as a fence on the landed
//! code, this slice moving no production code.

use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::render::render_json;

fn renders(text: &str) -> String {
    render_json(&JsonValue::text(text))
}

#[test]
fn test_the_named_escapes_stand_at_their_exact_bytes() {
    assert_eq!(
        renders("\"\\\n\r\t\u{0008}\u{000C}"),
        "\"\\\"\\\\\\n\\r\\t\\b\\f\"\n"
    );
}

#[test]
fn test_every_character_of_the_pass_through_range_stands_as_itself() {
    let range: String = (0x20u32..=0x7E)
        .filter_map(char::from_u32)
        .filter(|ch| *ch != '"' && *ch != '\\')
        .collect();
    assert_eq!(range.chars().count(), 93);
    assert_eq!(renders(&range), format!("\"{range}\"\n"));
    assert_eq!(renders("a\"b\\c"), "\"a\\\"b\\\\c\"\n");
}

#[test]
fn test_the_remaining_basic_plane_stands_in_its_unicode_form_at_its_class_edges() {
    assert_eq!(
        renders("\u{0000}\u{001F}\u{007F}\u{0080}\u{00AB}\u{ABCD}\u{FFFF}"),
        "\"\\u0000\\u001f\\u007f\\u0080\\u00ab\\uabcd\\uffff\"\n"
    );
}

#[test]
fn test_every_character_past_the_basic_plane_stands_as_a_surrogate_pair() {
    assert_eq!(
        renders("\u{10000}\u{1F600}\u{10FFFF}"),
        "\"\\ud800\\udc00\\ud83d\\ude00\\udbff\\udfff\"\n"
    );
}

#[test]
fn test_the_key_seat_and_the_value_seat_stand_under_the_one_table_and_the_document_stands_pure_ascii(
) {
    let doc = JsonValue::object(vec![(
        "a\"b\u{00E9}\u{10000}",
        JsonValue::text("\"\\\n\u{0001}\u{007F} ~"),
    )]);
    let out = render_json(&doc);
    assert_eq!(
        out,
        "{\n  \"a\\\"b\\u00e9\\ud800\\udc00\": \"\\\"\\\\\\n\\u0001\\u007f ~\"\n}\n"
    );
    assert!(
        out.bytes().all(|byte| byte < 0x80),
        "document stands pure ASCII"
    );
    assert_eq!(renders(""), "\"\"\n");
}
