//! Recorded conformance for the Rust port slice 1.
//!
//! Every expectation in tests/conformance/port1_cases.json was
//! recorded from the landed Python by the probe generator. The
//! recordings are never hand edited. This suite recreates the
//! inputs and compares the crate's answers against them.

use hookdoctor::dates::parse_instant;
use hookdoctor::examine::{build_report, matcher_matches, validate_report};
use hookdoctor::ingest::{parse_config, parse_session_text, HookConfig, TraceRecord};
use hookdoctor::jsonvalue::{parse_json, JsonValue, ParseFailure};
use hookdoctor::reader::read_json_document;
use hookdoctor::render::render_json;
use hookdoctor::report::render_markdown;

fn recordings() -> JsonValue {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/conformance/port1_cases.json"
    );
    let text = std::fs::read_to_string(path).expect("recordings readable");
    parse_json(&text).expect("recordings parse")
}

fn section<'a>(root: &'a JsonValue, name: &str) -> &'a [JsonValue] {
    root.get(name)
        .and_then(JsonValue::as_list)
        .expect("section stands")
}

fn text_of(value: &JsonValue, key: &str) -> String {
    value
        .get(key)
        .and_then(JsonValue::as_text)
        .unwrap_or_else(|| panic!("case carries text field {key}"))
        .to_string()
}

fn outcome_of(value: &JsonValue) -> String {
    text_of(value.get("answer").unwrap_or(value), "outcome")
}

fn answer_of(case: &JsonValue) -> &JsonValue {
    case.get("answer").expect("case carries an answer")
}

fn hook_value(hook: &HookConfig) -> JsonValue {
    JsonValue::object(vec![
        ("id", JsonValue::text(hook.id.clone())),
        ("event", JsonValue::text(hook.event.clone())),
        ("matcher", JsonValue::optional_text(&hook.matcher)),
        ("hook_type", JsonValue::text(hook.hook_type.clone())),
        ("command", JsonValue::text(hook.command.clone())),
    ])
}

fn record_value(record: &TraceRecord) -> JsonValue {
    let exit_code = match &record.exit_code {
        None => JsonValue::Null,
        Some(text) => match text.parse::<i64>() {
            Ok(integer) => JsonValue::Int(integer),
            Err(_) => JsonValue::BigInt(text.clone()),
        },
    };
    JsonValue::object(vec![
        ("kind", JsonValue::text(record.kind.clone())),
        ("timestamp", JsonValue::optional_text(&record.timestamp)),
        ("session", JsonValue::optional_text(&record.session)),
        ("file_index", JsonValue::Int(record.file_index)),
        ("line", JsonValue::Int(record.line)),
        ("tool_name", JsonValue::optional_text(&record.tool_name)),
        ("call_id", JsonValue::optional_text(&record.call_id)),
        ("is_error", JsonValue::Bool(record.is_error)),
        ("text", JsonValue::text(record.text.clone())),
        ("hook_name", JsonValue::optional_text(&record.hook_name)),
        ("hook_event", JsonValue::optional_text(&record.hook_event)),
        ("command", JsonValue::optional_text(&record.command)),
        ("content", JsonValue::text(record.content.clone())),
        ("stdout", JsonValue::text(record.stdout.clone())),
        ("exit_code", exit_code),
        (
            "attachment_type",
            JsonValue::optional_text(&record.attachment_type),
        ),
        (
            "commands",
            JsonValue::List(
                record
                    .commands
                    .iter()
                    .map(|command| JsonValue::text(command.clone()))
                    .collect(),
            ),
        ),
        (
            "prevented_continuation",
            JsonValue::Bool(record.prevented_continuation),
        ),
        ("has_output", JsonValue::Bool(record.has_output)),
    ])
}

fn depth_of(value: &JsonValue) -> usize {
    match value {
        JsonValue::List(items) => 1 + items.iter().map(depth_of).max().unwrap_or(0),
        _ => 0,
    }
}

