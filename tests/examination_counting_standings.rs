//! The pins of slice 50, the examination counting
//! standings, over hooks and records the pins build
//! in code and hand to the examination directly.
//! HD2-4 and HD2-5 and HD2-6 and HD2-7 state the
//! counts of an entry at their summary and record
//! grains, and the pins below state the dispatch
//! that produces those counts whole at the walk
//! grain, the matched count and the ran reading
//! and the effects dispatch and the sums and the
//! class derivation and the refusal reach and the
//! window bound wording. Every pin stands as a
//! fence on the landed code, this slice moving
//! no production code.

use hookdoctor::examine::{examine_hooks, unconfigured_runs};
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

fn entry<'a>(entries: &'a [JsonValue], id: &str) -> &'a JsonValue {
    entries
        .iter()
        .find(|e| e.get("id").and_then(JsonValue::as_text) == Some(id))
        .expect("entry stands")
}

fn count(evidence: &JsonValue, key: &str) -> i64 {
    match evidence.get(key) {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("count {key} stands"),
    }
}

fn class_of(e: &JsonValue) -> String {
    e.get("class")
        .and_then(JsonValue::as_text)
        .expect("class stands")
        .to_string()
}

fn effects_of(e: &JsonValue) -> (i64, i64, i64) {
    let effects = e
        .get("evidence")
        .expect("evidence")
        .get("effects")
        .expect("effects");
    (
        count(effects, "denial"),
        count(effects, "modification"),
        count(effects, "consumed_output"),
    )
}

fn matched_ran(e: &JsonValue) -> (i64, i64) {
    let evidence = e.get("evidence").expect("evidence");
    (count(evidence, "matched"), count(evidence, "ran"))
}

#[test]
fn test_a_stop_subject_counts_as_matched_for_every_stop_hook_and_as_ran_only_for_the_hook_its_commands_name(
) {
    // A stop record naming no configured command stands
    // matched and not ran for every stop hook.
    let stop_hook = hook("Stop", None, "stopper.sh");
    let mut other_hook = hook("Stop", None, "sweeper.sh");
    other_hook.id = "Stop:1:0".to_string();
    let mut unnamed = record("stop", 1);
    unnamed.commands = vec!["other.sh".to_string()];
    let (entries, _) = examine_hooks(
        &[stop_hook.clone(), other_hook],
        &[unnamed],
        None,
        None,
        &[],
    )
    .unwrap();
    for id in ["Stop:0:0", "Stop:1:0"] {
        let e = entry(&entries, id);
        assert_eq!(matched_ran(e), (1, 0));
        assert_eq!(class_of(e), "silent");
        assert_eq!(effects_of(e), (0, 0, 0));
    }

    // A stop record naming the command with continuation
    // prevented and output standing stands ran with a
    // denial and consumed output and no modification.
    let mut named = record("stop", 1);
    named.commands = vec!["stopper.sh".to_string()];
    named.prevented_continuation = true;
    named.has_output = true;
    let (entries, _) = examine_hooks(&[stop_hook.clone()], &[named], None, None, &[]).unwrap();
    let e = entry(&entries, "Stop:0:0");
    assert_eq!(matched_ran(e), (1, 1));
    assert_eq!(effects_of(e), (1, 0, 1));
    assert_eq!(class_of(e), "firing");

    // A stop record naming the command with neither flag
    // stands ran with no effect.
    let mut bare = record("stop", 1);
    bare.commands = vec!["stopper.sh".to_string()];
    let (entries, _) = examine_hooks(&[stop_hook], &[bare], None, None, &[]).unwrap();
    let e = entry(&entries, "Stop:0:0");
    assert_eq!(matched_ran(e), (1, 1));
    assert_eq!(effects_of(e), (0, 0, 0));
    assert_eq!(class_of(e), "firing-without-effect");
}

#[test]
fn test_a_subject_after_the_tool_runs_counts_no_effect_and_the_subject_before_it_counts_the_denial_of_the_same_call(
) {
    let pre = hook("PreToolUse", Some("Bash"), "guard.sh");
    let mut post = hook("PostToolUse", Some("Bash"), "audit.sh");
    post.id = "PostToolUse:0:0".to_string();

    // The paired call. The hook before the tool runs counts
    // the denial of the drawn outcome and the hook after
    // it counts no effect of any kind over the same call.
    let mut call = record("tool_call", 1);
    call.tool_name = Some("Bash".to_string());
    call.call_id = Some("c1".to_string());
    let mut outcome = record("tool_outcome", 2);
    outcome.call_id = Some("c1".to_string());
    outcome.is_error = true;
    outcome.text = "request denied".to_string();
    let (entries, _) = examine_hooks(
        &[pre.clone(), post.clone()],
        &[call, outcome],
        None,
        None,
        &[],
    )
    .unwrap();
    let pre_e = entry(&entries, "PreToolUse:0:0");
    assert_eq!(matched_ran(pre_e), (1, 1));
    assert_eq!(effects_of(pre_e), (1, 0, 0));
    assert_eq!(class_of(pre_e), "firing");
    let post_e = entry(&entries, "PostToolUse:0:0");
    assert_eq!(matched_ran(post_e), (1, 1));
    assert_eq!(effects_of(post_e), (0, 0, 0));
    assert_eq!(class_of(post_e), "firing-without-effect");

    // The unpaired call. No outcome stands drawn, so no
    // subject after the tool runs stands built at all.
    let mut call = record("tool_call", 1);
    call.tool_name = Some("Bash".to_string());
    call.call_id = Some("c1".to_string());
    let (entries, _) = examine_hooks(&[pre, post], &[call], None, None, &[]).unwrap();
    assert_eq!(matched_ran(entry(&entries, "PreToolUse:0:0")), (1, 1));
    let post_e = entry(&entries, "PostToolUse:0:0");
    assert_eq!(matched_ran(post_e), (0, 0));
    assert_eq!(class_of(post_e), "silent");
}

