//! The pins of slice 48, the reader door wording
//! standings, over documents and files the pins
//! hand to the public reader functions directly
//! and over session texts the seventh pin hands
//! to the public session reading. PORT-2 states
//! the reader its grammar and the shape of the
//! duplicate message alone, and GR-11 defers the
//! wording of the decoder its failures to the
//! doors above it, and PORT2-3 names the reader
//! wording by its shape and states no wording,
//! and the pins below state the wordings whole,
//! the three texts as functions of the label and
//! of the named key and the seat map of the
//! module and the label population and the
//! silence of the sampling reading and the
//! session line pair at the ingest seat. Every
//! pin stands as a fence on the landed code,
//! this slice moving no production code.

use std::collections::BTreeSet;
use std::path::PathBuf;

use hookdoctor::ingest::parse_session_text;
use hookdoctor::jsonvalue::parse_json;
use hookdoctor::reader::{parse_output_text, read_file_bytes, read_json_document, read_json_file};

fn message_of(text: &str, label: &str) -> String {
    match read_json_document(text, label) {
        Ok(_) => panic!("the document read where a refusal stood expected"),
        Err(error) => error.message().to_string(),
    }
}

fn fixture_dir(tag: &str) -> PathBuf {
    let dir = std::env::current_dir().unwrap().join(format!(
        "reader_wording_fixtures_{}_{}",
        tag,
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_the_invalid_wording_stands_at_the_document_door_for_every_refused_shape_and_label() {
    let shapes = [
        "",
        "  ",
        "{",
        "[1,]",
        "tru",
        "true false",
        "\"\\ud800\"",
        "{\"a\":1",
        "01",
        "{\"a\" 1}",
    ];
    for shape in shapes {
        assert_eq!(
            message_of(shape, "configuration"),
            "the configuration is not valid JSON",
            "shape {shape:?}"
        );
    }
    assert_eq!(
        message_of("{", "evidence document"),
        "the evidence document is not valid JSON"
    );
    assert_eq!(message_of("{", ""), "the  is not valid JSON");
    // The document at the bound reads whole and the document one
    // past it refuses in the same words, on a large stack thread.
    let deep_ok = format!("{}{}", "[".repeat(4000), "]".repeat(4000));
    let deep_past = format!("{}{}", "[".repeat(4001), "]".repeat(4001));
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            assert!(read_json_document(&deep_ok, "configuration").is_ok());
            message_of(&deep_past, "configuration")
        })
        .unwrap();
    assert_eq!(
        handle.join().unwrap(),
        "the configuration is not valid JSON"
    );
}

#[test]
fn test_the_duplicate_wording_names_the_key_the_failure_names_in_its_decoded_spelling() {
    assert_eq!(
        message_of("{\"a\":1,\"a\":2}", "configuration"),
        "the configuration carries the key a more than once"
    );
    // The first key standing twice in written order is named.
    assert_eq!(
        message_of("{\"b\":1,\"a\":2,\"b\":3,\"a\":4}", "configuration"),
        "the configuration carries the key b more than once"
    );
    // The named key is the key whose repeat stands first, not the
    // first key of the object.
    assert_eq!(
        message_of("{\"a\":1,\"b\":2,\"b\":3,\"a\":4}", "configuration"),
        "the configuration carries the key b more than once"
    );
    // An inner object is checked before the object carrying it.
    assert_eq!(
        message_of("{\"z\":1,\"o\":{\"k\":1,\"k\":2},\"z\":3}", "configuration"),
        "the configuration carries the key k more than once"
    );
    // A duplicate inside a list element names its key.
    assert_eq!(
        message_of("[{\"q\":1,\"q\":2}]", "report"),
        "the report carries the key q more than once"
    );
    // The named key stands in its decoded spelling.
    assert_eq!(
        message_of("{\"a\":1,\"\\u0061\":2}", "configuration"),
        "the configuration carries the key a more than once"
    );
    assert_eq!(
        message_of("{\"a\\\"b\":1,\"a\\\"b\":2}", "configuration"),
        "the configuration carries the key a\"b more than once"
    );
    // The same key in different objects is no repeat.
    assert!(read_json_document("{\"a\":{\"a\":1},\"b\":{\"a\":2}}", "configuration").is_ok());
}

#[test]
fn test_the_could_not_be_read_wording_stands_at_both_file_readings() {
    let missing = PathBuf::from("reader-wording-absent-seat/nope.json");
    let error = read_json_file(&missing, "report").unwrap_err();
    assert_eq!(error.message(), "the report could not be read");
    let error = read_json_file(&missing, "configuration").unwrap_err();
    assert_eq!(error.message(), "the configuration could not be read");
    let error = read_file_bytes(&missing, "session file").unwrap_err();
    assert_eq!(error.message(), "the session file could not be read");
    let error = read_file_bytes(&missing, "markdown report").unwrap_err();
    assert_eq!(error.message(), "the markdown report could not be read");
    // A directory standing as the path refuses the same way.
    let error = read_json_file(std::path::Path::new("."), "configuration").unwrap_err();
    assert_eq!(error.message(), "the configuration could not be read");
    let error = read_file_bytes(std::path::Path::new("."), "session file").unwrap_err();
    assert_eq!(error.message(), "the session file could not be read");
    // The byte reading passes every standing of its bytes whole.
    let dir = fixture_dir("bytes");
    let seat = dir.join("bytes.bin");
    std::fs::write(&seat, [0x7b_u8, 0xff, 0x7d]).unwrap();
    assert_eq!(std::fs::read(&seat).unwrap(), [0x7b_u8, 0xff, 0x7d]);
    assert_eq!(
        read_file_bytes(&seat, "session file").unwrap(),
        [0x7b_u8, 0xff, 0x7d]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_the_file_door_words_its_decode_seat_and_its_parse_seat_with_the_one_invalid_wording() {
    let dir = fixture_dir("door");
    let invalid = dir.join("invalid.json");
    let dup = dir.join("dup.json");
    let notutf8 = dir.join("notutf8.json");
    std::fs::write(&invalid, "{no").unwrap();
    std::fs::write(&dup, "{\"a\":1,\"a\":2}").unwrap();
    std::fs::write(&notutf8, [0x7b_u8, 0xff, 0x7d]).unwrap();
    // The fixtures stand as written before any reader call.
    assert_eq!(std::fs::read(&invalid).unwrap(), b"{no");
    assert_eq!(std::fs::read(&dup).unwrap(), b"{\"a\":1,\"a\":2}");
    assert_eq!(std::fs::read(&notutf8).unwrap(), [0x7b_u8, 0xff, 0x7d]);
    // Bytes that do not decode word as the parse refusal words.
    let error = read_json_file(&notutf8, "configuration").unwrap_err();
    assert_eq!(error.message(), "the configuration is not valid JSON");
    // Text that does not parse words alike, at the parse seat.
    let error = read_json_file(&invalid, "configuration").unwrap_err();
    assert_eq!(error.message(), "the configuration is not valid JSON");
    // A duplicate through the file door words as at the document door.
    let error = read_json_file(&dup, "evidence document").unwrap_err();
    assert_eq!(
        error.message(),
        "the evidence document carries the key a more than once"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_the_sampling_reading_raises_no_refusal_and_carries_no_wording_at_any_standing() {
    assert_eq!(parse_output_text(""), None);
    assert_eq!(parse_output_text("not json"), None);
    assert_eq!(parse_output_text("[1,2]"), None);
    assert_eq!(parse_output_text("\"text\""), None);
    assert_eq!(parse_output_text("42"), None);
    // A repeated key inside an object leaves its last value standing.
    let sampled = parse_output_text("{\"a\":1,\"a\":2}").expect("the object samples");
    assert_eq!(sampled.get("a"), Some(&parse_json("2").unwrap()));
    let sampled = parse_output_text("{\"a\":1}").expect("the object samples");
    assert_eq!(sampled.get("a"), Some(&parse_json("1").unwrap()));
}

#[test]
fn test_no_refusal_of_the_reader_door_carries_a_wording_beyond_the_three_wordings_of_this_section()
{
    let labels = [
        "configuration",
        "session file",
        "evidence document",
        "report",
        "markdown report",
    ];
    let mut expected = BTreeSet::new();
    for label in labels {
        expected.insert(format!("the {label} is not valid JSON"));
        expected.insert(format!("the {label} carries the key a more than once"));
        expected.insert(format!("the {label} could not be read"));
    }
    let mut raised = BTreeSet::new();
    let missing = PathBuf::from("reader-wording-absent-seat/nope.json");
    let dir = fixture_dir("totality");
    let notutf8 = dir.join("notutf8.json");
    std::fs::write(&notutf8, [0x7b_u8, 0xff, 0x7d]).unwrap();
    assert_eq!(std::fs::read(&notutf8).unwrap(), [0x7b_u8, 0xff, 0x7d]);
    for label in labels {
        raised.insert(message_of("{", label));
        raised.insert(message_of("{\"a\":1,\"a\":2}", label));
        raised.insert(
            read_json_file(&missing, label)
                .unwrap_err()
                .message()
                .to_string(),
        );
        raised.insert(
            read_json_file(&notutf8, label)
                .unwrap_err()
                .message()
                .to_string(),
        );
    }
    for label in ["session file", "markdown report"] {
        raised.insert(
            read_file_bytes(&missing, label)
                .unwrap_err()
                .message()
                .to_string(),
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(raised, expected);
}

#[test]
fn test_the_session_line_pair_stands_at_the_ingest_seat_in_its_own_subject() {
    // A text carrying no line reads as no records and no refusal.
    let records = parse_session_text("", 0).expect("the empty session reads");
    assert!(records.is_empty());
    // The invalid failure words in the session line subject.
    let error = parse_session_text("{no", 0).unwrap_err();
    assert_eq!(error.message(), "a session line is not valid JSON");
    // The duplicate failure words in the same subject, naming its key.
    let error = parse_session_text("{\"a\":1,\"a\":2}", 0).unwrap_err();
    assert_eq!(
        error.message(),
        "a session line carries the key a more than once"
    );
    // The named key stands in its decoded spelling at this seat too.
    let error = parse_session_text("{\"a\":1,\"\\u0061\":2}", 0).unwrap_err();
    assert_eq!(
        error.message(),
        "a session line carries the key a more than once"
    );
}