fn check_reader(root: &JsonValue) -> usize {
    let mut count = 0;
    for case in section(root, "reader") {
        count += 1;
        let input = text_of(case, "input");
        let answer = answer_of(case);
        match outcome_of(case).as_str() {
            "value" => {
                let value =
                    parse_json(&input).unwrap_or_else(|_| panic!("reader case refused: {input:?}"));
                assert_eq!(
                    render_json(&value),
                    text_of(answer, "rendered"),
                    "reader case {input:?}"
                );
            }
            "duplicate" => match parse_json(&input) {
                Err(ParseFailure::DuplicateKey(key)) => {
                    assert_eq!(key, text_of(answer, "key"), "reader dup case {input:?}")
                }
                other => panic!("reader dup case {input:?} answered {other:?}"),
            },
            "invalid" => {
                assert_eq!(
                    parse_json(&input),
                    Err(ParseFailure::Invalid),
                    "reader invalid case {input:?}"
                );
            }
            other => panic!("unknown reader outcome {other}"),
        }
    }
    count
}

fn check_boundary(root: &JsonValue) -> usize {
    let mut count = 0;
    for case in section(root, "boundary") {
        count += 1;
        let name = text_of(case, "name");
        let input = text_of(case, "input");
        match text_of(case, "rust").as_str() {
            "refuse" if name == "out of grammar matcher at the matcher door" => {
                assert!(
                    hookdoctor::regexlite::fullmatch(&input, "bash").is_err(),
                    "boundary case {name}"
                );
            }
            "refuse" => {
                assert_eq!(
                    parse_json(&input),
                    Err(ParseFailure::Invalid),
                    "boundary case {name}"
                );
            }
            "refuse-config" => {
                let document =
                    read_json_document(&input, "configuration").expect("boundary config decodes");
                let error = parse_config(&document).expect_err("boundary config accepted");
                assert_eq!(
                    error.message(),
                    "a matcher is not a valid pattern",
                    "boundary case {name}"
                );
            }
            "refuse-matcher" => {
                assert!(
                    hookdoctor::regexlite::compile(&input).is_err(),
                    "boundary case {name}"
                );
            }
            "accept-depth-4000" => {
                let value =
                    parse_json(&input).unwrap_or_else(|_| panic!("boundary case {name} refused"));
                assert_eq!(depth_of(&value), 4000, "boundary case {name}");
            }
            "render-digits" => {
                let value =
                    parse_json(&input).unwrap_or_else(|_| panic!("boundary case {name} refused"));
                assert_eq!(
                    render_json(&value),
                    format!("{input}\n"),
                    "boundary case {name}"
                );
            }
            other => panic!("unknown boundary expectation {other}"),
        }
    }
    count
}

fn check_config(root: &JsonValue) -> usize {
    let mut count = 0;
    for case in section(root, "config") {
        count += 1;
        let input = text_of(case, "input");
        let answer = answer_of(case);
        let document = read_json_document(&input, "configuration")
            .unwrap_or_else(|error| panic!("config case decode: {input:?}: {error}"));
        match outcome_of(case).as_str() {
            "hooks" => {
                let hooks = parse_config(&document)
                    .unwrap_or_else(|error| panic!("config case refused: {input:?}: {error}"));
                let built: Vec<JsonValue> = hooks.iter().map(hook_value).collect();
                let expected = answer
                    .get("hooks")
                    .and_then(JsonValue::as_list)
                    .expect("recorded hooks stand");
                assert_eq!(built.len(), expected.len(), "config case {input:?}");
                for (mine, theirs) in built.iter().zip(expected.iter()) {
                    assert_eq!(mine, theirs, "config case {input:?}");
                }
            }
            "error" => {
                let error =
                    parse_config(&document).expect_err(&format!("config case accepted: {input:?}"));
                assert_eq!(
                    error.message(),
                    text_of(answer, "message"),
                    "config case {input:?}"
                );
            }
            other => panic!("unknown config outcome {other}"),
        }
    }
    count
}

