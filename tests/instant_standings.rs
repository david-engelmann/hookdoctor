//! The pins of slice 20, the instant grammar, over
//! timestamp texts the suite hands to the instant
//! reading directly. The reading stands stated at rule
//! level in PORT-5 by reference to the timestamp probe
//! prose, and the pins below walk the grammar the
//! obligations of the slice state at the families the
//! executed cells cover, the calendar dates and the
//! week dates and the forms that refuse at the date
//! seat and the separator and the time forms and the
//! fraction and the offset and its fraction. Every pin
//! stands as a fence on the landed code, this slice
//! moving no production code.

use hookdoctor::dates::parse_instant;

#[test]
fn test_the_calendar_dates_stand_or_refuse_by_civil_validity_at_their_edges() {
    assert_eq!(
        parse_instant("2024-02-29T00:00:00Z"),
        Some(1709164800000000)
    );
    assert_eq!(parse_instant("2025-02-29T00:00:00Z"), None);
    assert_eq!(parse_instant("2026-04-31T00:00:00Z"), None);
    assert_eq!(parse_instant("2026-01-00T00:00:00Z"), None);
    assert_eq!(parse_instant("2026-00-15T00:00:00Z"), None);
    assert_eq!(parse_instant("2026-01-32T00:00:00Z"), None);
}

#[test]
fn test_the_week_dates_stand_by_the_thursday_of_their_week_in_the_extended_and_basic_forms_and_the_mixed_forms_refuse(
) {
    assert_eq!(
        parse_instant("2020-W53-7T00:00:00Z"),
        Some(1609632000000000)
    );
    assert_eq!(parse_instant("2021-W53-1T00:00:00Z"), None);
    assert_eq!(
        parse_instant("2026-W53-7T00:00:00Z"),
        Some(1798934400000000)
    );
    assert_eq!(parse_instant("2026-W00-1T00:00:00Z"), None);
    assert_eq!(parse_instant("2026-W01-0T00:00:00Z"), None);
    assert_eq!(parse_instant("2026-W01-8T00:00:00Z"), None);
    assert_eq!(parse_instant("2026W011T09:00:00Z"), Some(1766998800000000));
    assert_eq!(parse_instant("2026W01-1T00:00:00Z"), None);
    assert_eq!(parse_instant("2026-W011T00:00:00Z"), None);
    assert_eq!(
        parse_instant("2026-W01-1T00:00:00Z"),
        Some(1766966400000000)
    );
    assert_eq!(
        parse_instant("2026-W52-7T00:00:00Z"),
        Some(1798329600000000)
    );
    assert_eq!(
        parse_instant("2019-W52-7T00:00:00Z"),
        Some(1577577600000000)
    );
    assert_eq!(parse_instant("2019-W53-1T00:00:00Z"), None);
}

#[test]
fn test_the_ordinal_and_reduced_date_forms_refuse_and_a_bare_date_refuses_where_its_day_stands_outside_its_month(
) {
    assert_eq!(parse_instant("2026015"), None);
    assert_eq!(parse_instant("2026-015"), None);
    assert_eq!(parse_instant("2026-01"), None);
    assert_eq!(parse_instant("2026-02-29"), None);
}

#[test]
fn test_the_separator_stands_as_any_one_character_and_a_doubled_separator_refuses_and_the_time_may_stand_as_the_hour_alone(
) {
    assert_eq!(parse_instant("2026-01-01Z09:00:00"), Some(1767258000000000));
    assert_eq!(
        parse_instant("2026-01-01é09:00:00Z"),
        Some(1767258000000000)
    );
    assert_eq!(parse_instant("2026-01-01TT09:00:00Z"), None);
    assert_eq!(parse_instant("2026-01-01T09"), Some(1767258000000000));
}

#[test]
fn test_the_time_components_refuse_past_their_ranges_and_the_basic_run_reads_its_pairs_and_its_fraction_digits(
) {
    assert_eq!(parse_instant("2026-01-01T09:60:00Z"), None);
    assert_eq!(
        parse_instant("2026-01-01T23:59:59Z"),
        Some(1767311999000000)
    );
    assert_eq!(parse_instant("2026-01-01T0900"), Some(1767258000000000));
    assert_eq!(parse_instant("2026-01-01T09000"), None);
    assert_eq!(
        parse_instant("2026-01-01T09000012Z"),
        Some(1767258000120000)
    );
    assert_eq!(parse_instant("2026-01-01T09000012"), Some(1767258000120000));
}

#[test]
fn test_the_fraction_reads_as_a_fraction_of_a_second_at_whichever_component_it_follows_and_truncates_past_six_digits_and_an_empty_fraction_at_the_end_of_the_text_refuses(
) {
    assert_eq!(
        parse_instant("2026-01-01T09:00:00:5Z"),
        Some(1767258000500000)
    );
    assert_eq!(parse_instant("2026-01-01T09,5Z"), Some(1767258000500000));
    assert_eq!(
        parse_instant("2026-01-01T09:00:00.5"),
        Some(1767258000500000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00.1234567Z"),
        Some(1767258000123456)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00,1234567Z"),
        Some(1767258000123456)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00.000001Z"),
        Some(1767258000000001)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00.0000001Z"),
        Some(1767258000000000)
    );
    assert_eq!(parse_instant("2026-01-01T09:00:00,"), None);
}

#[test]
fn test_the_offset_sums_its_components_and_refuses_at_a_full_day_and_a_trailing_z_after_an_offset_refuses(
) {
    assert_eq!(
        parse_instant("2026-01-01T09:00:00-05"),
        Some(1767276000000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+05:30"),
        Some(1767238200000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00-00:00:30"),
        Some(1767258030000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T12:00:00+00:00"),
        Some(1767268800000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+23:59:59"),
        Some(1767171601000000)
    );
    assert_eq!(parse_instant("2026-01-01T09:00:00+24:00:00.0"), None);
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+00:00:00.0"),
        Some(1767258000000000)
    );
    assert_eq!(parse_instant("2026-01-01T09:00:00+00:00Z"), None);
}

#[test]
fn test_the_offset_fraction_counts_where_the_whole_part_stands_nonzero_and_drops_where_the_whole_part_stands_at_zero(
) {
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+01.5"),
        Some(1767254399500000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+01:00.5"),
        Some(1767254399500000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+00:30.5"),
        Some(1767256199500000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+00:00:30.5"),
        Some(1767257969500000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+23:59:59.9"),
        Some(1767171600100000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+00.5"),
        Some(1767258000000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+00:00.5"),
        Some(1767258000000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+00:00:00.5"),
        Some(1767258000000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00-00.5"),
        Some(1767258000000000)
    );
    assert_eq!(
        parse_instant("2026-01-01T09:00:00+00:30.0"),
        Some(1767256200000000)
    );
}
