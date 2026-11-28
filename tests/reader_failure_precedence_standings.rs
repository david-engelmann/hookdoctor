//! The pins of slice 60, the reader door
//! failure precedence standings, over
//! documents the suite writes as literals
//! and hands to the document reading
//! directly and over documents handed to
//! the decoder itself and over files the
//! suite writes itself at the file door
//! and over one full call of the shipped
//! binary. R2-1 states the duplicate
//! refusal and GR states the grammar and
//! RD-2 and RD-3 state the reader door
//! wordings and DP states the precedence
//! at the session line seat alone, and
//! this slice states the order the two
//! failures of the parse part in at the
//! document reading and at the file door
//! behind it, at RP-1 to RP-5. Every pin
//! stands as a fence on the landed code,
//! this slice moving no production code.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::jsonvalue::{parse_json, ParseFailure};
use hookdoctor::reader::{read_json_document, read_json_file};

fn dup(label: &str, key: &str) -> String {
    format!("the {label} carries the key {key} more than once")
}

fn invalid(label: &str) -> String {
    format!("the {label} is not valid JSON")
}

fn refusal(text: &str, label: &str) -> String {
    match read_json_document(text, label) {
        Ok(_) => panic!("expected a refusal"),
        Err(error) => error.message().to_string(),
    }
}

fn file_refusal(path: &Path, label: &str) -> String {
    match read_json_file(path, label) {
        Ok(_) => panic!("expected a refusal"),
        Err(error) => error.message().to_string(),
    }
}

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

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("hookdoctor-rp-standings-{}", std::process::id()))
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale cell directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a cell directory stands");
    dir
}

