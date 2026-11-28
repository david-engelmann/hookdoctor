//! The pins of slice 59, the decoder
//! failure precedence standings, over
//! session texts the suite writes as
//! literals and hands to the session
//! reading directly and over documents
//! handed to the decoder itself and over
//! one full call of the shipped binary.
//! R2-1 states the duplicate refusal and
//! GR states the grammar and IN-6 states
//! the no type wording and RD-8 states
//! the session line pair of wordings, and
//! this slice states the order the two
//! failures of the parse part in and
//! their order against the no type
//! reading, at DP-1 to DP-5. Every pin
//! stands as a fence on the landed code,
//! this slice moving no production code.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::ingest::parse_session_text;
use hookdoctor::jsonvalue::{parse_json, ParseFailure};

const INVALID_WORDING: &str = "a session line is not valid JSON";
const NO_TYPE_WORDING: &str = "a session line carries no type";

fn dup_wording(key: &str) -> String {
    format!("a session line carries the key {key} more than once")
}

fn refusal(text: &str) -> String {
    match parse_session_text(text, 0) {
        Ok(records) => panic!("expected a refusal, got {} records", records.len()),
        Err(error) => error.message().to_string(),
    }
}

#[test]
fn test_a_duplicate_carried_by_an_object_that_closes_before_the_parse_reaches_a_token_that_does_not_read_names_its_key_at_the_session_seat_and_at_the_decoder_itself(
) {
    let line = r#"{"type":"user","type":"user"} trailing"#;
    assert_eq!(refusal(line), dup_wording("type"));

    let line = r#"{"type":"user","message":{"a":1,"a":2},"x":@}"#;
    assert_eq!(refusal(line), dup_wording("a"));

    let line = r#"{"type":"user","message":{"content":[{"b":1,"b":2},@]}}"#;
    assert_eq!(refusal(line), dup_wording("b"));

    match parse_json(r#"{"a":1,"a":2} @"#) {
        Err(ParseFailure::DuplicateKey(key)) => assert_eq!(key, "a"),
        other => panic!("expected the duplicate failure, got {other:?}"),
    }
}

#[test]
fn test_a_token_that_does_not_read_standing_before_a_duplicated_object_closes_refuses_the_line_as_not_valid_json_at_the_session_seat_and_at_the_decoder_itself(
) {
    let line = r#"{"type":"user","type":"user","x":@}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    let line = r#"{"type":"user","type":@}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    let line = r#"{"type":@,"a":1,"a":2}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    let line = r#"{"type":"user","message":{"a":1,"a":2,"z":@}}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    match parse_json(r#"{"a":1,"a":@}"#) {
        Err(ParseFailure::Invalid) => {}
        other => panic!("expected the invalid failure, got {other:?}"),
    }
}

#[test]
fn test_where_one_line_carries_duplicates_in_more_than_one_object_or_more_than_once_in_one_object_the_first_object_to_close_names_its_first_repeated_key_in_written_order(
) {
    let line = r#"{"type":"user","message":{"content":[{"a":1,"a":2},{"b":1,"b":2}]}}"#;
    assert_eq!(refusal(line), dup_wording("a"));

    let line = r#"{"type":"user","a":1,"o":{"b":1,"b":2},"a":3}"#;
    assert_eq!(refusal(line), dup_wording("b"));

    let line = r#"{"type":"user","type":"user","m":1,"m":2}"#;
    assert_eq!(refusal(line), dup_wording("type"));
}

#[test]
fn test_both_failures_of_the_parse_stand_ahead_of_the_no_type_reading_and_the_no_type_wording_stands_only_for_a_line_that_decodes_whole(
) {
    let line = r#"{"a":1,"a":2}"#;
    assert_eq!(refusal(line), dup_wording("a"));

    let line = r#"{"a":@}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    let line = r#"{"a":1}"#;
    assert_eq!(refusal(line), NO_TYPE_WORDING);

    let line = r#"[1,2]"#;
    assert_eq!(refusal(line), NO_TYPE_WORDING);
}

