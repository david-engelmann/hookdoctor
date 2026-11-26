//! The pins of slice 51, the civil conversion
//! standings, over dates and day numbers and
//! timestamp texts the pins hand to the dates
//! doors directly. Slice 20 states the instant
//! grammar the doors read and slice 22 states
//! the evidence dates choice, and the pins below
//! state the arithmetic the grammar stands on,
//! the conversion pair whole and the floor and
//! the range of the reading and the absent
//! weekday the week construction fixes. Every
//! pin stands as a fence on the landed code,
//! this slice moving no production code.

use hookdoctor::dates::{civil_from_days, days_from_civil, earliest_instant, parse_instant};

#[test]
fn test_the_epoch_anchor_stands_at_zero_and_the_first_day_of_year_one_and_the_last_day_of_year_9999_read_at_both_doors(
) {
    assert_eq!(days_from_civil(1970, 1, 1), 0);
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(days_from_civil(1969, 12, 31), -1);
    assert_eq!(civil_from_days(-1), (1969, 12, 31));
    assert_eq!(days_from_civil(1970, 1, 2), 1);
    assert_eq!(civil_from_days(1), (1970, 1, 2));
    assert_eq!(days_from_civil(1, 1, 1), -719162);
    assert_eq!(civil_from_days(-719162), (1, 1, 1));
    assert_eq!(days_from_civil(9999, 12, 31), 2932896);
    assert_eq!(civil_from_days(2932896), (9999, 12, 31));
}

#[test]
fn test_the_two_conversion_readings_undo_each_other_over_the_whole_domain_the_pair_runs_over() {
    let dates = [
        (1, 1, 1),
        (0, 2, 29),
        (0, 3, 1),
        (-400, 1, 1),
        (10000, 1, 1),
        (9999, 12, 31),
        (1970, 1, 1),
        (2024, 2, 29),
        (1900, 3, 1),
        (4, 2, 29),
        (100, 3, 1),
        (400, 2, 29),
    ];
    for (year, month, day) in dates {
        assert_eq!(
            civil_from_days(days_from_civil(year, month, day)),
            (year, month, day)
        );
    }
    let numbers = [0, -1, -719162, -719528, 2932896, 10957, -719469];
    for number in numbers {
        let (read_year, read_month, read_day) = civil_from_days(number);
        assert_eq!(days_from_civil(read_year, read_month, read_day), number);
    }
}

#[test]
fn test_two_dates_one_day_apart_stand_one_day_number_apart_at_every_boundary_and_the_year_spans_carry_the_leap_standing(
) {
    assert_eq!(
        days_from_civil(1900, 3, 1) - days_from_civil(1900, 2, 28),
        1
    );
    assert_eq!(
        days_from_civil(2023, 3, 1) - days_from_civil(2023, 2, 28),
        1
    );
    assert_eq!(
        days_from_civil(2024, 3, 1) - days_from_civil(2024, 2, 28),
        2
    );
    assert_eq!(
        days_from_civil(2024, 2, 29) - days_from_civil(2024, 2, 28),
        1
    );
    assert_eq!(
        days_from_civil(2000, 2, 29) - days_from_civil(2000, 2, 28),
        1
    );
    assert_eq!(
        days_from_civil(2023, 2, 1) - days_from_civil(2023, 1, 31),
        1
    );
    assert_eq!(
        days_from_civil(2023, 5, 1) - days_from_civil(2023, 4, 30),
        1
    );
    assert_eq!(
        days_from_civil(2024, 1, 1) - days_from_civil(2023, 12, 31),
        1
    );
    assert_eq!(
        days_from_civil(1900, 1, 1) - days_from_civil(1899, 12, 31),
        1
    );
    assert_eq!(
        days_from_civil(2025, 1, 1) - days_from_civil(2024, 1, 1),
        366
    );
    assert_eq!(
        days_from_civil(2024, 1, 1) - days_from_civil(2023, 1, 1),
        365
    );
    assert_eq!(
        days_from_civil(1901, 1, 1) - days_from_civil(1900, 1, 1),
        365
    );
    assert_eq!(
        days_from_civil(2001, 1, 1) - days_from_civil(2000, 1, 1),
        366
    );
}

