//! The pins of slice 56, the near miss build
//! order standings, over records the suite builds
//! in code and hands to the examination directly
//! and over session files the suite writes itself
//! for the shipped binary. The order of the near
//! miss list stands stated at NM-1 in its four
//! components and the dedup at NM-2, and this
//! slice states what order the sort preserves
//! where the whole key ties and what the build
//! order behind it is, at BO-1 to BO-6. Every
//! pin stands as a fence on the landed code,
//! this slice moving no production code.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use hookdoctor::examine::examine_hooks;
use hookdoctor::ingest::{HookConfig, TraceRecord};
use hookdoctor::jsonvalue::{parse_json, JsonValue};

fn record(kind: &str, session: Option<&str>, file_index: i64, line: i64) -> TraceRecord {
    TraceRecord {
        kind: kind.to_string(),
        timestamp: None,
        session: session.map(str::to_string),
        file_index,
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

fn tool_call(
    session: Option<&str>,
    file_index: i64,
    line: i64,
    stamp: &str,
    tool: &str,
    id: &str,
) -> TraceRecord {
    let mut built = record("tool_call", session, file_index, line);
    built.tool_name = Some(tool.to_string());
    built.call_id = Some(id.to_string());
    built.timestamp = Some(stamp.to_string());
    built
}

fn tool_outcome(
    session: Option<&str>,
    file_index: i64,
    line: i64,
    stamp: &str,
    id: &str,
) -> TraceRecord {
    let mut built = record("tool_outcome", session, file_index, line);
    built.call_id = Some(id.to_string());
    built.timestamp = Some(stamp.to_string());
    built
}

fn hook(id: &str, event: &str, matcher: &str, command: &str) -> HookConfig {
    HookConfig {
        id: id.to_string(),
        event: event.to_string(),
        matcher: Some(matcher.to_string()),
        hook_type: "command".to_string(),
        command: command.to_string(),
    }
}

fn labels(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

fn near_misses(
    hooks: &[HookConfig],
    records: &[TraceRecord],
    sources: &[String],
) -> Vec<JsonValue> {
    let (_entries, misses) =
        examine_hooks(hooks, records, None, None, sources).expect("the examination stands");
    misses
}

fn miss_text(miss: &JsonValue, field: &str) -> Option<String> {
    match miss.get(field) {
        Some(JsonValue::Text(text)) => Some(text.clone()),
        Some(JsonValue::Null) => None,
        _ => panic!("the miss carries its {field}"),
    }
}

fn miss_shape(
    miss: &JsonValue,
) -> (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
) {
    let line = match miss.get("line") {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the miss carries its line"),
    };
    (
        miss_text(miss, "hook_id").expect("the miss carries its hook"),
        miss_text(miss, "reason").expect("the miss carries its reason"),
        miss_text(miss, "event").expect("the miss carries its event"),
        miss_text(miss, "tool"),
        miss_text(miss, "timestamp"),
        miss_text(miss, "source"),
        line,
    )
}

fn shapes(
    misses: &[JsonValue],
) -> Vec<(
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
)> {
    misses.iter().map(miss_shape).collect()
}

fn shape(
    hook_id: &str,
    reason: &str,
    event: &str,
    tool: &str,
    stamp: &str,
    source: &str,
    line: i64,
) -> (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
) {
    (
        hook_id.to_string(),
        reason.to_string(),
        event.to_string(),
        Some(tool.to_string()),
        Some(stamp.to_string()),
        Some(source.to_string()),
        line,
    )
}

const T: &str = "2026-01-01T09:00:00Z";

/// The settings document the door cells stand on,
/// one configured hook standing silent over the
/// calls the session files carry.
const SETTINGS: &str = "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\", \"hooks\": [{\"type\": \"command\", \"command\": \"guard.sh\"}]}]}}";

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
        .join(format!("hookdoctor-bo-standings-{}", std::process::id()))
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

fn word(text: &str) -> OsString {
    OsString::from(text)
}

fn session_line(tool: &str, id: &str, stamp: &str) -> String {
    let mut line = String::new();
    line.push_str("{\"type\": \"assistant\", \"timestamp\": \"");
    line.push_str(stamp);
    line.push_str("\", \"sessionId\": \"sess\", \"message\": {\"content\": [{\"type\": \"tool_use\", \"name\": \"");
    line.push_str(tool);
    line.push_str("\", \"id\": \"");
    line.push_str(id);
    line.push_str("\"}]}}\n");
    line
}

fn report_tools(run: &DoorRun) -> Vec<Option<String>> {
    assert_eq!(run.code, Some(0));
    assert!(run.stdout.is_empty());
    assert!(run.stderr.is_empty());
    assert!(run.output.exists());
    assert!(run.report.exists());
    let text = std::fs::read_to_string(&run.output).expect("the report document reads");
    let document = parse_json(&text).expect("the report document parses");
    let misses = match document.get("near_misses") {
        Some(JsonValue::List(items)) => items.clone(),
        _ => panic!("the report carries its near miss list"),
    };
    misses.iter().map(|miss| miss_text(miss, "tool")).collect()
}

fn door_call(dir: &Path, sessions: Vec<PathBuf>) -> DoorRun {
    let config = dir.join("settings.json");
    std::fs::write(&config, SETTINGS).expect("the settings document writes");
    let mut args = vec![word("--config"), config.into_os_string()];
    for session in sessions {
        args.push(word("--session"));
        args.push(session.into_os_string());
    }
    args.push(word("--output"));
    args.push(dir.join("output.json").into_os_string());
    args.push(word("--report"));
    args.push(dir.join("report.md").into_os_string());
    run_call(dir, args)
}

#[test]
fn test_two_misses_tying_on_the_whole_key_across_two_files_under_one_label_stand_in_record_order_and_flip_with_the_records_and_part_on_the_line_and_on_the_instant(
) {
    let hooks = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        "Bash",
        "guard-bash.sh",
    )];
    let sources = labels(&["same.jsonl", "same.jsonl"]);

    let records = [
        tool_call(Some("sess-a"), 0, 1, T, "bash", "call-a"),
        tool_call(Some("sess-b"), 1, 1, T, "BASH", "call-b"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "BASH",
                T,
                "same.jsonl",
                1
            ),
        ]
    );

    let records = [
        tool_call(Some("sess-b"), 1, 1, T, "BASH", "call-b"),
        tool_call(Some("sess-a"), 0, 1, T, "bash", "call-a"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "BASH",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "same.jsonl",
                1
            ),
        ]
    );

    let records = [
        tool_call(Some("sess-a"), 0, 2, T, "bash", "call-a"),
        tool_call(Some("sess-b"), 1, 1, T, "BASH", "call-b"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "BASH",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "same.jsonl",
                2
            ),
        ]
    );

    let records = [
        tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T10:00:00Z",
            "bash",
            "call-a",
        ),
        tool_call(
            Some("sess-b"),
            1,
            1,
            "2026-01-01T08:00:00Z",
            "BASH",
            "call-b",
        ),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "BASH",
                "2026-01-01T08:00:00Z",
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                "2026-01-01T10:00:00Z",
                "same.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_at_a_paired_call_the_subject_after_the_tool_stands_in_the_build_at_its_call_so_its_miss_stands_ahead_at_a_full_tie_however_the_outcomes_stand_interleaved(
) {
    let hooks = [hook(
        "PostToolUse:0:0",
        "PostToolUse",
        "Bash",
        "log-bash.sh",
    )];
    let sources = labels(&["same.jsonl", "same.jsonl"]);

    let records = [
        tool_call(Some("sess-a"), 0, 9, T, "bash", "c1"),
        tool_call(Some("sess-b"), 1, 9, T, "Bash", "c2"),
        tool_outcome(Some("sess-a"), 0, 9, T, "c1"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PostToolUse:0:0",
                "name-form",
                "PostToolUse",
                "bash",
                T,
                "same.jsonl",
                9
            ),
            shape(
                "PostToolUse:0:0",
                "paired-event",
                "PreToolUse",
                "Bash",
                T,
                "same.jsonl",
                9
            ),
        ]
    );

    let records = [
        tool_call(Some("sess-a"), 0, 9, T, "bash", "c1"),
        tool_call(Some("sess-b"), 1, 8, T, "Bash", "c2"),
        tool_outcome(Some("sess-a"), 0, 9, T, "c1"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PostToolUse:0:0",
                "paired-event",
                "PreToolUse",
                "Bash",
                T,
                "same.jsonl",
                8
            ),
            shape(
                "PostToolUse:0:0",
                "name-form",
                "PostToolUse",
                "bash",
                T,
                "same.jsonl",
                9
            ),
        ]
    );

    let records = [
        tool_call(Some("sess-a"), 0, 3, T, "bash", "c1"),
        tool_call(Some("sess-b"), 1, 4, T, "BASH", "c2"),
        tool_outcome(Some("sess-b"), 1, 9, T, "c2"),
        tool_outcome(Some("sess-a"), 0, 9, T, "c1"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PostToolUse:0:0",
                "name-form",
                "PostToolUse",
                "bash",
                T,
                "same.jsonl",
                9
            ),
            shape(
                "PostToolUse:0:0",
                "name-form",
                "PostToolUse",
                "BASH",
                T,
                "same.jsonl",
                9
            ),
        ]
    );
}

