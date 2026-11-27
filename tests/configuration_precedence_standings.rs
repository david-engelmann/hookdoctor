//! The pins of slice 54, the configuration refusal
//! precedence statement, over documents the pins write
//! as literals and over synthetic settings and session
//! files the door pin writes itself. Where more than
//! one defect stands in one document the refusal that
//! fires is the refusal of the first defect in the walk
//! order, and the pins below walk that order at its
//! seats, the head seats and the seats inside one group
//! and inside one entry and the order across groups and
//! across events, and at the door where the wording
//! names the refusal line. Every pin stands as a fence
//! on the landed code, this slice moving no production
//! code.

use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::ingest::{parse_config, HookConfig};
use hookdoctor::reader::read_json_document;

fn config(text: &str) -> Result<Vec<HookConfig>, String> {
    let document =
        read_json_document(text, "configuration").map_err(|error| error.message().to_string())?;
    parse_config(&document).map_err(|error| error.message().to_string())
}

fn config_refusal(text: &str) -> String {
    config(text).expect_err("the configuration reading refuses the document")
}

/// The session document the door cells stand on, one
/// line standing clean so the configuration reading
/// stands as the only refusal in play.
const SESSION_TEXT: &str = "{\"type\": \"system\", \"subtype\": \"other\"}\n";

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
/// settings document the suite writes itself.
struct DoorRun {
    _dir: ScratchDir,
    output: PathBuf,
    report: PathBuf,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_door(name: &str, settings: &str) -> DoorRun {
    let dir = std::env::temp_dir()
        .join(format!("hookdoctor-cp-standings-{}", std::process::id()))
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale cell directory clears");
    }
    std::fs::create_dir_all(&dir).expect("a cell directory stands");
    let settings_path = dir.join("settings.json");
    let session_path = dir.join("session.jsonl");
    std::fs::write(&settings_path, settings).expect("the settings document writes");
    std::fs::write(&session_path, SESSION_TEXT).expect("the session document writes");
    let output = dir.join("output.json");
    let report = dir.join("report.md");
    let run = Command::new(env!("CARGO_BIN_EXE_hookdoctor"))
        .arg("--config")
        .arg(&settings_path)
        .arg("--session")
        .arg(&session_path)
        .arg("--output")
        .arg(&output)
        .arg("--report")
        .arg(&report)
        .output()
        .expect("the shipped binary runs");
    DoorRun {
        _dir: ScratchDir::hold(&dir),
        output,
        report,
        code: run.status.code(),
        stdout: run.stdout,
        stderr: run.stderr,
    }
}

#[test]
fn test_the_document_and_hooks_value_refusals_stand_ahead_of_the_walk_and_an_event_list_refusal_fires_at_its_event_its_turn(
) {
    assert_eq!(
        config_refusal("[1, 2]"),
        "the configuration is not a JSON object"
    );
    assert_eq!(
        config_refusal("{\"hooks\": [1]}"),
        "the hooks value is not an object"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": 5, \"Stop\": [{\"matcher\": 5, \"hooks\": [{\"command\": \"x.sh\"}]}]}}"
        ),
        "a hooks event does not carry a list of groups"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"x.sh\"}]}], \"Stop\": 5}}"
        ),
        "a hooks event does not carry a list of groups"
    );
    assert!(config("{\"hooks\": null}")
        .expect("the reading yields")
        .is_empty());
}

#[test]
fn test_inside_one_group_the_hooks_list_refusal_stands_ahead_of_the_matcher_refusals_and_the_matcher_refusals_stand_ahead_of_the_entry_refusals(
) {
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [\"loose\"]}}"),
        "a matcher group is not an object"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"matcher\": 5}]}}"),
        "a matcher group carries no hooks list"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"[\"}]}}"),
        "a matcher group carries no hooks list"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"[\", \"hooks\": 5}]}}"),
        "a matcher group carries no hooks list"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"matcher\": 5, \"hooks\": [{\"type\": \"command\"}]}]}}"
        ),
        "a matcher is not a string"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"[\", \"hooks\": [7]}]}}"),
        "a matcher is not a valid pattern"
    );
}

