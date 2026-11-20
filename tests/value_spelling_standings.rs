//! The pins of slice 16, the whole value spelling
//! statement, over documents the suite hands to the
//! rendering directly. The rendering spells every value
//! it prints through the one value spelling and reads
//! every count through the one count reading, and the
//! pins below walk the two rules at the lines they
//! govern, the header lines and the session sources
//! line and the hook entry lines and its counts and its
//! date lines and the near misses line and the two
//! fixed lines and the unconfigured runs line beyond
//! its identity. Every pin stands as a fence on the
//! landed code, this slice moving no production code.

use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::report::render_markdown;

fn base_entry() -> JsonValue {
    JsonValue::object(vec![
        ("command", JsonValue::text("run.sh")),
        ("id", JsonValue::text("PreToolUse:0:0")),
        ("event", JsonValue::text("PreToolUse")),
        ("matcher", JsonValue::Null),
        ("class", JsonValue::text("firing")),
        (
            "evidence",
            JsonValue::object(vec![
                ("matched", JsonValue::Int(1)),
                ("ran", JsonValue::Int(1)),
                (
                    "effects",
                    JsonValue::object(vec![
                        ("denial", JsonValue::Int(0)),
                        ("modification", JsonValue::Int(0)),
                        ("consumed_output", JsonValue::Int(0)),
                    ]),
                ),
                ("first_ran_at", JsonValue::Null),
                ("last_ran_at", JsonValue::Null),
            ]),
        ),
    ])
}

fn base_fields() -> Vec<(String, JsonValue)> {
    vec![
        ("format".to_string(), JsonValue::text("hookdoctor-report/1")),
        ("config_source".to_string(), JsonValue::text("probe")),
        (
            "session_sources".to_string(),
            JsonValue::List(vec![JsonValue::text("a.jsonl")]),
        ),
        (
            "window".to_string(),
            JsonValue::object(vec![("start", JsonValue::Null), ("end", JsonValue::Null)]),
        ),
        ("hooks".to_string(), JsonValue::List(vec![base_entry()])),
        ("near_misses".to_string(), JsonValue::List(vec![])),
    ]
}

fn set(fields: &mut Vec<(String, JsonValue)>, key: &str, value: JsonValue) {
    for pair in fields.iter_mut() {
        if pair.0 == key {
            pair.1 = value;
            return;
        }
    }
    fields.push((key.to_string(), value));
}

fn drop_key(fields: &mut Vec<(String, JsonValue)>, key: &str) {
    fields.retain(|pair| pair.0 != key);
}

fn page(fields: Vec<(String, JsonValue)>) -> String {
    render_markdown(&JsonValue::Object(fields))
}

fn page_with(key: &str, value: JsonValue) -> String {
    let mut fields = base_fields();
    set(&mut fields, key, value);
    page(fields)
}

fn page_without(key: &str) -> String {
    let mut fields = base_fields();
    drop_key(&mut fields, key);
    page(fields)
}

fn entry_with(key: &str, value: Option<JsonValue>) -> JsonValue {
    let JsonValue::Object(mut pairs) = base_entry() else {
        panic!("the base entry is an object")
    };
    match value {
        Some(value) => set(&mut pairs, key, value),
        None => drop_key(&mut pairs, key),
    }
    JsonValue::Object(pairs)
}

fn evidence_entry(key: &str, mut value: Option<JsonValue>) -> JsonValue {
    let JsonValue::Object(mut pairs) = base_entry() else {
        panic!("the base entry is an object")
    };
    for pair in pairs.iter_mut() {
        if pair.0 == "evidence" {
            let JsonValue::Object(mut evidence) = pair.1.clone() else {
                panic!("the base evidence is an object")
            };
            match value.take() {
                Some(value) => set(&mut evidence, key, value),
                None => drop_key(&mut evidence, key),
            }
            pair.1 = JsonValue::Object(evidence);
        }
    }
    JsonValue::Object(pairs)
}

fn entry_page(entry: JsonValue) -> String {
    page_with("hooks", JsonValue::List(vec![entry]))
}

