//! The pins of slice 52, the window word
//! standings, over the shipped binary the pins
//! drive with window words handed as byte
//! strings at the seats the obligations name.
//! PORT2-3 states the door and its window words
//! reading through the timestamp reading and
//! EX-8 states the bound wordings, and the pins
//! below state the byte reading of the window
//! words, the lossy reading whole and its
//! separator standing and its strict seats and
//! the spelling the bound carries into the
//! outputs and the parting of the words of the
//! door. Every pin stands as a fence on the
//! landed code, this slice moving no production
//! code.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use hookdoctor::reader::read_json_document;

/// The synthetic configuration the pins drive
/// the binary over, one command hook standing
/// under the Bash matcher.
const SETTINGS_TEXT: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

/// The synthetic session the pins drive the
/// binary over, one assistant tool call standing
/// inside the window words the pins hand it.
const SESSION_TEXT: &str = r#"{"type": "assistant", "timestamp": "2026-01-01T00:00:00Z", "sessionId": "s1", "message": {"content": [{"type": "tool_use", "name": "Bash", "id": "c1"}]}}"#;

/// A scratch directory that removes itself when
/// it leaves scope, so a panicking assertion
/// leaves no state behind.
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

fn run_check(dir: &Path, flag: &str, target: &Path, word: &OsString) -> Output {
    Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .arg("--config")
        .arg(dir.join("settings.json"))
        .arg("--session")
        .arg(dir.join("session.jsonl"))
        .arg(flag)
        .arg(target)
        .arg("--window-start")
        .arg(word)
        .output()
        .expect("the binary runs")
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

fn markdown_window_line(dir: &Path) -> String {
    let text = std::fs::read_to_string(dir.join("report.md")).expect("the markdown stands");
    text.lines()
        .find(|line| line.starts_with("Window "))
        .expect("the markdown carries its window line")
        .to_string()
}

fn assert_refused(output: &Output, dir: &Path, wording: &str) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "the word refuses at exit code 2"
    );
    let stderr = String::from_utf8(output.stderr.clone()).expect("standard error is text");
    assert_eq!(stderr, format!("hookdoctor error: {wording}\n"));
    assert!(
        !dir.join("report.json").exists(),
        "the refusal writes no report"
    );
    assert!(
        !dir.join("report.md").exists(),
        "the refusal writes no markdown"
    );
}

