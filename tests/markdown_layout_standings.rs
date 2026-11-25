//! The pins of slice 47, the markdown layout
//! standings, over report documents the pins
//! build at the builder seats of the value type
//! and hand to the public markdown rendering.
//! PORT-7 states the markdown rendering as byte
//! for byte the Python rendering, line for line,
//! at the parity grain alone, and slice 7 states
//! the check mode of the report and states no
//! layout, and VS states the spelling of every
//! value a line prints and states no line, and
//! MI states the identity of an unconfigured
//! run and states no section, and the pins below
//! state the layout of the rendering whole, the
//! lines and their order and the blank lines
//! between them and the standings of its
//! sections and the ending of the document.
//! Every pin stands as a fence on the landed
//! code, this slice moving no production code.

use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::report::render_markdown;

fn text(value: &str) -> JsonValue {
    JsonValue::text(value)
}

fn head_text(format: &str, source: &str, sources: &str, window: &str) -> String {
    format!(
        "# hookdoctor examination\n\nFormat {format}\nConfiguration source {source}\nSession sources {sources}\nWindow start {window}\n"
    )
}

fn empty_head_text() -> String {
    head_text("None", "None", "none", "None end None")
}

fn full_hook() -> JsonValue {
    JsonValue::object(vec![
        ("command", text("zeta hook")),
        ("id", text("zeta:0:0")),
        ("event", text("PreToolUse")),
        ("matcher", text("Bash")),
        ("class", text("firing")),
        (
            "evidence",
            JsonValue::object(vec![
                ("matched", JsonValue::Int(3)),
                ("ran", JsonValue::Int(2)),
                (
                    "effects",
                    JsonValue::object(vec![
                        ("denial", JsonValue::Int(1)),
                        ("modification", JsonValue::Int(0)),
                        ("consumed_output", JsonValue::Int(2)),
                    ]),
                ),
                ("first_ran_at", text("2026-01-01T00:00:00Z")),
                ("last_ran_at", text("2026-01-01T00:05:00Z")),
            ]),
        ),
    ])
}

fn bare_hook() -> JsonValue {
    JsonValue::object(vec![
        ("command", text("alpha hook")),
        ("id", text("alpha:1:0")),
        ("event", text("Stop")),
        ("matcher", JsonValue::Null),
        ("class", text("silent")),
    ])
}

const FULL_BLOCK: &str = "### zeta hook\n\nIdentifier zeta:0:0\nEvent PreToolUse\nMatcher Bash\nClass firing\nMatched 3\nRan 2\nEffects denial 1, modification 0, consumed output 2\nFirst ran at 2026-01-01T00:00:00Z\nLast ran at 2026-01-01T00:05:00Z\n";
const BARE_BLOCK: &str = "### alpha hook\n\nIdentifier alpha:1:0\nEvent Stop\nMatcher None\nClass silent\nMatched 0\nRan 0\nEffects denial 0, modification 0, consumed output 0\nFirst ran at None\nLast ran at None\n";

fn first_miss() -> JsonValue {
    JsonValue::object(vec![
        ("hook_id", text("zeta:0:0")),
        ("event", text("PreToolUse")),
        ("timestamp", text("2026-01-01T00:01:00Z")),
        ("source", text("a.jsonl")),
        ("line", JsonValue::Int(7)),
        ("detail", text("the matcher refused the subject")),
    ])
}

const FIRST_MISS_LINE: &str = "- Hook zeta:0:0 missed a PreToolUse subject at 2026-01-01T00:01:00Z in a.jsonl on line 7. the matcher refused the subject.";

fn first_run() -> JsonValue {
    JsonValue::object(vec![
        ("command", text("make build")),
        ("event", text("PostToolUse")),
        ("source", text("a.jsonl")),
        ("line", JsonValue::Int(12)),
    ])
}

const FIRST_RUN_LINE: &str = "- make build ran at PostToolUse in a.jsonl on line 12.";

#[test]
fn test_the_document_opens_with_its_head_and_its_empty_sections_stand_at_their_empty_forms() {
    let empty_document = JsonValue::object(vec![]);
    let expected_empty =
        empty_head_text() + "\n## Hooks\n\nNo hooks are configured.\n\n## Near misses\n\nNone.\n";
    assert_eq!(render_markdown(&empty_document), expected_empty);
    let headed_document = JsonValue::object(vec![
        ("format", text("hookdoctor-report/1")),
        ("config_source", text("settings.json")),
        (
            "session_sources",
            JsonValue::List(vec![text("a.jsonl"), text("b.jsonl")]),
        ),
        (
            "window",
            JsonValue::object(vec![
                ("start", text("2026-01-01T00:00:00Z")),
                ("end", text("2026-01-02T00:00:00Z")),
            ]),
        ),
        ("hooks", JsonValue::List(vec![])),
        ("near_misses", JsonValue::List(vec![])),
    ]);
    let expected_headed = head_text(
        "hookdoctor-report/1",
        "settings.json",
        "a.jsonl, b.jsonl",
        "2026-01-01T00:00:00Z end 2026-01-02T00:00:00Z",
    ) + "\n## Hooks\n\nNo hooks are configured.\n\n## Near misses\n\nNone.\n";
    assert_eq!(render_markdown(&headed_document), expected_headed);
}