fn deep_line(duplicate_first: bool) -> String {
    let opens = "[".repeat(4100);
    let closes = "]".repeat(4100);
    if duplicate_first {
        format!(r#"{{"type":"user","message":{{"content":[{{"a":1,"a":2}},{opens}{closes}]}}}}"#)
    } else {
        format!(r#"{{"type":"user","message":{{"content":[{opens}{closes},{{"a":1,"a":2}}]}}}}"#)
    }
}

fn refusal_on_large_stack(text: String) -> String {
    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || refusal(&text))
        .expect("thread stands");
    handle.join().expect("thread completes")
}

#[test]
fn test_a_lone_surrogate_refuses_as_not_valid_json_and_a_repeat_closing_before_a_lone_surrogate_and_before_the_nesting_bound_names_its_key_and_the_bound_reached_first_refuses_as_not_valid_json(
) {
    let line = r#"{"type":"user","s":"\uD800"}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    let line = r#"{"type":"user","o":{"a":1,"a":2},"s":"\uD800"}"#;
    assert_eq!(refusal(line), dup_wording("a"));

    // The descent to the bound overflows a plain test thread stack in
    // this harness, so the deep lines run on a spawned thread with an
    // explicit large stack, in the form the ninth pin of slice 40
    // carries.
    assert_eq!(refusal_on_large_stack(deep_line(true)), dup_wording("a"));
    assert_eq!(refusal_on_large_stack(deep_line(false)), INVALID_WORDING);
}

/// The settings document the door cells stand on,
/// one Stop hook standing configured over the
/// session the suite writes.
const SETTINGS: &str =
    "{\"hooks\": {\"Stop\": [{\"hooks\": [{\"type\": \"command\", \"command\": \"wrap.sh\"}]}]}}";

/// A scratch directory that removes itself when it
/// leaves scope, so a panicking assertion leaves no
/// state behind and concurrent test processes never
/// share a path.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn hold(path: &Path) -> ScratchDir {
        ScratchDir(path.to_path_buf())
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The standing of one full call of the door over a
/// cell directory the suite prepares itself.
struct DoorRun {
    _dir: ScratchDir,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn door_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("hookdoctor-dp-standings-{}", std::process::id()))
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale cell directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a cell directory stands");
    dir
}

fn run_call(dir: &Path, args: Vec<OsString>) -> DoorRun {
    let run = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .args(&args)
        .output()
        .expect("the shipped binary runs");
    DoorRun {
        _dir: ScratchDir::hold(dir),
        code: run.status.code(),
        stdout: run.stdout,
        stderr: run.stderr,
    }
}

fn word(text: &str) -> OsString {
    OsString::from(text)
}

fn refusal_line(run: &DoorRun) -> String {
    assert_eq!(run.code, Some(2));
    assert!(run.stdout.is_empty());
    String::from_utf8(run.stderr.clone()).expect("the refusal line is text")
}

#[test]
fn test_at_the_door_a_line_whose_inner_object_closes_its_repeat_before_a_later_invalid_token_refuses_under_the_duplicate_wording_and_a_line_whose_invalid_token_stands_ahead_of_its_repeat_refuses_under_the_invalid_wording(
) {
    let dir = door_dir("inner-repeat-later-token");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(
        &session,
        "{\"type\":\"user\",\"message\":{\"a\":1,\"a\":2},\"x\":@}\n",
    )
    .expect("the session writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            settings.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
        ],
    );
    assert_eq!(
        refusal_line(&run),
        "hookdoctor error: a session line carries the key a more than once\n"
    );

    let dir = door_dir("token-ahead-of-repeat");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"type\":@,\"a\":1,\"a\":2}\n").expect("the session writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            settings.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
        ],
    );
    assert_eq!(
        refusal_line(&run),
        "hookdoctor error: a session line is not valid JSON\n"
    );
}
