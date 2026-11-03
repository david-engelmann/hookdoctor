//! The examination that classifies configured hooks against a
//! trace. This is the Rust port of the Python examination module.

use std::collections::{HashMap, HashSet};

use crate::dates::{earliest_instant, parse_instant};
use crate::errors::{HookdoctorError, ReportValidationError};
use crate::ingest::{HookConfig, TraceRecord};
use crate::jsonvalue::JsonValue;
use crate::reader::parse_output_text;
use crate::regexlite::{self, is_word_char};

pub const REPORT_FORMAT: &str = "hookdoctor-report/1";

const CLASS_FIRING: &str = "firing";
const CLASS_SILENT: &str = "silent";
const CLASS_WITHOUT_EFFECT: &str = "firing-without-effect";

/// Whether a matcher is a plain name under the probed rule.
pub fn is_plain_matcher(matcher: &str) -> bool {
    !matcher.is_empty()
        && matcher
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Apply the probed harness matcher rule to one tool name.
///
/// Mirrors `matcher_matches` in the Python examination module.
pub fn matcher_matches(
    matcher: Option<&str>,
    tool_name: Option<&str>,
) -> Result<bool, HookdoctorError> {
    let tool_name = match tool_name {
        None => return Ok(false),
        Some(name) => name,
    };
    match matcher {
        None => Ok(true),
        Some("") | Some("*") => Ok(true),
        Some(pattern) => {
            if is_plain_matcher(pattern) {
                Ok(pattern == tool_name)
            } else {
                regexlite::fullmatch(pattern, tool_name)
                    .map_err(|_| HookdoctorError::new("a matcher is not a valid pattern"))
            }
        }
    }
}

fn window_bound(text: Option<&str>, label: &str) -> Result<Option<i128>, HookdoctorError> {
    match text {
        None => Ok(None),
        Some(value) => match parse_instant(value) {
            Some(instant) => Ok(Some(instant)),
            None => Err(HookdoctorError::new(format!(
                "the window {label} is not a timestamp"
            ))),
        },
    }
}

fn in_window(record: &TraceRecord, start: Option<i128>, end: Option<i128>) -> bool {
    if start.is_none() && end.is_none() {
        return true;
    }
    let instant = match record.timestamp.as_deref().and_then(parse_instant) {
        Some(instant) => instant,
        None => return false,
    };
    if let Some(start) = start {
        if instant < start {
            return false;
        }
    }
    if let Some(end) = end {
        if instant >= end {
            return false;
        }
    }
    true
}

#[derive(Default, Clone, Copy)]
struct OutputFacts {
    denial: bool,
    modification: bool,
    consumed_output: bool,
}

const ERROR_ATTACHMENT_TYPES: [&str; 2] = ["hook_blocking_error", "hook_non_blocking_error"];

fn output_facts(record: &TraceRecord) -> OutputFacts {
    let mut facts = OutputFacts::default();
    let error_record = matches!(
        &record.attachment_type,
        Some(name) if ERROR_ATTACHMENT_TYPES.contains(&name.as_str())
    );
    if record.attachment_type.as_deref() == Some("hook_blocking_error") {
        facts.denial = true;
    }
    if !record.content.is_empty() && !error_record {
        facts.consumed_output = true;
    }
    for text in [&record.content, &record.stdout] {
        let output = match parse_output_text(text) {
            Some(value) => value,
            None => continue,
        };
        let mut candidates = vec![output.clone()];
        if let Some(specific) = output.get("hookSpecificOutput") {
            if specific.is_object() {
                candidates.push(specific.clone());
            }
        }
        for candidate in &candidates {
            if candidate.get("decision") == Some(&JsonValue::Text("block".to_string())) {
                facts.denial = true;
            }
            if candidate.get("permissionDecision") == Some(&JsonValue::Text("deny".to_string())) {
                facts.denial = true;
            }
            if candidate.get("updatedInput").is_some() {
                facts.modification = true;
            }
            if let Some(JsonValue::Text(text)) = candidate.get("additionalContext") {
                if !text.is_empty() && !error_record {
                    facts.consumed_output = true;
                }
            }
        }
    }
    facts
}

/// Whether the character stands in the Python whitespace reading.
fn is_space(ch: char) -> bool {
    regexlite::py_is_space(ch)
}

/// The word bounded occurrences of the denial word in a lowered
/// text, as (start, end) character positions.
fn denial_word_spans(chars: &[char]) -> Vec<(usize, usize)> {
    let word: Vec<char> = "denied".chars().collect();
    let mut spans = Vec::new();
    if chars.len() < word.len() {
        return spans;
    }
    for start in 0..=(chars.len() - word.len()) {
        if chars[start..start + word.len()] != word[..] {
            continue;
        }
        let before = start == 0 || !is_word_char(chars[start - 1]);
        let after = start + word.len() == chars.len() || !is_word_char(chars[start + word.len()]);
        if before && after {
            spans.push((start, start + word.len()));
        }
    }
    spans
}

fn denial_word_present(lowered: &str) -> bool {
    let chars: Vec<char> = lowered.chars().collect();
    !denial_word_spans(&chars).is_empty()
}

/// The negated denial reading, matched against a lowered text the
/// way the fixed negated pattern reads it.
fn negated_denial_present(chars: &[char]) -> bool {
    for (start, _end) in denial_word_spans(chars) {
        // The text before the denial word must end with one of the
        // negation forms over a run of whitespace.
        let mut head = start;
        let mut spaces = 0;
        while head > 0 && is_space(chars[head - 1]) {
            head -= 1;
            spaces += 1;
        }
        if spaces == 0 {
            continue;
        }
        let before = &chars[..head];
        let ends_with = |tail: &[char]| -> bool {
            before.len() >= tail.len() && &before[before.len() - tail.len()..] == tail
        };
        let boundary_before = |tail: &[char]| -> bool {
            let start_at = before.len() - tail.len();
            start_at == 0 || !is_word_char(before[start_at - 1])
        };
        let not_tail: Vec<char> = "not".chars().collect();
        if ends_with(&not_tail) && boundary_before(&not_tail) {
            return true;
        }
        let never_tail: Vec<char> = "never".chars().collect();
        if ends_with(&never_tail) && boundary_before(&never_tail) {
            return true;
        }
        let nt_straight: Vec<char> = "n't".chars().collect();
        let nt_curly: Vec<char> = "n\u{2019}t".chars().collect();
        if ends_with(&nt_straight) || ends_with(&nt_curly) {
            return true;
        }
    }
    false
}

/// The name shaped party a denial is attributed to, or None.
///
/// Mirrors `_denial_attribution` in the Python examination module.
fn denial_attribution(chars: &[char]) -> Option<String> {
    for start in 0..chars.len() {
        let word: Vec<char> = "denied".chars().collect();
        if start + word.len() > chars.len() {
            break;
        }
        if chars[start..start + word.len()] != word[..] {
            continue;
        }
        if start > 0 && is_word_char(chars[start - 1]) {
            continue;
        }
        let mut pos = start + word.len();
        let mut spaces = 0;
        while pos < chars.len() && is_space(chars[pos]) {
            pos += 1;
            spaces += 1;
        }
        if spaces == 0 {
            continue;
        }
        if chars[pos..].len() < 2 || chars[pos] != 'b' || chars[pos + 1] != 'y' {
            continue;
        }
        pos += 2;
        let mut spaces = 0;
        while pos < chars.len() && is_space(chars[pos]) {
            pos += 1;
            spaces += 1;
        }
        if spaces == 0 {
            continue;
        }
        let party_start = pos;
        while pos < chars.len()
            && (chars[pos].is_ascii_lowercase()
                || chars[pos].is_ascii_digit()
                || matches!(chars[pos], '_' | '.' | '-' | '/'))
        {
            pos += 1;
        }
        if pos == party_start {
            continue;
        }
        let party: String = chars[party_start..pos].iter().collect();
        if party.chars().any(|ch| matches!(ch, '-' | '.' | '_' | '/')) {
            return Some(party);
        }
        return None;
    }
    None
}

/// Apply the denial reading of HD2-5 to one outcome text.
fn denial_by_outcome_text(hook: &HookConfig, text: &str) -> bool {
    let lowered = text.to_lowercase();
    let lowered_chars: Vec<char> = lowered.chars().collect();
    if negated_denial_present(&lowered_chars) {
        return false;
    }
    let attribution = denial_attribution(&lowered_chars);
    if text.contains(&hook.command) {
        return attribution.is_none()
            || attribution.as_deref() == Some(hook.command.to_lowercase().as_str());
    }
    if !denial_word_present(&lowered) {
        return false;
    }
    attribution.is_none()
}

/// Decide whether a run record name is this hook own name form.
fn name_form_agrees(hook: &HookConfig, name: &Option<String>) -> Option<bool> {
    let name = match name {
        None => return None,
        Some(value) if value.is_empty() => return None,
        Some(value) => value,
    };
    if *name == hook.command {
        return Some(true);
    }
    if *name == hook.event {
        return Some(hook.matcher.is_none() || hook.matcher.as_deref() == Some(""));
    }
    let prefix = format!("{}:", hook.event);
    if let Some(suffix) = name.strip_prefix(&prefix) {
        return Some(
            suffix == hook.matcher.as_deref().unwrap_or("") || suffix == hook.command.as_str(),
        );
    }
    Some(false)
}

/// Decide whether a hook run record identifies this hook.
fn run_agrees(hook: &HookConfig, record: &TraceRecord) -> bool {
    if record.hook_event.as_deref() != Some(hook.event.as_str()) {
        return false;
    }
    let mut verdicts: Vec<bool> = Vec::new();
    if let Some(command) = &record.command {
        if !command.is_empty() {
            verdicts.push(*command == hook.command);
        }
    }
    if let Some(verdict) = name_form_agrees(hook, &record.hook_name) {
        verdicts.push(verdict);
    }
    !verdicts.is_empty() && verdicts.iter().all(|verdict| *verdict)
}

fn levenshtein(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    if left == right {
        return 0;
    }
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (i, a) in left.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, b) in right.iter().enumerate() {
            current.push(std::cmp::min(
                std::cmp::min(previous[j + 1] + 1, current[j] + 1),
                previous[j] + if a == b { 0 } else { 1 },
            ));
        }
        previous = current;
    }
    previous[right.len()]
}

