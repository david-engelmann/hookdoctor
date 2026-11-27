//! The pins of slice 55, the window bound precedence
//! standings, over hooks and records the pins build
//! in code and hand to the examination directly and
//! over synthetic settings and session files the
//! door pins write themselves. HD2-9 states the
//! window itself and EX-8 states the bound wording
//! whole, and the pins below state the order of
//! the two bound readings and their seats, the
//! start standing read ahead of the end at both
//! walks and the window refusal standing ahead of
//! every matcher refusal and inside the building
//! and the validations and at the door behind the
//! ingestion and the labels. Every pin stands as
//! a fence on the landed code, this slice moving
//! no production code.

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::examine::{
    build_report, examine_hooks, unconfigured_runs, validate_evidence, validate_report,
};
use hookdoctor::ingest::{HookConfig, TraceRecord};
use hookdoctor::jsonvalue::JsonValue;

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

fn hook(event: &str, matcher: Option<&str>, command: &str) -> HookConfig {
    HookConfig {
        id: format!("{event}:0:0"),
        event: event.to_string(),
        matcher: matcher.map(str::to_string),
        hook_type: "command".to_string(),
        command: command.to_string(),
    }
}

fn tool_call(line: i64) -> TraceRecord {
    let mut subject = record("tool_call", line);
    subject.tool_name = Some("Bash".to_string());
    subject.call_id = Some("c1".to_string());
    subject
}

fn tool_outcome(line: i64) -> TraceRecord {
    let mut subject = record("tool_outcome", line);
    subject.call_id = Some("c1".to_string());
    subject
}

fn run_record(line: i64) -> TraceRecord {
    let mut subject = record("hook_run", line);
    subject.hook_name = Some("guard".to_string());
    subject.hook_event = Some("PreToolUse".to_string());
    subject.command = Some("guard.sh".to_string());
    subject
}

fn examination_refusal(
    hooks: &[HookConfig],
    records: &[TraceRecord],
    start: Option<&str>,
    end: Option<&str>,
) -> String {
    examine_hooks(hooks, records, start, end, &[])
        .expect_err("the examination refuses the call")
        .message()
        .to_string()
}

fn unconfigured_refusal(
    hooks: &[HookConfig],
    records: &[TraceRecord],
    start: Option<&str>,
    end: Option<&str>,
) -> String {
    unconfigured_runs(hooks, records, start, end, &[])
        .expect_err("the unconfigured walk refuses the call")
        .message()
        .to_string()
}

/// The settings document the door cells stand on, one
/// configured hook standing clean so the window and
/// the seat it pairs with stand as the only refusals
/// in play.
const SETTINGS: &str = "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\", \"hooks\": [{\"type\": \"command\", \"command\": \"guard.sh\"}]}]}}";

/// The settings document carrying an entry with no
/// command, a configuration defect standing in play
/// beside the window of its cell.
const DEFECT_SETTINGS: &str = "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\", \"hooks\": [{\"type\": \"command\"}]}]}}";

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
    output: PathBuf,
    report: PathBuf,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn door_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("hookdoctor-wb-standings-{}", std::process::id()))
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale cell directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a cell directory stands");
    dir
}

fn run_call(dir: &Path, args: Vec<OsString>) -> DoorRun {
    let output = dir.join("output.json");
    let report = dir.join("report.md");
    let run = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .args(&args)
        .output()
        .expect("the shipped binary runs");
    DoorRun {
        _dir: ScratchDir::hold(dir),
        output,
        report,
        code: run.status.code(),
        stdout: run.stdout,
        stderr: run.stderr,
    }
}

fn assert_refusal(run: &DoorRun, wording: &str) {
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        format!("hookdoctor error: {wording}\n").into_bytes()
    );
    assert!(run.stdout.is_empty());
    assert!(!run.output.exists());
    assert!(!run.report.exists());
}

fn word(text: &str) -> OsString {
    OsString::from(text)
}

#[test]
fn test_at_the_examination_walk_the_start_bound_stands_read_ahead_of_the_end_bound_and_two_bounds_that_do_not_read_name_the_start(
) {
    let stop_hook = hook("Stop", None, "stopper.sh");
    assert_eq!(
        examination_refusal(&[stop_hook.clone()], &[], Some("junk"), Some("junk2")),
        "the window start is not a timestamp"
    );
    assert_eq!(
        examination_refusal(&[stop_hook.clone()], &[], Some("junk"), None),
        "the window start is not a timestamp"
    );
    assert_eq!(
        examination_refusal(&[stop_hook.clone()], &[], None, Some("junk2")),
        "the window end is not a timestamp"
    );
    let guard = hook("PreToolUse", Some("Bash"), "guard.sh");
    assert_eq!(
        examination_refusal(&[guard], &[tool_call(1)], Some("junk"), Some("junk2")),
        "the window start is not a timestamp"
    );
}