#[test]
fn test_the_first_day_of_each_month_stands_at_the_count_of_the_days_the_earlier_months_of_its_year_carry(
) {
    let common = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    for (index, offset) in common.iter().enumerate() {
        let month = index as i64 + 1;
        assert_eq!(
            days_from_civil(2023, month, 1) - days_from_civil(2023, 1, 1),
            *offset
        );
    }
    let leap = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];
    for (index, offset) in leap.iter().enumerate() {
        let month = index as i64 + 1;
        assert_eq!(
            days_from_civil(2024, month, 1) - days_from_civil(2024, 1, 1),
            *offset
        );
    }
}

#[test]
fn test_the_pair_reads_an_out_of_range_month_or_day_forward_over_its_boundary_and_refuses_nothing()
{
    assert_eq!(days_from_civil(2023, 2, 30), days_from_civil(2023, 3, 2));
    assert_eq!(days_from_civil(2023, 13, 1), days_from_civil(2024, 1, 1));
    assert_eq!(days_from_civil(2023, 1, 0), days_from_civil(2022, 12, 31));
    assert_eq!(days_from_civil(2023, 0, 1), days_from_civil(2022, 12, 1));
}

#[test]
fn test_the_reading_stands_at_its_floor_at_the_year_one_spellings_and_at_its_ceiling_under_the_largest_offset_and_refuses_a_full_day_offset(
) {
    assert_eq!(earliest_instant(), -62135596800000000);
    assert_eq!(
        earliest_instant(),
        i128::from(days_from_civil(1, 1, 1)) * 86_400_000_000
    );
    assert_eq!(parse_instant("0001-01-01"), Some(-62135596800000000));
    assert_eq!(parse_instant("00010101"), Some(-62135596800000000));
    assert_eq!(
        parse_instant("0001-01-01T00:00:00Z"),
        Some(-62135596800000000)
    );
    assert_eq!(parse_instant("0001-W01-1"), Some(-62135596800000000));
    assert_eq!(
        parse_instant("0001-01-01T00:00:01Z"),
        Some(-62135596799000000)
    );
    assert_eq!(parse_instant("0001-01-02"), Some(-62135510400000000));
    assert_eq!(parse_instant("1969-12-31T23:59:59.999999Z"), Some(-1));
    assert_eq!(parse_instant("1969-12-31T00:00:00Z"), Some(-86400000000));
    assert_eq!(
        parse_instant("9999-12-31T23:59:59.999999Z"),
        Some(253402300799999999)
    );
    assert_eq!(
        parse_instant("9999-12-31T23:59:59.999999-23:59"),
        Some(253402387139999999)
    );
    assert_eq!(
        parse_instant("9999-12-31T23:59:59.999999-23:59:59.999999"),
        Some(253402387199999998)
    );
    assert_eq!(parse_instant("9999-12-31T23:59:59.999999-24:00"), None);
}

#[test]
fn test_a_week_date_carrying_no_weekday_reads_at_the_monday_of_its_week() {
    assert_eq!(parse_instant("2020-W01"), Some(1577664000000000));
    assert_eq!(parse_instant("2020W01"), Some(1577664000000000));
    assert_eq!(parse_instant("2020-W01T00:00:00Z"), Some(1577664000000000));
    assert_eq!(
        parse_instant("2020-W01-1T00:00:00Z"),
        Some(1577664000000000)
    );
    assert_eq!(
        parse_instant("2019-12-30T00:00:00Z"),
        Some(1577664000000000)
    );
    assert_eq!(
        parse_instant("2020-W01-2T00:00:00Z"),
        Some(1577750400000000)
    );
}