#[test]
fn test_the_header_lines_spell_every_kind_through_the_value_spelling() {
    assert!(page_without("format").contains("Format None"));
    assert!(page_with("format", JsonValue::Null).contains("Format None"));
    assert!(page_with("format", JsonValue::Bool(true)).contains("Format True"));
    assert!(page_with("format", JsonValue::Bool(false)).contains("Format False"));
    assert!(page_with("format", JsonValue::Int(-3)).contains("Format -3"));
    assert!(page_with("format", JsonValue::Int(0)).contains("Format 0"));
    let digits = JsonValue::BigInt("123456789012345678901234567890".to_string());
    assert!(page_with("format", digits).contains("Format 123456789012345678901234567890"));
    assert!(page_with("format", JsonValue::Float(1.5)).contains("Format 1.5"));
    let nan_page = page_with("format", JsonValue::Float(f64::NAN));
    assert!(nan_page.contains("Format nan"), "{nan_page}");
    assert!(
        !nan_page.contains("Format NaN"),
        "the float at a header field takes the markdown spelling, so the encoder word stands nowhere at the Format line: {nan_page}"
    );
    assert!(page_with("format", JsonValue::text("plain")).contains("Format plain"));
    assert!(page_with("format", JsonValue::List(vec![])).contains("Format []"));
    assert!(page_with("format", JsonValue::Object(vec![])).contains("Format {}"));
    let list_page = page_with(
        "format",
        JsonValue::List(vec![JsonValue::Int(1), JsonValue::Int(2)]),
    );
    assert!(list_page.contains("Format [\n  1,\n  2\n]"), "{list_page}");
    assert!(
        list_page.contains("Format [\n  1,\n  2\n]\n\nConfiguration source probe"),
        "{list_page}"
    );
    assert!(
        !list_page.contains("Format [1, 2]"),
        "a container at a header field takes the JSON rendering whole, so the one line repr stands nowhere at the Format line: {list_page}"
    );
    let object_page = page_with("format", JsonValue::object(vec![("b", JsonValue::Int(1))]));
    assert!(
        object_page.contains("Format {\n  \"b\": 1\n}"),
        "{object_page}"
    );
    assert!(page_with("config_source", JsonValue::Int(7)).contains("Configuration source 7"));
    let source_page = page_with("config_source", JsonValue::List(vec![JsonValue::Int(1)]));
    assert!(
        source_page.contains("Configuration source [\n  1\n]"),
        "{source_page}"
    );
    assert!(
        source_page.contains("Configuration source [\n  1\n]\n\nSession sources a.jsonl"),
        "{source_page}"
    );
    let window = JsonValue::object(vec![
        ("start", JsonValue::Float(1.5)),
        ("end", JsonValue::List(vec![JsonValue::Int(1)])),
    ]);
    let window_page = page_with("window", window);
    assert!(
        window_page.contains("Window start 1.5 end [\n  1\n]"),
        "{window_page}"
    );
    assert!(page_without("window").contains("Window start None end None"));
}

#[test]
fn test_the_session_sources_line_joins_its_labels_and_spells_its_items() {
    let two = JsonValue::List(vec![JsonValue::text("a"), JsonValue::text("b")]);
    assert!(page_with("session_sources", two).contains("Session sources a, b"));
    assert!(page_without("session_sources").contains("Session sources none"));
    assert!(page_with("session_sources", JsonValue::List(vec![])).contains("Session sources none"));
    let one_empty = JsonValue::List(vec![JsonValue::text("")]);
    assert!(page_with("session_sources", one_empty).contains("Session sources none"));
    let int_item = JsonValue::List(vec![JsonValue::Int(3)]);
    assert!(page_with("session_sources", int_item).contains("Session sources 3"));
    let null_item = JsonValue::List(vec![JsonValue::Null]);
    assert!(page_with("session_sources", null_item).contains("Session sources None"));
    let list_item = JsonValue::List(vec![JsonValue::List(vec![JsonValue::Int(1)])]);
    let list_page = page_with("session_sources", list_item);
    assert!(
        list_page.contains("Session sources [\n  1\n]"),
        "{list_page}"
    );
    assert!(page_with("session_sources", JsonValue::text("x")).contains("Session sources none"));
}

#[test]
fn test_the_hook_lines_spell_their_fields_through_the_value_spelling() {
    let first = {
        let JsonValue::Object(mut pairs) = base_entry() else {
            panic!("the base entry is an object")
        };
        set(&mut pairs, "command", JsonValue::Int(7));
        set(&mut pairs, "event", JsonValue::Float(1.5));
        set(
            &mut pairs,
            "matcher",
            JsonValue::BigInt("42424242424242424242".to_string()),
        );
        set(&mut pairs, "class", JsonValue::Bool(true));
        JsonValue::Object(pairs)
    };
    let second = {
        let JsonValue::Object(mut pairs) = base_entry() else {
            panic!("the base entry is an object")
        };
        drop_key(&mut pairs, "command");
        set(&mut pairs, "id", JsonValue::List(vec![JsonValue::Int(1)]));
        set(&mut pairs, "event", JsonValue::Null);
        set(&mut pairs, "matcher", JsonValue::text("auth*"));
        set(&mut pairs, "class", JsonValue::Null);
        JsonValue::Object(pairs)
    };
    let page = page_with("hooks", JsonValue::List(vec![first, second]));
    assert!(page.contains("### 7"), "{page}");
    assert!(page.contains("Identifier PreToolUse:0:0"), "{page}");
    assert!(page.contains("Event 1.5"), "{page}");
    assert!(page.contains("Matcher 42424242424242424242"), "{page}");
    assert!(page.contains("Class True"), "{page}");
    assert!(page.contains("### None"), "{page}");
    assert!(page.contains("Identifier [\n  1\n]"), "{page}");
    assert!(page.contains("Event None"), "{page}");
    assert!(page.contains("Matcher auth*"), "{page}");
    assert!(page.contains("Class None"), "{page}");
}

