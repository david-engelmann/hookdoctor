//! Ingestion of local hook configurations and session trace files.
//!
//! Both ingesters work on data already read from local files.
//! Nothing in this module touches a network or a path. This is the
//! Rust port of the Python ingestion module, the substrate the
//! examination consumes every input through.

use crate::errors::HookdoctorError;
use crate::jsonvalue::{parse_json, py_truthy, JsonValue, ParseFailure};
use crate::regexlite;

/// One configured hook entry flattened from its matcher group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookConfig {
    pub id: String,
    pub event: String,
    pub matcher: Option<String>,
    pub hook_type: String,
    pub command: String,
}

/// One dated record built from one piece of a session line.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceRecord {
    pub kind: String,
    pub timestamp: Option<String>,
    pub session: Option<String>,
    pub file_index: i64,
    pub line: i64,
    pub tool_name: Option<String>,
    pub call_id: Option<String>,
    pub is_error: bool,
    pub text: String,
    pub hook_name: Option<String>,
    pub hook_event: Option<String>,
    pub command: Option<String>,
    pub content: String,
    pub stdout: String,
    pub exit_code: Option<String>,
    pub attachment_type: Option<String>,
    pub commands: Vec<String>,
    pub prevented_continuation: bool,
    pub has_output: bool,
}