#[test]
fn test_a_hook_block_stands_for_each_entry_in_the_order_the_entries_stand_in_the_document() {
    let document = JsonValue::object(vec![(
        "hooks",
        JsonValue::List(vec![full_hook(), bare_hook()]),
    )]);
    let expected = empty_head_text()
        + "\n## Hooks\n\n"
        + FULL_BLOCK
        + "\n"
        + BARE_BLOCK
        + "\n## Near misses\n\nNone.\n";
    assert_eq!(render_markdown(&document), expected);
}

#[test]
fn test_a_count_reads_as_the_integer_itself_where_it_stands_as_an_integer_and_as_zero_at_every_other_kind(
) {
    let hook = JsonValue::object(vec![
        ("command", text("count hook")),
        ("id", text("c:0:0")),
        ("event", text("PreToolUse")),
        ("matcher", text("Bash")),
        ("class", text("firing without effect")),
        (
            "evidence",
            JsonValue::object(vec![
                (
                    "matched",
                    JsonValue::BigInt("99999999999999999999999999".to_string()),
                ),
                ("ran", JsonValue::Float(2.5)),
                (
                    "effects",
                    JsonValue::object(vec![
                        ("denial", text("x")),
                        ("modification", JsonValue::Int(4)),
                        ("consumed_output", JsonValue::BigInt("42".to_string())),
                    ]),
                ),
                ("first_ran_at", JsonValue::Int(7)),
            ]),
        ),
    ]);
    let document = JsonValue::object(vec![("hooks", JsonValue::List(vec![hook]))]);
    let expected = empty_head_text()
        + "\n## Hooks\n\n### count hook\n\nIdentifier c:0:0\nEvent PreToolUse\nMatcher Bash\nClass firing without effect\nMatched 0\nRan 0\nEffects denial 0, modification 4, consumed output 0\nFirst ran at 7\nLast ran at None\n"
        + "\n## Near misses\n\nNone.\n";
    assert_eq!(render_markdown(&document), expected);
}

#[test]
fn test_a_line_stands_for_each_near_miss_in_document_order_and_no_line_where_no_miss_stands() {
    let second_miss = JsonValue::object(vec![
        ("hook_id", text("alpha:1:0")),
        ("event", text("Stop")),
        ("timestamp", text("2026-01-01T00:02:00Z")),
        ("source", text("b.jsonl")),
        ("line", text("9")),
        ("detail", text("no detail stands")),
    ]);
    let document = JsonValue::object(vec![(
        "near_misses",
        JsonValue::List(vec![first_miss(), second_miss]),
    )]);
    let expected = empty_head_text()
        + "\n## Hooks\n\nNo hooks are configured.\n\n## Near misses\n\n"
        + FIRST_MISS_LINE
        + "\n- Hook alpha:1:0 missed a Stop subject at 2026-01-01T00:02:00Z in b.jsonl on line 0. no detail stands.\n";
    assert_eq!(render_markdown(&document), expected);
}

#[test]
fn test_the_unconfigured_runs_section_stands_only_where_its_field_stands_as_a_populated_list() {
    let expected_empty =
        empty_head_text() + "\n## Hooks\n\nNo hooks are configured.\n\n## Near misses\n\nNone.\n";
    let absent_document = JsonValue::object(vec![]);
    assert_eq!(render_markdown(&absent_document), expected_empty);
    let empty_document = JsonValue::object(vec![("unconfigured_runs", JsonValue::List(vec![]))]);
    assert_eq!(render_markdown(&empty_document), expected_empty);
    let second_run = JsonValue::object(vec![
        ("command", text("")),
        ("name", text("named run")),
        ("event", text("Stop")),
        ("source", text("b.jsonl")),
        ("line", JsonValue::Int(3)),
    ]);
    let populated_document = JsonValue::object(vec![(
        "unconfigured_runs",
        JsonValue::List(vec![first_run(), second_run]),
    )]);
    let expected_populated = expected_empty
        + "\n## Unconfigured runs\n\n"
        + FIRST_RUN_LINE
        + "\n- named run ran at Stop in b.jsonl on line 3.\n";
    assert_eq!(render_markdown(&populated_document), expected_populated);
}

#[test]
fn test_the_document_ends_with_exactly_one_newline_and_no_line_stands_beyond_the_lines_of_this_section(
) {
    let document = JsonValue::object(vec![
        ("hooks", JsonValue::List(vec![full_hook()])),
        ("near_misses", JsonValue::List(vec![first_miss()])),
        ("unconfigured_runs", JsonValue::List(vec![first_run()])),
    ]);
    let rendered = render_markdown(&document);
    let expected = empty_head_text()
        + "\n## Hooks\n\n"
        + FULL_BLOCK
        + "\n## Near misses\n\n"
        + FIRST_MISS_LINE
        + "\n\n## Unconfigured runs\n\n"
        + FIRST_RUN_LINE
        + "\n";
    assert_eq!(rendered, expected);
    assert_eq!(rendered.matches('\n').count(), 28);
    assert!(rendered.ends_with('\n'));
    assert!(!rendered.ends_with("\n\n"));
    let away_document = JsonValue::object(vec![
        ("hooks", JsonValue::List(vec![full_hook()])),
        ("near_misses", JsonValue::List(vec![first_miss()])),
    ]);
    let expected_away = empty_head_text()
        + "\n## Hooks\n\n"
        + FULL_BLOCK
        + "\n## Near misses\n\n"
        + FIRST_MISS_LINE
        + "\n";
    assert_eq!(render_markdown(&away_document), expected_away);
}