#[test]
fn test_inside_one_entry_the_shape_refusal_stands_first_and_the_command_refusal_stands_ahead_of_the_type_refusal(
) {
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [7, {\"type\": \"command\"}]}]}}"
        ),
        "a hook entry is not an object"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"type\": 5}]}]}}"),
        "a hook entry carries no command"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{}, {\"command\": \"x.sh\", \"type\": \"\"}]}]}}"
        ),
        "a hook entry carries no command"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"x.sh\", \"type\": 5}]}]}}"
        ),
        "a hook entry carries no type"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"a.sh\"}, {\"command\": \"b.sh\", \"type\": \"\"}]}]}}"
        ),
        "a hook entry carries no type"
    );
}

#[test]
fn test_a_defect_of_an_earlier_group_fires_ahead_of_every_defect_of_a_later_group_whatever_kind_each_defect_stands_in(
) {
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{}]}, {\"matcher\": 5, \"hooks\": [{\"command\": \"x.sh\"}]}]}}"
        ),
        "a hook entry carries no command"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"matcher\": 5, \"hooks\": [{\"command\": \"x.sh\"}]}, {\"matcher\": \"Bash\"}]}}"
        ),
        "a matcher is not a string"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\"}, {\"hooks\": [{}]}]}}"
        ),
        "a matcher group carries no hooks list"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"x.sh\"}]}, \"loose\"]}}"
        ),
        "a matcher group is not an object"
    );
}

#[test]
fn test_the_events_stand_taken_in_the_order_they_stand_written_and_no_order_of_their_names_and_an_empty_matcher_raises_neither_matcher_refusal(
) {
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"Zeta\": [{\"matcher\": 5, \"hooks\": [{\"command\": \"x.sh\"}]}], \"Alpha\": 5}}"
        ),
        "a matcher is not a string"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"Alpha\": 5, \"Zeta\": [{\"matcher\": 5, \"hooks\": [{\"command\": \"x.sh\"}]}]}}"
        ),
        "a hooks event does not carry a list of groups"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{}]}], \"Stop\": 5}}"),
        "a hook entry carries no command"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"\", \"hooks\": [{}]}]}}"),
        "a hook entry carries no command"
    );
}

#[test]
fn test_at_the_door_the_first_defect_of_the_document_names_the_refusal_line_and_a_clean_document_writes_both_outputs(
) {
    let run = run_door(
        "list-before-matcher",
        "{\"hooks\": {\"PreToolUse\": [{\"matcher\": 5}]}}",
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        b"hookdoctor error: a matcher group carries no hooks list\n".to_vec()
    );
    assert!(run.stdout.is_empty());
    assert!(!run.output.exists());
    assert!(!run.report.exists());
    let run = run_door(
        "entry-of-first-group",
        "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{}]}, {\"matcher\": 5, \"hooks\": [{\"command\": \"x.sh\"}]}]}}",
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        b"hookdoctor error: a hook entry carries no command\n".to_vec()
    );
    assert!(run.stdout.is_empty());
    assert!(!run.output.exists());
    assert!(!run.report.exists());
    let run = run_door(
        "single-defect",
        "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"x.sh\", \"type\": 5}]}]}}",
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        b"hookdoctor error: a hook entry carries no type\n".to_vec()
    );
    assert!(run.stdout.is_empty());
    assert!(!run.output.exists());
    assert!(!run.report.exists());
    let run = run_door(
        "clean-document",
        "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\", \"hooks\": [{\"command\": \"x.sh\"}]}]}}",
    );
    assert_eq!(run.code, Some(0));
    assert!(run.stdout.is_empty());
    assert!(run.stderr.is_empty());
    assert!(run.output.exists());
    assert!(run.report.exists());
}