fn check_session(root: &JsonValue) -> usize {
    let mut count = 0;
    for case in section(root, "session") {
        count += 1;
        let input = text_of(case, "input");
        let answer = answer_of(case);
        match outcome_of(case).as_str() {
            "records" => {
                let records = parse_session_text(&input, 0)
                    .unwrap_or_else(|error| panic!("session case refused: {input:?}: {error}"));
                let built: Vec<JsonValue> = records.iter().map(record_value).collect();
                let expected = answer
                    .get("records")
                    .and_then(JsonValue::as_list)
                    .expect("recorded records stand");
                assert_eq!(built.len(), expected.len(), "session case {input:?}");
                for (mine, theirs) in built.iter().zip(expected.iter()) {
                    assert_eq!(mine, theirs, "session case {input:?}");
                }
            }
            "error" => {
                let error = parse_session_text(&input, 0)
                    .expect_err(&format!("session case accepted: {input:?}"));
                assert_eq!(
                    error.message(),
                    text_of(answer, "message"),
                    "session case {input:?}"
                );
            }
            other => panic!("unknown session outcome {other}"),
        }
    }
    count
}

fn check_instant(root: &JsonValue) -> usize {
    let mut count = 0;
    for case in section(root, "instant") {
        count += 1;
        let input = text_of(case, "input");
        let answer = answer_of(case);
        match outcome_of(case).as_str() {
            "none" => {
                assert_eq!(parse_instant(&input), None, "instant case {input:?}");
            }
            "instant" => {
                let micros = match answer.get("micros") {
                    Some(JsonValue::Int(value)) => *value as i128,
                    other => panic!("instant case {input:?} micros {other:?}"),
                };
                assert_eq!(
                    parse_instant(&input),
                    Some(micros),
                    "instant case {input:?}"
                );
            }
            other => panic!("unknown instant outcome {other}"),
        }
    }
    count
}

fn optional_text_of(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Null => None,
        JsonValue::Text(text) => Some(text.clone()),
        other => panic!("expected text or null, found {other:?}"),
    }
}

fn check_matcher(root: &JsonValue) -> usize {
    let mut count = 0;
    for case in section(root, "matcher") {
        count += 1;
        let matcher = optional_text_of(case.get("matcher").expect("matcher stands"));
        let tool = optional_text_of(case.get("tool").expect("tool stands"));
        let answer = answer_of(case);
        match outcome_of(case).as_str() {
            "bool" => {
                let expected = matches!(answer.get("value"), Some(JsonValue::Bool(true)));
                let result = matcher_matches(matcher.as_deref(), tool.as_deref())
                    .unwrap_or_else(|error| panic!("matcher case refused: {error}"));
                assert_eq!(result, expected, "matcher case {matcher:?} on {tool:?}");
            }
            "error" => {
                let error = matcher_matches(matcher.as_deref(), tool.as_deref())
                    .expect_err("matcher case accepted");
                assert_eq!(error.message(), text_of(answer, "message"));
            }
            other => panic!("unknown matcher outcome {other}"),
        }
    }
    count
}

fn build_from_case(
    case: &JsonValue,
) -> Result<(Vec<HookConfig>, Vec<TraceRecord>), hookdoctor::errors::HookdoctorError> {
    let document = read_json_document(&text_of(case, "config"), "configuration")?;
    let hooks = parse_config(&document)?;
    let mut records: Vec<TraceRecord> = Vec::new();
    let sessions = case
        .get("sessions")
        .and_then(JsonValue::as_list)
        .expect("sessions stand");
    for (index, text) in sessions.iter().enumerate() {
        let text = text.as_text().expect("session text stands");
        let mut built = parse_session_text(text, index as i64)?;
        records.append(&mut built);
    }
    Ok((hooks, records))
}

