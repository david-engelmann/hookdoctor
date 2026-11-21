//! The pins of slice 25, the number equality
//! standings, over values the suite builds by
//! hand. The number reading decides wherever
//! two values of the model compare, and the
//! pins below walk its arms, the gate and the
//! float against float arm and the mixed arm
//! with its exact text and the integer text
//! arm, at the shapes where each arm can part
//! from its neighbours. Every pin stands as a
//! fence on the landed code, this slice moving
//! no production code.

use hookdoctor::jsonvalue::JsonValue;

#[test]
fn test_two_floats_stand_equal_where_their_values_agree_and_two_not_a_number_floats_stand_equal_and_a_not_a_number_float_stands_unequal_to_any_other_float_and_an_infinity_stands_equal_only_to_the_infinity_of_its_own_sign_and_the_two_zeros_stand_equal_across_their_signs(
) {
    assert_eq!(JsonValue::Float(1.5), JsonValue::Float(1.5));
    assert_ne!(JsonValue::Float(1.5), JsonValue::Float(2.5));
    assert_eq!(JsonValue::Float(f64::NAN), JsonValue::Float(f64::NAN));
    assert_ne!(JsonValue::Float(f64::NAN), JsonValue::Float(1.0));
    assert_ne!(JsonValue::Float(1.0), JsonValue::Float(f64::NAN));
    assert_eq!(
        JsonValue::Float(f64::INFINITY),
        JsonValue::Float(f64::INFINITY)
    );
    assert_eq!(
        JsonValue::Float(f64::NEG_INFINITY),
        JsonValue::Float(f64::NEG_INFINITY)
    );
    assert_ne!(
        JsonValue::Float(f64::INFINITY),
        JsonValue::Float(f64::NEG_INFINITY)
    );
    assert_eq!(JsonValue::Float(0.0), JsonValue::Float(-0.0));
}

#[test]
fn test_a_whole_float_stands_equal_to_the_integer_and_the_big_integer_of_its_value_in_both_directions_and_a_fractional_float_stands_equal_to_no_integer_not_even_the_integer_its_rounding_names_and_a_negative_zero_float_stands_equal_to_the_integer_zero(
) {
    assert_eq!(JsonValue::Float(2.0), JsonValue::Int(2));
    assert_eq!(JsonValue::Int(2), JsonValue::Float(2.0));
    assert_eq!(JsonValue::Float(2.0), JsonValue::BigInt("2".to_string()));
    assert_eq!(JsonValue::BigInt("2".to_string()), JsonValue::Float(2.0));
    assert_eq!(JsonValue::Float(-3.0), JsonValue::Int(-3));
    assert_ne!(JsonValue::Float(1.5), JsonValue::Int(2));
    assert_ne!(JsonValue::Float(1.5), JsonValue::Int(1));
    assert_ne!(JsonValue::Float(2.5), JsonValue::Int(2));
    assert_ne!(JsonValue::Int(2), JsonValue::Float(1.5));
    assert_eq!(JsonValue::Float(-0.0), JsonValue::Int(0));
    assert_eq!(JsonValue::Int(0), JsonValue::Float(-0.0));
}

#[test]
fn test_the_float_at_two_to_the_fifty_third_stands_equal_to_the_integer_of_its_value_and_unequal_to_the_integer_one_above_it_and_a_whole_float_stands_equal_to_a_big_integer_at_three_magnitudes_where_the_stored_text_is_the_exact_integer_the_float_holds_and_an_infinity_and_a_not_a_number_float_stand_equal_to_no_integer_kind_value(
) {
    assert_eq!(
        JsonValue::Float(9007199254740992.0),
        JsonValue::Int(9007199254740992)
    );
    assert_ne!(
        JsonValue::Float(9007199254740992.0),
        JsonValue::Int(9007199254740993)
    );
    assert_ne!(
        JsonValue::Int(9007199254740993),
        JsonValue::Float(9007199254740992.0)
    );
    assert_eq!(
        JsonValue::Float(1e19),
        JsonValue::BigInt("10000000000000000000".to_string())
    );
    assert_eq!(
        JsonValue::Float(9223372036854775808.0),
        JsonValue::BigInt("9223372036854775808".to_string())
    );
    assert_eq!(
        JsonValue::Float(1e22),
        JsonValue::BigInt("10000000000000000000000".to_string())
    );
    assert_eq!(
        JsonValue::BigInt("10000000000000000000".to_string()),
        JsonValue::Float(1e19)
    );
    assert_ne!(
        JsonValue::Float(f64::INFINITY),
        JsonValue::BigInt("10000000000000000000000".to_string())
    );
    assert_ne!(JsonValue::Float(f64::INFINITY), JsonValue::Int(1));
    assert_ne!(
        JsonValue::Float(f64::NAN),
        JsonValue::BigInt("1".to_string())
    );
    assert_ne!(JsonValue::Float(f64::NAN), JsonValue::Int(0));
}

