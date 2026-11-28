//! The pins of slice 61, the doc comment cure
//! standings, over the production sources the
//! text pins read as data and over hooks and
//! records the suite builds in code and hands
//! to the library directly and over settings
//! and session files the suite writes itself
//! for the shipped binary. WW states the
//! window words and EP states the validation
//! error parting and PORT2-3 and SF state the
//! door and its label resolutions, and this
//! slice states the standings of the two
//! cured comments and of the label
//! resolution order at GL-1 to GL-6. The two
//! text pins read the comments as data and
//! drive nothing, and the behaviour pins tie
//! each cured clause to the seat it names, so
//! no silent parting between the text and
//! the code stands open at either comment.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use hookdoctor::errors::ReportValidationError;
use hookdoctor::examine::{
    build_evidence, build_report, validate_evidence, validate_markdown, validate_report,
};
use hookdoctor::ingest::{HookConfig, TraceRecord};
use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::reader::read_json_document;
use hookdoctor::report::render_markdown;

/// The synthetic configuration the door pins
/// drive the binary over, one command hook
/// standing under the Bash matcher.
const SETTINGS_TEXT: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

/// The synthetic session the door pins drive
/// the binary over, one assistant tool call
/// standing inside it.
const SESSION_TEXT: &str = r#"{"type": "assistant", "timestamp": "2026-01-01T00:00:00Z", "sessionId": "s1", "message": {"content": [{"type": "tool_use", "name": "Bash", "id": "c1"}]}}"#;

/// A scratch directory that removes itself
/// when it leaves scope, so a panicking
/// assertion leaves no state behind.
struct ScratchDir(PathBuf);

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn pin_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale pin directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a pin directory stands");
    dir
}

fn cell_dir(pin: &Path, cell: &str) -> PathBuf {
    let dir = pin.join(cell);
    std::fs::create_dir_all(&dir).expect("a cell directory stands");
    std::fs::write(dir.join("settings.json"), SETTINGS_TEXT).expect("the configuration stands");
    std::fs::write(dir.join("session.jsonl"), format!("{SESSION_TEXT}\n"))
        .expect("the session stands");
    dir
}

fn text_word(text: &str) -> OsString {
    OsString::from(text)
}

#[cfg(unix)]
fn byte_word(bytes: &[u8]) -> OsString {
    OsString::from_vec(bytes.to_vec())
}

fn run_report(dir: &Path, extra: &[(&str, OsString)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_hookdoctor"));
    command
        .arg("--config")
        .arg(dir.join("settings.json"))
        .arg("--session")
        .arg(dir.join("session.jsonl"))
        .arg("--output")
        .arg(dir.join("report.json"))
        .arg("--report")
        .arg(dir.join("report.md"));
    for (flag, value) in extra {
        command.arg(flag).arg(value);
    }
    command.output().expect("the binary runs")
}

fn window_bound(dir: &Path, member: &str) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("report.json")).expect("the report stands");
    let document = read_json_document(&text, "report").expect("the report parses");
    let window = document
        .get("window")
        .expect("the report carries its window");
    window
        .get(member)
        .and_then(|value| value.as_text())
        .map(str::to_string)
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn refusal_line(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(2), "the door refuses at exit 2");
    assert!(
        output.stdout.is_empty(),
        "the door writes nothing to the standard output"
    );
    stderr_text(output)
}

fn record(kind: &str, line: i64) -> TraceRecord {
    TraceRecord {
        kind: kind.to_string(),
        timestamp: Some("2026-01-01T00:00:00Z".to_string()),
        session: Some("sess".to_string()),
        file_index: 0,
        line,
        tool_name: None,
        call_id: None,
        is_error: false,
        text: String::new(),
        hook_name: None,
        hook_event: None,
        command: None,
        content: String::new(),
        stdout: String::new(),
        exit_code: None,
        attachment_type: None,
        commands: Vec::new(),
        prevented_continuation: false,
        has_output: false,
    }
}

fn tool_call(line: i64) -> TraceRecord {
    let mut built = record("tool_call", line);
    built.tool_name = Some("Bash".to_string());
    built.call_id = Some("c1".to_string());
    built
}