#[test]
fn test_at_the_unconfigured_walk_the_start_bound_stands_read_ahead_of_the_end_bound_and_two_bounds_that_do_not_read_name_the_start(
) {
    let stop_hook = hook("Stop", None, "stopper.sh");
    assert_eq!(
        unconfigured_refusal(&[stop_hook.clone()], &[], Some("junk"), Some("junk2")),
        "the window start is not a timestamp"
    );
    assert_eq!(
        unconfigured_refusal(&[stop_hook.clone()], &[], Some("junk"), None),
        "the window start is not a timestamp"
    );
    assert_eq!(
        unconfigured_refusal(&[stop_hook.clone()], &[], None, Some("junk2")),
        "the window end is not a timestamp"
    );
    assert_eq!(
        unconfigured_refusal(
            &[stop_hook],
            &[run_record(1), run_record(2)],
            Some("junk"),
            Some("junk2")
        ),
        "the window start is not a timestamp"
    );
}

#[test]
fn test_a_window_refusal_fires_ahead_of_every_matcher_refusal_at_the_counting_seat_and_at_the_near_miss_paired_seat_alike(
) {
    let bad_hook = hook("PreToolUse", Some("["), "guard.sh");
    assert_eq!(
        examination_refusal(&[bad_hook.clone()], &[tool_call(1)], Some("junk"), None),
        "the window start is not a timestamp"
    );
    assert_eq!(
        examination_refusal(&[bad_hook.clone()], &[tool_call(1)], None, Some("junk2")),
        "the window end is not a timestamp"
    );
    let prompt_hook = hook("UserPromptSubmit", Some("["), "prompt.sh");
    assert_eq!(
        examination_refusal(&[prompt_hook], &[record("prompt", 1)], Some("junk"), None),
        "the window start is not a timestamp"
    );
    assert_eq!(
        examination_refusal(
            &[bad_hook.clone()],
            &[tool_call(1), tool_outcome(2)],
            None,
            Some("junk2")
        ),
        "the window end is not a timestamp"
    );
    assert_eq!(
        examination_refusal(&[bad_hook], &[tool_call(1)], None, None),
        "a matcher is not a valid pattern"
    );
    let post_hook = hook("PostToolUse", Some("["), "guard.sh");
    assert_eq!(
        examination_refusal(&[post_hook.clone()], &[tool_call(1)], Some("junk"), None),
        "the window start is not a timestamp"
    );
    assert_eq!(
        examination_refusal(&[post_hook.clone()], &[tool_call(1)], None, Some("junk2")),
        "the window end is not a timestamp"
    );
    assert_eq!(
        examination_refusal(&[post_hook], &[tool_call(1)], None, None),
        "a matcher is not a valid pattern"
    );
}

#[test]
fn test_the_report_building_takes_the_examination_walk_ahead_of_the_unconfigured_walk_and_a_validation_rebuild_refuses_on_the_window_ahead_of_any_comparison(
) {
    let stop_hook = hook("Stop", None, "stopper.sh");
    let err = build_report(
        &[stop_hook.clone()],
        &[],
        "cfg",
        &[],
        Some("junk"),
        Some("junk2"),
    )
    .expect_err("the report building refuses the call");
    assert_eq!(err.message(), "the window start is not a timestamp");
    let err = build_report(&[stop_hook.clone()], &[], "cfg", &[], None, Some("junk2"))
        .expect_err("the report building refuses the call");
    assert_eq!(err.message(), "the window end is not a timestamp");
    let supplied = JsonValue::object(vec![]);
    let err = validate_report(
        &supplied,
        &[stop_hook.clone()],
        &[],
        "cfg",
        &[],
        Some("junk"),
        None,
    )
    .expect_err("the validation refuses the call");
    assert_eq!(err.message(), "the window start is not a timestamp");
    let err = validate_report(
        &supplied,
        &[stop_hook.clone()],
        &[],
        "cfg",
        &[],
        None,
        Some("junk2"),
    )
    .expect_err("the validation refuses the call");
    assert_eq!(err.message(), "the window end is not a timestamp");
    let err = validate_evidence(
        &supplied,
        &[stop_hook.clone()],
        &[],
        Some("junk"),
        Some("junk2"),
    )
    .expect_err("the validation refuses the call");
    assert_eq!(err.message(), "the window start is not a timestamp");
    let err = hookdoctor::examine::validate_markdown(
        b"{}",
        &[stop_hook.clone()],
        &[],
        "cfg",
        &[],
        Some("junk"),
        None,
    )
    .expect_err("the validation refuses the call");
    assert_eq!(err.message(), "the window start is not a timestamp");
    let err = build_report(&[stop_hook.clone()], &[], "cfg", &[], Some("junk"), None)
        .expect_err("the report building refuses the call");
    assert_eq!(err.message(), "the window start is not a timestamp");
    let err =
        hookdoctor::examine::build_evidence(&[stop_hook.clone()], &[], Some("junk"), Some("junk2"))
            .expect_err("the evidence building refuses the call");
    assert_eq!(err.message(), "the window start is not a timestamp");
    let err = validate_evidence(&supplied, &[stop_hook], &[], None, Some("junk2"))
        .expect_err("the validation refuses the call");
    assert_eq!(err.message(), "the window end is not a timestamp");
}