#[test]
fn test_the_hooks_counts_read_every_non_integer_standing_as_zero() {
    let matched = |value: Option<JsonValue>| entry_page(evidence_entry("matched", value));
    assert!(matched(Some(JsonValue::Int(3))).contains("Matched 3"));
    let big = JsonValue::BigInt("9999999999999999999".to_string());
    assert!(matched(Some(big)).contains("Matched 0"));
    assert!(matched(Some(JsonValue::Float(1.5))).contains("Matched 0"));
    assert!(matched(Some(JsonValue::Float(f64::NAN))).contains("Matched 0"));
    assert!(matched(Some(JsonValue::Bool(true))).contains("Matched 0"));
    assert!(matched(Some(JsonValue::text("3"))).contains("Matched 0"));
    assert!(matched(Some(JsonValue::Null)).contains("Matched 0"));
    assert!(matched(None).contains("Matched 0"));
    let ran_big = JsonValue::BigInt("9999999999999999999".to_string());
    assert!(entry_page(evidence_entry("ran", Some(ran_big))).contains("Ran 0"));
    let effects = JsonValue::object(vec![
        ("denial", JsonValue::Float(2.5)),
        ("modification", JsonValue::text("x")),
        ("consumed_output", JsonValue::Bool(false)),
    ]);
    assert!(entry_page(evidence_entry("effects", Some(effects)))
        .contains("Effects denial 0, modification 0, consumed output 0"));
    let effects = JsonValue::object(vec![
        ("denial", JsonValue::Int(2)),
        ("modification", JsonValue::Int(0)),
        ("consumed_output", JsonValue::Int(1)),
    ]);
    assert!(entry_page(evidence_entry("effects", Some(effects)))
        .contains("Effects denial 2, modification 0, consumed output 1"));
    let bare = entry_page(entry_with("evidence", None));
    assert!(bare.contains("Matched 0"), "{bare}");
    assert!(bare.contains("Ran 0"), "{bare}");
    assert!(
        bare.contains("Effects denial 0, modification 0, consumed output 0"),
        "{bare}"
    );
}

#[test]
fn test_the_first_and_last_ran_lines_spell_their_standings_as_they_stand() {
    let first = |value: Option<JsonValue>| entry_page(evidence_entry("first_ran_at", value));
    assert!(first(Some(JsonValue::text("noon"))).contains("First ran at noon"));
    assert!(first(Some(JsonValue::Int(9))).contains("First ran at 9"));
    let nan_page = first(Some(JsonValue::Float(f64::NAN)));
    assert!(nan_page.contains("First ran at nan"), "{nan_page}");
    assert!(
        !nan_page.contains("First ran at NaN"),
        "a float at a date line takes the markdown spelling, so the encoder word stands nowhere at the First ran line: {nan_page}"
    );
    let list = JsonValue::List(vec![JsonValue::text("a"), JsonValue::Int(2)]);
    let list_page = first(Some(list));
    assert!(
        list_page.contains("First ran at [\n  \"a\",\n  2\n]"),
        "{list_page}"
    );
    let last = |value: Option<JsonValue>| entry_page(evidence_entry("last_ran_at", value));
    assert!(last(Some(JsonValue::Bool(false))).contains("Last ran at False"));
    let object = JsonValue::object(vec![("b", JsonValue::Int(1))]);
    let object_page = last(Some(object));
    assert!(
        object_page.contains("Last ran at {\n  \"b\": 1\n}"),
        "{object_page}"
    );
    assert!(first(None).contains("First ran at None"));
    assert!(last(None).contains("Last ran at None"));
}