fn name_form_key(name: &str) -> String {
    name.to_lowercase().replace(['-', '_'], "")
}

struct Subject {
    event: String,
    tool_name: Option<String>,
    record_index: usize,
    outcome_index: Option<usize>,
}

fn subjects(records: &[TraceRecord]) -> Vec<Subject> {
    let mut outcome_queues: HashMap<Option<String>, Vec<usize>> = HashMap::new();
    for (index, record) in records.iter().enumerate() {
        if record.kind == "tool_outcome" {
            outcome_queues
                .entry(record.call_id.clone())
                .or_default()
                .push(index);
        }
    }
    let mut consumed: HashMap<Option<String>, usize> = HashMap::new();
    let mut built: Vec<Subject> = Vec::new();
    for (index, record) in records.iter().enumerate() {
        match record.kind.as_str() {
            "tool_call" => {
                let queue = outcome_queues.get(&record.call_id);
                let position = consumed.get(&record.call_id).copied().unwrap_or(0);
                let outcome_index = queue.and_then(|items| items.get(position).copied());
                if outcome_index.is_some() {
                    consumed.insert(record.call_id.clone(), position + 1);
                }
                built.push(Subject {
                    event: "PreToolUse".to_string(),
                    tool_name: record.tool_name.clone(),
                    record_index: index,
                    outcome_index,
                });
                if let Some(outcome) = outcome_index {
                    built.push(Subject {
                        event: "PostToolUse".to_string(),
                        tool_name: record.tool_name.clone(),
                        record_index: outcome,
                        outcome_index: Some(outcome),
                    });
                }
            }
            "prompt" => built.push(Subject {
                event: "UserPromptSubmit".to_string(),
                tool_name: None,
                record_index: index,
                outcome_index: None,
            }),
            "stop" => built.push(Subject {
                event: "Stop".to_string(),
                tool_name: None,
                record_index: index,
                outcome_index: None,
            }),
            "hook_run" => {
                if let Some(event) = &record.hook_event {
                    built.push(Subject {
                        event: event.clone(),
                        tool_name: None,
                        record_index: index,
                        outcome_index: None,
                    });
                }
            }
            _ => {}
        }
    }
    built
}