#[test]
fn test_a_prompt_subject_matches_under_any_matcher_and_counts_no_effect() {
    // A prompt subject carries no tool, so the matcher
    // reading stands untaken and the subject matches
    // under a standing matcher as under none.
    let prompt_hook = hook("UserPromptSubmit", Some("Bash"), "prompt.sh");
    let prompt = record("prompt", 1);
    let (entries, _) = examine_hooks(&[prompt_hook], &[prompt.clone()], None, None, &[]).unwrap();
    let e = entry(&entries, "UserPromptSubmit:0:0");
    assert_eq!(matched_ran(e), (1, 1));
    assert_eq!(effects_of(e), (0, 0, 0));
    assert_eq!(class_of(e), "firing-without-effect");

    let matcherless = hook("UserPromptSubmit", None, "prompt.sh");
    let (entries, _) = examine_hooks(&[matcherless], &[prompt], None, None, &[]).unwrap();
    let e = entry(&entries, "UserPromptSubmit:0:0");
    assert_eq!(matched_ran(e), (1, 1));
    assert_eq!(effects_of(e), (0, 0, 0));
    assert_eq!(class_of(e), "firing-without-effect");
}

#[test]
fn test_one_run_record_counts_at_most_one_effect_of_each_kind_and_two_run_records_count_two_of_each(
) {
    // One run record whose sampling shows a denial at two
    // arms counts one denial, its facts standing as
    // booleans over the whole record.
    let run_hook = hook("PreToolUse", Some("Bash"), "guard.sh");
    let mut run = record("hook_run", 1);
    run.hook_event = Some("PreToolUse".to_string());
    run.command = Some("guard.sh".to_string());
    run.content = "{\"decision\": \"block\", \"additionalContext\": \"ctx\"}".to_string();
    run.stdout = "{\"permissionDecision\": \"deny\", \"updatedInput\": {}}".to_string();
    let (entries, _) = examine_hooks(&[run_hook.clone()], &[run.clone()], None, None, &[]).unwrap();
    let e = entry(&entries, "PreToolUse:0:0");
    assert_eq!(matched_ran(e), (1, 1));
    assert_eq!(effects_of(e), (1, 1, 1));
    assert_eq!(class_of(e), "firing");

    // Two such records count two of each kind, the sums
    // standing over the ran subjects.
    let (entries, _) = examine_hooks(&[run_hook], &[run.clone(), run], None, None, &[]).unwrap();
    let e = entry(&entries, "PreToolUse:0:0");
    assert_eq!(matched_ran(e), (2, 2));
    assert_eq!(effects_of(e), (2, 2, 2));
    assert_eq!(class_of(e), "firing");
}

#[test]
fn test_a_tool_less_subject_under_a_hook_whose_matcher_is_not_a_valid_pattern_refuses_nothing_and_a_tool_subject_under_it_refuses_the_examination(
) {
    // The prompt subject carries no tool, so the matcher
    // reading stands untaken and nothing refuses.
    let bad_prompt = hook("UserPromptSubmit", Some("["), "prompt.sh");
    let prompt = record("prompt", 1);
    let (entries, _) = examine_hooks(&[bad_prompt], &[prompt], None, None, &[]).unwrap();
    let e = entry(&entries, "UserPromptSubmit:0:0");
    assert_eq!(matched_ran(e), (1, 1));

    // The tool subject stands judged against the matcher,
    // and the invalid pattern refuses the examination.
    let bad_pre = hook("PreToolUse", Some("["), "guard.sh");
    let mut call = record("tool_call", 1);
    call.tool_name = Some("Bash".to_string());
    call.call_id = Some("c1".to_string());
    let err = examine_hooks(&[bad_pre], &[call], None, None, &[]).unwrap_err();
    assert_eq!(err.message(), "a matcher is not a valid pattern");
}

#[test]
fn test_a_window_bound_that_does_not_read_as_an_instant_refuses_with_the_wording_of_its_own_bound()
{
    let any_hook = hook("Stop", None, "stopper.sh");
    let err = examine_hooks(&[any_hook.clone()], &[], Some("junk"), None, &[]).unwrap_err();
    assert_eq!(err.message(), "the window start is not a timestamp");
    let err = examine_hooks(&[any_hook.clone()], &[], None, Some("junk"), &[]).unwrap_err();
    assert_eq!(err.message(), "the window end is not a timestamp");
    let err = unconfigured_runs(&[any_hook], &[], Some("junk"), None, &[]).unwrap_err();
    assert_eq!(err.message(), "the window start is not a timestamp");
}
