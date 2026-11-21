//! The pins of slice 26, the integer digit
//! limit standings, over texts the suite
//! builds by repetition and hands to the
//! two doors of the decoder directly and
//! to the sampling reader and the labelled
//! document reader at the public surface.
//! The decoder holds integer tokens to the
//! Python conversion limit on the sampling
//! route alone, and the pins below walk
//! the boundary digits at both doors and
//! the count that draws the boundary and
//! the float standing outside it. Every
//! pin stands as a fence on the landed
//! code, this slice moving no production
//! code.

use hookdoctor::jsonvalue::{parse_json, parse_json_lenient, JsonValue, ParseFailure};
use hookdoctor::reader::{parse_output_text, read_json_document};

#[test]
fn test_an_integer_token_at_the_conversion_limit_reads_through_the_sampling_door_as_a_big_integer_of_its_exact_text_and_the_same_token_one_digit_past_the_limit_stands_refused_there_as_invalid(
) {
    let below_limit = "1".repeat(4299);
    assert_eq!(
        parse_json_lenient(&below_limit),
        Ok(JsonValue::BigInt(below_limit.clone()))
    );
    let at_limit = "1".repeat(4300);
    assert_eq!(
        parse_json_lenient(&at_limit),
        Ok(JsonValue::BigInt(at_limit.clone()))
    );
    let past_limit = "1".repeat(4301);
    assert_eq!(parse_json_lenient(&past_limit), Err(ParseFailure::Invalid));
}

#[test]
fn test_a_hook_output_text_carrying_an_integer_token_past_the_limit_at_a_field_or_nested_in_a_list_samples_as_no_output_and_the_same_text_with_the_token_at_the_limit_samples_with_the_big_integer_standing_at_its_field(
) {
    let at_limit = format!("{{\"count\": {}}}", "1".repeat(4300));
    let sampled = parse_output_text(&at_limit).expect("the text samples");
    assert_eq!(
        sampled.get("count"),
        Some(&JsonValue::BigInt("1".repeat(4300)))
    );
    let past_limit = format!("{{\"count\": {}}}", "1".repeat(4301));
    assert_eq!(parse_output_text(&past_limit), None);
    let nested = format!("{{\"counts\": [{}]}}", "1".repeat(4301));
    assert_eq!(parse_output_text(&nested), None);
}

#[test]
fn test_integer_tokens_at_and_past_the_conversion_limit_stand_through_the_refusing_door_as_big_integers_of_their_exact_text_and_a_negative_token_past_the_limit_carries_its_sign_in_its_text(
) {
    let at_limit = "1".repeat(4300);
    assert_eq!(
        parse_json(&at_limit),
        Ok(JsonValue::BigInt(at_limit.clone()))
    );
    let past_limit = "1".repeat(4301);
    assert_eq!(
        parse_json(&past_limit),
        Ok(JsonValue::BigInt(past_limit.clone()))
    );
    let fives = "5".repeat(5000);
    assert_eq!(parse_json(&fives), Ok(JsonValue::BigInt(fives.clone())));
    let negative = format!("-{}", "1".repeat(4301));
    assert_eq!(
        parse_json(&negative),
        Ok(JsonValue::BigInt(negative.clone()))
    );
}

#[test]
fn test_a_document_carrying_an_integer_token_past_the_limit_decodes_whole_through_the_labelled_document_reader_with_the_big_integer_standing_at_its_seat(
) {
    let text = format!("{{\"count\": {}}}", "1".repeat(4301));
    let document = read_json_document(&text, "evidence").expect("the document decodes");
    assert_eq!(
        document.get("count"),
        Some(&JsonValue::BigInt("1".repeat(4301)))
    );
}

#[test]
fn test_a_number_token_whose_integer_part_runs_past_the_limit_and_carries_a_fraction_reads_as_a_float_at_both_doors_and_an_exponent_form_at_the_same_shape_reads_as_a_float_at_both_doors_and_fraction_digits_past_the_limit_change_nothing_at_either_door(
) {
    let fraction = format!("{}.5", "1".repeat(4301));
    let expected = JsonValue::Float(fraction.parse::<f64>().expect("the spelling reads"));
    assert_eq!(parse_json(&fraction), Ok(expected.clone()));
    assert_eq!(parse_json_lenient(&fraction), Ok(expected));
    let exponent = format!("{}e2", "1".repeat(4301));
    assert!(matches!(parse_json(&exponent), Ok(JsonValue::Float(_))));
    assert!(matches!(
        parse_json_lenient(&exponent),
        Ok(JsonValue::Float(_))
    ));
    let fraction_digits = format!("1.{}", "5".repeat(4301));
    assert!(matches!(
        parse_json(&fraction_digits),
        Ok(JsonValue::Float(_))
    ));
    assert!(matches!(
        parse_json_lenient(&fraction_digits),
        Ok(JsonValue::Float(_))
    ));
    let exponent_digits = format!("1e{}", "5".repeat(4301));
    assert!(matches!(
        parse_json(&exponent_digits),
        Ok(JsonValue::Float(_))
    ));
    assert!(matches!(
        parse_json_lenient(&exponent_digits),
        Ok(JsonValue::Float(_))
    ));
}

#[test]
fn test_a_negative_integer_token_at_exactly_the_limit_reads_at_the_sampling_door_with_its_sign_standing_outside_the_count_and_a_negative_token_one_digit_past_the_limit_stands_refused_there(
) {
    let at_limit = format!("-{}", "1".repeat(4300));
    assert_eq!(
        parse_json_lenient(&at_limit),
        Ok(JsonValue::BigInt(at_limit.clone()))
    );
    let past_limit = format!("-{}", "1".repeat(4301));
    assert_eq!(parse_json_lenient(&past_limit), Err(ParseFailure::Invalid));
}
