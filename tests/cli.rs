//! The recorded command line battery of obligation PORT2-4.
//!
//! The recordings were made by running the landed Python command
//! line as a black box over invocations drawn from its own suite
//! and from the argument edges the contract probe recorded. Each
//! case stands in a scratch directory with its input files, and
//! its steps run there in order. The comparison is at content
//! level, the exit code and the standard error text and the
//! written files byte for byte. For a step the Python argument
//! reader refused, the recording carries no standard error text,
//! because the contract states the argument reader texts stand
//! outside the port, and the step instead asserts the binary
//! writes its own usage line byte for byte, the line standing in
//! the runner below as the pin it is, and nothing to standard
//! output.
//!
//! A written file name is read as the platform reads it, so a
//! backslash standing in a name is a character of the name and
//! never a separator on this platform, and the comparison keeps
//! the name as it stands.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::reader::read_json_document;

/// The usage line of the shell with its newline, carried here as
/// the pin a usage refusal step byte compares its standard error
/// against, so a step the argument reader refused cannot pass on
/// the tool refusal line standing in its place.
const USAGE_LINE: &str = "usage: hookdoctor --config PATH --session PATH [--session PATH ...] --output PATH --report PATH [--config-label TEXT] [--session-label TEXT ...] [--window-start TIMESTAMP] [--window-end TIMESTAMP]\n";

fn recordings() -> Vec<JsonValue> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/conformance/cli_cases.json"
    );
    let text = std::fs::read_to_string(path).expect("the battery recordings stand in the tree");
    let document =
        read_json_document(&text, "battery recordings").expect("the battery recordings parse");
    document
        .get("cases")
        .and_then(|cases| cases.as_list())
        .expect("the battery recordings carry a case list")
        .to_vec()
}

fn text_of(value: &JsonValue) -> &str {
    value.as_text().unwrap_or("")
}

fn hex_bytes(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    assert!(
        bytes.len().is_multiple_of(2),
        "hex text carries whole bytes"
    );
    (0..bytes.len())
        .step_by(2)
        .map(|at| {
            let pair = std::str::from_utf8(&bytes[at..at + 2]).expect("hex text is text");
            u8::from_str_radix(pair, 16).expect("hex text is hex")
        })
        .collect()
}

fn case_dir(index: usize, name: &str) -> PathBuf {
    let slug: String = name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect();
    std::env::temp_dir()
        .join(format!("hookdoctor-cli-battery-{}", std::process::id()))
        .join(format!("case-{index:03}-{slug}"))
}

/// A scratch directory that removes itself when it leaves scope,
/// so a panicking assertion leaves no state behind and concurrent
/// test processes never share a path.
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

fn collect_files(root: &Path, dir: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
    let entries = std::fs::read_dir(dir).expect("a case directory reads");
    for entry in entries {
        let path = entry.expect("a case directory entry reads").path();
        if path.is_dir() {
            collect_files(root, &path, into);
        } else {
            let rel = path
                .strip_prefix(root)
                .expect("a written file stands under the case directory")
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&path).expect("a written file reads");
            into.insert(rel, bytes);
        }
    }
}

fn run_case(index: usize, case: &JsonValue) {
    let name = text_of(case.get("name").expect("each case carries a name")).to_string();
    let dir = case_dir(index, &name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale case directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a case directory stands");
    let _scratch = ScratchDir::hold(&dir);
    let inputs = case
        .get("inputs")
        .and_then(|value| value.as_list())
        .expect("each case carries its inputs")
        .clone();
    let mut input_names: Vec<String> = Vec::new();
    for input in &inputs {
        let rel = text_of(input.get("path").expect("an input carries its path"));
        input_names.push(rel.to_string());
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("an input directory stands");
        }
        let bytes = match input.get("hex") {
            Some(hex) => hex_bytes(text_of(hex)),
            None => text_of(input.get("text").expect("an input carries its text"))
                .as_bytes()
                .to_vec(),
        };
        std::fs::write(&path, bytes).expect("an input file is written");
    }
    let dir_text = dir
        .to_str()
        .expect("a case directory path is text")
        .to_string();
    let steps = case
        .get("steps")
        .and_then(|value| value.as_list())
        .expect("each case carries its steps");
    for step in steps {
        let argv: Vec<String> = step
            .get("argv")
            .and_then(|value| value.as_list())
            .expect("each step carries its arguments")
            .iter()
            .map(|part| text_of(part).replace("{DIR}", &dir_text))
            .collect();
        let output = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
            .args(&argv)
            .current_dir(&dir)
            .output()
            .expect("the binary runs");
        let expected_exit = match step.get("exit") {
            Some(JsonValue::Int(code)) => *code,
            _ => panic!("each step carries its exit code"),
        };
        assert_eq!(
            output.status.code(),
            Some(expected_exit as i32),
            "case {name} step {argv:?} exits with a different code"
        );
        match step.get("stderr") {
            Some(JsonValue::Text(expected)) => {
                let stderr =
                    String::from_utf8(output.stderr.clone()).expect("standard error is text");
                assert_eq!(
                    stderr, *expected,
                    "case {name} step {argv:?} speaks a different standard error"
                );
            }
            _ => {
                assert_eq!(
                    output.stderr,
                    USAGE_LINE.as_bytes(),
                    "case {name} step {argv:?} writes its own usage line to standard error"
                );
            }
        }
        let stdout_empty = match step.get("stdout_empty") {
            Some(JsonValue::Bool(standing)) => *standing,
            _ => panic!("each step carries its standard output standing"),
        };
        if stdout_empty {
            assert!(
                output.stdout.is_empty(),
                "case {name} step {argv:?} writes to standard output"
            );
        } else {
            assert!(
                !output.stdout.is_empty(),
                "case {name} step {argv:?} answers the help flag on standard output"
            );
        }
    }
    let mut written: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    collect_files(&dir, &dir, &mut written);
    for rel in &input_names {
        written.remove(rel);
    }
    let mut expected: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    if let Some(outputs) = case.get("outputs").and_then(|value| value.as_list()) {
        for output in outputs {
            let rel = text_of(output.get("path").expect("an output carries its path"));
            let text = text_of(output.get("text").expect("an output carries its text"));
            expected.insert(rel.to_string(), text.as_bytes().to_vec());
        }
    }
    assert_eq!(
        written, expected,
        "case {name} writes different files than the record"
    );
    if name == "the outputs carry no local paths" {
        for bytes in written.values() {
            assert!(
                !bytes
                    .windows(dir_text.len())
                    .any(|window| window == dir_text.as_bytes()),
                "case {name} writes a local path into an output"
            );
        }
    }
    std::fs::remove_dir_all(&dir).expect("a case directory clears");
}

