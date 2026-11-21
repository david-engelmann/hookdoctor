//! The pins of slice 24, the object pair standings,
//! over values the suite builds by hand and over
//! texts the suite hands to the sampling reading
//! directly. The model its object stands as a list
//! of pairs in written order, and the pins below
//! walk the two arms the obligations of the slice
//! state, the read that takes the first standing
//! pair and the sampling replacement that keeps
//! the first position and the last value, and the
//! one consequence the read arm decides at
//! equality. Every pin stands as a fence on the
//! landed code, this slice moving no production
//! code.

use hookdoctor::jsonvalue::{parse_json_lenient, JsonValue};

#[test]
fn test_a_read_returns_the_first_value_at_a_repeated_name_and_the_standing_value_at_a_single_name_and_nothing_at_an_absent_name_or_over_another_kind(
) {
    let object = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(2)),
    ]);
    assert_eq!(object.get("a"), Some(&JsonValue::Int(1)));
    assert_eq!(object.get("b"), Some(&JsonValue::Int(9)));
    assert_eq!(object.get("absent"), None);
    assert_eq!(JsonValue::List(vec![JsonValue::Int(1)]).get("a"), None);
    assert_eq!(JsonValue::text("a").get("a"), None);
}

#[test]
fn test_two_identical_objects_each_carrying_a_key_twice_with_differing_values_do_not_stand_equal_and_the_same_objects_with_agreeing_values_stand_equal(
) {
    let differing = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(2)),
    ]);
    let differing_twin = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(2)),
    ]);
    assert_ne!(differing, differing_twin);
    let agreeing = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(1)),
    ]);
    let agreeing_twin = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(1)),
    ]);
    assert_eq!(agreeing, agreeing_twin);
    let single = JsonValue::object(vec![("a", JsonValue::Int(1)), ("b", JsonValue::Int(9))]);
    assert_ne!(differing, single);
}

#[test]
fn test_a_sampled_object_carries_a_repeated_key_once_at_the_position_of_its_first_standing_with_the_value_of_its_last_standing(
) {
    let first = parse_json_lenient("{\"a\": 1, \"b\": 2, \"a\": 3}").expect("the text reads");
    assert_eq!(
        first.as_object(),
        Some(&vec![
            ("a".to_string(), JsonValue::Int(3)),
            ("b".to_string(), JsonValue::Int(2)),
        ])
    );
    let mirrored = parse_json_lenient("{\"b\": 2, \"a\": 1, \"b\": 3}").expect("the text reads");
    assert_eq!(
        mirrored.as_object(),
        Some(&vec![
            ("b".to_string(), JsonValue::Int(3)),
            ("a".to_string(), JsonValue::Int(1)),
        ])
    );
    let thrice = parse_json_lenient("{\"a\": 1, \"a\": 2, \"a\": 3}").expect("the text reads");
    assert_eq!(
        thrice.as_object(),
        Some(&vec![("a".to_string(), JsonValue::Int(3))])
    );
}

#[test]
fn test_a_sampled_repeat_settles_inside_each_object_on_its_own_pairs_and_a_value_of_one_kind_replaces_a_value_of_another_kind_whole(
) {
    let nested = parse_json_lenient("{\"x\": {\"a\": 1, \"b\": 5, \"a\": 2}, \"a\": 9}")
        .expect("the text reads");
    let outer_pairs = nested.as_object().expect("the value stands as an object");
    assert_eq!(outer_pairs.len(), 2);
    assert_eq!(outer_pairs[0].0, "x");
    assert_eq!(outer_pairs[1], ("a".to_string(), JsonValue::Int(9)));
    let inner = nested.get("x").expect("the outer name stands");
    assert_eq!(
        inner.as_object(),
        Some(&vec![
            ("a".to_string(), JsonValue::Int(2)),
            ("b".to_string(), JsonValue::Int(5)),
        ])
    );
    let object_then_text =
        parse_json_lenient("{\"a\": {\"k\": 1}, \"a\": \"text\"}").expect("the text reads");
    assert_eq!(
        object_then_text.as_object(),
        Some(&vec![("a".to_string(), JsonValue::text("text"))])
    );
    let text_then_object =
        parse_json_lenient("{\"a\": \"text\", \"a\": {\"k\": 1}}").expect("the text reads");
    assert_eq!(
        text_then_object.as_object(),
        Some(&vec![(
            "a".to_string(),
            JsonValue::object(vec![("k", JsonValue::Int(1))])
        )])
    );
}

#[test]
fn test_a_read_of_a_sampled_object_returns_the_last_value_at_a_repeated_name_at_an_outer_seat_and_at_a_nested_seat(
) {
    let outer = parse_json_lenient("{\"a\": 1, \"b\": 2, \"a\": 3}").expect("the text reads");
    assert_eq!(outer.get("a"), Some(&JsonValue::Int(3)));
    let nested = parse_json_lenient("{\"x\": {\"a\": 1, \"a\": 2}}").expect("the text reads");
    assert_eq!(
        nested.get("x").and_then(|inner| inner.get("a")),
        Some(&JsonValue::Int(2))
    );
}