fn hook(event: &str, matcher: Option<&str>, command: &str) -> HookConfig {
    HookConfig {
        id: format!("{event}:0:0"),
        event: event.to_string(),
        matcher: matcher.map(str::to_string),
        hook_type: "command".to_string(),
        command: command.to_string(),
    }
}

fn stop_hook() -> HookConfig {
    hook("Stop", None, "stopper.sh")
}

fn sources() -> Vec<String> {
    vec!["sess".to_string()]
}

fn tamper(document: &JsonValue, field: &str, value: JsonValue) -> JsonValue {
    let pairs = document
        .as_object()
        .expect("the document is an object")
        .clone();
    let mut out: Vec<(&str, JsonValue)> = Vec::new();
    let mut found = false;
    for (key, standing) in &pairs {
        if key == field {
            out.push((key.as_str(), value.clone()));
            found = true;
        } else {
            out.push((key.as_str(), standing.clone()));
        }
    }
    if !found {
        out.push((field, value));
    }
    JsonValue::object(out)
}

/// The doc comment block standing above the
/// line the anchor names, its lines read as
/// data and joined as one text, standing as
/// the text pins their reading of a comment.
fn doc_block_above(source: &str, anchor: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.contains(anchor))
        .expect("the anchor stands in the source");
    let mut block: Vec<&str> = Vec::new();
    let mut i = at;
    while i > 0 {
        let line = lines[i - 1];
        if line.trim_start().starts_with("///") {
            block.push(line.trim_start().trim_start_matches('/').trim());
            i -= 1;
        } else if line.trim_start().starts_with("#[") && block.is_empty() {
            i -= 1;
        } else {
            break;
        }
    }
    block.reverse();
    block.join(" ")
}

#[test]
fn test_the_doc_comment_over_the_window_word_helper_states_the_lossy_reading_and_the_timestamp_reading_as_the_decider_and_names_the_separator_seat_and_the_fraction_tail_seat_and_carries_no_falsified_sentence(
) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main_text = std::fs::read_to_string(root.join("src/main.rs")).expect("the shell reads");
    let block = doc_block_above(&main_text, "fn word_text");
    assert!(
        block.contains("lossy conversion")
            && block.contains("each invalid sequence standing as the replacement character"),
        "the helper comment states the lossy reading whole: {block}"
    );
    assert!(
        block.contains("the timestamp reading then decides the word whole"),
        "the helper comment names the timestamp reading as the decider of the word whole: {block}"
    );
    assert!(
        block.contains("separator seat") && block.contains("fraction tail"),
        "the helper comment names the separator seat and the fraction tail seat as accepting seats: {block}"
    );
    assert!(
        !block.contains("no timestamp that grammar accepts can contain"),
        "the falsified sentence of the landed comment stands absent from the helper comment whole: {block}"
    );
    assert!(
        !block.contains("cannot contain the replacement character")
            && !block.contains("can never contain")
            && !block.contains("no accepted timestamp can contain")
            && !block.contains("refuses as no timestamp whole"),
        "no spelling of the falsified claim family stands in the helper comment whole: {block}"
    );
    for sentence in block.split(". ") {
        if let Some(at) = sentence.find("contain") {
            let before = &sentence[..at];
            assert!(
                !before.contains("cannot")
                    && !before.contains("can not")
                    && !before.contains("never")
                    && !before.contains(" not ")
                    && !before.contains(" no ")
                    && !before.starts_with("no "),
                "no negated form of the falsified claim stands ahead of the containment word in its sentence: {sentence}"
            );
        }
    }
}