fn subject_matches(
    hook: &HookConfig,
    subject: &Subject,
    records: &[TraceRecord],
) -> Result<bool, HookdoctorError> {
    if subject.event != hook.event {
        return Ok(false);
    }
    let record = &records[subject.record_index];
    if record.kind == "hook_run" {
        return Ok(run_agrees(hook, record));
    }
    if subject.tool_name.is_none() {
        return Ok(true);
    }
    matcher_matches(hook.matcher.as_deref(), subject.tool_name.as_deref())
}

fn subject_ran(hook: &HookConfig, subject: &Subject, records: &[TraceRecord]) -> bool {
    let record = &records[subject.record_index];
    if record.kind == "hook_run" {
        return run_agrees(hook, record);
    }
    if subject.event == "Stop" {
        return record.commands.contains(&hook.command);
    }
    true
}

#[derive(Default, Clone, Copy)]
struct EffectCounts {
    denial: i64,
    modification: i64,
    consumed_output: i64,
}

impl EffectCounts {
    fn total(&self) -> i64 {
        self.denial + self.modification + self.consumed_output
    }

    fn add(&mut self, other: &EffectCounts) {
        self.denial += other.denial;
        self.modification += other.modification;
        self.consumed_output += other.consumed_output;
    }

    fn as_value(&self) -> JsonValue {
        JsonValue::object(vec![
            ("denial", JsonValue::Int(self.denial)),
            ("modification", JsonValue::Int(self.modification)),
            ("consumed_output", JsonValue::Int(self.consumed_output)),
        ])
    }
}