#[test]
fn test_a_duplicate_carried_by_an_object_that_closes_before_the_parse_reaches_a_token_that_does_not_read_names_its_key_at_the_document_reading_and_at_the_decoder_itself(
) {
    // A duplicate carried by an object that closes before trailing
    // content that does not read.
    assert_eq!(
        refusal("{\"a\":1,\"a\":2} xyz", "configuration"),
        dup("configuration", "a")
    );

    // A duplicated top object with content standing past the value.
    assert_eq!(
        refusal("{\"a\":1,\"a\":2} {\"b\":", "configuration"),
        dup("configuration", "a")
    );

    // An inner object closing its repeat ahead of a later token.
    assert_eq!(
        refusal("{\"o\":{\"b\":1,\"b\":2},\"z\":@}", "configuration"),
        dup("configuration", "b")
    );

    // A duplicate inside a list element ahead of a later token.
    assert_eq!(
        refusal("[{\"c\":1,\"c\":2},@]", "configuration"),
        dup("configuration", "c")
    );

    // The repeated key written under an escape names the decoded
    // character in the wording.
    assert_eq!(
        refusal(r#"{"a":1,"\u0061":2} xyz"#, "configuration"),
        dup("configuration", "a")
    );

    // The same parting under the evidence document label.
    assert_eq!(
        refusal("{\"o\":{\"b\":1,\"b\":2},\"z\":@}", "evidence document"),
        dup("evidence document", "b")
    );

    // A bare token standing past the whole value behind a closed
    // repeat.
    assert_eq!(
        refusal("{\"a\":1,\"a\":2} @", "configuration"),
        dup("configuration", "a")
    );

    // A lone surrogate string standing past the whole value
    // behind a closed repeat.
    assert_eq!(
        refusal(r#"{"a":1,"a":2} "\uD800""#, "configuration"),
        dup("configuration", "a")
    );

    // The repeated key written under an escape standing first,
    // the plain spelling behind it, the repeat closing ahead of
    // trailing content.
    assert_eq!(
        refusal(r#"{"\u0061":1,"a":2} xyz"#, "configuration"),
        dup("configuration", "a")
    );

    // Both spellings of the repeated key written under escapes.
    assert_eq!(
        refusal(r#"{"\u0061":1,"\u0061":2}"#, "configuration"),
        dup("configuration", "a")
    );

    // The same parting under the session transcript label.
    assert_eq!(
        refusal("{\"a\":1,\"a\":2} xyz", "session transcript"),
        dup("session transcript", "a")
    );

    // The same parting under a caller label no standing text
    // names.
    assert_eq!(
        refusal("{\"a\":1,\"a\":2} xyz", "intake packet"),
        dup("intake packet", "a")
    );

    match parse_json("{\"a\":1,\"a\":2} xyz") {
        Err(ParseFailure::DuplicateKey(key)) => assert_eq!(key, "a"),
        other => panic!("expected the duplicate failure, got {other:?}"),
    }

    // The escaped shape at the decoder itself, the duplicate
    // failure carrying the decoded key.
    match parse_json(r#"{"\u0061":1,"a":2} xyz"#) {
        Err(ParseFailure::DuplicateKey(key)) => assert_eq!(key, "a"),
        other => panic!("expected the duplicate failure, got {other:?}"),
    }
}

#[test]
fn test_a_token_that_does_not_read_standing_before_a_duplicated_object_closes_refuses_the_document_as_not_valid_json_at_the_document_reading_and_at_the_decoder_itself(
) {
    // A token that does not read standing as the value of the
    // repeated key inside the carrying object.
    assert_eq!(
        refusal("{\"a\":1,\"a\":@}", "configuration"),
        invalid("configuration")
    );

    // A token standing ahead of the repeat.
    assert_eq!(
        refusal("{\"a\":@,\"a\":2}", "configuration"),
        invalid("configuration")
    );

    // A token inside the inner carrying object.
    assert_eq!(
        refusal("{\"o\":{\"b\":1,\"b\":@},\"b\":2}", "configuration"),
        invalid("configuration")
    );

    // A lone surrogate standing inside the carrying object ahead of
    // its close.
    assert_eq!(
        refusal(r#"{"a":1,"a":2,"s":"\uD800"}"#, "configuration"),
        invalid("configuration")
    );

    // A token inside the object standing as the first value of
    // the key that repeats behind it.
    assert_eq!(
        refusal("{\"a\":{\"x\":@},\"a\":2}", "configuration"),
        invalid("configuration")
    );

    // A token inside the object standing as the repeated key its
    // own value.
    assert_eq!(
        refusal("{\"a\":1,\"a\":{\"x\":@}}", "configuration"),
        invalid("configuration")
    );

    // A token inside the list standing as the repeated key its
    // own value.
    assert_eq!(
        refusal("{\"a\":1,\"a\":[1,@]}", "configuration"),
        invalid("configuration")
    );

    // A literal that does not read standing as the repeated key
    // its own value.
    assert_eq!(
        refusal("{\"a\":1,\"a\":nul}", "configuration"),
        invalid("configuration")
    );

    match parse_json("{\"a\":1,\"a\":@}") {
        Err(ParseFailure::Invalid) => {}
        other => panic!("expected the invalid failure, got {other:?}"),
    }

    // The token inside the repeated key its own value at the
    // decoder itself.
    match parse_json("{\"a\":1,\"a\":{\"x\":@}}") {
        Err(ParseFailure::Invalid) => {}
        other => panic!("expected the invalid failure, got {other:?}"),
    }
}

#[test]
fn test_where_one_document_carries_duplicates_in_more_than_one_object_or_more_than_once_in_one_object_the_first_object_to_close_names_its_first_repeated_key_in_written_order(
) {
    // The first object to close names its key.
    assert_eq!(
        refusal("{\"x\":{\"b\":1,\"b\":2},\"a\":1,\"a\":2}", "report"),
        dup("report", "b")
    );

    // The inner repeat is named ahead of the outer repeat written
    // before it.
    assert_eq!(
        refusal("{\"a\":1,\"o\":{\"b\":1,\"b\":2},\"a\":3}", "report"),
        dup("report", "b")
    );

    // The first key standing twice in written order inside one
    // object.
    assert_eq!(
        refusal("{\"a\":1,\"b\":1,\"b\":2,\"a\":2}", "report"),
        dup("report", "b")
    );
}

#[test]
fn test_at_the_file_door_the_closed_first_shape_names_its_key_and_the_token_first_shape_and_the_bytes_that_do_not_decode_refuse_as_not_valid_json_and_the_path_that_does_not_stand_refuses_as_could_not_be_read(
) {
    let dir = scratch_dir("file-door");
    let _hold = ScratchDir::hold(&dir);

    let closed = dir.join("closed.json");
    std::fs::write(&closed, "{\"a\":1,\"a\":2} xyz").expect("the file writes");
    assert_eq!(
        file_refusal(&closed, "evidence document"),
        dup("evidence document", "a")
    );

    let token = dir.join("token.json");
    std::fs::write(&token, r#"{"a":@,"\u0061":2}"#).expect("the file writes");
    assert_eq!(
        file_refusal(&token, "evidence document"),
        invalid("evidence document")
    );

    let raw = dir.join("raw.json");
    std::fs::write(&raw, [0xff, 0xfe]).expect("the file writes");
    assert_eq!(
        file_refusal(&raw, "evidence document"),
        invalid("evidence document")
    );

    let missing = dir.join("missing.json");
    assert_eq!(
        file_refusal(&missing, "evidence document"),
        "the evidence document could not be read"
    );

    // Bytes that do not decode standing behind a document that
    // would name its key. The decode seat fires ahead of the
    // parse whole.
    let mut bytes = b"{\"a\":1,\"a\":2}".to_vec();
    bytes.push(0xff);
    let behind = dir.join("behind.json");
    std::fs::write(&behind, &bytes).expect("the file writes");
    assert_eq!(
        file_refusal(&behind, "evidence document"),
        invalid("evidence document")
    );

    // A directory standing at the path refuses as could not be
    // read.
    assert_eq!(
        file_refusal(&dir, "evidence document"),
        "the evidence document could not be read"
    );

    // The token first shape through the file door in its plain
    // form.
    let plain = dir.join("plain.json");
    std::fs::write(&plain, "{\"a\":1,\"a\":@}").expect("the file writes");
    assert_eq!(
        file_refusal(&plain, "evidence document"),
        invalid("evidence document")
    );

    // An empty file decodes whole as empty text and the parse
    // refuses it under the invalid wording.
    let empty = dir.join("empty.json");
    std::fs::write(&empty, "").expect("the file writes");
    assert_eq!(
        file_refusal(&empty, "evidence document"),
        invalid("evidence document")
    );

    // The path that does not stand under the session transcript
    // label.
    assert_eq!(
        file_refusal(&missing, "session transcript"),
        "the session transcript could not be read"
    );
}

fn deep_document(lists: usize, inner: &str) -> String {
    let opens = "[".repeat(lists);
    let closes = "]".repeat(lists);
    format!("{opens}{inner}{closes}")
}

fn refusal_on_large_stack(text: String, label: &'static str) -> String {
    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || refusal(&text, label))
        .expect("thread stands");
    handle.join().expect("thread completes")
}

fn decodes_on_large_stack(text: String) -> bool {
    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || read_json_document(&text, "configuration").is_ok())
        .expect("thread stands");
    handle.join().expect("thread completes")
}

#[test]
fn test_a_lone_surrogate_refuses_as_not_valid_json_and_a_repeat_closing_before_a_lone_surrogate_and_before_the_nesting_bound_names_its_key_and_the_bound_reached_first_refuses_as_not_valid_json_at_the_document_reading(
) {
    // A lone surrogate standing alone in a document.
    assert_eq!(
        refusal(r#"{"a":1,"s":"\uD800"}"#, "configuration"),
        invalid("configuration")
    );

    // A repeat closed in an inner object ahead of a lone surrogate
    // standing in the outer object.
    assert_eq!(
        refusal(r#"{"o":{"a":1,"a":2},"s":"\uD800"}"#, "configuration"),
        dup("configuration", "a")
    );

    // A lone low surrogate standing alone in a document.
    assert_eq!(
        refusal(r#"{"s":"\uDC00"}"#, "configuration"),
        invalid("configuration")
    );

    // A lone surrogate standing as the repeated key its own
    // value.
    assert_eq!(
        refusal(r#"{"a":1,"a":"\uD800"}"#, "configuration"),
        invalid("configuration")
    );

    // A repeat closing inside a list element ahead of a lone
    // surrogate standing as a later element.
    assert_eq!(
        refusal(r#"[{"a":1,"a":2},"\uD800"]"#, "configuration"),
        dup("configuration", "a")
    );

    // A lone surrogate standing in an inner object ahead of the
    // outer object its repeat closing.
    assert_eq!(
        refusal(r#"{"o":{"s":"\uD800"},"a":1,"a":2}"#, "configuration"),
        invalid("configuration")
    );

    // The descent to the bound overflows a plain test thread stack in
    // this harness, so the deep documents run on a spawned thread
    // with an explicit large stack, in the form the pins of slices
    // 40 and 59 carry.
    assert_eq!(
        refusal_on_large_stack(deep_document(3999, "{\"a\":1,\"a\":2}"), "configuration"),
        dup("configuration", "a")
    );

    let past = format!("{}1", "[".repeat(4001));
    assert_eq!(
        refusal_on_large_stack(past, "configuration"),
        invalid("configuration")
    );

    // The duplicated object entered at the bound itself in
    // lists stands refused by the bound ahead of its check.
    assert_eq!(
        refusal_on_large_stack(deep_document(4000, "{\"a\":1,\"a\":2}"), "configuration"),
        invalid("configuration")
    );

    // The bound at its edge through object nesting. The
    // innermost object standing at depth 3999 names its key.
    let doc = format!(
        "{}{{\"a\":1,\"a\":2}}{}",
        "{\"k\":".repeat(3999),
        "}".repeat(3999)
    );
    assert_eq!(
        refusal_on_large_stack(doc, "configuration"),
        dup("configuration", "a")
    );

    // The innermost object standing at depth 4000 stands
    // refused by the bound ahead of its check.
    let doc = format!(
        "{}{{\"a\":1,\"a\":2}}{}",
        "{\"k\":".repeat(4000),
        "}".repeat(4000)
    );
    assert_eq!(
        refusal_on_large_stack(doc, "configuration"),
        invalid("configuration")
    );

    // A document nested exactly at the bound with no duplicate
    // decodes whole, the bound refusing only past itself.
    let at_bound = format!("{}1{}", "[".repeat(4000), "]".repeat(4000));
    assert!(decodes_on_large_stack(at_bound));
}

/// The standing of one full call of the door over a
/// cell directory the suite prepares itself.
struct DoorRun {
    _dir: ScratchDir,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
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
fn test_at_the_door_a_settings_document_whose_object_closes_its_repeat_before_a_later_invalid_token_refuses_under_the_duplicate_wording_and_a_settings_document_whose_invalid_token_stands_ahead_of_its_repeat_refuses_under_the_invalid_wording(
) {
    let dir = scratch_dir("closed-first-settings");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, "{\"a\":1,\"a\":2} xyz").expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"type\":\"user\"}\n").expect("the session writes");
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
        "hookdoctor error: the configuration carries the key a more than once\n"
    );

    let dir = scratch_dir("token-first-settings");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, r#"{"a":@,"\u0061":2}"#).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"type\":\"user\"}\n").expect("the session writes");
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
        "hookdoctor error: the configuration is not valid JSON\n"
    );

    let dir = scratch_dir("check-evidence-settings");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, "{}").expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"type\":\"user\"}\n").expect("the session writes");
    let evidence = dir.join("evidence.json");
    std::fs::write(&evidence, "{\"a\":1,\"a\":2} xyz").expect("the evidence document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            settings.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-evidence"),
            evidence.into_os_string(),
        ],
    );
    assert_eq!(
        refusal_line(&run),
        "hookdoctor error: the evidence document carries the key a more than once\n"
    );

    let dir = scratch_dir("check-report-settings");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, "{}").expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{\"type\":\"user\"}\n").expect("the session writes");
    let supplied = dir.join("report.json");
    std::fs::write(&supplied, r#"{"a":@,"\u0061":2}"#).expect("the report document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            settings.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-report"),
            supplied.into_os_string(),
        ],
    );
    assert_eq!(
        refusal_line(&run),
        "hookdoctor error: the report is not valid JSON\n"
    );
}