/// The separator standing of WW-3. A window word
/// carrying a raw byte at the separator seat of
/// the timestamp grammar reads as a timestamp,
/// the byte standing as the replacement character
/// at that seat, at the start word and at the end
/// word alike, and a truncated two byte sequence
/// standing alone at that seat reads the same way.
#[cfg(unix)]
#[test]
fn test_a_window_word_carrying_a_raw_byte_at_the_separator_seat_reads_and_spells_the_replacement_character_there(
) {
    let pin = pin_dir("hookdoctor-window-word-separator");
    let _scratch = ScratchDir(pin.clone());

    let start_cell = cell_dir(&pin, "start-byte");
    let output = run_report(
        &start_cell,
        &[("--window-start", byte_word(b"2025-12-31\xff00:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat of the start word reads"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&start_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00Z"),
        "the bound stands spelled with the replacement character at that seat"
    );
    assert_eq!(
        markdown_window_line(&start_cell),
        "Window start 2025-12-31\u{FFFD}00:00:00Z end None"
    );

    let end_cell = cell_dir(&pin, "end-byte");
    let output = run_report(
        &end_cell,
        &[
            ("--window-start", text_word("2025-01-01T00:00:00Z")),
            ("--window-end", byte_word(b"2027-01-01\xff00:00:00Z")),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat of the end word reads"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&end_cell, "end").as_deref(),
        Some("2027-01-01\u{FFFD}00:00:00Z"),
        "the end bound stands spelled with the replacement character at that seat"
    );
    assert_eq!(
        window_bound(&end_cell, "start").as_deref(),
        Some("2025-01-01T00:00:00Z"),
        "the start bound stands as handed"
    );

    let truncated_cell = cell_dir(&pin, "truncated");
    let output = run_report(
        &truncated_cell,
        &[("--window-start", byte_word(b"2025-12-31\xc300:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a truncated sequence at the separator seat reads as the one character"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&truncated_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00Z"),
        "the one sequence stands as the one replacement character"
    );

    let trunc3_start_cell = cell_dir(&pin, "trunc3-start");
    let output = run_report(
        &trunc3_start_cell,
        &[("--window-start", byte_word(b"2025-12-31\xe2\x8200:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a truncated three byte sequence at the separator seat reads as the one character"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&trunc3_start_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00Z"),
        "the truncated sequence stands as the one replacement character"
    );

    let trunc4_start_cell = cell_dir(&pin, "trunc4-start");
    let output = run_report(
        &trunc4_start_cell,
        &[(
            "--window-start",
            byte_word(b"2025-12-31\xf0\x9f\x9800:00:00Z"),
        )],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a truncated four byte sequence at the separator seat reads as the one character"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&trunc4_start_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00Z"),
        "the truncated sequence stands as the one replacement character"
    );

    let lead3_start_cell = cell_dir(&pin, "lead3-start");
    let output = run_report(
        &lead3_start_cell,
        &[("--window-start", byte_word(b"2025-12-31\xe200:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a lone three byte lead at the separator seat reads as the one character"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&lead3_start_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00Z"),
        "the lone lead stands as the one replacement character"
    );

    let trunc3_end_cell = cell_dir(&pin, "trunc3-end");
    let output = run_report(
        &trunc3_end_cell,
        &[
            ("--window-start", text_word("2025-01-01T00:00:00Z")),
            ("--window-end", byte_word(b"2027-01-01\xe2\x8200:00:00Z")),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a truncated sequence at the separator seat of the end word reads as the one character"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&trunc3_end_cell, "end").as_deref(),
        Some("2027-01-01\u{FFFD}00:00:00Z"),
        "the end bound stands spelled with the replacement character at that seat"
    );

    let week_ext_cell = cell_dir(&pin, "week-ext");
    let output = run_report(
        &week_ext_cell,
        &[("--window-start", byte_word(b"2026-W01-1\xff00:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat of the week extended form reads"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&week_ext_cell, "start").as_deref(),
        Some("2026-W01-1\u{FFFD}00:00:00Z"),
        "the bound stands spelled with the replacement character at that seat"
    );

    let week_basic_cell = cell_dir(&pin, "week-basic");
    let output = run_report(
        &week_basic_cell,
        &[("--window-start", byte_word(b"2026W011\xff00:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat of the week basic form reads"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&week_basic_cell, "start").as_deref(),
        Some("2026W011\u{FFFD}00:00:00Z"),
        "the bound stands spelled with the replacement character at that seat"
    );

    let week_bare_cell = cell_dir(&pin, "week-bare");
    let output = run_report(
        &week_bare_cell,
        &[("--window-start", byte_word(b"2026W01\xff00:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat of the bare week form reads"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&week_bare_cell, "start").as_deref(),
        Some("2026W01\u{FFFD}00:00:00Z"),
        "the bound stands spelled with the replacement character at that seat"
    );

    let cal_basic_cell = cell_dir(&pin, "cal-basic");
    let output = run_report(
        &cal_basic_cell,
        &[("--window-start", byte_word(b"20251231\xff000000Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat of the calendar basic form reads"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&cal_basic_cell, "start").as_deref(),
        Some("20251231\u{FFFD}000000Z"),
        "the bound stands spelled with the replacement character at that seat"
    );

    let offset_tail_cell = cell_dir(&pin, "offset-tail");
    let output = run_report(
        &offset_tail_cell,
        &[("--window-start", byte_word(b"2025-12-31\xff00:00:00+01:00"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat reads with an offset tail standing"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&offset_tail_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00+01:00"),
        "the bound stands spelled with the replacement character at that seat"
    );

    let lowered_zed_cell = cell_dir(&pin, "lowered-zed");
    let output = run_report(
        &lowered_zed_cell,
        &[("--window-start", byte_word(b"2025-12-31\xff00:00:00z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "a raw byte at the separator seat reads with a lowered zed tail standing"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&lowered_zed_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00z"),
        "the bound stands spelled with the replacement character at that seat"
    );

    let both_words_cell = cell_dir(&pin, "both-words");
    let output = run_report(
        &both_words_cell,
        &[
            ("--window-start", byte_word(b"2025-12-31\xff00:00:00Z")),
            ("--window-end", byte_word(b"2027-01-01\xff00:00:00Z")),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "both window words carrying a raw byte at the separator seat read together"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&both_words_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00Z"),
        "the start bound stands spelled with the replacement character at that seat"
    );
    assert_eq!(
        window_bound(&both_words_cell, "end").as_deref(),
        Some("2027-01-01\u{FFFD}00:00:00Z"),
        "the end bound stands spelled with the replacement character at that seat"
    );
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The character itself under WW-2 and WW-4. A
/// window word carrying the replacement character
/// as UTF-8 text at the separator seat stands
/// accepted with the word standing whole in the
/// report, and the same character standing at a
/// digit seat reads as no timestamp and refuses
/// in the wording of the bound.
#[test]
fn test_a_window_word_carrying_the_replacement_character_itself_stands_whole_in_the_report_and_refuses_at_a_digit_seat(
) {
    let pin = pin_dir("hookdoctor-window-word-character");
    let _scratch = ScratchDir(pin.clone());

    let whole_cell = cell_dir(&pin, "whole");
    let output = run_report(
        &whole_cell,
        &[("--window-start", text_word("2025-12-31\u{FFFD}00:00:00Z"))],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "the character itself at the separator seat reads"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    assert_eq!(
        window_bound(&whole_cell, "start").as_deref(),
        Some("2025-12-31\u{FFFD}00:00:00Z"),
        "the word stands whole in the report"
    );

    let digit_cell = cell_dir(&pin, "digit");
    let output = run_report(
        &digit_cell,
        &[("--window-start", text_word("2025-12-3\u{FFFD}T00:00:00Z"))],
    );
    assert_refused(&output, &digit_cell, "the window start is not a timestamp");
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The strict seats of WW-4. A raw byte standing
/// in a window word at a seat the grammar reads
/// strictly refuses in the wording of its bound
/// and writes nothing, at a digit seat and at the
/// tail and standing twice at the separator seat
/// and at the stray byte seat ahead of an offset.
#[cfg(unix)]
#[test]
fn test_a_window_word_carrying_a_raw_byte_at_a_strict_seat_refuses_in_the_wording_of_its_bound_and_writes_nothing(
) {
    let pin = pin_dir("hookdoctor-window-word-strict");
    let _scratch = ScratchDir(pin.clone());

    let digit_cell = cell_dir(&pin, "digit");
    let output = run_report(
        &digit_cell,
        &[("--window-start", byte_word(b"2025-12-3\xffT00:00:00Z"))],
    );
    assert_refused(&output, &digit_cell, "the window start is not a timestamp");

    let tail_cell = cell_dir(&pin, "tail");
    let output = run_report(
        &tail_cell,
        &[
            ("--window-start", text_word("2025-01-01T00:00:00Z")),
            ("--window-end", byte_word(b"2027-01-01T00:00:00\xff")),
        ],
    );
    assert_refused(&output, &tail_cell, "the window end is not a timestamp");

    let pair_cell = cell_dir(&pin, "pair");
    let output = run_report(
        &pair_cell,
        &[("--window-start", byte_word(b"2025-12-31\xff\xfe00:00:00Z"))],
    );
    assert_refused(&output, &pair_cell, "the window start is not a timestamp");

    let stray_cell = cell_dir(&pin, "stray");
    let output = run_report(
        &stray_cell,
        &[(
            "--window-start",
            byte_word(b"2025-12-31T00:00:00\xff+01:00"),
        )],
    );
    assert_refused(&output, &stray_cell, "the window start is not a timestamp");

    let end_digit_cell = cell_dir(&pin, "end-digit");
    let output = run_report(
        &end_digit_cell,
        &[
            ("--window-start", text_word("2025-01-01T00:00:00Z")),
            ("--window-end", byte_word(b"2027-01-0\xffT00:00:00Z")),
        ],
    );
    assert_refused(
        &output,
        &end_digit_cell,
        "the window end is not a timestamp",
    );
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The check modes under WW-5. The documents of a
/// run standing under a start word carrying a raw
/// byte at its separator seat stand validated by
/// the evidence check and the report check and
/// the markdown check under the same word.
#[cfg(unix)]
#[test]
fn test_the_three_check_modes_validate_their_documents_under_a_window_word_carrying_a_raw_byte_at_the_separator_seat(
) {
    let pin = pin_dir("hookdoctor-window-word-checks");
    let _scratch = ScratchDir(pin.clone());
    let cell = cell_dir(&pin, "documents");
    let word = byte_word(b"2025-12-31\xff00:00:00Z");
    let evidence_path = cell.join("evidence.json");
    let output = run_report(
        &cell,
        &[
            ("--window-start", word.clone()),
            ("--evidence", OsString::from(&evidence_path)),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "the run under the raw byte word builds its documents"
    );
    assert!(evidence_path.exists(), "the evidence document stands");
    for (flag, target) in [
        ("--check-evidence", evidence_path.clone()),
        ("--check-report", cell.join("report.json")),
        ("--check-markdown", cell.join("report.md")),
    ] {
        let check = run_check(&cell, flag, &target, &word);
        assert_eq!(
            check.status.code(),
            Some(0),
            "the check mode validates its document under the same word"
        );
        assert!(check.stderr.is_empty(), "the validation stands in silence");
    }
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The unchanged passage of WW-2. Window words
/// standing whole as UTF-8 text pass to the
/// timestamp reading unchanged and stand in the
/// report byte for byte as handed, in the letter
/// separator form and the offset spelling and the
/// lowered zed and the week date form.
#[test]
fn test_window_words_standing_whole_as_utf8_text_in_their_standing_forms_pass_into_the_report_byte_for_byte(
) {
    let pin = pin_dir("hookdoctor-window-word-whole");
    let _scratch = ScratchDir(pin.clone());
    for (cell, word) in [
        ("letter", "2025-12-31T00:00:00Z"),
        ("offset", "2025-12-31T00:00:00+01:00"),
        ("lowered", "2025-12-31T00:00:00z"),
        ("week", "2026-W01-1T00:00:00Z"),
    ] {
        let dir = cell_dir(&pin, cell);
        let output = run_report(&dir, &[("--window-start", text_word(word))]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "a whole word in its standing form reads"
        );
        assert!(output.stderr.is_empty(), "the reading stands in silence");
        assert_eq!(
            window_bound(&dir, "start").as_deref(),
            Some(word),
            "the word stands in the report as handed"
        );
        let report = std::fs::read_to_string(dir.join("report.json")).expect("the report stands");
        assert!(
            report.contains(word),
            "the report carries the word byte for byte"
        );
    }
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The parting of WW-6. No other word of the door
/// takes the lossy reading, a session label word
/// carrying a raw byte refusing in its own wording
/// while a configuration file whose name carries
/// a raw byte stands read whole as a path word.
#[cfg(unix)]
#[test]
fn test_a_session_label_word_carrying_a_raw_byte_refuses_in_its_own_wording_and_a_configuration_file_named_with_a_raw_byte_reads_whole(
) {
    use std::os::unix::ffi::OsStrExt;

    let pin = pin_dir("hookdoctor-window-word-parting");
    let _scratch = ScratchDir(pin.clone());

    let label_cell = cell_dir(&pin, "label");
    let output = run_report(&label_cell, &[("--session-label", byte_word(b"lab\xffel"))]);
    assert_refused(&output, &label_cell, "a session label is not UTF-8 text");

    let name_cell = cell_dir(&pin, "name");
    let mut named_settings = name_cell.as_os_str().as_bytes().to_vec();
    named_settings.extend_from_slice(b"/set\xfftings.json");
    let named_settings_path = PathBuf::from(OsString::from_vec(named_settings));
    std::fs::write(&named_settings_path, SETTINGS_TEXT)
        .expect("the configuration stands under its byte name");
    let output = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .arg("--config")
        .arg(&named_settings_path)
        .arg("--session")
        .arg(name_cell.join("session.jsonl"))
        .arg("--config-label")
        .arg("given-config")
        .arg("--session-label")
        .arg("given-session")
        .arg("--output")
        .arg(name_cell.join("report.json"))
        .arg("--report")
        .arg(name_cell.join("report.md"))
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(0),
        "a configuration named with a raw byte reads whole"
    );
    assert!(output.stderr.is_empty(), "the reading stands in silence");
    let report = std::fs::read_to_string(name_cell.join("report.json")).expect("the report stands");
    let document = read_json_document(&report, "report").expect("the report parses");
    assert_eq!(
        document
            .get("config_source")
            .and_then(|value| value.as_text()),
        Some("given-config"),
        "the given configuration label stands in the report"
    );
    assert!(
        report.contains("\"given-session\""),
        "the given session label stands in the report"
    );
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}
