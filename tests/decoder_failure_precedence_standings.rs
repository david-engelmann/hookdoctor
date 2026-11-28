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
use hookdoctor::jsonvalue::{parse_json, parse_json_lenient, JsonValue, ParseFailure};

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

fn escape_spelling(hex: &str) -> String {
    // The six characters of one unicode escape spelling of a key,
    // built from their pieces so the spelling stands in the line
    // exactly as a document would carry it.
    let mut out = String::new();
    out.push(char::from_u32(0x5c).expect("a backslash stands"));
    out.push('u');
    out.push_str(hex);
    out
}

fn nested_document(lists: usize) -> String {
    let opens = "[".repeat(lists);
    let closes = "]".repeat(lists);
    format!("{opens}{{\"a\":1,\"a\":2}}{closes}")
}

fn decoder_failure_on_large_stack(text: String) -> ParseFailure {
    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || match parse_json(&text) {
            Err(failure) => failure,
            Ok(_) => panic!("expected a failure, got a value"),
        })
        .expect("thread stands");
    handle.join().expect("thread completes")
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

    // A repeat spelled with an escape names the decoded key, the
    // identity of a repeated key standing in its decoded spelling.
    let line = format!(
        "{{\"type\":\"user\",\"{}ype\":\"user\"}}",
        escape_spelling("0074")
    );
    assert_eq!(refusal(&line), dup_wording("type"));

    let line = format!(
        "{{\"o\":{{\"a\":1,\"{}\":2}},\"x\":@}}",
        escape_spelling("0061")
    );
    assert_eq!(refusal(&line), dup_wording("a"));

    // The token stands inside a following sibling object, behind
    // the element whose repeat closes first.
    let line = r#"{"type":"user","message":{"content":[{"b":1,"b":2},{"c":@}]}}"#;
    assert_eq!(refusal(line), dup_wording("b"));

    match parse_json(r#"{"a":1,"a":2} @"#) {
        Err(ParseFailure::DuplicateKey(key)) => assert_eq!(key, "a"),
        other => panic!("expected the duplicate failure, got {other:?}"),
    }

    // Two further trailing shapes at the decoder, a duplicate
    // inside a list standing as the whole value with content
    // behind it and a duplicate in the value its top object with
    // a second value behind it.
    match parse_json(r#"[{"a":1,"a":2}] x"#) {
        Err(ParseFailure::DuplicateKey(key)) => assert_eq!(key, "a"),
        other => panic!("expected the duplicate failure, got {other:?}"),
    }
    match parse_json(r#"{"a":1,"a":2} {"b":1,"b":2}"#) {
        Err(ParseFailure::DuplicateKey(key)) => assert_eq!(key, "a"),
        other => panic!("expected the duplicate failure, got {other:?}"),
    }

    // The sampling route of the decoder raises no duplicate
    // failure at any standing, keeping the last value standing,
    // and its grammar stands whole.
    match parse_json_lenient(r#"{"a":1,"a":2}"#) {
        Ok(value) => assert_eq!(value.get("a"), Some(&JsonValue::Int(2))),
        Err(failure) => panic!("expected a value, got {failure:?}"),
    }
    match parse_json_lenient(r#"{"a":1,"a":2} @"#) {
        Err(ParseFailure::Invalid) => {}
        other => panic!("expected the invalid failure, got {other:?}"),
    }
    match parse_json_lenient(r#"{"a":1,"a":@}"#) {
        Err(ParseFailure::Invalid) => {}
        other => panic!("expected the invalid failure, got {other:?}"),
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

    // The token stands as the first value of the key that repeats
    // behind it.
    let line = r#"{"a":@,"a":2}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    // The token stands inside the object that stands as the
    // repeated key its value, and that object never closes.
    let line = r#"{"a":{"x":@},"a":2}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    // The token stands in a key position after the repeat stands
    // written whole, and the carrying object never closes.
    let line = r#"{"type":"user","type":"user",@}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    // A number token that does not read stands behind the repeat,
    // and the carrying object never closes.
    let line = r#"{"a":1,"a":2,"n":01}"#;
    assert_eq!(refusal(line), INVALID_WORDING);

    // The token stands in a list element ahead of the element that
    // carries the repeat, and the list its parse meets it first.
    let line = r#"{"type":"user","message":{"content":[@,{"b":1,"b":2}]}}"#;
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

    // The control of the decoded spelling seat. Two keys whose
    // decoded spellings part stand as two keys, and the line
    // decodes whole and carries no type.
    let line = format!("{{\"a\":1,\"{}\":2}}", escape_spelling("0062"));
    assert_eq!(refusal(&line), NO_TYPE_WORDING);
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

    // A lone low surrogate partakes in the same order, alone as
    // not valid JSON and behind a closed repeat under the
    // duplicate wording, its spelling built from its pieces in
    // the form of the escape spelling helper.
    let low_surrogate = escape_spelling("DC00");
    let line = format!("{{\"type\":\"user\",\"s\":\"{low_surrogate}\"}}");
    assert_eq!(refusal(&line), INVALID_WORDING);

    let line = format!("{{\"type\":\"user\",\"o\":{{\"a\":1,\"a\":2}},\"s\":\"{low_surrogate}\"}}");
    assert_eq!(refusal(&line), dup_wording("a"));

    // The descent to the bound overflows a plain test thread stack in
    // this harness, so the deep lines run on a spawned thread with an
    // explicit large stack, in the form the ninth pin of slice 40
    // carries.
    assert_eq!(refusal_on_large_stack(deep_line(true)), dup_wording("a"));
    assert_eq!(refusal_on_large_stack(deep_line(false)), INVALID_WORDING);

    // The bound at its exact edge, at the decoder. The innermost
    // list of 3999 lists stands entered at depth 3998 and its
    // item, the duplicated object, stands entered at depth 3999,
    // below the bound of 4000, so the object parses and its
    // check fires at its close. With 4000 lists the duplicated
    // object would stand entered at depth 4000, where the descent
    // stands refused, so the bound fires ahead of any duplicate
    // the object carries.
    assert_eq!(
        decoder_failure_on_large_stack(nested_document(3999)),
        ParseFailure::DuplicateKey("a".to_string())
    );
    assert_eq!(
        decoder_failure_on_large_stack(nested_document(4000)),
        ParseFailure::Invalid
    );
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

    // Both failures stand ahead of the no type reading at the
    // door itself, with the decoded control landing under it.
    let dir = door_dir("duplicate-beside-no-type");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"a\":1,\"a\":2}\n").expect("the session writes");
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

    let dir = door_dir("invalid-beside-no-type");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"a\":@}\n").expect("the session writes");
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

    let dir = door_dir("no-type-control");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"a\":1}\n").expect("the session writes");
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
        "hookdoctor error: a session line carries no type\n"
    );
}
