//! The pins of slice 53, the scan point
//! precedence standings, over the shipped
//! binary the pins drive at the seats the
//! obligations name. PORT2-2 names the usage
//! refusal classes of the door and AR states
//! the help seats and the separator and the
//! value seat reading, and the pins below
//! state where in the walk of the scan each
//! refusal fires and where the help answers.
//! Every pin stands as a fence on the landed
//! code, this slice moving no production
//! code.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The usage line of the shell, standing as
/// the one refusal line of the scan.
const USAGE: &str = "usage: hookdoctor --config PATH --session PATH [--session PATH ...] --output PATH --report PATH [--config-label TEXT] [--session-label TEXT ...] [--window-start TIMESTAMP] [--window-end TIMESTAMP] [--evidence PATH] [--check-evidence PATH] [--check-report PATH] [--check-markdown PATH]";

/// The synthetic configuration the door cells
/// drive the binary over, one command hook
/// standing under the Bash matcher.
const SETTINGS_TEXT: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

/// The synthetic session the door cells drive
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

fn word(text: &str) -> OsString {
    OsString::from(text)
}

fn invoke(args: &[OsString]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .args(args)
        .output()
        .expect("the binary runs")
}

fn words(texts: &[&str]) -> Vec<OsString> {
    texts.iter().map(|text| word(text)).collect()
}

fn full_args(dir: &Path, extra: &[&str]) -> Vec<OsString> {
    let mut args = vec![
        word("--config"),
        dir.join("settings.json").into_os_string(),
        word("--session"),
        dir.join("session.jsonl").into_os_string(),
        word("--output"),
        dir.join("report.json").into_os_string(),
        word("--report"),
        dir.join("report.md").into_os_string(),
    ];
    args.extend(words(extra));
    args
}

fn assert_usage_refusal(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "the call refuses at exit code 2"
    );
    assert!(
        output.stdout.is_empty(),
        "the refusal writes no standard output"
    );
    assert_eq!(
        String::from_utf8(output.stderr.clone()).expect("standard error is text"),
        format!("{USAGE}\n"),
        "the refusal stands as the usage line alone"
    );
}

fn assert_help_answered(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "the help answers at exit code 0"
    );
    assert!(output.stderr.is_empty(), "the answer stands in silence");
    assert_eq!(
        String::from_utf8(output.stdout.clone()).expect("standard output is text"),
        format!("{USAGE}\n\nexamine configured hooks against session execution traces\n"),
        "the answer stands as the help text whole"
    );
}

fn assert_no_outputs(dir: &Path) {
    assert!(
        !dir.join("report.json").exists(),
        "the call writes no report output"
    );
    assert!(
        !dir.join("report.md").exists(),
        "the call writes no markdown output"
    );
}