#[test]
fn test_at_the_door_the_configuration_ingestion_and_the_session_ingestion_and_the_label_seats_each_refuse_ahead_of_a_window_refusal(
) {
    let dir = door_dir("missing-config");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            dir.join("no-config.json").into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the configuration could not be read");

    let dir = door_dir("defect-config");
    let config = dir.join("settings.json");
    std::fs::write(&config, DEFECT_SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "a hook entry carries no command");

    let dir = door_dir("missing-session");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            dir.join("no-session.jsonl").into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the session file could not be read");

    let dir = door_dir("raw-byte-label");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--session-label"),
            OsString::from_vec(vec![0xff]),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "a session label is not UTF-8 text");

    let dir = door_dir("label-count-mismatch");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--session-label"),
            word("one"),
            word("--session-label"),
            word("two"),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "session labels do not match the session files");

    let dir = door_dir("defect-config-end-bad");
    let config = dir.join("settings.json");
    std::fs::write(&config, DEFECT_SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-end"),
            word("junk2"),
        ],
    );
    assert_refusal(&run, "a hook entry carries no command");

    let dir = door_dir("missing-session-end-bad");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            dir.join("no-session.jsonl").into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-end"),
            word("junk2"),
        ],
    );
    assert_refusal(&run, "the session file could not be read");

    let dir = door_dir("config-label-raw-byte");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--config-label"),
            OsString::from_vec(vec![0xff]),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the configuration label is not UTF-8 text");

    let dir = door_dir("malformed-config");
    let config = dir.join("settings.json");
    std::fs::write(&config, "{not json").expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the configuration is not valid JSON");

    let dir = door_dir("session-line-invalid");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "{bad line\n").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "a session line is not valid JSON");

    let dir = door_dir("session-not-utf8");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, vec![0xff_u8, 0xfe, 0x0a]).expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the session file is not UTF-8 text");
}

#[test]
fn test_at_the_door_two_window_words_that_do_not_read_name_the_start_and_a_checked_document_stands_read_ahead_of_the_window_refusal_and_its_disagreement_behind_it(
) {
    let dir = door_dir("both-words-bad");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
            word("--window-end"),
            word("junk2"),
        ],
    );
    assert_refusal(&run, "the window start is not a timestamp");

    let dir = door_dir("end-word-bad");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-end"),
            word("junk2"),
        ],
    );
    assert_refusal(&run, "the window end is not a timestamp");

    let dir = door_dir("checked-report-missing");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-report"),
            dir.join("no-report.json").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the report could not be read");

    let dir = door_dir("checked-report-disagrees");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let supplied = dir.join("supplied.json");
    std::fs::write(&supplied, "{}").expect("the supplied document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-report"),
            supplied.into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the window start is not a timestamp");

    let dir = door_dir("plain-start-alone");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the window start is not a timestamp");

    let dir = door_dir("checked-report-disagrees-end-bad");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let supplied = dir.join("supplied.json");
    std::fs::write(&supplied, "{}").expect("the supplied document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-report"),
            supplied.into_os_string(),
            word("--window-end"),
            word("junk2"),
        ],
    );
    assert_refusal(&run, "the window end is not a timestamp");

    let dir = door_dir("checked-evidence-missing");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-evidence"),
            dir.join("no-evidence.json").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the evidence document could not be read");

    let dir = door_dir("checked-markdown-missing");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-markdown"),
            dir.join("no-report.md").into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the markdown report could not be read");

    let dir = door_dir("checked-evidence-disagrees");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let supplied = dir.join("supplied.json");
    std::fs::write(&supplied, "{}").expect("the supplied document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--check-evidence"),
            supplied.into_os_string(),
            word("--window-start"),
            word("junk"),
        ],
    );
    assert_refusal(&run, "the window start is not a timestamp");

    let dir = door_dir("clean-call");
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let session = dir.join("session.jsonl");
    std::fs::write(&session, "").expect("the session document writes");
    let run = run_call(
        &dir,
        vec![
            word("--config"),
            config.into_os_string(),
            word("--session"),
            session.into_os_string(),
            word("--output"),
            dir.join("output.json").into_os_string(),
            word("--report"),
            dir.join("report.md").into_os_string(),
            word("--window-start"),
            word("2025-01-01T00:00:00Z"),
            word("--window-end"),
            word("2027-01-01T00:00:00Z"),
        ],
    );
    assert_eq!(run.code, Some(0));
    assert!(run.stdout.is_empty());
    assert!(run.stderr.is_empty());
    assert!(run.output.exists());
    assert!(run.report.exists());
}
