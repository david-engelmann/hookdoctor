//! The pins of slice 39, the outcome pairing
//! standings over records carrying no call
//! identifier, over records the suite builds
//! in code and hands to the examination
//! directly. The pairing of the identified
//! form stands stated at FIX-6 and fenced by
//! its recording cases, and the identifierless
//! form stands stated at OP-1 to OP-6 of this
//! slice and fenced by the pins below. The
//! shape stands unreachable at the trace door
//! under IN-7 and IN-8, so the pins stand at
//! the library grain, over records built in
//! code. Every pin stands as a fence on the
//! landed code, this slice moving no
//! production code.

use hookdoctor::examine::examine_hooks;
use hookdoctor::ingest::{HookConfig, TraceRecord};
use hookdoctor::jsonvalue::JsonValue;

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

fn call(
    line: i64,
    session: Option<&str>,
    file_index: i64,
    tool: &str,
    id: Option<&str>,
    stamp: &str,
) -> TraceRecord {
    let mut built = record("tool_call", session, file_index, line);
    built.tool_name = Some(tool.to_string());
    built.call_id = id.map(str::to_string);
    built.timestamp = Some(stamp.to_string());
    built
}

fn outcome(
    line: i64,
    session: Option<&str>,
    file_index: i64,
    id: Option<&str>,
    error: bool,
    text: &str,
    stamp: &str,
) -> TraceRecord {
    let mut built = record("tool_outcome", session, file_index, line);
    built.call_id = id.map(str::to_string);
    built.is_error = error;
    built.text = text.to_string();
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

fn pair_hooks() -> Vec<HookConfig> {
    vec![
        hook("PreToolUse:0:0", "PreToolUse", "Bash", "bash-hook.sh"),
        hook("PreToolUse:1:0", "PreToolUse", "Edit", "edit-hook.sh"),
        hook("PostToolUse:0:0", "PostToolUse", "Bash", "bash-hook.sh"),
        hook("PostToolUse:1:0", "PostToolUse", "Edit", "edit-hook.sh"),
    ]
}

fn examine(
    records: &[TraceRecord],
    sources: &[&str],
    start: Option<&str>,
    end: Option<&str>,
) -> Vec<JsonValue> {
    let owned: Vec<String> = sources.iter().map(|text| text.to_string()).collect();
    let (entries, _misses) =
        examine_hooks(&pair_hooks(), records, start, end, &owned).expect("the examination stands");
    entries
}

fn entry(entries: &[JsonValue], id: &str) -> JsonValue {
    entries
        .iter()
        .find(|candidate| candidate.get("id") == Some(&JsonValue::text(id.to_string())))
        .cloned()
        .unwrap_or_else(|| panic!("the entries carry the hook {id}"))
}

fn evidence_int(found: &JsonValue, field: &str) -> i64 {
    match found
        .get("evidence")
        .and_then(|evidence| evidence.get(field))
    {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the entry carries its {field}"),
    }
}

fn matched(found: &JsonValue) -> i64 {
    evidence_int(found, "matched")
}

fn denial(found: &JsonValue) -> i64 {
    match found
        .get("evidence")
        .and_then(|evidence| evidence.get("effects"))
        .and_then(|effects| effects.get("denial"))
    {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the entry carries its denial count"),
    }
}

fn ran_stamp(found: &JsonValue, field: &str) -> Option<String> {
    match found
        .get("evidence")
        .and_then(|evidence| evidence.get(field))
    {
        Some(JsonValue::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

#[test]
fn test_identifierless_calls_pair_identifierless_outcomes_by_queue_position_in_file_order() {
    let for_bash = "bash-hook.sh refused the call";
    let for_edit = "edit-hook.sh refused the call";
    // The aligned cell. Two identifierless calls on two tools
    // and two identifierless error outcomes, each outcome
    // crediting its own call its hook in its text. The first
    // call takes the first outcome and the second call takes
    // the second, so both hooks stand denied once.
    let aligned = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            call(2, None, 0, "Edit", None, "2026-01-01T00:00:02Z"),
            outcome(3, None, 0, None, true, for_bash, "2026-01-01T00:00:11Z"),
            outcome(4, None, 0, None, true, for_edit, "2026-01-01T00:00:12Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&aligned, "PreToolUse:0:0")), 1);
    assert_eq!(denial(&entry(&aligned, "PreToolUse:1:0")), 1);
    assert_eq!(matched(&entry(&aligned, "PostToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&aligned, "PostToolUse:1:0")), 1);
    assert_eq!(
        ran_stamp(&entry(&aligned, "PostToolUse:0:0"), "first_ran_at"),
        Some("2026-01-01T00:00:11Z".to_string())
    );
    assert_eq!(
        ran_stamp(&entry(&aligned, "PostToolUse:1:0"), "first_ran_at"),
        Some("2026-01-01T00:00:12Z".to_string())
    );
    // The crossed cell. The same calls with the outcomes
    // exchanged, each outcome crediting the other call its
    // hook. The pairing reads no content, so the first call
    // still takes the first outcome and neither hook stands
    // denied, where a pairing reading outcome content would
    // deny both. The subjects stand all the same.
    let crossed = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            call(2, None, 0, "Edit", None, "2026-01-01T00:00:02Z"),
            outcome(3, None, 0, None, true, for_edit, "2026-01-01T00:00:11Z"),
            outcome(4, None, 0, None, true, for_bash, "2026-01-01T00:00:12Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&crossed, "PreToolUse:0:0")), 0);
    assert_eq!(denial(&entry(&crossed, "PreToolUse:1:0")), 0);
    assert_eq!(matched(&entry(&crossed, "PostToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&crossed, "PostToolUse:1:0")), 1);
    assert_eq!(
        ran_stamp(&entry(&crossed, "PostToolUse:0:0"), "first_ran_at"),
        Some("2026-01-01T00:00:11Z".to_string())
    );
    // The interleaved cell. The identifierless call stands
    // between two identified calls and the identifierless
    // outcome between the identified outcomes. The shared
    // stream threads through untouched, the identifierless
    // call drawing the identifierless outcome and the
    // identified calls drawing their own plain outcomes.
    let interleaved = examine(
        &[
            call(1, None, 0, "Bash", Some("x"), "2026-01-01T00:00:01Z"),
            call(2, None, 0, "Edit", None, "2026-01-01T00:00:02Z"),
            call(3, None, 0, "Bash", Some("y"), "2026-01-01T00:00:03Z"),
            outcome(4, None, 0, Some("x"), false, "ok", "2026-01-01T00:00:11Z"),
            outcome(5, None, 0, None, true, for_edit, "2026-01-01T00:00:12Z"),
            outcome(6, None, 0, Some("y"), false, "ok", "2026-01-01T00:00:13Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&interleaved, "PreToolUse:1:0")), 1);
    assert_eq!(denial(&entry(&interleaved, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&interleaved, "PostToolUse:0:0")), 2);
    assert_eq!(matched(&entry(&interleaved, "PostToolUse:1:0")), 1);
    assert_eq!(
        ran_stamp(&entry(&interleaved, "PostToolUse:1:0"), "first_ran_at"),
        Some("2026-01-01T00:00:12Z".to_string())
    );
}

#[test]
fn test_the_identifierless_queue_is_one_queue_across_tools_sessions_and_files() {
    let for_bash = "bash-hook.sh refused the call";
    let for_edit = "edit-hook.sh refused the call";
    // The head of queue cell. One denying outcome at the head
    // of the shared queue and one plain outcome behind it,
    // with the first call in file order naming the second
    // tool. The first call takes the head of the queue
    // whichever tool it names.
    let head = examine(
        &[
            call(1, None, 0, "Edit", None, "2026-01-01T00:00:01Z"),
            call(2, None, 0, "Bash", None, "2026-01-01T00:00:02Z"),
            outcome(3, None, 0, None, true, for_edit, "2026-01-01T00:00:11Z"),
            outcome(4, None, 0, None, false, "ok", "2026-01-01T00:00:12Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&head, "PreToolUse:1:0")), 1);
    assert_eq!(denial(&entry(&head, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&head, "PostToolUse:1:0")), 1);
    assert_eq!(matched(&entry(&head, "PostToolUse:0:0")), 1);
    // The cross session cell. The call stands in one session
    // and its outcome in another, and they pair all the same.
    let cross_session = examine(
        &[
            call(1, Some("sess-a"), 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(
                2,
                Some("sess-b"),
                0,
                None,
                true,
                for_bash,
                "2026-01-01T00:00:11Z",
            ),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&cross_session, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&cross_session, "PostToolUse:0:0")), 1);
    // The cross file cell. The call stands in the first file
    // and its outcome in the second, and they pair all the
    // same.
    let cross_file = examine(
        &[
            call(1, Some("sess"), 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(
                2,
                Some("sess"),
                1,
                None,
                true,
                for_bash,
                "2026-01-01T00:00:11Z",
            ),
        ],
        &["first.jsonl", "second.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&cross_file, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&cross_file, "PostToolUse:0:0")), 1);
    // The three sessions cell. Two calls stand in two
    // sessions and one outcome in a third. The first call in
    // file order takes the one outcome and the second call
    // stands unpaired.
    let three = examine(
        &[
            call(1, Some("sess-a"), 0, "Bash", None, "2026-01-01T00:00:01Z"),
            call(2, Some("sess-b"), 0, "Edit", None, "2026-01-01T00:00:02Z"),
            outcome(
                3,
                Some("sess-c"),
                0,
                None,
                true,
                for_bash,
                "2026-01-01T00:00:11Z",
            ),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&three, "PreToolUse:0:0")), 1);
    assert_eq!(denial(&entry(&three, "PreToolUse:1:0")), 0);
    assert_eq!(matched(&entry(&three, "PostToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&three, "PostToolUse:1:0")), 0);
}

#[test]
fn test_identifierless_pairing_never_crosses_the_identifier_boundary() {
    let deny = "bash-hook.sh refused the call";
    // An identifierless call facing an identified outcome
    // alone draws nothing, building no PostToolUse subject
    // and counting no denial.
    let none_call = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, Some("y"), true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&none_call, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&none_call, "PostToolUse:0:0")), 0);
    // An identified call facing an identifierless outcome
    // alone draws nothing the same way.
    let id_call = examine(
        &[
            call(1, None, 0, "Bash", Some("y"), "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, None, true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&id_call, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&id_call, "PostToolUse:0:0")), 0);
    // An empty text call does not draw the shared queue.
    let empty_call = examine(
        &[
            call(1, None, 0, "Bash", Some(""), "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, None, true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&empty_call, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&empty_call, "PostToolUse:0:0")), 0);
    // An identifierless call does not draw the empty queue.
    let empty_outcome = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, Some(""), true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&empty_outcome, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&empty_outcome, "PostToolUse:0:0")), 0);
    // The control. An empty text call pairs an empty text
    // outcome, the empty identifier keying a queue of its
    // own.
    let empty_pair = examine(
        &[
            call(1, None, 0, "Bash", Some(""), "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, Some(""), true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&empty_pair, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&empty_pair, "PostToolUse:0:0")), 1);
    // The exhaustion cell. Two calls on one identifier with
    // one outcome on it leave the second call unpaired, and
    // the identifierless outcome standing beside them is
    // never spilled into.
    let exhausted = examine(
        &[
            call(1, None, 0, "Bash", Some("x"), "2026-01-01T00:00:01Z"),
            call(2, None, 0, "Bash", Some("x"), "2026-01-01T00:00:02Z"),
            outcome(3, None, 0, Some("x"), false, "ok", "2026-01-01T00:00:11Z"),
            outcome(4, None, 0, None, true, deny, "2026-01-01T00:00:12Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&exhausted, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&exhausted, "PostToolUse:0:0")), 1);
}

#[test]
fn test_identifierless_calls_beyond_the_outcomes_stand_unpaired_and_surplus_outcomes_stand_unread()
{
    let for_bash = "bash-hook.sh refused the call";
    // Two identifierless calls with one identifierless
    // outcome. The first call draws it and its denial counts
    // once, and the second call stands unpaired, building no
    // PostToolUse subject of its own.
    let beyond = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            call(2, None, 0, "Edit", None, "2026-01-01T00:00:02Z"),
            outcome(3, None, 0, None, true, for_bash, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&beyond, "PreToolUse:0:0")), 1);
    assert_eq!(denial(&entry(&beyond, "PreToolUse:1:0")), 0);
    assert_eq!(matched(&entry(&beyond, "PostToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&beyond, "PostToolUse:1:0")), 0);
    // One identifierless call with two identifierless
    // outcomes. The call draws the first, and the second
    // outcome stands unread, no subject standing on it and
    // its stamp appearing in no evidence date.
    let surplus = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, None, true, for_bash, "2026-01-01T00:00:11Z"),
            outcome(3, None, 0, None, true, for_bash, "2026-01-01T00:00:12Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&surplus, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&surplus, "PostToolUse:0:0")), 1);
    assert_eq!(
        ran_stamp(&entry(&surplus, "PostToolUse:0:0"), "first_ran_at"),
        Some("2026-01-01T00:00:11Z".to_string())
    );
    assert_eq!(
        ran_stamp(&entry(&surplus, "PostToolUse:0:0"), "last_ran_at"),
        Some("2026-01-01T00:00:11Z".to_string())
    );
}

#[test]
fn test_identifierless_pairing_stands_indifferent_to_the_file_order_of_call_and_outcome() {
    let deny = "bash-hook.sh refused the call";
    // The outcome standing behind its call pairs all the
    // same as any outcome does.
    let behind = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, None, true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&behind, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&behind, "PostToolUse:0:0")), 1);
    // The outcome standing ahead of its call pairs all the
    // same, the queues standing built over the whole record
    // list before any call is read.
    let ahead = examine(
        &[
            outcome(1, None, 0, None, true, deny, "2026-01-01T00:00:11Z"),
            call(2, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&ahead, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&ahead, "PostToolUse:0:0")), 1);
    assert_eq!(
        ran_stamp(&entry(&ahead, "PostToolUse:0:0"), "first_ran_at"),
        Some("2026-01-01T00:00:11Z".to_string())
    );
    // The outcome standing at the head of the file, ahead of
    // identified traffic that pairs its own, still pairs the
    // identifierless call standing behind that traffic.
    let head = examine(
        &[
            outcome(1, None, 0, None, true, deny, "2026-01-01T00:00:11Z"),
            call(2, None, 0, "Bash", Some("x"), "2026-01-01T00:00:01Z"),
            outcome(3, None, 0, Some("x"), false, "ok", "2026-01-01T00:00:12Z"),
            call(4, None, 0, "Bash", None, "2026-01-01T00:00:02Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&head, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&head, "PostToolUse:0:0")), 2);
}

#[test]
fn test_the_identifierless_queues_stand_built_over_the_windowed_records() {
    let deny = "bash-hook.sh refused the call";
    // The control. With no window standing, the call pairs
    // its outcome and the denial counts.
    let control = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, None, true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        None,
    );
    assert_eq!(denial(&entry(&control, "PreToolUse:0:0")), 1);
    assert_eq!(matched(&entry(&control, "PostToolUse:0:0")), 1);
    // The window ends between the call and its outcome, so
    // the outcome never enters the queue the call draws
    // from, and the call stands unpaired with no denial.
    let windowed = examine(
        &[
            call(1, None, 0, "Bash", None, "2026-01-01T00:00:01Z"),
            outcome(2, None, 0, None, true, deny, "2026-01-01T00:00:11Z"),
        ],
        &["only.jsonl"],
        None,
        Some("2026-01-01T00:00:05Z"),
    );
    assert_eq!(denial(&entry(&windowed, "PreToolUse:0:0")), 0);
    assert_eq!(matched(&entry(&windowed, "PostToolUse:0:0")), 0);
}