/// One input file of one recorded battery case, standing as the
/// single source of the synthetic documents the named pins below
/// drive the binary over.
fn recorded_input(case_name: &str, input: &str) -> String {
    let cases = recordings();
    let case = cases
        .iter()
        .find(|case| case.get("name").and_then(|value| value.as_text()) == Some(case_name))
        .expect("the named battery case stands in the recordings");
    let inputs = case
        .get("inputs")
        .and_then(|value| value.as_list())
        .expect("the named case carries its inputs");
    inputs
        .iter()
        .find(|entry| entry.get("path").and_then(|value| value.as_text()) == Some(input))
        .and_then(|entry| entry.get("text"))
        .map(|text| text_of(text).to_string())
        .expect("the named input stands in the case")
}

fn pin_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale pin directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a pin directory stands");
    dir
}

#[test]
fn test_cli_battery_matches_the_python_record() {
    let cases = recordings();
    assert_eq!(cases.len(), 45, "the battery carries 45 cases");
    let sections: Vec<(&str, usize)> = vec![
        ("behaviours", 9),
        ("arguments", 25),
        ("refusals", 9),
        ("write_failure", 2),
    ];
    for (section, count) in &sections {
        let standing = cases
            .iter()
            .filter(|case| case.get("section").and_then(|value| value.as_text()) == Some(section))
            .count();
        assert_eq!(
            standing, *count,
            "the battery section {section} carries its recorded count"
        );
    }
    for (index, case) in cases.iter().enumerate() {
        run_case(index, case);
    }
}

/// The configuration label pin of obligation PORT2-3. The label
/// flows into the outputs as data, standing in the report as it
/// was given, so a label carrying a byte that is not UTF-8 is
/// refused by the shell in its own wording where the Python dies
/// uncaught on such a word, and nothing is written.
#[cfg(unix)]
#[test]
fn test_a_configuration_label_carrying_a_non_utf8_byte_is_refused() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let dir = pin_dir("hookdoctor-cli-config-label");
    let _scratch = ScratchDir::hold(&dir);
    let settings = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-settings.json",
    );
    let session = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-session.jsonl",
    );
    let settings_path = dir.join("settings.json");
    std::fs::write(&settings_path, settings.as_bytes()).expect("the configuration stands");
    let session_path = dir.join("session.jsonl");
    std::fs::write(&session_path, session.as_bytes()).expect("the session stands");
    let output_path = dir.join("report.json");
    let output = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .arg("--config")
        .arg(&settings_path)
        .arg("--session")
        .arg(&session_path)
        .arg("--config-label")
        .arg(OsString::from_vec(b"lab\xffel".to_vec()))
        .arg("--output")
        .arg(&output_path)
        .arg("--report")
        .arg(dir.join("report.md"))
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(2),
        "a configuration label carrying a raw byte refuses"
    );
    let stderr = String::from_utf8(output.stderr).expect("standard error is text");
    assert_eq!(
        stderr,
        "hookdoctor error: the configuration label is not UTF-8 text\n"
    );
    assert!(!output_path.exists(), "the refused label writes nothing");
    std::fs::remove_dir_all(&dir).expect("a pin directory clears");
}