fn subject_effects(hook: &HookConfig, subject: &Subject, records: &[TraceRecord]) -> EffectCounts {
    let mut effects = EffectCounts::default();
    let record = &records[subject.record_index];
    if record.kind == "hook_run" {
        let facts = output_facts(record);
        if facts.denial {
            effects.denial = 1;
        }
        if facts.modification {
            effects.modification = 1;
        }
        if facts.consumed_output {
            effects.consumed_output = 1;
        }
        return effects;
    }
    if record.kind == "stop" {
        if record.prevented_continuation {
            effects.denial = 1;
        }
        if record.has_output {
            effects.consumed_output = 1;
        }
        return effects;
    }
    if subject.event == "PreToolUse" && record.kind == "tool_call" {
        if let Some(outcome_index) = subject.outcome_index {
            let outcome = &records[outcome_index];
            if outcome.is_error && denial_by_outcome_text(hook, &outcome.text) {
                effects.denial = 1;
            }
        }
        return effects;
    }
    effects
}

fn near_miss_reason(
    hook: &HookConfig,
    subject: &Subject,
) -> Result<Option<&'static str>, HookdoctorError> {
    if subject.event == hook.event {
        if let Some(tool) = &subject.tool_name {
            if let Some(matcher) = &hook.matcher {
                if is_plain_matcher(matcher) {
                    if tool != matcher && name_form_key(tool) == name_form_key(matcher) {
                        return Ok(Some("name-form"));
                    }
                    if levenshtein(matcher, tool) == 1 {
                        return Ok(Some("one-letter"));
                    }
                    return Ok(None);
                }
            }
        }
    }
    let paired = match hook.event.as_str() {
        "PreToolUse" => Some("PostToolUse"),
        "PostToolUse" => Some("PreToolUse"),
        _ => None,
    };
    if let Some(paired) = paired {
        if subject.event == paired {
            if let Some(_tool) = &subject.tool_name {
                if matcher_matches(hook.matcher.as_deref(), subject.tool_name.as_deref())? {
                    return Ok(Some("paired-event"));
                }
            }
        }
    }
    if subject.event == format!("Subagent{}", hook.event)
        || hook.event == format!("Subagent{}", subject.event)
    {
        return Ok(Some("subagent-prefix"));
    }
    Ok(None)
}

