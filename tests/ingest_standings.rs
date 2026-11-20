//! The pins of slice 17, the ingest statement, over
//! settings documents and session texts the pins write
//! as literals. The two readings turn recorded text
//! into the records the examination governs, and the
//! pins below walk the readings at their seats, the
//! configuration reading at its document and its hooks
//! value and its groups and its entries, and the
//! session reading at its lines and its blocks and its
//! flags and its run and stop records. Every pin stands
//! as a fence on the landed code, this slice moving no
//! production code.

use hookdoctor::ingest::{parse_config, parse_session_text, HookConfig, TraceRecord};
use hookdoctor::reader::read_json_document;

fn config(text: &str) -> Result<Vec<HookConfig>, String> {
    let document =
        read_json_document(text, "configuration").map_err(|error| error.message().to_string())?;
    parse_config(&document).map_err(|error| error.message().to_string())
}

fn config_refusal(text: &str) -> String {
    config(text).expect_err("the configuration reading refuses the document")
}

fn session(text: &str) -> Result<Vec<TraceRecord>, String> {
    parse_session_text(text, 0).map_err(|error| error.message().to_string())
}

fn session_refusal(text: &str) -> String {
    session(text).expect_err("the session reading refuses the text")
}

fn read_session(text: &str) -> Vec<TraceRecord> {
    session(text).expect("the session reading yields its records")
}

fn only(records: &[TraceRecord]) -> &TraceRecord {
    assert_eq!(records.len(), 1, "the reading yields one record");
    &records[0]
}

#[test]
fn test_a_configuration_document_and_a_hooks_value_in_another_kind_are_refused_and_a_null_hooks_value_yields_no_hooks(
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
        config_refusal("{\"hooks\": \"text\"}"),
        "the hooks value is not an object"
    );
    assert!(config("{\"hooks\": null}")
        .expect("the reading yields")
        .is_empty());
    assert!(config("{\"other\": 1}")
        .expect("the reading yields")
        .is_empty());
}

#[test]
fn test_a_hooks_event_value_and_a_group_in_another_kind_and_a_group_carrying_no_hooks_list_are_refused_and_an_empty_group_still_occupies_its_position(
) {
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": {\"matcher\": \"Bash\"}}}"),
        "a hooks event does not carry a list of groups"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [\"loose\"]}}"),
        "a matcher group is not an object"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"hooks\": {\"command\": \"x\"}}]}}"),
        "a matcher group carries no hooks list"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"hooks\": \"x\"}]}}"),
        "a matcher group carries no hooks list"
    );
    let hooks = config(
        "{\"hooks\": {\"PreToolUse\": [], \"Stop\": [{\"hooks\": []}, {\"matcher\": \"\", \"hooks\": [{\"command\": \"a.sh\"}, {\"command\": \"b.sh\", \"type\": \"prompt\"}]}]}}",
    )
    .expect("the reading yields");
    let ids: Vec<&str> = hooks.iter().map(|hook| hook.id.as_str()).collect();
    assert_eq!(ids, ["Stop:1:0", "Stop:1:1"]);
    assert_eq!(hooks[0].matcher.as_deref(), Some(""));
    assert_eq!(hooks[0].hook_type, "command");
    assert_eq!(hooks[0].command, "a.sh");
    assert_eq!(hooks[1].hook_type, "prompt");
    assert_eq!(hooks[1].command, "b.sh");
}