/// The session label pin of obligation PORT2-3, the same reading
/// for the repeating label of the session files.
#[cfg(unix)]
#[test]
fn test_a_session_label_carrying_a_non_utf8_byte_is_refused() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let dir = pin_dir("hookdoctor-cli-session-label");
    let _scratch = ScratchDir::hold(&dir);
    let settings = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-settings.json",
    );
    let session = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-session.jsonl",
    );
    let settings_path = dir.join("settings.json");
    std::fs::write(&settings_path, settings.as_bytes()).expect("the configuration stands");
    let session_path = dir.join("session.jsonl");
    std::fs::write(&session_path, session.as_bytes()).expect("the session stands");
    let output_path = dir.join("report.json");
    let output = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .arg("--config")
        .arg(&settings_path)
        .arg("--session")
        .arg(&session_path)
        .arg("--session-label")
        .arg(OsString::from_vec(b"lab\xffel".to_vec()))
        .arg("--output")
        .arg(&output_path)
        .arg("--report")
        .arg(dir.join("report.md"))
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(2),
        "a session label carrying a raw byte refuses"
    );
    let stderr = String::from_utf8(output.stderr).expect("standard error is text");
    assert_eq!(
        stderr,
        "hookdoctor error: a session label is not UTF-8 text\n"
    );
    assert!(!output_path.exists(), "the refused label writes nothing");
    std::fs::remove_dir_all(&dir).expect("a pin directory clears");
}

/// The nesting band pin of obligation PORT2-3, stated there as a
/// port limit. A value nested inside the configuration at 4500
/// deep in a field the reading does not read completes under the
/// Python, and the port refuses the same document whole at its
/// stated reader bound of 4000, in the reader wording, writing
/// nothing.
#[test]
fn test_a_document_nested_past_the_reader_bound_is_refused_by_the_command_line() {
    let dir = pin_dir("hookdoctor-cli-nesting");
    let _scratch = ScratchDir::hold(&dir);
    let settings = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-settings.json",
    );
    let session = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-session.jsonl",
    );
    let nest = format!("{}{}", "[".repeat(4500), "]".repeat(4500));
    let deep = settings.replacen('{', &format!("{{\"deep\": {nest}, "), 1);
    assert_ne!(
        deep, settings,
        "the configuration object stands in the recording"
    );
    let deep_path = dir.join("deep.json");
    std::fs::write(&deep_path, deep.as_bytes()).expect("the deep document stands");
    let session_path = dir.join("session.jsonl");
    std::fs::write(&session_path, session.as_bytes()).expect("the session stands");
    let output_path = dir.join("report.json");
    let output = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .arg("--config")
        .arg(&deep_path)
        .arg("--session")
        .arg(&session_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--report")
        .arg(dir.join("report.md"))
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(2),
        "a document nested past the reader bound refuses"
    );
    let stderr = String::from_utf8(output.stderr).expect("standard error is text");
    assert_eq!(
        stderr,
        "hookdoctor error: the configuration is not valid JSON\n"
    );
    assert!(!output_path.exists(), "the refused document writes nothing");
    std::fs::remove_dir_all(&dir).expect("a pin directory clears");
}

/// The argument bytes pin of obligation PORT2-3. The shell takes
/// its arguments as the operating system hands them over, so a
/// configuration and a session whose file names carry a byte
/// that is not UTF-8 read whole under labels the caller gives,
/// and the run they name builds its outputs.
#[cfg(unix)]
#[test]
fn test_a_file_name_carrying_a_non_utf8_byte_is_read() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let dir = pin_dir("hookdoctor-cli-nonutf8");
    let _scratch = ScratchDir::hold(&dir);
    let settings = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-settings.json",
    );
    let session = recorded_input(
        "deterministic outputs over the examples",
        "synthetic-session.jsonl",
    );
    let mut bad_settings = dir.as_os_str().as_bytes().to_vec();
    bad_settings.extend_from_slice(b"/set\xfftings.json");
    let bad_settings_path = PathBuf::from(OsString::from_vec(bad_settings));
    std::fs::write(&bad_settings_path, settings.as_bytes())
        .expect("the configuration stands under its byte name");
    let mut bad_session = dir.as_os_str().as_bytes().to_vec();
    bad_session.extend_from_slice(b"/sess\xffion.jsonl");
    let bad_session_path = PathBuf::from(OsString::from_vec(bad_session));
    std::fs::write(&bad_session_path, session.as_bytes())
        .expect("the session stands under its byte name");
    let output = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .arg("--config")
        .arg(bad_settings_path.as_os_str())
        .arg("--session")
        .arg(bad_session_path.as_os_str())
        .arg("--config-label")
        .arg("given-config")
        .arg("--session-label")
        .arg("given-session")
        .arg("--output")
        .arg(dir.join("report.json"))
        .arg("--report")
        .arg(dir.join("report.md"))
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(0),
        "documents named with a raw byte build under given labels"
    );
    assert!(output.stderr.is_empty());
    let report = std::fs::read_to_string(dir.join("report.json")).expect("the report stands");
    assert!(
        report.contains("\"config_source\": \"given-config\""),
        "the given configuration label stands in the report"
    );
    assert!(
        report.contains("\"given-session\""),
        "the given session label stands in the report"
    );
    std::fs::remove_dir_all(&dir).expect("a pin directory clears");
}