#[test]
fn test_the_doc_sentence_over_the_validation_error_type_names_the_three_disagreement_seats_and_the_crossed_rebuild_refusal_carrying_its_message_whole(
) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let errors_text =
        std::fs::read_to_string(root.join("src/errors.rs")).expect("the error module reads");
    let block = doc_block_above(&errors_text, "pub struct ReportValidationError");
    assert!(
        block.contains("supplied report")
            && block.contains("disagrees with the rebuilt examination"),
        "the type sentence names the report disagreement at its seat: {block}"
    );
    assert!(
        block.contains("markdown report"),
        "the type sentence names the markdown disagreement at its seat: {block}"
    );
    assert!(
        block.contains("evidence document"),
        "the type sentence names the evidence disagreement at its seat: {block}"
    );
    assert!(
        block.contains("crosses into this type") && block.contains("carrying its message whole"),
        "the type sentence names the crossed rebuild refusal carrying its message whole: {block}"
    );
    assert!(
        !block.contains("Raised when a report disagrees with a rebuilt examination"),
        "the narrow sentence of the landed comment stands absent as the type its whole raising: {block}"
    );
    assert!(
        !block.contains("Raised when a report disagrees")
            && !block.contains("Raised where a report disagrees")
            && !block.contains("Raised at the report disagreement"),
        "no narrow spelling of the landed sentence stands in the type sentence whole: {block}"
    );
    assert!(
        block.starts_with("Raised at the validation seats"),
        "the type sentence opens on the whole life at the validation seats: {block}"
    );
}

#[test]
fn test_a_window_word_carrying_a_byte_that_is_not_utf_8_text_at_the_separator_seat_reads_as_a_timestamp_at_the_start_word_and_at_the_end_word(
) {
    let pin = pin_dir("hookdoctor-gl-separator");
    let _guard = ScratchDir(pin.clone());
    let start_cell = cell_dir(&pin, "start");
    let output = run_report(
        &start_cell,
        &[("--window-start", byte_word(b"2025-12-31\xff00:00:00Z"))],
    );
    assert!(
        output.status.success(),
        "the separator seat run at the start word succeeds: {}",
        stderr_text(&output)
    );
    let bound = window_bound(&start_cell, "start").expect("the start bound stands spelled");
    assert!(
        bound.contains('\u{FFFD}'),
        "the spelled start bound carries the replacement character at the separator seat: {bound}"
    );
    let end_cell = cell_dir(&pin, "end");
    let output = run_report(
        &end_cell,
        &[("--window-end", byte_word(b"2027-01-01\xff00:00:00Z"))],
    );
    assert!(
        output.status.success(),
        "the separator seat run at the end word succeeds: {}",
        stderr_text(&output)
    );
    let bound = window_bound(&end_cell, "end").expect("the end bound stands spelled");
    assert!(
        bound.contains('\u{FFFD}'),
        "the spelled end bound carries the replacement character at the separator seat: {bound}"
    );
    let truncated_cell = cell_dir(&pin, "truncated");
    let output = run_report(
        &truncated_cell,
        &[("--window-start", byte_word(b"2025-12-31\xe2\x8200:00:00Z"))],
    );
    assert!(
        output.status.success(),
        "the truncated sequence run at the separator seat succeeds: {}",
        stderr_text(&output)
    );
    let bound = window_bound(&truncated_cell, "start").expect("the start bound stands spelled");
    assert_eq!(
        bound.matches('\u{FFFD}').count(),
        1,
        "the spelled bound carries the replacement character exactly once for the one truncated sequence: {bound}"
    );
    assert_eq!(
        bound, "2025-12-31\u{FFFD}00:00:00Z",
        "the spelled bound stands exact at the separator seat: {bound}"
    );
}

