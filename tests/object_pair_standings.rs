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
    assert_ne!(single, differing);
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

#[test]
fn test_an_object_carrying_a_key_twice_with_differing_values_against_the_same_key_carrying_agreeing_values_reads_unequal_in_the_one_order_and_equal_in_the_other(
) {
    let repeated = JsonValue::object(vec![("a", JsonValue::Int(1)), ("a", JsonValue::Int(2))]);
    let agreeing = JsonValue::object(vec![("a", JsonValue::Int(1)), ("a", JsonValue::Int(1))]);
    assert_ne!(repeated, agreeing);
    assert_eq!(agreeing, repeated);
    let mirrored = JsonValue::object(vec![("a", JsonValue::Int(2)), ("a", JsonValue::Int(1))]);
    let mirrored_agreeing =
        JsonValue::object(vec![("a", JsonValue::Int(2)), ("a", JsonValue::Int(2))]);
    assert_ne!(mirrored, mirrored_agreeing);
    assert_eq!(mirrored_agreeing, mirrored);
    let spread = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(2)),
    ]);
    let spread_agreeing = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(1)),
    ]);
    assert_ne!(spread, spread_agreeing);
    assert_eq!(spread_agreeing, spread);
    let spread_twin = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(9)),
        ("a", JsonValue::Int(2)),
    ]);
    assert_ne!(spread, spread_twin);
    let nested = JsonValue::object(vec![(
        "x",
        JsonValue::object(vec![("a", JsonValue::Int(1)), ("a", JsonValue::Int(2))]),
    )]);
    let nested_agreeing = JsonValue::object(vec![(
        "x",
        JsonValue::object(vec![("a", JsonValue::Int(1)), ("a", JsonValue::Int(1))]),
    )]);
    assert_ne!(nested, nested_agreeing);
    assert_eq!(nested_agreeing, nested);
    let carried = JsonValue::List(vec![repeated.clone()]);
    let carried_agreeing = JsonValue::List(vec![agreeing.clone()]);
    assert_ne!(carried, carried_agreeing);
    assert_eq!(carried_agreeing, carried);
    let text_repeated = JsonValue::object(vec![
        ("a", JsonValue::text("one")),
        ("a", JsonValue::text("two")),
    ]);
    let text_agreeing = JsonValue::object(vec![
        ("a", JsonValue::text("one")),
        ("a", JsonValue::text("one")),
    ]);
    assert_ne!(text_repeated, text_agreeing);
    assert_eq!(text_agreeing, text_repeated);
    let shuffled = JsonValue::object(vec![("a", JsonValue::Int(1)), ("b", JsonValue::Int(9))]);
    let shuffled_turned =
        JsonValue::object(vec![("b", JsonValue::Int(9)), ("a", JsonValue::Int(1))]);
    assert_eq!(shuffled, shuffled_turned);
    assert_eq!(shuffled_turned, shuffled);
    assert_ne!(repeated, mirrored);
    assert_ne!(mirrored, repeated);
}

#[test]
fn test_an_object_carrying_a_key_twice_against_an_object_carrying_that_key_once_and_a_second_key_once_reads_equal_in_the_one_order_and_unequal_in_the_other(
) {
    let subset = JsonValue::object(vec![("a", JsonValue::Int(1)), ("a", JsonValue::Int(1))]);
    let carrying = JsonValue::object(vec![("a", JsonValue::Int(1)), ("b", JsonValue::Int(1))]);
    assert_eq!(subset, carrying);
    assert_ne!(carrying, subset);
    let subset_mirrored =
        JsonValue::object(vec![("a", JsonValue::Int(2)), ("a", JsonValue::Int(2))]);
    let carrying_mirrored =
        JsonValue::object(vec![("a", JsonValue::Int(2)), ("b", JsonValue::Int(7))]);
    assert_eq!(subset_mirrored, carrying_mirrored);
    assert_ne!(carrying_mirrored, subset_mirrored);
    let spread = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(5)),
        ("a", JsonValue::Int(1)),
    ]);
    let spread_carrying = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(5)),
        ("c", JsonValue::Int(9)),
    ]);
    assert_eq!(spread, spread_carrying);
    assert_ne!(spread_carrying, spread);
    let triple = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("a", JsonValue::Int(1)),
        ("a", JsonValue::Int(1)),
    ]);
    let triple_carrying = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(1)),
        ("c", JsonValue::Int(1)),
    ]);
    assert_eq!(triple, triple_carrying);
    assert_ne!(triple_carrying, triple);
    let paired = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(2)),
        ("b", JsonValue::Int(2)),
    ]);
    let paired_carrying = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(2)),
        ("c", JsonValue::Int(3)),
        ("d", JsonValue::Int(3)),
    ]);
    assert_eq!(paired, paired_carrying);
    assert_ne!(paired_carrying, paired);
    let nested = JsonValue::object(vec![(
        "x",
        JsonValue::object(vec![("a", JsonValue::Int(1)), ("a", JsonValue::Int(1))]),
    )]);
    let nested_carrying = JsonValue::object(vec![(
        "x",
        JsonValue::object(vec![("a", JsonValue::Int(1)), ("b", JsonValue::Int(1))]),
    )]);
    assert_eq!(nested, nested_carrying);
    assert_ne!(nested_carrying, nested);
    let carried = JsonValue::List(vec![subset.clone()]);
    let carried_carrying = JsonValue::List(vec![carrying.clone()]);
    assert_eq!(carried, carried_carrying);
    assert_ne!(carried_carrying, carried);
    let text_subset = JsonValue::object(vec![
        ("a", JsonValue::text("x")),
        ("a", JsonValue::text("x")),
    ]);
    let text_carrying = JsonValue::object(vec![
        ("a", JsonValue::text("x")),
        ("b", JsonValue::text("y")),
    ]);
    assert_eq!(text_subset, text_carrying);
    assert_ne!(text_carrying, text_subset);
    let differing_subset =
        JsonValue::object(vec![("a", JsonValue::Int(1)), ("a", JsonValue::Int(2))]);
    assert_ne!(differing_subset, carrying);
    assert_ne!(carrying, differing_subset);
    let absent_other = JsonValue::object(vec![("b", JsonValue::Int(1)), ("b", JsonValue::Int(1))]);
    assert_ne!(subset, absent_other);
    assert_ne!(absent_other, subset);
    let parted = JsonValue::object(vec![("a", JsonValue::Int(2)), ("b", JsonValue::Int(1))]);
    assert_ne!(subset, parted);
    assert_ne!(parted, subset);
    let longer = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(1)),
        ("c", JsonValue::Int(1)),
    ]);
    assert_ne!(subset, longer);
    assert_ne!(longer, subset);
    let equal_keys = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(2)),
    ]);
    let equal_keys_turned = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(2)),
        ("b", JsonValue::Int(2)),
    ]);
    assert_eq!(equal_keys, equal_keys_turned);
    assert_eq!(equal_keys_turned, equal_keys);
    let shared = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(3)),
    ]);
    let shared_parted = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("b", JsonValue::Int(4)),
        ("c", JsonValue::Int(9)),
    ]);
    assert_ne!(shared, shared_parted);
    assert_ne!(shared_parted, shared);
    let absent_keyed = JsonValue::object(vec![
        ("a", JsonValue::Int(1)),
        ("a", JsonValue::Int(1)),
        ("z", JsonValue::Int(9)),
    ]);
    assert_ne!(absent_keyed, triple_carrying);
    assert_ne!(triple_carrying, absent_keyed);
}