#[test]
fn test_a_matcher_and_a_hook_entry_in_another_kind_are_refused_and_an_entry_carrying_no_command_and_an_entry_carrying_no_type_standing_are_refused(
) {
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"matcher\": 9, \"hooks\": []}]}}"),
        "a matcher is not a string"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"matcher\": [\"Bash\"], \"hooks\": []}]}}"),
        "a matcher is not a string"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"hooks\": [7]}]}}"),
        "a hook entry is not an object"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"type\": \"command\", \"command\": 9}]}]}}"
        ),
        "a hook entry carries no command"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"\"}]}]}}"),
        "a hook entry carries no command"
    );
    assert_eq!(
        config_refusal("{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"type\": \"command\"}]}]}}"),
        "a hook entry carries no command"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"x.sh\", \"type\": null}]}]}}"
        ),
        "a hook entry carries no type"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"x.sh\", \"type\": 3}]}]}}"
        ),
        "a hook entry carries no type"
    );
    assert_eq!(
        config_refusal(
            "{\"hooks\": {\"PreToolUse\": [{\"hooks\": [{\"command\": \"x.sh\", \"type\": \"\"}]}]}}"
        ),
        "a hook entry carries no type"
    );
    let hooks = config(
        "{\"hooks\": {\"PreToolUse\": [{\"matcher\": \"Bash\", \"hooks\": [{\"command\": \"g.sh\"}]}]}}",
    )
    .expect("the reading yields");
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].matcher.as_deref(), Some("Bash"));
    assert_eq!(hooks[0].hook_type, "command");
    let hooks = config(
        "{\"hooks\": {\"PreToolUse\": [{\"matcher\": null, \"hooks\": [{\"command\": \"g.sh\"}]}]}}",
    )
    .expect("the reading yields");
    assert_eq!(hooks[0].matcher, None);
}

