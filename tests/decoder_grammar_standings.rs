//! The pins of slice 40, the decoder
//! grammar standings, over documents the
//! suite writes as literals and hands to
//! the two doors of the decoder directly.
//! PORT-2 states the reader by reference
//! to the grammar the Python decoder
//! reads, and the pins below state that
//! grammar at its edges, the literal
//! words and the whitespace set and the
//! number token and its float range and
//! the string shape and its escapes and
//! the surrogate standings and the
//! container discipline and the nesting
//! bound. Every pin stands as a fence on
//! the landed code, this slice moving no
//! production code.

use hookdoctor::jsonvalue::{parse_json, parse_json_lenient, JsonValue, ParseFailure};

fn ok(text: &str) -> JsonValue {
    parse_json(text).unwrap_or_else(|failure| panic!("refused {text:?} with {failure:?}"))
}

fn bad(text: &str) {
    assert_eq!(
        parse_json(text),
        Err(ParseFailure::Invalid),
        "accepted {text:?}"
    );
}

fn float_of(text: &str) -> f64 {
    match ok(text) {
        JsonValue::Float(value) => value,
        other => panic!("{text:?} gave {other:?}, not a float"),
    }
}

fn text_of(text: &str) -> String {
    match ok(text) {
        JsonValue::Text(value) => value,
        other => panic!("{text:?} gave {other:?}, not a text"),
    }
}

fn lenient_text_of(text: &str) -> String {
    match parse_json_lenient(text) {
        Ok(JsonValue::Text(value)) => value,
        other => panic!("lenient {text:?} gave {other:?}, not a text"),
    }
}

#[test]
fn test_the_literal_words_stand_as_their_values_and_the_signed_non_finite_words_stand_as_their_floats_and_a_partway_or_suffixed_word_refuses(
) {
    assert_eq!(ok("true"), JsonValue::Bool(true));
    assert_eq!(ok("false"), JsonValue::Bool(false));
    assert_eq!(ok("null"), JsonValue::Null);
    assert!(float_of("NaN").is_nan());
    assert_eq!(float_of("Infinity"), f64::INFINITY);
    assert_eq!(float_of("-Infinity"), f64::NEG_INFINITY);
    assert!(float_of("-NaN").is_nan());
    for refused in ["tru", "nul", "fals", "truex", "nan", "inf", "None", "nulll"] {
        bad(refused);
    }
}

#[test]
fn test_the_decoder_skips_exactly_space_and_tab_and_line_feed_and_carriage_return_and_no_other_character_standing_as_whitespace(
) {
    for padded in [" 42 ", "\t42", "\n42", "\r42", "\r\n42", "42\t"] {
        assert_eq!(ok(padded), JsonValue::Int(42), "padded {padded:?}");
    }
    assert_eq!(
        ok("[1,\n2]"),
        JsonValue::List(vec![JsonValue::Int(1), JsonValue::Int(2)])
    );
    assert_eq!(
        ok("{ \"a\" : 1 }"),
        JsonValue::Object(vec![("a".to_string(), JsonValue::Int(1))])
    );
    bad("\u{b}42");
    bad("\u{c}42");
    bad("\u{a0}42");
    bad("42\u{b}");
    bad("[1,\u{c}2]");
}

#[test]
fn test_the_number_token_shape_refuses_a_leading_zero_and_a_point_with_no_digit_after_it_and_an_exponent_with_no_digit_after_it_and_a_lone_minus_and_a_leading_point_and_a_leading_plus(
) {
    assert_eq!(ok("0"), JsonValue::Int(0));
    assert_eq!(ok("-0"), JsonValue::Int(0));
    assert_eq!(ok("10"), JsonValue::Int(10));
    assert_eq!(ok("-17"), JsonValue::Int(-17));
    assert_eq!(float_of("0.5"), 0.5);
    assert_eq!(float_of("1e1"), 10.0);
    assert_eq!(float_of("1E1"), 10.0);
    assert_eq!(float_of("1e+1"), 10.0);
    assert_eq!(float_of("1e-1"), 0.1);
    let negative_zero = float_of("-0.0");
    assert!(negative_zero == 0.0 && negative_zero.is_sign_negative());
    let positive_zero = float_of("0e0");
    assert!(positive_zero == 0.0 && positive_zero.is_sign_positive());
    for refused in ["01", "00", "1.", "1e", "1e+", "-", "-.5", ".5", "+1", "0x1"] {
        bad(refused);
    }
}