#[test]
fn test_a_truth_value_stands_equal_to_the_integer_and_the_big_integer_and_the_float_of_its_digit_in_both_directions_and_unequal_to_the_numbers_of_the_other_digit_and_to_a_fractional_float(
) {
    assert_eq!(JsonValue::Bool(true), JsonValue::Int(1));
    assert_eq!(JsonValue::Int(1), JsonValue::Bool(true));
    assert_eq!(JsonValue::Bool(false), JsonValue::Int(0));
    assert_eq!(JsonValue::Int(0), JsonValue::Bool(false));
    assert_eq!(JsonValue::Bool(true), JsonValue::BigInt("1".to_string()));
    assert_eq!(JsonValue::BigInt("0".to_string()), JsonValue::Bool(false));
    assert_eq!(JsonValue::Bool(true), JsonValue::Float(1.0));
    assert_eq!(JsonValue::Float(0.0), JsonValue::Bool(false));
    assert_ne!(JsonValue::Bool(true), JsonValue::Int(0));
    assert_ne!(JsonValue::Bool(true), JsonValue::BigInt("0".to_string()));
    assert_ne!(JsonValue::Bool(true), JsonValue::Float(0.0));
    assert_ne!(JsonValue::Bool(false), JsonValue::Int(1));
    assert_ne!(JsonValue::Bool(true), JsonValue::Float(1.5));
}

#[test]
fn test_an_integer_stands_equal_to_the_big_integer_of_its_spelling_and_a_big_integer_carrying_a_leading_zero_text_or_a_negative_zero_text_stands_unequal_to_the_integer_it_names(
) {
    assert_eq!(JsonValue::Int(7), JsonValue::BigInt("7".to_string()));
    assert_eq!(JsonValue::BigInt("7".to_string()), JsonValue::Int(7));
    assert_eq!(JsonValue::Int(-7), JsonValue::BigInt("-7".to_string()));
    assert_ne!(JsonValue::BigInt("07".to_string()), JsonValue::Int(7));
    assert_ne!(JsonValue::Int(7), JsonValue::BigInt("07".to_string()));
    assert_ne!(JsonValue::BigInt("-0".to_string()), JsonValue::Int(0));
    assert_ne!(JsonValue::Int(0), JsonValue::BigInt("-0".to_string()));
}

#[test]
fn test_a_not_a_number_float_standing_inside_two_lists_stands_equal_and_a_count_spelled_as_a_whole_float_inside_two_objects_stands_equal_where_one_spelled_as_a_fractional_float_stands_unequal_and_a_number_stands_unequal_to_a_text_of_its_own_spelling(
) {
    assert_eq!(
        JsonValue::List(vec![JsonValue::Float(f64::NAN)]),
        JsonValue::List(vec![JsonValue::Float(f64::NAN)])
    );
    let whole = JsonValue::object(vec![("count", JsonValue::Float(1.0))]);
    let rebuilt = JsonValue::object(vec![("count", JsonValue::Int(1))]);
    assert_eq!(whole, rebuilt);
    let fractional = JsonValue::object(vec![("count", JsonValue::Float(1.5))]);
    let rounded = JsonValue::object(vec![("count", JsonValue::Int(2))]);
    assert_ne!(fractional, rounded);
    assert_ne!(JsonValue::Int(42), JsonValue::text("42"));
    assert_ne!(JsonValue::text("42"), JsonValue::Int(42));
    assert_ne!(JsonValue::Float(1.5), JsonValue::text("1.5"));
}