#[test]
fn test_a_field_of_the_session_reading_standing_in_another_kind_reads_as_though_it_stood_absent() {
    let records = read_session(
        "{\"type\": \"summary\", \"timestamp\": 9, \"sessionId\": [\"x\"], \"note\": \"n\"}",
    );
    let record = only(&records);
    assert_eq!(record.timestamp, None);
    assert_eq!(record.session, None);
    let records = read_session(
        "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_blocking_error\", \"hookName\": 9, \"toolUseID\": 4, \"content\": 5, \"command\": \"g.sh\"}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "hook_run");
    assert_eq!(record.hook_name, None);
    assert_eq!(record.call_id, None);
    assert_eq!(record.content, "");
    assert_eq!(record.command.as_deref(), Some("g.sh"));
    let records = read_session(
        "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_success\", \"hookName\": \"PostToolUse:Bash\", \"hookEvent\": 9}}",
    );
    let record = only(&records);
    assert_eq!(record.hook_name.as_deref(), Some("PostToolUse:Bash"));
    assert_eq!(record.hook_event.as_deref(), Some("PostToolUse"));
}

#[test]
fn test_the_session_reading_splits_its_text_at_the_python_line_boundary_set_and_skips_a_line_carrying_only_python_whitespace_and_counts_the_split_lines_from_one(
) {
    let line = "{\"type\": \"summary\"}";
    let separators = [
        '\n', '\r', '\u{B}', '\u{C}', '\u{1C}', '\u{1D}', '\u{1E}', '\u{85}', '\u{2028}',
        '\u{2029}',
    ];
    let mut text = String::new();
    text.push_str(line);
    for separator in separators.iter() {
        text.push(*separator);
        text.push_str(line);
    }
    let records = read_session(&text);
    assert_eq!(records.len(), separators.len() + 1);
    for (index, record) in records.iter().enumerate() {
        assert_eq!(record.kind, "other");
        assert_eq!(record.line, (index + 1) as i64);
    }
    let records =
        read_session("{\"type\": \"summary\"}\r\n{\"type\": \"summary\"}\r{\"type\": \"summary\"}");
    let lines: Vec<i64> = records.iter().map(|record| record.line).collect();
    assert_eq!(lines, [1, 2, 3]);
    let records = read_session("{\"type\": \"summary\"}\n\n{\"type\": \"summary\"}");
    let lines: Vec<i64> = records.iter().map(|record| record.line).collect();
    assert_eq!(lines, [1, 3]);
    let records = read_session("\u{A0}\n{\"type\": \"summary\"}");
    let record = only(&records);
    assert_eq!(record.line, 2);
    let records = read_session("{\"type\": \"summary\"}\n");
    let record = only(&records);
    assert_eq!(record.line, 1);
}

#[test]
fn test_a_session_line_that_does_not_stand_as_an_object_and_a_line_carrying_no_type_key_are_refused_under_the_one_no_type_wording_and_a_type_standing_in_another_kind_builds_an_other_record(
) {
    assert_eq!(session_refusal("[1, 2]"), "a session line carries no type");
    assert_eq!(
        session_refusal("\"a string line\""),
        "a session line carries no type"
    );
    assert_eq!(
        session_refusal("{\"note\": \"x\"}"),
        "a session line carries no type"
    );
    let records = read_session("{\"type\": 5, \"note\": \"typed number\"}");
    let record = only(&records);
    assert_eq!(record.kind, "other");
}

#[test]
fn test_an_assistant_line_yields_one_tool_call_record_for_each_block_whose_name_and_identifier_both_stand_as_text_and_one_other_record_where_no_block_qualifies(
) {
    let records = read_session(
        "{\"type\": \"assistant\", \"timestamp\": \"2026-01-01T09:00:00Z\", \"sessionId\": \"s1\", \"message\": {\"content\": [{\"type\": \"tool_use\", \"name\": \"Bash\", \"id\": \"c1\"}, {\"type\": \"tool_use\", \"name\": \"Edit\"}, {\"type\": \"tool_use\", \"id\": \"c3\"}, {\"type\": \"tool_use\", \"name\": 7, \"id\": \"c4\"}, {\"type\": \"text\", \"text\": \"hi\"}]}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "tool_call");
    assert_eq!(record.tool_name.as_deref(), Some("Bash"));
    assert_eq!(record.call_id.as_deref(), Some("c1"));
    assert_eq!(record.timestamp.as_deref(), Some("2026-01-01T09:00:00Z"));
    assert_eq!(record.session.as_deref(), Some("s1"));
    assert_eq!(record.line, 1);
    let records = read_session(
        "{\"type\": \"assistant\", \"message\": {\"content\": [{\"type\": \"tool_use\", \"name\": \"\", \"id\": \"\"}]}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "tool_call");
    assert_eq!(record.tool_name.as_deref(), Some(""));
    assert_eq!(record.call_id.as_deref(), Some(""));
    let records = read_session(
        "{\"type\": \"assistant\", \"message\": \"plain text message\"}\n{\"type\": \"assistant\", \"message\": {\"content\": \"plain content\"}}\n{\"type\": \"assistant\"}",
    );
    assert_eq!(records.len(), 3);
    for (index, record) in records.iter().enumerate() {
        assert_eq!(record.kind, "other");
        assert_eq!(record.line, (index + 1) as i64);
    }
    let records = read_session(
        "{\"type\": \"assistant\", \"message\": {\"content\": [{\"type\": \"text\", \"text\": \"hi\"}]}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "other");
}

#[test]
fn test_a_user_line_yields_its_outcome_records_in_block_order_and_one_prompt_record_after_them_carrying_its_texts_joined_with_a_newline(
) {
    let records = read_session(
        "{\"type\": \"user\", \"message\": {\"content\": [{\"type\": \"text\", \"text\": \"first\"}, {\"type\": \"tool_result\", \"tool_use_id\": \"c1\", \"content\": \"ok\"}, {\"type\": \"text\", \"text\": \"second\"}]}}",
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].kind, "tool_outcome");
    assert_eq!(records[0].call_id.as_deref(), Some("c1"));
    assert_eq!(records[0].text, "ok");
    assert_eq!(records[1].kind, "prompt");
    assert_eq!(records[1].text, "first\nsecond");
    let records = read_session("{\"type\": \"user\", \"message\": {\"content\": \"hello\"}}");
    let record = only(&records);
    assert_eq!(record.kind, "prompt");
    assert_eq!(record.text, "hello");
    let records = read_session(
        "{\"type\": \"user\", \"message\": {\"content\": [{\"type\": \"tool_result\", \"tool_use_id\": 9, \"content\": \"x\"}, {\"type\": \"text\", \"text\": \"kept\"}]}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "prompt");
    assert_eq!(record.text, "kept");
    let records = read_session(
        "{\"type\": \"user\", \"message\": {\"content\": [\"loose\", {\"type\": \"text\", \"text\": \"kept\"}]}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "prompt");
    assert_eq!(record.text, "kept");
    let records = read_session(
        "{\"type\": \"user\", \"message\": {\"content\": 42}}\n{\"type\": \"user\", \"message\": {}}\n{\"type\": \"user\"}",
    );
    assert_eq!(records.len(), 3);
    for record in &records {
        assert_eq!(record.kind, "other");
    }
}

#[test]
fn test_an_outcome_text_stands_as_its_content_where_the_content_stands_as_text_and_as_the_newline_join_of_its_text_blocks_where_the_content_stands_as_a_list_and_as_empty_text_in_every_other_standing(
) {
    let records = read_session(
        "{\"type\": \"user\", \"message\": {\"content\": [{\"type\": \"tool_result\", \"tool_use_id\": \"c1\", \"content\": [{\"type\": \"text\", \"text\": \"alpha\"}, {\"type\": \"image\", \"text\": \"not text block\"}, {\"type\": \"text\", \"text\": 9}, {\"type\": \"text\", \"text\": \"beta\"}, \"loose\", {\"type\": \"text\"}]}]}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "tool_outcome");
    assert_eq!(record.text, "alpha\nbeta");
    let records = read_session(
        "{\"type\": \"user\", \"message\": {\"content\": [{\"type\": \"tool_result\", \"tool_use_id\": \"c1\", \"content\": {\"text\": \"object content\"}}, {\"type\": \"tool_result\", \"tool_use_id\": \"c2\"}]}}",
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].text, "");
    assert_eq!(records[1].text, "");
}

#[test]
fn test_the_truthiness_reading_governs_the_error_flag_of_an_outcome_and_the_prevented_continuation_and_has_output_flags_of_a_stop_record(
) {
    let flags = |text: &str| -> Vec<bool> {
        read_session(text)
            .iter()
            .map(|record| record.is_error)
            .collect()
    };
    let outcome = |value: &str| -> String {
        format!(
            "{{\"type\": \"user\", \"message\": {{\"content\": [{{\"type\": \"tool_result\", \"tool_use_id\": \"c1\", \"is_error\": {value}, \"content\": \"x\"}}]}}}}"
        )
    };
    assert_eq!(flags(&outcome("\"false\"")), [true]);
    assert_eq!(flags(&outcome("0")), [false]);
    assert_eq!(flags(&outcome("\"\"")), [false]);
    assert_eq!(flags(&outcome("[]")), [false]);
    assert_eq!(flags(&outcome("[0]")), [true]);
    assert_eq!(flags(&outcome("{}")), [false]);
    assert_eq!(flags(&outcome("{\"a\": 1}")), [true]);
    assert_eq!(flags(&outcome("null")), [false]);
    assert_eq!(flags(&outcome("0.0")), [false]);
    assert_eq!(flags(&outcome("2.5")), [true]);
    let records = read_session(
        "{\"type\": \"user\", \"message\": {\"content\": [{\"type\": \"tool_result\", \"tool_use_id\": \"c1\", \"content\": \"x\"}]}}",
    );
    assert_eq!(only(&records).is_error, false);
    let records = read_session(
        "{\"type\": \"system\", \"subtype\": \"stop_hook_summary\", \"hookInfos\": {\"command\": \"a.sh\"}, \"preventedContinuation\": \"no\", \"hasOutput\": 0}",
    );
    let record = only(&records);
    assert_eq!(record.prevented_continuation, true);
    assert_eq!(record.has_output, false);
    let records = read_session(
        "{\"type\": \"system\", \"subtype\": \"stop_hook_summary\", \"hookInfos\": [], \"preventedContinuation\": false, \"hasOutput\": true}",
    );
    let record = only(&records);
    assert_eq!(record.prevented_continuation, false);
    assert_eq!(record.has_output, true);
}

#[test]
fn test_an_attachment_line_yields_a_run_record_only_where_its_attachment_names_one_of_the_three_hook_kinds_and_the_run_exit_code_stands_as_the_digits_of_an_integer_token_of_either_width_and_as_no_value_in_every_other_standing(
) {
    let records = read_session(
        "{\"type\": \"attachment\", \"attachment\": [1, 2]}\n{\"type\": \"attachment\", \"attachment\": {\"type\": 9}}\n{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_other\"}}\n{\"type\": \"attachment\"}",
    );
    assert_eq!(records.len(), 4);
    for record in &records {
        assert_eq!(record.kind, "other");
    }
    let exits = |text: &str| -> Vec<Option<String>> {
        read_session(text)
            .iter()
            .map(|record| record.exit_code.clone())
            .collect()
    };
    let hook_line = |value: &str| -> String {
        format!(
            "{{\"type\": \"attachment\", \"attachment\": {{\"type\": \"hook_success\", \"hookName\": \"PreToolUse\", \"exitCode\": {value}}}}}"
        )
    };
    assert_eq!(exits(&hook_line("2.0")), [None]);
    assert_eq!(exits(&hook_line("\"3\"")), [None]);
    assert_eq!(exits(&hook_line("true")), [None]);
    assert_eq!(exits(&hook_line("3")), [Some("3".to_string())]);
    assert_eq!(
        exits(&hook_line("1234567890123456789012345")),
        [Some("1234567890123456789012345".to_string())]
    );
    assert_eq!(exits(&hook_line("-7")), [Some("-7".to_string())]);
    let records = read_session(
        "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_non_blocking_error\", \"hookName\": \"Stop\", \"hookEvent\": \"Stop\", \"content\": \"ctx\", \"stdout\": \"out\", \"exitCode\": 1}}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "hook_run");
    assert_eq!(
        record.attachment_type.as_deref(),
        Some("hook_non_blocking_error")
    );
    assert_eq!(record.content, "ctx");
    assert_eq!(record.stdout, "out");
    assert_eq!(record.exit_code.as_deref(), Some("1"));
    assert_eq!(record.command, None);
    let records = read_session(
        "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_blocking_error\", \"hookName\": 9, \"toolUseID\": 4, \"content\": 5, \"command\": \"g.sh\"}}",
    );
    let record = only(&records);
    assert_eq!(record.content, "");
    assert_eq!(record.stdout, "");
}

#[test]
fn test_a_system_line_whose_subtype_stands_as_the_stop_summary_yields_a_stop_record_whose_commands_stand_as_the_command_texts_of_the_entries_of_its_list_that_stand_as_objects_carrying_a_text_command(
) {
    let records = read_session(
        "{\"type\": \"system\", \"subtype\": \"stop_hook_summary\", \"hookInfos\": {\"command\": \"a.sh\"}, \"preventedContinuation\": \"no\", \"hasOutput\": 0}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "stop");
    assert!(record.commands.is_empty());
    let records = read_session(
        "{\"type\": \"system\", \"subtype\": \"stop_hook_summary\", \"hookInfos\": [{\"command\": \"a.sh\"}, \"loose\", {\"command\": 3}, {\"other\": 1}, {\"command\": \"b.sh\"}], \"preventedContinuation\": false, \"hasOutput\": true}",
    );
    let record = only(&records);
    assert_eq!(record.kind, "stop");
    assert_eq!(record.commands, ["a.sh", "b.sh"]);
    let records = read_session("{\"type\": \"system\", \"subtype\": \"stop_hook_summary\"}");
    let record = only(&records);
    assert_eq!(record.kind, "stop");
    assert!(record.commands.is_empty());
    let records = read_session("{\"type\": \"system\", \"subtype\": \"other_summary\"}");
    assert_eq!(only(&records).kind, "other");
    let records = read_session("{\"type\": \"system\", \"subtype\": 5}");
    assert_eq!(only(&records).kind, "other");
}

#[test]
fn test_the_hook_event_of_a_run_record_stands_as_its_event_field_where_one_stands_and_stands_derived_from_its_hook_name_behind_an_absent_or_an_empty_event_field_and_stands_absent_where_no_derivation_yields_an_event(
) {
    let event = |text: &str| -> Option<String> { only(&read_session(text)).hook_event.clone() };
    assert_eq!(
        event(
            "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_success\", \"hookName\": \"PreToolUse:Bash\", \"hookEvent\": \"\", \"command\": \"g.sh\"}}"
        )
        .as_deref(),
        Some("PreToolUse")
    );
    assert_eq!(
        event(
            "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_success\", \"hookName\": \":Bash\", \"command\": \"g.sh\"}}"
        ),
        None
    );
    assert_eq!(
        event(
            "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_success\", \"hookName\": \"PostToolUse:Bash\", \"hookEvent\": 9}}"
        )
        .as_deref(),
        Some("PostToolUse")
    );
    assert_eq!(
        event(
            "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_success\", \"hookName\": \"PreToolUse:Bash\", \"hookEvent\": \"Stop\"}}"
        )
        .as_deref(),
        Some("Stop")
    );
    assert_eq!(
        event(
            "{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_success\", \"hookName\": \"PreToolUse\", \"exitCode\": 3}}"
        ),
        None
    );
    assert_eq!(
        event("{\"type\": \"attachment\", \"attachment\": {\"type\": \"hook_success\"}}"),
        None
    );
}