fn near_miss_detail(reason: &str, hook: &HookConfig, subject: &Subject) -> String {
    let tool = subject.tool_name.as_deref().unwrap_or("no tool");
    let matcher = hook.matcher.as_deref().unwrap_or("");
    match reason {
        "name-form" => format!(
            "tool {tool} differs from matcher {matcher} by letter case or separator form only"
        ),
        "one-letter" => format!("tool {tool} is one letter away from matcher {matcher}"),
        "paired-event" => format!(
            "event {} is the paired event of {} for tool {tool} the matcher matches",
            subject.event, hook.event
        ),
        _ => format!(
            "event {} differs from {} by the Subagent prefix",
            subject.event, hook.event
        ),
    }
}

/// Return one entry per hook and the near miss list for silent
/// hooks, as JSON values in the report shape.
///
/// Mirrors `examine_hooks` in the Python examination module.
pub fn examine_hooks(
    hooks: &[HookConfig],
    records: &[TraceRecord],
    window_start: Option<&str>,
    window_end: Option<&str>,
    session_sources: &[String],
) -> Result<(Vec<JsonValue>, Vec<JsonValue>), HookdoctorError> {
    let start_instant = window_bound(window_start, "start")?;
    let end_instant = window_bound(window_end, "end")?;
    let windowed: Vec<TraceRecord> = records
        .iter()
        .filter(|record| in_window(record, start_instant, end_instant))
        .cloned()
        .collect();
    let subjects = subjects(&windowed);

    let source_of = |record: &TraceRecord| -> Option<String> {
        let index = record.file_index;
        if index >= 0 && (index as usize) < session_sources.len() {
            Some(session_sources[index as usize].clone())
        } else {
            record.session.clone()
        }
    };

    let mut entries: Vec<JsonValue> = Vec::new();
    let mut near_misses: Vec<JsonValue> = Vec::new();
    for hook in hooks {
        let mut matched: i64 = 0;
        let mut ran_subjects: Vec<&Subject> = Vec::new();
        let mut effect_counts = EffectCounts::default();
        for subject in &subjects {
            if !subject_matches(hook, subject, &windowed)? {
                continue;
            }
            matched += 1;
            if !subject_ran(hook, subject, &windowed) {
                continue;
            }
            ran_subjects.push(subject);
            let effects = subject_effects(hook, subject, &windowed);
            effect_counts.add(&effects);
        }
        let hook_class = if ran_subjects.is_empty() {
            CLASS_SILENT
        } else if effect_counts.total() > 0 {
            CLASS_FIRING
        } else {
            CLASS_WITHOUT_EFFECT
        };
        let mut stamps: Vec<(i128, String)> = Vec::new();
        for subject in &ran_subjects {
            let record = &windowed[subject.record_index];
            if let Some(text) = &record.timestamp {
                if let Some(instant) = parse_instant(text) {
                    stamps.push((instant, text.clone()));
                }
            }
        }
        let first = stamps.iter().min().map(|(_, text)| text.clone());
        let last = stamps.iter().max().map(|(_, text)| text.clone());
        entries.push(JsonValue::object(vec![
            ("id", JsonValue::text(hook.id.clone())),
            ("event", JsonValue::text(hook.event.clone())),
            ("matcher", JsonValue::optional_text(&hook.matcher)),
            ("command", JsonValue::text(hook.command.clone())),
            ("hook_type", JsonValue::text(hook.hook_type.clone())),
            ("class", JsonValue::text(hook_class)),
            (
                "evidence",
                JsonValue::object(vec![
                    ("matched", JsonValue::Int(matched)),
                    ("ran", JsonValue::Int(ran_subjects.len() as i64)),
                    ("effects", effect_counts.as_value()),
                    ("first_ran_at", JsonValue::optional_text(&first)),
                    ("last_ran_at", JsonValue::optional_text(&last)),
                ]),
            ),
        ]));
        if hook_class == CLASS_SILENT {
            let mut seen: HashSet<(String, String, Option<String>, i64, i64)> = HashSet::new();
            for subject in &subjects {
                let reason = match near_miss_reason(hook, subject)? {
                    Some(reason) => reason,
                    None => continue,
                };
                let record = &windowed[subject.record_index];
                let key = (
                    reason.to_string(),
                    subject.event.clone(),
                    subject.tool_name.clone(),
                    record.file_index,
                    record.line,
                );
                if !seen.insert(key) {
                    continue;
                }
                near_misses.push(JsonValue::object(vec![
                    ("hook_id", JsonValue::text(hook.id.clone())),
                    ("reason", JsonValue::text(reason)),
                    ("event", JsonValue::text(subject.event.clone())),
                    ("tool", JsonValue::optional_text(&subject.tool_name)),
                    ("timestamp", JsonValue::optional_text(&record.timestamp)),
                    ("source", JsonValue::optional_text(&source_of(record))),
                    ("line", JsonValue::Int(record.line)),
                    (
                        "detail",
                        JsonValue::text(near_miss_detail(reason, hook, subject)),
                    ),
                ]));
            }
        }
    }
    let order: HashMap<String, usize> = hooks
        .iter()
        .enumerate()
        .map(|(index, hook)| (hook.id.clone(), index))
        .collect();
    let earliest = earliest_instant();
    near_misses.sort_by(|left, right| {
        let key = |miss: &JsonValue| -> (usize, i128, String, i64) {
            let hook_order = order
                .get(
                    miss.get("hook_id")
                        .and_then(JsonValue::as_text)
                        .unwrap_or(""),
                )
                .copied()
                .unwrap_or(usize::MAX);
            let instant = miss
                .get("timestamp")
                .and_then(JsonValue::as_text)
                .and_then(parse_instant)
                .unwrap_or(earliest);
            let source = miss
                .get("source")
                .and_then(JsonValue::as_text)
                .unwrap_or("")
                .to_string();
            let line = match miss.get("line") {
                Some(JsonValue::Int(value)) => *value,
                _ => 0,
            };
            (hook_order, instant, source, line)
        };
        key(left).cmp(&key(right))
    });
    Ok((entries, near_misses))
}