#[test]
fn test_two_hooks_whose_misses_tie_behind_the_hook_stand_in_configuration_order_and_flip_with_the_configuration(
) {
    let sources = labels(&["only.jsonl"]);
    let records = [tool_call(Some("sess"), 0, 1, T, "bash", "call-1")];

    let hooks = [
        hook("PreToolUse:0:0", "PreToolUse", "Bash", "guard-one.sh"),
        hook("PreToolUse:1:0", "PreToolUse", "Bash", "guard-two.sh"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "only.jsonl",
                1
            ),
            shape(
                "PreToolUse:1:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "only.jsonl",
                1
            ),
        ]
    );

    let hooks = [
        hook("PreToolUse:1:0", "PreToolUse", "Bash", "guard-two.sh"),
        hook("PreToolUse:0:0", "PreToolUse", "Bash", "guard-one.sh"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:1:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "only.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "only.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_the_gathering_stands_in_build_order_inside_a_hook_with_three_misses_tying_whole_and_the_key_deciding_the_head_where_it_parts(
) {
    let hooks = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        "Bash",
        "guard-bash.sh",
    )];
    let sources = labels(&["same.jsonl", "same.jsonl", "same.jsonl"]);

    let records = [
        tool_call(Some("sess-a"), 0, 1, T, "bash", "call-a"),
        tool_call(Some("sess-b"), 1, 1, T, "BASH", "call-b"),
        tool_call(Some("sess-c"), 2, 1, T, "bAsh", "call-c"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "BASH",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bAsh",
                T,
                "same.jsonl",
                1
            ),
        ]
    );

    let records = [
        tool_call(
            Some("sess-a"),
            0,
            1,
            "2026-01-01T10:00:00Z",
            "bash",
            "call-a",
        ),
        tool_call(Some("sess-b"), 1, 1, T, "BASH", "call-b"),
        tool_call(Some("sess-c"), 2, 1, T, "bAsh", "call-c"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "BASH",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bAsh",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "bash",
                "2026-01-01T10:00:00Z",
                "same.jsonl",
                1
            ),
        ]
    );

    let hooks = [
        hook("PreToolUse:0:0", "PreToolUse", "Edit", "guard-edit.sh"),
        hook("PreToolUse:1:0", "PreToolUse", "Bash", "guard-bash.sh"),
    ];
    let records = [
        tool_call(Some("sess-a"), 0, 1, T, "edit", "call-e"),
        tool_call(Some("sess-b"), 1, 1, T, "bash", "call-a"),
        tool_call(Some("sess-c"), 2, 1, T, "BASH", "call-b"),
    ];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![
            shape(
                "PreToolUse:0:0",
                "name-form",
                "PreToolUse",
                "edit",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:1:0",
                "name-form",
                "PreToolUse",
                "bash",
                T,
                "same.jsonl",
                1
            ),
            shape(
                "PreToolUse:1:0",
                "name-form",
                "PreToolUse",
                "BASH",
                T,
                "same.jsonl",
                1
            ),
        ]
    );
}