fn sources_of(case: &JsonValue) -> Vec<String> {
    case.get("session_sources")
        .and_then(JsonValue::as_list)
        .expect("sources stand")
        .iter()
        .map(|source| source.as_text().expect("source text stands").to_string())
        .collect()
}

fn optional_field(case: &JsonValue, key: &str) -> Option<String> {
    match case.get(key) {
        Some(JsonValue::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

fn check_examine(root: &JsonValue) -> usize {
    let mut count = 0;
    for case in section(root, "examine") {
        count += 1;
        let name = text_of(case, "name");
        let answer = answer_of(case);
        let built = build_from_case(case);
        match outcome_of(case).as_str() {
            "report" => {
                let (hooks, records) =
                    built.unwrap_or_else(|error| panic!("examine case {name} refused: {error}"));
                let report = build_report(
                    &hooks,
                    &records,
                    &text_of(case, "config_source"),
                    &sources_of(case),
                    optional_field(case, "window_start").as_deref(),
                    optional_field(case, "window_end").as_deref(),
                )
                .unwrap_or_else(|error| panic!("examine case {name} build refused: {error}"));
                assert_eq!(
                    render_json(&report),
                    text_of(answer, "json"),
                    "examine case {name} json"
                );
                assert_eq!(
                    render_markdown(&report),
                    text_of(answer, "markdown"),
                    "examine case {name} markdown"
                );
            }
            "error" => {
                let message = match built {
                    Err(error) => error.message().to_string(),
                    Ok((hooks, records)) => build_report(
                        &hooks,
                        &records,
                        &text_of(case, "config_source"),
                        &sources_of(case),
                        optional_field(case, "window_start").as_deref(),
                        optional_field(case, "window_end").as_deref(),
                    )
                    .expect_err(&format!("examine case {name} accepted"))
                    .message()
                    .to_string(),
                };
                assert_eq!(message, text_of(answer, "message"), "examine case {name}");
            }
            other => panic!("unknown examine outcome {other}"),
        }
    }
    count
}

fn check_validate(root: &JsonValue) -> usize {
    let fixture = section(root, "examine")
        .iter()
        .find(|case| text_of(case, "name") == "report test fixture")
        .expect("the report fixture stands")
        .clone();
    let (hooks, records) = build_from_case(&fixture).expect("fixture builds");
    let sources = sources_of(&fixture);
    let config_source = text_of(&fixture, "config_source");
    let mut count = 0;
    for case in section(root, "validate") {
        count += 1;
        let name = text_of(case, "name");
        let report = case.get("report").expect("report stands").clone();
        let answer = answer_of(case);
        match outcome_of(case).as_str() {
            "valid" => {
                assert_eq!(
                    validate_report(
                        &report,
                        &hooks,
                        &records,
                        &config_source,
                        &sources,
                        None,
                        None
                    ),
                    Ok(true),
                    "validate case {name}"
                );
            }
            "error" => {
                let error = validate_report(
                    &report,
                    &hooks,
                    &records,
                    &config_source,
                    &sources,
                    None,
                    None,
                )
                .expect_err(&format!("validate case {name} accepted"));
                assert_eq!(
                    error.message(),
                    text_of(answer, "message"),
                    "validate case {name}"
                );
            }
            other => panic!("unknown validate outcome {other}"),
        }
    }
    count
}

fn run_all() {
    let root = recordings();
    let mut total = 0;
    total += check_reader(&root);
    total += check_boundary(&root);
    total += check_config(&root);
    total += check_session(&root);
    total += check_instant(&root);
    total += check_matcher(&root);
    total += check_examine(&root);
    total += check_validate(&root);
    assert_eq!(total, 343, "every recorded case ran");
}

#[test]
fn port_slice_1_conformance() {
    // The nesting boundary case recurses thousands of frames deep
    // in the reader, so the suite runs on a thread with a large
    // stack, as the sibling port suites do.
    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(run_all)
        .expect("conformance thread stands");
    handle.join().expect("conformance thread finishes");
}
