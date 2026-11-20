//! The pins of slice 18, the output sampling statement,
//! over synthetic settings and session texts the pins
//! write as literals. The examination reads the output
//! of a run record out of the record's two texts, each
//! text sampled on its own, and reads the effect fields
//! at two candidates of each sampled object, and the
//! pins below walk the sampling at the seats the
//! recordings leave unwatched, the content seat and
//! the candidates and the presence reading and the
//! error record seats. Every pin stands as a fence on
//! the landed code, this slice moving no production
//! code.

use hookdoctor::examine::examine_hooks;
use hookdoctor::ingest::{parse_config, parse_session_text};
use hookdoctor::jsonvalue::parse_json;
use hookdoctor::JsonValue;

const SETTINGS: &str = r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "guard.sh"}]}]}}"#;

fn quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn run_line(attachment_type: &str, content: &str, stdout: &str) -> String {
    format!(
        "{{\"type\":\"attachment\",\"timestamp\":\"2026-01-01T00:00:02Z\",\"sessionId\":\"s1\",\"attachment\":{{\"type\":\"{attachment_type}\",\"hookName\":\"guard.sh\",\"command\":\"guard.sh\",\"toolUseId\":\"c1\",\"content\":{},\"exitCode\":0,\"stdout\":{}}}}}",
        quoted(content),
        quoted(stdout)
    )
}

fn effects_of(attachment_type: &str, content: &str, stdout: &str) -> (i64, i64, i64) {
    let session = [
        r#"{"type":"assistant","timestamp":"2026-01-01T00:00:02Z","sessionId":"s1","message":{"content":[{"type":"tool_use","name":"Bash","id":"c1","input":{"command":"ls"}}]}}"#.to_string(),
        run_line(attachment_type, content, stdout),
        r#"{"type":"user","timestamp":"2026-01-01T00:00:03Z","sessionId":"s1","message":{"content":[{"type":"tool_result","tool_use_id":"c1","is_error":false,"content":"ok"}]}}"#.to_string(),
    ]
    .join("\n");
    let document = parse_json(SETTINGS).expect("the settings parse");
    let hooks = parse_config(&document).expect("the configuration reads");
    let records = parse_session_text(&session, 0).expect("the session reads");
    let (entries, _near) = examine_hooks(&hooks, &records, None, None, &["session".to_string()])
        .expect("the examination stands");
    let entry = entries
        .iter()
        .find(|entry| entry.get("command").and_then(JsonValue::as_text) == Some("guard.sh"))
        .expect("the hook stands in the report");
    let effects = entry
        .get("evidence")
        .and_then(|evidence| evidence.get("effects"))
        .expect("the effects stand");
    let read = |key: &str| match effects.get(key) {
        Some(JsonValue::Int(value)) => *value,
        _ => panic!("the effect {key} stands as an integer"),
    };
    (
        read("denial"),
        read("modification"),
        read("consumed_output"),
    )
}

#[test]
fn test_the_output_of_a_run_record_stands_read_from_its_two_texts_each_sampled_on_its_own_with_a_text_that_does_not_read_as_an_object_sampling_as_no_output_and_a_repeated_key_leaving_its_last_value_standing(
) {
    assert_eq!(effects_of("hook_success", "", ""), (0, 0, 0));
    assert_eq!(
        effects_of("hook_success", "{\"decision\": \"block\"}", ""),
        (1, 0, 1)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "{\"hookSpecificOutput\": {\"permissionDecision\": \"deny\"}}",
            ""
        ),
        (1, 0, 1)
    );
    assert_eq!(effects_of("hook_success", "", "[1, 2]"), (0, 0, 0));
    assert_eq!(effects_of("hook_success", "", "not json"), (0, 0, 0));
    assert_eq!(effects_of("hook_success", "", "\"block\""), (0, 0, 0));
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"decision\": \"allow\", \"decision\": \"block\"}"
        ),
        (1, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "{\"decision\": \"block\"}",
            "{\"hookSpecificOutput\": {\"permissionDecision\": \"deny\"}}"
        ),
        (1, 0, 1)
    );
}

#[test]
fn test_at_each_sampled_object_the_effect_fields_stand_read_at_two_candidates_the_object_itself_and_its_hook_specific_output_child_where_the_child_stands_as_an_object_a_child_in_any_other_kind_adding_no_candidate(
) {
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": \"just text\", \"decision\": \"allow\"}"
        ),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": \"{\\\"decision\\\": \\\"block\\\"}\"}"
        ),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": [{\"decision\": \"block\"}]}"
        ),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": 7, \"decision\": \"allow\"}"
        ),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"decision\": \"allow\", \"hookSpecificOutput\": {\"permissionDecision\": \"deny\"}}"
        ),
        (1, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"permissionDecision\": \"deny\", \"hookSpecificOutput\": {\"decision\": \"allow\"}}"
        ),
        (1, 0, 0)
    );
}

#[test]
fn test_a_decision_standing_as_the_word_block_or_a_permission_decision_standing_as_the_word_deny_at_either_candidate_makes_a_denial_and_the_sampled_decision_arms_carry_no_error_guard(
) {
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": {\"decision\": \"block\"}}"
        ),
        (1, 0, 0)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"permissionDecision\": \"deny\"}"),
        (1, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "{\"hookSpecificOutput\": {\"permissionDecision\": \"deny\"}}",
            ""
        ),
        (1, 0, 1)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"decision\": \"allow\", \"hookSpecificOutput\": {\"decision\": \"allow\"}}"
        ),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"permissionDecision\": \"allow\"}"),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"decision\": 1}"),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of("hook_non_blocking_error", "", "{\"decision\": \"block\"}"),
        (1, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_non_blocking_error",
            "",
            "{\"hookSpecificOutput\": {\"permissionDecision\": \"deny\"}}"
        ),
        (1, 0, 0)
    );
}

#[test]
fn test_replacement_input_standing_at_either_candidate_makes_a_modification_by_its_standing_alone_a_replacement_standing_as_null_counting_as_surely_as_one_standing_as_an_object(
) {
    assert_eq!(
        effects_of("hook_success", "", "{\"updatedInput\": null}"),
        (0, 1, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": {\"updatedInput\": null}}"
        ),
        (0, 1, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"updatedInput\": {\"command\": \"x\"}}"
        ),
        (0, 1, 0)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"updatedInput\": false}"),
        (0, 1, 0)
    );
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": {\"updatedInput\": []}}"
        ),
        (0, 1, 0)
    );
    assert_eq!(
        effects_of("hook_success", "{\"updatedInput\": null}", ""),
        (0, 1, 1)
    );
}

#[test]
fn test_additional_context_standing_as_nonempty_text_at_either_candidate_makes_consumed_output_and_standing_empty_or_in_another_kind_counts_nothing_and_context_sampled_out_of_an_error_record_counts_nothing(
) {
    assert_eq!(
        effects_of(
            "hook_success",
            "",
            "{\"hookSpecificOutput\": {\"additionalContext\": \"ctx\"}}"
        ),
        (0, 0, 1)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"additionalContext\": \"ctx\"}"),
        (0, 0, 1)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"additionalContext\": \"\"}"),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"additionalContext\": 7}"),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of("hook_success", "", "{\"additionalContext\": [\"x\"]}"),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_non_blocking_error",
            "",
            "{\"additionalContext\": \"ctx\"}"
        ),
        (0, 0, 0)
    );
    assert_eq!(
        effects_of(
            "hook_non_blocking_error",
            "",
            "{\"hookSpecificOutput\": {\"additionalContext\": \"ctx\"}}"
        ),
        (0, 0, 0)
    );
}