#[test]
fn test_a_float_token_past_the_float_range_stands_as_the_infinity_of_its_sign_and_a_float_token_below_the_smallest_subnormal_stands_as_the_zero_of_its_sign(
) {
    assert_eq!(float_of("1e999"), f64::INFINITY);
    assert_eq!(float_of("-1e999"), f64::NEG_INFINITY);
    let underflow = float_of("1e-999");
    assert!(underflow == 0.0 && underflow.is_sign_positive());
    let negative_underflow = float_of("-1e-999");
    assert!(negative_underflow == 0.0 && negative_underflow.is_sign_negative());
    assert_eq!(float_of("5e-324"), f64::from_bits(1));
    assert_eq!(float_of("1.7976931348623157e308"), f64::MAX);
}

#[test]
fn test_the_string_escapes_stand_as_their_characters_and_the_unicode_escape_takes_exactly_four_hex_digits_of_either_case_and_an_unknown_escape_and_a_short_unicode_escape_refuse(
) {
    assert_eq!(
        text_of("\"\\\"\\\\\\/\\b\\f\\n\\r\\t\""),
        "\"\\/\u{8}\u{c}\n\r\t"
    );
    assert_eq!(text_of("\"\\u0041\""), "A");
    assert_eq!(text_of("\"\\u00E9\""), "\u{e9}");
    assert_eq!(text_of("\"\\u00e9\""), "\u{e9}");
    bad("\"\\q\"");
    bad("\"\\a\"");
    bad("\"\\u041\"");
    bad("\"\\u04G1\"");
    bad("\"abc");
    bad("'a'");
}

#[test]
fn test_a_raw_control_character_inside_a_string_refuses_and_a_raw_character_of_any_other_kind_stands_as_itself(
) {
    bad("\"a\tb\"");
    bad("\"a\nb\"");
    assert_eq!(text_of("\"a\u{7f}b\""), "a\u{7f}b");
    assert_eq!(text_of("\"\u{1F600}\""), "\u{1F600}");
    assert_eq!(text_of("\"\u{e9}\""), "\u{e9}");
    assert_eq!(text_of("\"\\uD83D\\uDE00\""), "\u{1F600}");
}

#[test]
fn test_a_surrogate_unit_standing_unpaired_refuses_at_the_refusing_route_and_reads_as_the_replacement_character_at_the_sampling_route_with_a_following_escape_reading_on_its_own_terms(
) {
    for unpaired in [
        "\"\\uD800\"",
        "\"\\uDC00\"",
        "\"\\uD800\\u0041\"",
        "\"\\uD800x\"",
        "\"\\uD800\\uD800\"",
    ] {
        bad(unpaired);
    }
    assert_eq!(lenient_text_of("\"\\uD800\""), "\u{FFFD}");
    assert_eq!(lenient_text_of("\"\\uDC00\""), "\u{FFFD}");
    assert_eq!(lenient_text_of("\"\\uD800\\u0041\""), "\u{FFFD}A");
    assert_eq!(lenient_text_of("\"\\uD800x\""), "\u{FFFD}x");
    assert_eq!(lenient_text_of("\"\\uD800\\uD800\""), "\u{FFFD}\u{FFFD}");
    assert_eq!(text_of("\"\\uD800\\uDC00\""), "\u{10000}");
    assert_eq!(lenient_text_of("\"\\uD800\\uDC00\""), "\u{10000}");
}

#[test]
fn test_the_containers_refuse_a_trailing_comma_and_a_missing_comma_and_a_missing_colon_and_an_unclosed_shape_and_a_document_carrying_content_past_its_value_refuses(
) {
    assert_eq!(ok("[]"), JsonValue::List(vec![]));
    assert_eq!(ok("{}"), JsonValue::Object(vec![]));
    assert_eq!(
        ok("{\"\": \"\"}"),
        JsonValue::Object(vec![(String::new(), JsonValue::Text(String::new()))])
    );
    for refused in [
        "[1,2,]",
        "{\"a\":1,}",
        "[1 2]",
        "{\"a\" 1}",
        "[1,2",
        "{\"a\":1",
        "",
        "  ",
        "42 43",
        "{} []",
        "true false",
        "[1] x",
    ] {
        bad(refused);
    }
}

#[test]
fn test_a_container_opening_stands_at_four_thousand_levels_of_nesting_and_refuses_at_four_thousand_and_one(
) {
    // The descent to the bound overflows a plain test thread stack in
    // this harness, so the deep parses run on a spawned thread with an
    // explicit large stack, as the owner ruled for this pin.
    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(|| {
            let at_bound = "[".repeat(4000) + &"]".repeat(4000);
            assert!(parse_json(&at_bound).is_ok());
            assert!(parse_json_lenient(&at_bound).is_ok());
            let past_bound = "[".repeat(4001) + &"]".repeat(4001);
            assert_eq!(parse_json(&past_bound), Err(ParseFailure::Invalid));
            assert_eq!(parse_json_lenient(&past_bound), Err(ParseFailure::Invalid));
        })
        .expect("thread spawns");
    handle.join().expect("nesting pin thread holds");
}
