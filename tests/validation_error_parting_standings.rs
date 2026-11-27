//! The pins of slice 57, the validation error
//! type parting standings, over hooks and records
//! the suite builds in code and hands to the
//! library directly and over settings and session
//! files the suite writes itself for the shipped
//! binary. The check doors stand stated at CEV-3
//! and CHR-3 and CMD-3 and the window seat of the
//! crossing at WB-4, and this slice states the
//! parting of the two error types whole, at EP-1
//! to EP-6. Every pin stands as a fence on the
//! landed code, this slice moving no production
//! code.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::errors::{HookdoctorError, ReportValidationError};
use hookdoctor::examine::{
    build_evidence, build_report, examine_hooks, validate_evidence, validate_markdown,
    validate_report,
};
use hookdoctor::ingest::{parse_config, HookConfig, TraceRecord};
use hookdoctor::jsonvalue::JsonValue;
use hookdoctor::reader::read_json_document;
use hookdoctor::report::render_markdown;

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

fn standing_report() -> JsonValue {
    build_report(&[stop_hook()], &[], "cfg", &sources(), None, None).expect("the report builds")
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

#[test]
fn test_the_three_validations_raise_their_own_disagreement_wordings_in_the_validation_error_type_one_tampered_document_to_a_reading(
) {
    let report = standing_report();
    let tampered = tamper(&report, "format", JsonValue::text("hookdoctor-report/2"));
    let error: ReportValidationError = validate_report(
        &tampered,
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        None,
        None,
    )
    .expect_err("the tampered report refuses");
    assert_eq!(error.message(), "the report disagrees with the examination");

    let rendered = render_markdown(&report);
    let mut bytes = rendered.into_bytes();
    bytes[0] = b'X';
    let error: ReportValidationError =
        validate_markdown(&bytes, &[stop_hook()], &[], "cfg", &sources(), None, None)
            .expect_err("the tampered markdown refuses");
    assert_eq!(
        error.message(),
        "the markdown report disagrees with the examination"
    );

    let evidence = build_evidence(&[stop_hook()], &[], None, None).expect("the evidence builds");
    let tampered = tamper(
        &evidence,
        "format",
        JsonValue::text("hookdoctor-evidence/2"),
    );
    let error: ReportValidationError =
        validate_evidence(&tampered, &[stop_hook()], &[], None, None)
            .expect_err("the tampered evidence refuses");
    assert_eq!(
        error.message(),
        "the evidence document disagrees with the examination"
    );
}

#[test]
fn test_the_window_start_wording_the_window_end_wording_and_the_matcher_wording_cross_the_mapping_seat_into_the_validation_error_type_with_their_messages_whole(
) {
    let supplied = JsonValue::object(vec![]);
    let error: ReportValidationError = validate_report(
        &supplied,
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        Some("junk"),
        None,
    )
    .expect_err("the report validation refuses on the window");
    assert_eq!(error.message(), "the window start is not a timestamp");

    let error: ReportValidationError =
        validate_evidence(&supplied, &[stop_hook()], &[], None, Some("junk2"))
            .expect_err("the evidence validation refuses on the window");
    assert_eq!(error.message(), "the window end is not a timestamp");

    let error: ReportValidationError = validate_markdown(
        b"{}",
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        Some("junk"),
        None,
    )
    .expect_err("the markdown validation refuses on the window");
    assert_eq!(error.message(), "the window start is not a timestamp");

    let bad_hook = hook("PreToolUse", Some("["), "guard.sh");
    let error: ReportValidationError = validate_report(
        &supplied,
        &[bad_hook],
        &[tool_call(1)],
        "cfg",
        &sources(),
        None,
        None,
    )
    .expect_err("the validation refuses on the matcher");
    assert_eq!(error.message(), "a matcher is not a valid pattern");

    let error: ReportValidationError = validate_report(
        &supplied,
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        None,
        Some("junk"),
    )
    .expect_err("the report validation refuses on the window end");
    assert_eq!(error.message(), "the window end is not a timestamp");

    let error: ReportValidationError =
        validate_evidence(&supplied, &[stop_hook()], &[], Some("junk"), None)
            .expect_err("the evidence validation refuses on the window start");
    assert_eq!(error.message(), "the window start is not a timestamp");

    let error: ReportValidationError = validate_markdown(
        b"{}",
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        None,
        Some("junk"),
    )
    .expect_err("the markdown validation refuses on the window end");
    assert_eq!(error.message(), "the window end is not a timestamp");

    let bad_hook = hook("PreToolUse", Some("["), "guard.sh");
    let error: ReportValidationError = validate_markdown(
        b"{}",
        std::slice::from_ref(&bad_hook),
        &[tool_call(1)],
        "cfg",
        &sources(),
        None,
        None,
    )
    .expect_err("the markdown validation refuses on the matcher");
    assert_eq!(error.message(), "a matcher is not a valid pattern");

    let error: ReportValidationError =
        validate_evidence(&supplied, &[bad_hook], &[tool_call(1)], None, None)
            .expect_err("the evidence validation refuses on the matcher");
    assert_eq!(error.message(), "a matcher is not a valid pattern");
}

#[test]
fn test_the_builders_the_examination_the_reader_and_the_configuration_reading_raise_their_refusals_in_the_hookdoctor_error_type_at_their_own_seats(
) {
    let error: HookdoctorError =
        build_report(&[stop_hook()], &[], "cfg", &sources(), Some("junk"), None)
            .expect_err("the report building refuses on the window");
    assert_eq!(error.message(), "the window start is not a timestamp");

    let error: HookdoctorError = build_evidence(&[stop_hook()], &[], None, Some("junk2"))
        .expect_err("the evidence building refuses on the window");
    assert_eq!(error.message(), "the window end is not a timestamp");

    let bad_hook = hook("PreToolUse", Some("["), "guard.sh");
    let error: HookdoctorError = examine_hooks(&[bad_hook], &[tool_call(1)], None, None, &[])
        .expect_err("the examination refuses on the matcher");
    assert_eq!(error.message(), "a matcher is not a valid pattern");

    let error: HookdoctorError =
        read_json_document("not json", "document").expect_err("the reader refuses an invalid text");
    assert_eq!(error.message(), "the document is not valid JSON");

    let error: HookdoctorError =
        parse_config(&JsonValue::text("not an object")).expect_err("the configuration refuses");
    assert_eq!(error.message(), "the configuration is not a JSON object");

    let error: HookdoctorError = hookdoctor::ingest::parse_session_text("not json", 0)
        .expect_err("the session reading refuses an invalid line");
    assert_eq!(error.message(), "a session line is not valid JSON");

    let error: HookdoctorError = read_json_document("{\"a\":1,\"a\":2}", "document")
        .expect_err("the reader refuses a repeated key");
    assert_eq!(
        error.message(),
        "the document carries the key a more than once"
    );

    let document = JsonValue::object(vec![("hooks", JsonValue::text("not an object"))]);
    let error: HookdoctorError =
        parse_config(&document).expect_err("the configuration reading refuses");
    assert_eq!(error.message(), "the hooks value is not an object");

    let error: HookdoctorError =
        build_report(&[stop_hook()], &[], "cfg", &sources(), None, Some("junk"))
            .expect_err("the report building refuses on the window end");
    assert_eq!(error.message(), "the window end is not a timestamp");

    let error: HookdoctorError = build_evidence(&[stop_hook()], &[], Some("junk"), None)
        .expect_err("the evidence building refuses on the window start");
    assert_eq!(error.message(), "the window start is not a timestamp");
}

#[test]
fn test_a_standing_report_and_its_rendering_and_a_standing_evidence_document_validate_as_true_at_the_three_validations(
) {
    let report = standing_report();
    assert_eq!(
        validate_report(&report, &[stop_hook()], &[], "cfg", &sources(), None, None),
        Ok(true)
    );
    let rendered = render_markdown(&report);
    assert_eq!(
        validate_markdown(
            rendered.as_bytes(),
            &[stop_hook()],
            &[],
            "cfg",
            &sources(),
            None,
            None
        ),
        Ok(true)
    );
    let evidence = build_evidence(&[stop_hook()], &[], None, None).expect("the evidence builds");
    assert_eq!(
        validate_evidence(&evidence, &[stop_hook()], &[], None, None),
        Ok(true)
    );
}

#[test]
fn test_the_one_conversion_carries_the_disagreement_and_crossed_messages_across_whole_and_each_type_displays_as_its_message(
) {
    let evidence = build_evidence(&[stop_hook()], &[], None, None).expect("the evidence builds");
    let tampered = tamper(
        &evidence,
        "format",
        JsonValue::text("hookdoctor-evidence/2"),
    );
    let error: ReportValidationError =
        validate_evidence(&tampered, &[stop_hook()], &[], None, None)
            .expect_err("the tampered evidence refuses");
    assert_eq!(
        error.to_string(),
        "the evidence document disagrees with the examination"
    );
    let converted = HookdoctorError::from(error);
    assert_eq!(
        converted.message(),
        "the evidence document disagrees with the examination"
    );
    assert_eq!(
        converted,
        HookdoctorError::new("the evidence document disagrees with the examination")
    );
    assert_eq!(
        converted.to_string(),
        "the evidence document disagrees with the examination"
    );

    let supplied = JsonValue::object(vec![]);
    let crossed: ReportValidationError =
        validate_evidence(&supplied, &[stop_hook()], &[], None, Some("junk2"))
            .expect_err("the evidence validation refuses on the window");
    let converted = HookdoctorError::from(crossed);
    assert_eq!(converted.message(), "the window end is not a timestamp");

    let crossed: ReportValidationError = validate_markdown(
        b"{}",
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        Some("junk"),
        None,
    )
    .expect_err("the markdown validation refuses on the window");
    assert_eq!(crossed.to_string(), "the window start is not a timestamp");
    let converted = HookdoctorError::from(crossed);
    assert_eq!(converted.message(), "the window start is not a timestamp");

    let report = standing_report();
    let tampered = tamper(&report, "format", JsonValue::text("hookdoctor-report/2"));
    let error: ReportValidationError = validate_report(
        &tampered,
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        None,
        None,
    )
    .expect_err("the tampered report refuses");
    let converted = HookdoctorError::from(error);
    assert_eq!(
        converted,
        HookdoctorError::new("the report disagrees with the examination")
    );

    let supplied = JsonValue::object(vec![]);
    let crossed: ReportValidationError = validate_report(
        &supplied,
        &[stop_hook()],
        &[],
        "cfg",
        &sources(),
        Some("junk"),
        None,
    )
    .expect_err("the report validation refuses on the window");
    let converted = HookdoctorError::from(crossed);
    assert_eq!(converted.message(), "the window start is not a timestamp");

    let rendered = render_markdown(&report);
    let mut bytes = rendered.into_bytes();
    bytes[0] = b'X';
    let error: ReportValidationError =
        validate_markdown(&bytes, &[stop_hook()], &[], "cfg", &sources(), None, None)
            .expect_err("the tampered markdown refuses");
    let converted = HookdoctorError::from(error);
    assert_eq!(
        converted.message(),
        "the markdown report disagrees with the examination"
    );
}

/// The settings document the door cells stand on,
/// one Stop hook standing configured over the
/// session the suite writes.
const SETTINGS: &str =
    "{\"hooks\": {\"Stop\": [{\"hooks\": [{\"type\": \"command\", \"command\": \"wrap.sh\"}]}]}}";

/// The session text the door cells stand on, one
/// tool call and its outcome under one session.
const SESSION: &str = "{\"type\":\"assistant\",\"timestamp\":\"2026-01-01T00:00:02Z\",\"sessionId\":\"s1\",\"message\":{\"content\":[{\"type\":\"tool_use\",\"name\":\"Bash\",\"id\":\"c1\",\"input\":{\"command\":\"ls\"}}]}}\n{\"type\":\"user\",\"timestamp\":\"2026-01-01T00:00:03Z\",\"sessionId\":\"s1\",\"message\":{\"content\":[{\"type\":\"tool_result\",\"tool_use_id\":\"c1\",\"is_error\":false,\"content\":\"ok\"}]}}\n";

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
        .join(format!("hookdoctor-ep-standings-{}", std::process::id()))
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

fn write_pair(dir: &Path) -> (PathBuf, PathBuf) {
    let settings = dir.join("settings.json");
    let session_path = dir.join("session.jsonl");
    std::fs::write(&settings, SETTINGS).expect("the settings document writes");
    std::fs::write(&session_path, SESSION).expect("the session writes");
    (settings, session_path)
}

fn refusal_line(run: &DoorRun) -> String {
    assert_eq!(run.code, Some(2));
    assert!(run.stdout.is_empty());
    String::from_utf8(run.stderr.clone()).expect("the refusal line is text")
}

#[test]
fn test_at_the_door_a_crossed_refusal_and_a_disagreement_and_a_reader_refusal_print_in_the_one_refusal_line_at_the_one_exit_with_no_mark_of_the_type_that_carried_them(
) {
    let dir = door_dir("crossed-window");
    let (settings, session_path) = write_pair(&dir);
    let supplied = dir.join("supplied.json");
    std::fs::write(&supplied, "{}").expect("the supplied report writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            settings.clone().into_os_string(),
            word("--session"),
            session_path.clone().into_os_string(),
            word("--check-report"),
            supplied.into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_eq!(
        refusal_line(&run),
        "hookdoctor error: the window start is not a timestamp\n"
    );

    let dir = door_dir("disagreement");
    let (settings, session_path) = write_pair(&dir);
    let output = dir.join("output.json");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            settings.clone().into_os_string(),
            word("--session"),
            session_path.clone().into_os_string(),
            word("--output"),
            output.clone().into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
        ],
    );
    assert_eq!(run.code, Some(0));
    assert!(run.stderr.is_empty());
    let built = std::fs::read_to_string(&output).expect("the built report reads");
    let tampered = built.replace("hookdoctor-report/1", "hookdoctor-report/2");
    assert!(tampered != built);
    let supplied = dir.join("supplied.json");
    std::fs::write(&supplied, tampered).expect("the supplied report writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            settings.clone().into_os_string(),
            word("--session"),
            session_path.into_os_string(),
            word("--check-report"),
            supplied.into_os_string(),
        ],
    );
    assert_eq!(
        refusal_line(&run),
        "hookdoctor error: the report disagrees with the examination\n"
    );

    let dir = door_dir("unreadable-configuration");
    let (_settings, session_path) = write_pair(&dir);
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            dir.join("missing.json").into_os_string(),
            word("--session"),
            session_path.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
        ],
    );
    assert_eq!(
        refusal_line(&run),
        "hookdoctor error: the configuration could not be read\n"
    );
}