/// The ambiguous point of PP-3. A shortened
/// long flag whose prefix names more than
/// one flag refuses at the point it stands,
/// ahead of a help flag behind it, in each
/// shortened spelling and in the inline
/// value form and with the short help
/// spelling standing behind it.
#[test]
fn test_an_ambiguous_shortened_flag_refuses_at_its_point_ahead_of_a_help_flag() {
    let pin = pin_dir("hookdoctor-scan-point-ambiguous");
    let _scratch = ScratchDir(pin.clone());

    for (cell, args) in [
        ("con-long-help", vec!["--con", "--help"]),
        ("sess-long-help", vec!["--sess", "--help"]),
        ("check-long-help", vec!["--check-", "--help"]),
        ("window-long-help", vec!["--w", "--help"]),
        ("con-short-help", vec!["--con", "-h"]),
        ("con-inline-help", vec!["--con=x", "--help"]),
    ] {
        let _cell = cell;
        let output = invoke(&words(&args));
        assert_usage_refusal(&output);
    }
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The valueless point of PP-4. A flag
/// standing without its value refuses at
/// the point it stands, ahead of a help
/// flag behind the word that leaves it
/// valueless and ahead of the end of the
/// walk where it stands last.
#[test]
fn test_a_flag_standing_without_its_value_refuses_at_its_point_ahead_of_a_help_flag() {
    let pin = pin_dir("hookdoctor-scan-point-valueless");
    let _scratch = ScratchDir(pin.clone());

    for args in [
        vec!["--config", "--bogus", "--help"],
        vec!["--config", "--session", "--help"],
        vec!["--config", "--con", "--help"],
        vec!["--config", "--", "--help"],
        vec!["--out", "--help"],
    ] {
        let output = invoke(&words(&args));
        assert_usage_refusal(&output);
    }

    let second_cell = cell_dir(&pin, "second-standing");
    let mut second_args = vec![
        word("--config"),
        second_cell.join("settings.json").into_os_string(),
        word("--config"),
        word("--help"),
    ];
    let output = invoke(&second_args);
    assert_usage_refusal(&output);
    second_args.clear();

    let last_cell = cell_dir(&pin, "standing-last");
    let output = invoke(&full_args(&last_cell, &["--window-start"]));
    assert_usage_refusal(&output);
    assert_no_outputs(&last_cell);
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The deferred boundary of PP-5. A word
/// the scan sets aside refuses only at the
/// end of the walk, so a point refusal
/// standing behind one still fires at its
/// own point and a help standing behind
/// one alone still answers, and an unknown
/// flag takes no value from the walk.
#[test]
fn test_words_set_aside_by_the_scan_refuse_only_at_the_end_of_the_walk() {
    let pin = pin_dir("hookdoctor-scan-point-deferred");
    let _scratch = ScratchDir(pin.clone());

    for args in [
        vec!["--bogus", "--con", "--help"],
        vec!["word", "--con", "--help"],
        vec!["-xh", "--con", "--help"],
    ] {
        let output = invoke(&words(&args));
        assert_usage_refusal(&output);
    }

    for args in [
        vec!["--bogus", "--help"],
        vec!["--bogus=x", "--help"],
        vec!["-xh", "--help"],
        vec!["--bogus", "-h"],
    ] {
        let output = invoke(&words(&args));
        assert_help_answered(&output);
    }
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The help point of PP-2 and PP-6. The
/// help flag answers at the point it
/// stands, ahead of a point refusal
/// standing behind it, in its long and
/// short and cluster and shortened long
/// spellings.
#[test]
fn test_the_help_flag_answers_at_its_point_ahead_of_the_point_refusals() {
    let pin = pin_dir("hookdoctor-scan-point-help");
    let _scratch = ScratchDir(pin.clone());

    for args in [
        vec!["--help", "--con"],
        vec!["-h", "--con"],
        vec!["-hx", "--con=x"],
        vec!["--he", "--sess"],
        vec!["--help", "--config"],
    ] {
        let output = invoke(&words(&args));
        assert_help_answered(&output);
    }
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The door axis of PP-6. Over full calls
/// on real synthetic files the clean call
/// writes both outputs, a point refusal at
/// the head or at the end writes neither,
/// a deferred word at the end writes
/// neither, and a help at the end answers
/// with neither written.
#[test]
fn test_the_point_and_deferred_refusals_write_neither_output_at_the_door() {
    let pin = pin_dir("hookdoctor-scan-point-door");
    let _scratch = ScratchDir(pin.clone());

    let clean_cell = cell_dir(&pin, "clean");
    let output = invoke(&full_args(&clean_cell, &[]));
    assert_eq!(
        output.status.code(),
        Some(0),
        "the clean call stands at exit code 0"
    );
    assert!(output.stdout.is_empty(), "the clean call stands in silence");
    assert!(output.stderr.is_empty(), "the clean call stands in silence");
    assert!(
        clean_cell.join("report.json").exists(),
        "the report stands written"
    );
    assert!(
        clean_cell.join("report.md").exists(),
        "the markdown stands written"
    );

    let end_cell = cell_dir(&pin, "point-at-end");
    let output = invoke(&full_args(&end_cell, &["--con"]));
    assert_usage_refusal(&output);
    assert_no_outputs(&end_cell);

    let head_cell = cell_dir(&pin, "point-at-head");
    let mut head_args = vec![word("--con")];
    head_args.extend(full_args(&head_cell, &[]));
    let output = invoke(&head_args);
    assert_usage_refusal(&output);
    assert_no_outputs(&head_cell);

    let deferred_cell = cell_dir(&pin, "deferred-at-end");
    let output = invoke(&full_args(&deferred_cell, &["--bogus"]));
    assert_usage_refusal(&output);
    assert_no_outputs(&deferred_cell);

    let help_cell = cell_dir(&pin, "help-at-end");
    let output = invoke(&full_args(&help_cell, &["--help"]));
    assert_help_answered(&output);
    assert_no_outputs(&help_cell);
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}

/// The stated controls of PP-2 and PP-4
/// and PP-5 at their boundary. The help
/// flag carrying an equals value refuses
/// at its point in each spelling, the
/// separator and the help stand in their
/// stated order, and the value seat takes
/// a negative number token and a lone dash
/// as values where the help itself leaves
/// a flag without its value.
#[test]
fn test_the_stated_controls_of_the_scan_stand_at_their_points_and_value_seats() {
    let pin = pin_dir("hookdoctor-scan-point-controls");
    let _scratch = ScratchDir(pin.clone());

    for args in [
        vec!["-h=x", "--help"],
        vec!["--help=x", "--help"],
        vec!["--h=x", "-h"],
        vec!["--", "-h"],
    ] {
        let output = invoke(&words(&args));
        assert_usage_refusal(&output);
    }

    let output = invoke(&words(&["-h", "--"]));
    assert_help_answered(&output);

    for (cell, value) in [("negative", "-5"), ("lone-dash", "-")] {
        let dir = cell_dir(&pin, cell);
        let args = vec![
            word("--config"),
            word(value),
            word("--session"),
            dir.join("session.jsonl").into_os_string(),
            word("--output"),
            dir.join("report.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
        ];
        let output = invoke(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "the value reading reaches the tool refusal"
        );
        assert!(
            output.stdout.is_empty(),
            "the refusal writes no standard output"
        );
        assert_eq!(
            String::from_utf8(output.stderr.clone()).expect("standard error is text"),
            "hookdoctor error: the configuration could not be read\n",
            "the refusal stands as the configuration reading wording"
        );
        assert_no_outputs(&dir);
    }

    let output = invoke(&words(&["--config", "--help"]));
    assert_usage_refusal(&output);
    std::fs::remove_dir_all(&pin).expect("a pin directory clears");
}