#[test]
fn test_a_window_word_carrying_a_byte_that_is_not_utf_8_text_in_the_fraction_tail_reads_as_a_timestamp_with_an_offset_following_and_with_a_trailing_zed_and_with_the_replacement_character_itself_standing_as_text_at_that_seat(
) {
    let pin = pin_dir("hookdoctor-gl-fraction");
    let _guard = ScratchDir(pin.clone());
    let offset_cell = cell_dir(&pin, "offset");
    let output = run_report(
        &offset_cell,
        &[(
            "--window-start",
            byte_word(b"2025-12-31T00:00:00.123456\xff+01:00"),
        )],
    );
    assert!(
        output.status.success(),
        "the fraction tail run with an offset following succeeds: {}",
        stderr_text(&output)
    );
    let bound = window_bound(&offset_cell, "start").expect("the start bound stands spelled");
    assert!(
        bound.contains('\u{FFFD}'),
        "the spelled bound carries the replacement character in the fraction tail: {bound}"
    );
    let zed_cell = cell_dir(&pin, "zed");
    let output = run_report(
        &zed_cell,
        &[(
            "--window-end",
            byte_word(b"2027-01-01T00:00:00.123456\xffZ"),
        )],
    );
    assert!(
        output.status.success(),
        "the fraction tail run with a trailing zed succeeds: {}",
        stderr_text(&output)
    );
    let bound = window_bound(&zed_cell, "end").expect("the end bound stands spelled");
    assert!(
        bound.contains('\u{FFFD}'),
        "the spelled end bound carries the replacement character in the fraction tail: {bound}"
    );
    let text_cell = cell_dir(&pin, "text");
    let output = run_report(
        &text_cell,
        &[(
            "--window-start",
            text_word("2025-12-31T00:00:00.123456\u{FFFD}+01:00"),
        )],
    );
    assert!(
        output.status.success(),
        "the fraction tail run with the replacement character itself as text succeeds: {}",
        stderr_text(&output)
    );
    let bound = window_bound(&text_cell, "start").expect("the start bound stands spelled");
    assert!(
        bound.contains('\u{FFFD}'),
        "the spelled bound carries the replacement character standing as text in the fraction tail: {bound}"
    );
    let truncated_cell = cell_dir(&pin, "truncated");
    let output = run_report(
        &truncated_cell,
        &[(
            "--window-start",
            byte_word(b"2025-12-31T00:00:00.123456\xe2\x82+01:00"),
        )],
    );
    assert!(
        output.status.success(),
        "the truncated sequence run in the fraction tail with an offset following succeeds: {}",
        stderr_text(&output)
    );
    let bound = window_bound(&truncated_cell, "start").expect("the start bound stands spelled");
    assert_eq!(
        bound.matches('\u{FFFD}').count(),
        1,
        "the spelled bound carries the replacement character exactly once for the one truncated sequence in the fraction tail: {bound}"
    );
}

#[test]
fn test_the_validation_error_stands_raised_at_the_disagreement_seat_of_the_report_validation_and_of_the_markdown_validation_and_of_the_evidence_validation_each_under_the_wording_of_its_own_reading(
) {
    let hooks = [stop_hook()];
    let report =
        build_report(&hooks, &[], "cfg", &sources(), None, None).expect("the report builds");
    let tampered_report = tamper(&report, "format", JsonValue::text("hookdoctor-report/2"));
    let error: ReportValidationError =
        validate_report(&tampered_report, &hooks, &[], "cfg", &sources(), None, None)
            .expect_err("the report disagreement refuses");
    assert_eq!(error.message(), "the report disagrees with the examination");

    let rendered = render_markdown(&report);
    let mut supplied = rendered.into_bytes();
    supplied[0] = b'X';
    let error: ReportValidationError =
        validate_markdown(&supplied, &hooks, &[], "cfg", &sources(), None, None)
            .expect_err("the markdown disagreement refuses");
    assert_eq!(
        error.message(),
        "the markdown report disagrees with the examination"
    );

    let evidence = build_evidence(&hooks, &[], None, None).expect("the evidence builds");
    let tampered_evidence = tamper(
        &evidence,
        "format",
        JsonValue::text("hookdoctor-evidence/2"),
    );
    let error: ReportValidationError =
        validate_evidence(&tampered_evidence, &hooks, &[], None, None)
            .expect_err("the evidence disagreement refuses");
    assert_eq!(
        error.message(),
        "the evidence document disagrees with the examination"
    );
}

