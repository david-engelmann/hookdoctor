//! The pins of slice 44, the JSON float spelling
//! standings, over values the pins read from
//! literal tokens through the public reader and
//! hand to the public float spelling and the
//! public rendering. PORT-7 states the JSON
//! rendering as byte for byte the Python
//! rendering, naming the number spelling nowhere
//! at its own grain, and the pins below state
//! that spelling at its fine grain, the words at
//! the values carrying no digits and the
//! shortest digits of the value itself and the
//! fixed and scientific layouts parting at the
//! exponent edges and the signed two digit
//! exponent and the round trip at the same bits.
//! Every pin stands as a fence on the landed
//! code, this slice moving no production code.

use hookdoctor::jsonvalue::{parse_json, JsonValue};
use hookdoctor::render::{json_float_text, render_json};

fn float_of(token: &str) -> f64 {
    match parse_json(token) {
        Ok(JsonValue::Float(value)) => value,
        _ => panic!("the token {token} reads as a float"),
    }
}

fn spelling_of(token: &str) -> String {
    json_float_text(float_of(token))
}

fn document_of(token: &str) -> String {
    render_json(&JsonValue::Float(float_of(token)))
}

#[test]
fn test_the_non_finite_floats_spell_as_their_words_and_the_zeros_carry_their_signs() {
    assert_eq!(spelling_of("NaN"), "NaN");
    assert_eq!(spelling_of("-NaN"), "NaN");
    let negative_nan = f64::from_bits(0xFFF8000000000000);
    assert_eq!(json_float_text(negative_nan), "NaN");
    assert_eq!(spelling_of("Infinity"), "Infinity");
    assert_eq!(spelling_of("-Infinity"), "-Infinity");
    assert_eq!(spelling_of("1e999"), "Infinity");
    assert_eq!(spelling_of("-1e999"), "-Infinity");
    assert_eq!(document_of("NaN"), "NaN\n");
    assert_eq!(document_of("-Infinity"), "-Infinity\n");
    assert_eq!(spelling_of("0.0"), "0.0");
    assert_eq!(spelling_of("-0.0"), "-0.0");
    assert_eq!(spelling_of("0e0"), "0.0");
    assert_eq!(spelling_of("-0e0"), "-0.0");
    assert_eq!(spelling_of("1e-999"), "0.0");
    assert_eq!(spelling_of("-1e-999"), "-0.0");
    assert_eq!(document_of("-0.0"), "-0.0\n");
}

#[test]
fn test_integral_and_fraction_floats_spell_in_fixed_form_with_the_values_own_digits() {
    let cells = [
        ("1.0", "1.0"),
        ("2.0", "2.0"),
        ("10.0", "10.0"),
        ("100.0", "100.0"),
        ("1e2", "100.0"),
        ("12300e-2", "123.0"),
        ("0.5", "0.5"),
        ("1.5", "1.5"),
        ("0.1", "0.1"),
        ("0.000123", "0.000123"),
        ("123.456", "123.456"),
        ("-123.456", "-123.456"),
        ("3.141592653589793", "3.141592653589793"),
        ("0.3333333333333333", "0.3333333333333333"),
        ("0.30000000000000004", "0.30000000000000004"),
        ("123456789.123456789", "123456789.12345679"),
        ("1.0000000000000002", "1.0000000000000002"),
        ("9007199254740993.0", "9007199254740992.0"),
    ];
    for (token, expected) in cells {
        assert_eq!(spelling_of(token), expected, "the token {token}");
    }
}

#[test]
fn test_the_fixed_and_scientific_forms_part_at_the_exponent_edges() {
    let cells = [
        ("1e15", "1000000000000000.0"),
        ("1.5e15", "1500000000000000.0"),
        ("1234567890123456.0", "1234567890123456.0"),
        ("9999999999999998.0", "9999999999999998.0"),
        ("1e16", "1e+16"),
        ("1.5e16", "1.5e+16"),
        ("12345678901234567.0", "1.2345678901234568e+16"),
        ("0.0001", "0.0001"),
        ("0.000123", "0.000123"),
        ("0.0009999", "0.0009999"),
        ("1e-5", "1e-05"),
        ("1.23e-5", "1.23e-05"),
        ("-1e16", "-1e+16"),
        ("-1e-5", "-1e-05"),
    ];
    for (token, expected) in cells {
        assert_eq!(spelling_of(token), expected, "the token {token}");
    }
}

#[test]
fn test_scientific_floats_spell_with_one_digit_and_a_signed_two_digit_exponent() {
    let cells = [
        ("1e16", "1e+16"),
        ("1.5e16", "1.5e+16"),
        ("1.23e-5", "1.23e-05"),
        ("1e-5", "1e-05"),
        ("2.5e-10", "2.5e-10"),
        ("1e-7", "1e-07"),
        ("1e100", "1e+100"),
        ("1e308", "1e+308"),
        ("1.7976931348623157e308", "1.7976931348623157e+308"),
        ("5e-324", "5e-324"),
        ("-5e-324", "-5e-324"),
        ("1e-323", "1e-323"),
        ("2.2250738585072014e-308", "2.2250738585072014e-308"),
        ("-1e16", "-1e+16"),
    ];
    for (token, expected) in cells {
        assert_eq!(spelling_of(token), expected, "the token {token}");
    }
}

#[test]
fn test_every_spelled_float_reads_back_as_the_same_float_and_none_spells_as_an_integer() {
    let tokens = [
        "0.0",
        "-0.0",
        "1.0",
        "-1.5",
        "0.1",
        "123.456",
        "0.30000000000000004",
        "123456789.123456789",
        "1.0000000000000002",
        "3.141592653589793",
        "1e15",
        "9999999999999998.0",
        "0.0001",
        "0.0009999",
        "1e16",
        "-1e16",
        "1.5e16",
        "1e-5",
        "-1e-5",
        "2.5e-10",
        "1e100",
        "1.7976931348623157e308",
        "5e-324",
        "-5e-324",
        "2.2250738585072014e-308",
    ];
    for token in tokens {
        let value = float_of(token);
        let spelled = json_float_text(value);
        assert!(
            spelled.contains('.') || spelled.contains('e'),
            "the spelling {spelled} of the token {token} carries a point or an exponent"
        );
        let back = float_of(&spelled);
        assert_eq!(back.to_bits(), value.to_bits(), "the token {token}");
    }
    let nan_back = float_of(&spelling_of("NaN"));
    assert!(nan_back.is_nan());
    let document =
        match parse_json("{\"b\": 0.1, \"a\": 1e16, \"c\": -0.0, \"d\": 1e999, \"e\": -1e-999}") {
            Ok(value) => value,
            Err(_) => panic!("the document reads"),
        };
    assert_eq!(
        render_json(&document),
        "{\n  \"a\": 1e+16,\n  \"b\": 0.1,\n  \"c\": -0.0,\n  \"d\": Infinity,\n  \"e\": -0.0\n}\n"
    );
}