/// Build the hookdoctor-report/1 document.
///
/// Mirrors `build_report` in the Python examination module.
pub fn build_report(
    hooks: &[HookConfig],
    records: &[TraceRecord],
    config_source: &str,
    session_sources: &[String],
    window_start: Option<&str>,
    window_end: Option<&str>,
) -> Result<JsonValue, HookdoctorError> {
    let (entries, near_misses) =
        examine_hooks(hooks, records, window_start, window_end, session_sources)?;
    Ok(JsonValue::object(vec![
        ("format", JsonValue::text(REPORT_FORMAT)),
        ("config_source", JsonValue::text(config_source)),
        (
            "session_sources",
            JsonValue::List(
                session_sources
                    .iter()
                    .map(|source| JsonValue::text(source.clone()))
                    .collect(),
            ),
        ),
        (
            "window",
            JsonValue::object(vec![
                (
                    "start",
                    JsonValue::optional_text(&window_start.map(str::to_string)),
                ),
                (
                    "end",
                    JsonValue::optional_text(&window_end.map(str::to_string)),
                ),
            ]),
        ),
        ("hooks", JsonValue::List(entries)),
        ("near_misses", JsonValue::List(near_misses)),
    ]))
}

/// Rebuild the expected report and refuse a supplied report that
/// differs in any field.
///
/// Mirrors `validate_report` in the Python examination module.
pub fn validate_report(
    report: &JsonValue,
    hooks: &[HookConfig],
    records: &[TraceRecord],
    config_source: &str,
    session_sources: &[String],
    window_start: Option<&str>,
    window_end: Option<&str>,
) -> Result<bool, ReportValidationError> {
    let expected = build_report(
        hooks,
        records,
        config_source,
        session_sources,
        window_start,
        window_end,
    )
    .map_err(|error| ReportValidationError::new(error.message().to_string()))?;
    if *report != expected {
        return Err(ReportValidationError::new(
            "the report disagrees with the examination",
        ));
    }
    Ok(true)
}