#[test]
fn test_at_the_door_two_session_files_under_one_file_name_carry_a_tying_pair_in_the_order_the_files_stand_handed_and_flip_with_the_handing(
) {
    let dir = door_dir("handed-in-order");
    let first = dir.join("first");
    let second = dir.join("second");
    std::fs::create_dir_all(&first).expect("the first directory stands");
    std::fs::create_dir_all(&second).expect("the second directory stands");
    let first_session = first.join("trace.jsonl");
    let second_session = second.join("trace.jsonl");
    std::fs::write(&first_session, session_line("bash", "call-a", T))
        .expect("the first session writes");
    std::fs::write(&second_session, session_line("BASH", "call-b", T))
        .expect("the second session writes");
    let run = door_call(&dir, vec![first_session.clone(), second_session.clone()]);
    assert_eq!(
        report_tools(&run),
        vec![Some("bash".to_string()), Some("BASH".to_string())]
    );

    let dir = door_dir("handed-flipped");
    let first = dir.join("first");
    let second = dir.join("second");
    std::fs::create_dir_all(&first).expect("the first directory stands");
    std::fs::create_dir_all(&second).expect("the second directory stands");
    let first_session = first.join("trace.jsonl");
    let second_session = second.join("trace.jsonl");
    std::fs::write(&first_session, session_line("bash", "call-a", T))
        .expect("the first session writes");
    std::fs::write(&second_session, session_line("BASH", "call-b", T))
        .expect("the second session writes");
    let run = door_call(&dir, vec![second_session, first_session]);
    assert_eq!(
        report_tools(&run),
        vec![Some("BASH".to_string()), Some("bash".to_string())]
    );

    let dir = door_dir("clean-call");
    let only = dir.join("trace.jsonl");
    std::fs::write(&only, session_line("Bash", "call-a", T)).expect("the session writes");
    let run = door_call(&dir, vec![only]);
    assert_eq!(report_tools(&run), Vec::<Option<String>>::new());
}

#[test]
fn test_a_call_carrying_one_miss_alone_stands_as_the_order_leaves_it_and_a_call_carrying_no_miss_stands_with_the_list_empty(
) {
    let hooks = [hook(
        "PreToolUse:0:0",
        "PreToolUse",
        "Bash",
        "guard-bash.sh",
    )];
    let sources = labels(&["only.jsonl"]);

    let records = [tool_call(Some("sess"), 0, 1, T, "bash", "call-1")];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(
        shapes(&misses),
        vec![shape(
            "PreToolUse:0:0",
            "name-form",
            "PreToolUse",
            "bash",
            T,
            "only.jsonl",
            1
        )]
    );

    let records = [tool_call(Some("sess"), 0, 1, T, "Bash", "call-1")];
    let misses = near_misses(&hooks, &records, &sources);
    assert_eq!(misses.len(), 0);
}