#[test]
fn test_the_near_miss_line_spells_its_five_fields_and_reads_its_count() {
    let first_miss = JsonValue::object(vec![
        ("hook_id", JsonValue::Int(9)),
        ("event", JsonValue::text("PreToolUse")),
        ("timestamp", JsonValue::Float(f64::NAN)),
        ("source", JsonValue::text("s.jsonl")),
        ("line", JsonValue::Float(2.5)),
        ("detail", JsonValue::Null),
    ]);
    let second_miss = JsonValue::object(vec![
        ("event", JsonValue::Int(4)),
        ("timestamp", JsonValue::text("t")),
        ("source", JsonValue::Null),
        ("line", JsonValue::Int(12)),
        ("detail", JsonValue::List(vec![JsonValue::Int(1)])),
    ]);
    let page = page_with(
        "near_misses",
        JsonValue::List(vec![first_miss, second_miss]),
    );
    assert!(
        page.contains("- Hook 9 missed a PreToolUse subject at nan in s.jsonl on line 0. None."),
        "{page}"
    );
    assert!(
        page.contains("- Hook None missed a 4 subject at t in None on line 12. [\n  1\n]"),
        "{page}"
    );
    assert!(
        !page.contains("on line 2.5."),
        "the count field of the miss line reads through the count reading, so the float standing stands nowhere as itself: {page}"
    );
    assert!(
        !page.contains("at NaN in"),
        "the timestamp field of the miss line takes the markdown spelling, so the encoder word stands nowhere in the line: {page}"
    );
}

#[test]
fn test_the_two_fixed_lines_stand_as_fixed_text() {
    let empty_hooks = page_with("hooks", JsonValue::List(vec![]));
    assert!(
        empty_hooks.contains("\nNo hooks are configured.\n"),
        "{empty_hooks}"
    );
    let nan_format = {
        let mut fields = base_fields();
        set(&mut fields, "format", JsonValue::Float(f64::NAN));
        set(&mut fields, "hooks", JsonValue::List(vec![]));
        page(fields)
    };
    assert!(
        nan_format.contains("\nNo hooks are configured.\n"),
        "{nan_format}"
    );
    assert!(page_with("hooks", JsonValue::text("x")).contains("\nNo hooks are configured.\n"));
    assert!(page_without("hooks").contains("\nNo hooks are configured.\n"));
    assert!(page_with("near_misses", JsonValue::List(vec![])).contains("## Near misses\n\nNone.\n"));
    assert!(page_with("near_misses", JsonValue::Object(vec![])).contains("\nNone.\n"));
    assert!(page_without("near_misses").contains("\nNone.\n"));
}

#[test]
fn test_the_unconfigured_runs_line_spells_its_event_and_source_and_reads_its_line() {
    fn run(
        command: &str,
        event: JsonValue,
        source: JsonValue,
        line: Option<JsonValue>,
    ) -> JsonValue {
        let mut fields = vec![
            ("event", event),
            ("kind", JsonValue::text("hook_run")),
            ("command", JsonValue::text(command)),
            ("name", JsonValue::Null),
            ("timestamp", JsonValue::Null),
            ("source", source),
        ];
        if let Some(line) = line {
            fields.push(("line", line));
        }
        JsonValue::object(fields)
    }
    let runs = JsonValue::List(vec![
        run(
            "x.sh",
            JsonValue::Float(1.5),
            JsonValue::Null,
            Some(JsonValue::Float(2.5)),
        ),
        run("y.sh", JsonValue::Null, JsonValue::Int(4), None),
        run(
            "z.sh",
            JsonValue::Null,
            JsonValue::text("session.jsonl"),
            Some(JsonValue::BigInt("9999999999999999999".to_string())),
        ),
    ]);
    let page = page_with("unconfigured_runs", runs);
    assert!(
        page.contains("- x.sh ran at 1.5 in None on line 0."),
        "{page}"
    );
    assert!(
        page.contains("- y.sh ran at None in 4 on line 0."),
        "{page}"
    );
    assert!(
        page.contains("- z.sh ran at None in session.jsonl on line 0."),
        "{page}"
    );
    assert!(
        !page.contains("on line 2.5."),
        "the line field of the runs line reads through the count reading, so the float standing stands nowhere as itself: {page}"
    );
}

#[test]
fn test_the_hooks_counts_at_the_matched_seat_read_a_fitting_big_integer_and_a_container_as_zero_and_the_boundary_integers_as_themselves(
) {
    let matched = |value: JsonValue| entry_page(evidence_entry("matched", Some(value)));
    let fitting = JsonValue::BigInt("42".to_string());
    assert!(matched(fitting).contains("Matched 0"));
    let list = JsonValue::List(vec![JsonValue::Int(1)]);
    assert!(matched(list).contains("Matched 0"));
    let object = JsonValue::object(vec![("a", JsonValue::Int(1))]);
    assert!(matched(object).contains("Matched 0"));
    assert!(matched(JsonValue::Int(-3)).contains("Matched -3"));
    assert!(matched(JsonValue::Int(i64::MIN)).contains("Matched -9223372036854775808"));
    assert!(matched(JsonValue::Int(i64::MAX)).contains("Matched 9223372036854775807"));
}