impl TraceRecord {
    fn new(
        kind: &str,
        timestamp: Option<String>,
        session: Option<String>,
        file_index: i64,
        line: i64,
    ) -> TraceRecord {
        TraceRecord {
            kind: kind.to_string(),
            timestamp,
            session,
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
}

const HOOK_ATTACHMENT_TYPES: [&str; 3] = [
    "hook_success",
    "hook_blocking_error",
    "hook_non_blocking_error",
];

fn text_field(value: Option<&JsonValue>) -> Option<String> {
    match value {
        Some(JsonValue::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

/// Flatten a settings document into hook records in document order.
///
/// Mirrors `parse_config` in the Python ingestion module.
pub fn parse_config(document: &JsonValue) -> Result<Vec<HookConfig>, HookdoctorError> {
    if document.as_object().is_none() {
        return Err(HookdoctorError::new(
            "the configuration is not a JSON object",
        ));
    }
    let hooks_value = document.get("hooks");
    if matches!(hooks_value, None | Some(JsonValue::Null)) {
        return Ok(Vec::new());
    }
    let hooks_object = match hooks_value.and_then(JsonValue::as_object) {
        Some(pairs) => pairs.clone(),
        None => return Err(HookdoctorError::new("the hooks value is not an object")),
    };
    let mut hooks = Vec::new();
    for (event, groups) in &hooks_object {
        let groups = match groups.as_list() {
            Some(items) => items.clone(),
            None => {
                return Err(HookdoctorError::new(
                    "a hooks event does not carry a list of groups",
                ))
            }
        };
        for (group_index, group) in groups.iter().enumerate() {
            if group.as_object().is_none() {
                return Err(HookdoctorError::new("a matcher group is not an object"));
            }
            let entries = match group.get("hooks").and_then(JsonValue::as_list) {
                Some(items) => items.clone(),
                None => {
                    return Err(HookdoctorError::new(
                        "a matcher group carries no hooks list",
                    ))
                }
            };
            let matcher = group.get("matcher");
            let matcher = match matcher {
                None | Some(JsonValue::Null) => None,
                Some(JsonValue::Text(text)) => Some(text.clone()),
                Some(_) => {
                    return Err(HookdoctorError::new("a matcher is not a string"));
                }
            };
            if let Some(pattern) = &matcher {
                if !pattern.is_empty() && !regexlite::pattern_compiles(pattern) {
                    return Err(HookdoctorError::new("a matcher is not a valid pattern"));
                }
            }
            for (entry_index, entry) in entries.iter().enumerate() {
                if entry.as_object().is_none() {
                    return Err(HookdoctorError::new("a hook entry is not an object"));
                }
                let command = match entry.get("command") {
                    Some(JsonValue::Text(text)) if !text.is_empty() => text.clone(),
                    _ => return Err(HookdoctorError::new("a hook entry carries no command")),
                };
                let hook_type = match entry.get("type") {
                    None => "command".to_string(),
                    Some(JsonValue::Text(text)) if !text.is_empty() => text.clone(),
                    _ => return Err(HookdoctorError::new("a hook entry carries no type")),
                };
                hooks.push(HookConfig {
                    id: format!("{event}:{group_index}:{entry_index}"),
                    event: event.clone(),
                    matcher: matcher.clone(),
                    hook_type,
                    command,
                });
            }
        }
    }
    Ok(hooks)
}

fn text_from_result_content(content: Option<&JsonValue>) -> String {
    match content {
        Some(JsonValue::Text(text)) => text.clone(),
        Some(JsonValue::List(blocks)) => {
            let mut parts: Vec<String> = Vec::new();
            for block in blocks {
                if block.is_object()
                    && block.get("type") == Some(&JsonValue::Text("text".to_string()))
                {
                    if let Some(text) = block.get("text").and_then(JsonValue::as_text) {
                        parts.push(text.to_string());
                    }
                }
            }
            parts.join("\n")
        }
        _ => String::new(),
    }
}

/// Whether a character ends a line under the Python splitlines
/// reading.
fn is_line_boundary(ch: char) -> bool {
    matches!(
        ch,
        '\n' | '\r'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{001C}'
            | '\u{001D}'
            | '\u{001E}'
            | '\u{0085}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

/// Split a text into lines the way the Python splitlines does,
/// with a carriage return and line feed pair counting once.
pub fn py_splitlines(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if is_line_boundary(ch) {
            if ch == '\r' && chars.get(index + 1) == Some(&'\n') {
                index += 1;
            }
            lines.push(std::mem::take(&mut current));
            index += 1;
        } else {
            current.push(ch);
            index += 1;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Whether the Python strip reading clears a line away.
fn line_is_blank(line: &str) -> bool {
    line.chars().all(regexlite::py_is_space)
}

/// Build dated trace records from the text of one session file.
///
/// Mirrors `parse_session_text` in the Python ingestion module,
/// with each line decoded under the duplicate refusal of obligation
/// R2-1, as the amended Python decodes it.
pub fn parse_session_text(
    text: &str,
    file_index: i64,
) -> Result<Vec<TraceRecord>, HookdoctorError> {
    let mut records: Vec<TraceRecord> = Vec::new();
    for (offset, raw) in py_splitlines(text).iter().enumerate() {
        let line_number = (offset + 1) as i64;
        if line_is_blank(raw) {
            continue;
        }
        let line_object = match parse_json(raw) {
            Ok(value) => value,
            Err(ParseFailure::DuplicateKey(key)) => {
                return Err(HookdoctorError::new(format!(
                    "a session line carries the key {key} more than once"
                )))
            }
            Err(ParseFailure::Invalid) => {
                return Err(HookdoctorError::new("a session line is not valid JSON"))
            }
        };
        let line_pairs = match line_object.as_object() {
            Some(pairs) => pairs,
            None => return Err(HookdoctorError::new("a session line carries no type")),
        };
        if !line_pairs.iter().any(|(name, _)| name == "type") {
            return Err(HookdoctorError::new("a session line carries no type"));
        }
        let timestamp = text_field(line_object.get("timestamp"));
        let session = text_field(line_object.get("sessionId"));
        let line_type = line_object.get("type").cloned().unwrap_or(JsonValue::Null);

        let push = |records: &mut Vec<TraceRecord>, record: TraceRecord| {
            records.push(record);
        };

        if line_type == JsonValue::Text("assistant".to_string()) {
            let message = line_object.get("message");
            let content = match message {
                Some(object) if object.is_object() => object.get("content"),
                _ => None,
            };
            let mut placed = false;
            if let Some(JsonValue::List(blocks)) = content {
                for block in blocks {
                    if block.is_object()
                        && block.get("type") == Some(&JsonValue::Text("tool_use".to_string()))
                        && block.get("name").and_then(JsonValue::as_text).is_some()
                        && block.get("id").and_then(JsonValue::as_text).is_some()
                    {
                        let mut record = TraceRecord::new(
                            "tool_call",
                            timestamp.clone(),
                            session.clone(),
                            file_index,
                            line_number,
                        );
                        record.tool_name = Some(
                            block
                                .get("name")
                                .and_then(JsonValue::as_text)
                                .unwrap()
                                .to_string(),
                        );
                        record.call_id = Some(
                            block
                                .get("id")
                                .and_then(JsonValue::as_text)
                                .unwrap()
                                .to_string(),
                        );
                        push(&mut records, record);
                        placed = true;
                    }
                }
            }
            if !placed {
                push(
                    &mut records,
                    TraceRecord::new("other", timestamp, session, file_index, line_number),
                );
            }
        } else if line_type == JsonValue::Text("user".to_string()) {
            let message = line_object.get("message");
            let content = match message {
                Some(object) if object.is_object() => object.get("content"),
                _ => None,
            };
            match content {
                Some(JsonValue::Text(text)) => {
                    let mut record =
                        TraceRecord::new("prompt", timestamp, session, file_index, line_number);
                    record.text = text.clone();
                    push(&mut records, record);
                }
                Some(JsonValue::List(blocks)) => {
                    let mut placed = false;
                    let mut prompt_texts: Vec<String> = Vec::new();
                    for block in blocks {
                        if !block.is_object() {
                            continue;
                        }
                        if block.get("type") == Some(&JsonValue::Text("tool_result".to_string()))
                            && block
                                .get("tool_use_id")
                                .and_then(JsonValue::as_text)
                                .is_some()
                        {
                            let mut record = TraceRecord::new(
                                "tool_outcome",
                                timestamp.clone(),
                                session.clone(),
                                file_index,
                                line_number,
                            );
                            record.call_id = Some(
                                block
                                    .get("tool_use_id")
                                    .and_then(JsonValue::as_text)
                                    .unwrap()
                                    .to_string(),
                            );
                            record.is_error = py_truthy(block.get("is_error"));
                            record.text = text_from_result_content(block.get("content"));
                            push(&mut records, record);
                            placed = true;
                        } else if block.get("type") == Some(&JsonValue::Text("text".to_string()))
                            && block.get("text").and_then(JsonValue::as_text).is_some()
                        {
                            prompt_texts.push(
                                block
                                    .get("text")
                                    .and_then(JsonValue::as_text)
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }
                    if !prompt_texts.is_empty() {
                        let mut record = TraceRecord::new(
                            "prompt",
                            timestamp.clone(),
                            session.clone(),
                            file_index,
                            line_number,
                        );
                        record.text = prompt_texts.join("\n");
                        push(&mut records, record);
                        placed = true;
                    }
                    if !placed {
                        push(
                            &mut records,
                            TraceRecord::new("other", timestamp, session, file_index, line_number),
                        );
                    }
                }
                _ => {
                    push(
                        &mut records,
                        TraceRecord::new("other", timestamp, session, file_index, line_number),
                    );
                }
            }
        } else if line_type == JsonValue::Text("attachment".to_string()) {
            let attachment = line_object.get("attachment");
            let attachment_type = match attachment {
                Some(object) if object.is_object() => text_field(object.get("type")),
                _ => None,
            };
            let is_hook = matches!(
                &attachment_type,
                Some(name) if HOOK_ATTACHMENT_TYPES.contains(&name.as_str())
            );
            if is_hook {
                let attachment = attachment.expect("attachment object stands");
                let hook_name = text_field(attachment.get("hookName"));
                let mut hook_event = text_field(attachment.get("hookEvent"));
                if hook_event.is_none() {
                    if let Some(name) = &hook_name {
                        if !name.is_empty() && name.contains(':') {
                            hook_event = Some(name.split(':').next().unwrap_or("").to_string());
                        }
                    }
                }
                let mut record =
                    TraceRecord::new("hook_run", timestamp, session, file_index, line_number);
                record.call_id = text_field(attachment.get("toolUseID"));
                record.hook_name = hook_name;
                record.hook_event = hook_event;
                record.command = text_field(attachment.get("command"));
                record.content = text_field(attachment.get("content")).unwrap_or_default();
                record.stdout = text_field(attachment.get("stdout")).unwrap_or_default();
                record.exit_code = match attachment.get("exitCode") {
                    Some(JsonValue::Int(integer)) => Some(integer.to_string()),
                    Some(JsonValue::BigInt(text)) => Some(text.clone()),
                    _ => None,
                };
                record.attachment_type = attachment_type;
                push(&mut records, record);
            } else {
                push(
                    &mut records,
                    TraceRecord::new("other", timestamp, session, file_index, line_number),
                );
            }
        } else if line_type == JsonValue::Text("system".to_string())
            && line_object.get("subtype") == Some(&JsonValue::Text("stop_hook_summary".to_string()))
        {
            let mut commands: Vec<String> = Vec::new();
            if let Some(JsonValue::List(infos)) = line_object.get("hookInfos") {
                for info in infos {
                    if info.is_object() {
                        if let Some(command) = info.get("command").and_then(JsonValue::as_text) {
                            commands.push(command.to_string());
                        }
                    }
                }
            }
            let mut record = TraceRecord::new("stop", timestamp, session, file_index, line_number);
            record.commands = commands;
            record.prevented_continuation = py_truthy(line_object.get("preventedContinuation"));
            record.has_output = py_truthy(line_object.get("hasOutput"));
            push(&mut records, record);
        } else {
            push(
                &mut records,
                TraceRecord::new("other", timestamp, session, file_index, line_number),
            );
        }
    }
    Ok(records)
}