#[test]
fn test_the_validation_error_stands_raised_at_the_mapping_seat_of_each_validation_carrying_the_window_wording_across_whole_and_at_the_report_validation_carrying_the_matcher_wording_across_whole(
) {
    let hooks = [stop_hook()];
    let report =
        build_report(&hooks, &[], "cfg", &sources(), None, None).expect("the report builds");
    let error: ReportValidationError = validate_report(
        &report,
        &hooks,
        &[],
        "cfg",
        &sources(),
        Some("not a timestamp"),
        None,
    )
    .expect_err("the crossed window refusal refuses at the report validation");
    assert_eq!(error.message(), "the window start is not a timestamp");

    let rendered = render_markdown(&report);
    let error: ReportValidationError = validate_markdown(
        rendered.as_bytes(),
        &hooks,
        &[],
        "cfg",
        &sources(),
        Some("not a timestamp"),
        None,
    )
    .expect_err("the crossed window refusal refuses at the markdown validation");
    assert_eq!(error.message(), "the window start is not a timestamp");

    let evidence = build_evidence(&hooks, &[], None, None).expect("the evidence builds");
    let error: ReportValidationError =
        validate_evidence(&evidence, &hooks, &[], Some("not a timestamp"), None)
            .expect_err("the crossed window refusal refuses at the evidence validation");
    assert_eq!(error.message(), "the window start is not a timestamp");

    let bad_hook = hook("PreToolUse", Some("["), "guard.sh");
    let supplied = JsonValue::object(vec![]);
    let error: ReportValidationError = validate_report(
        &supplied,
        &[bad_hook],
        &[tool_call(1)],
        "cfg",
        &sources(),
        None,
        None,
    )
    .expect_err("the crossed matcher refusal refuses at the report validation");
    assert_eq!(error.message(), "a matcher is not a valid pattern");

    let rendered = render_markdown(&report);
    let bad_hook = hook("PreToolUse", Some("["), "guard.sh");
    let error: ReportValidationError = validate_markdown(
        rendered.as_bytes(),
        &[bad_hook],
        &[tool_call(1)],
        "cfg",
        &sources(),
        None,
        None,
    )
    .expect_err("the crossed matcher refusal refuses at the markdown validation");
    assert_eq!(error.message(), "a matcher is not a valid pattern");

    let bad_hook = hook("PreToolUse", Some("["), "guard.sh");
    let error: ReportValidationError = validate_evidence(
        &JsonValue::object(vec![]),
        &[bad_hook],
        &[tool_call(1)],
        None,
        None,
    )
    .expect_err("the crossed matcher refusal refuses at the evidence validation");
    assert_eq!(error.message(), "a matcher is not a valid pattern");

    let error: ReportValidationError = validate_report(
        &report,
        &hooks,
        &[],
        "cfg",
        &sources(),
        None,
        Some("not a timestamp"),
    )
    .expect_err("the crossed window end refusal refuses at the report validation");
    assert_eq!(error.message(), "the window end is not a timestamp");
}

#[test]
fn test_where_a_configuration_label_and_a_session_label_both_carry_a_byte_that_is_not_utf_8_text_the_door_refuses_under_the_configuration_wording_and_each_label_alone_in_its_fault_refuses_in_its_own_wording_and_the_label_count_refusal_stands_ahead_of_both_resolutions(
) {
    let pin = pin_dir("hookdoctor-gl-rider");
    let _guard = ScratchDir(pin.clone());
    let both_cell = cell_dir(&pin, "both");
    let output = run_report(
        &both_cell,
        &[
            ("--config-label", byte_word(b"lab\xffel")),
            ("--session-label", byte_word(b"lab\xffel")),
        ],
    );
    assert_eq!(
        refusal_line(&output),
        "hookdoctor error: the configuration label is not UTF-8 text\n"
    );
    assert!(
        !both_cell.join("report.json").exists(),
        "the door writes nothing where both labels stand bad"
    );
    let session_cell = cell_dir(&pin, "session");
    let output = run_report(
        &session_cell,
        &[("--session-label", byte_word(b"lab\xffel"))],
    );
    assert_eq!(
        refusal_line(&output),
        "hookdoctor error: a session label is not UTF-8 text\n"
    );
    let config_cell = cell_dir(&pin, "config");
    let output = run_report(&config_cell, &[("--config-label", byte_word(b"lab\xffel"))]);
    assert_eq!(
        refusal_line(&output),
        "hookdoctor error: the configuration label is not UTF-8 text\n"
    );
    let count_cell = cell_dir(&pin, "count");
    let output = run_report(
        &count_cell,
        &[
            ("--config-label", byte_word(b"lab\xffel")),
            ("--session-label", text_word("one")),
            ("--session-label", text_word("two")),
        ],
    );
    assert_eq!(
        refusal_line(&output),
        "hookdoctor error: session labels do not match the session files\n"
    );
}
