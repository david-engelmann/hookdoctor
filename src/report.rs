//! Report assembly lives in the examination module, which builds
//! the report document. This module renders that document as
//! Markdown, line for line as the Python report module does.

use crate::jsonvalue::JsonValue;

/// Render a value the way a Python f-string spells it in the
/// Markdown lines.
fn fstring(value: Option<&JsonValue>) -> String {
    match value {
        None | Some(JsonValue::Null) => "None".to_string(),
        Some(JsonValue::Text(text)) => text.clone(),
        Some(JsonValue::Bool(flag)) => {
            if *flag {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        Some(JsonValue::Int(integer)) => integer.to_string(),
        Some(JsonValue::BigInt(text)) => text.clone(),
        Some(JsonValue::Float(number)) => crate::render::json_float_text(*number),
        Some(other) => crate::render::render_json(other),
    }
}

fn int_of(value: Option<&JsonValue>) -> i64 {
    match value {
        Some(JsonValue::Int(integer)) => *integer,
        _ => 0,
    }
}

fn effects_line(effects: Option<&JsonValue>) -> String {
    format!(
        "denial {}, modification {}, consumed output {}",
        int_of(effects.and_then(|value| value.get("denial"))),
        int_of(effects.and_then(|value| value.get("modification"))),
        int_of(effects.and_then(|value| value.get("consumed_output"))),
    )
}

/// Render the hookdoctor-report/1 document as Markdown.
///
/// Mirrors `render_markdown` in the Python report module.
pub fn render_markdown(report: &JsonValue) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# hookdoctor examination".to_string());
    lines.push(String::new());
    lines.push(format!("Format {}", fstring(report.get("format"))));
    lines.push(format!(
        "Configuration source {}",
        fstring(report.get("config_source"))
    ));
    let joined = match report.get("session_sources") {
        Some(JsonValue::List(items)) => items
            .iter()
            .map(|item| fstring(Some(item)))
            .collect::<Vec<String>>()
            .join(", "),
        _ => String::new(),
    };
    let sources = if joined.is_empty() {
        "none".to_string()
    } else {
        joined
    };
    lines.push(format!("Session sources {sources}"));
    let window = report.get("window");
    lines.push(format!(
        "Window start {} end {}",
        fstring(window.and_then(|value| value.get("start"))),
        fstring(window.and_then(|value| value.get("end"))),
    ));
    lines.push(String::new());
    lines.push("## Hooks".to_string());
    lines.push(String::new());
    let hooks = match report.get("hooks") {
        Some(JsonValue::List(items)) => items.clone(),
        _ => Vec::new(),
    };
    if hooks.is_empty() {
        lines.push("No hooks are configured.".to_string());
        lines.push(String::new());
    }
    for entry in &hooks {
        let evidence = entry.get("evidence");
        let effects = evidence.and_then(|value| value.get("effects"));
        lines.push(format!("### {}", fstring(entry.get("command"))));
        lines.push(String::new());
        lines.push(format!("Identifier {}", fstring(entry.get("id"))));
        lines.push(format!("Event {}", fstring(entry.get("event"))));
        lines.push(format!("Matcher {}", fstring(entry.get("matcher"))));
        lines.push(format!("Class {}", fstring(entry.get("class"))));
        lines.push(format!(
            "Matched {}",
            int_of(evidence.and_then(|value| value.get("matched")))
        ));
        lines.push(format!(
            "Ran {}",
            int_of(evidence.and_then(|value| value.get("ran")))
        ));
        lines.push(format!("Effects {}", effects_line(effects)));
        lines.push(format!(
            "First ran at {}",
            fstring(evidence.and_then(|value| value.get("first_ran_at")))
        ));
        lines.push(format!(
            "Last ran at {}",
            fstring(evidence.and_then(|value| value.get("last_ran_at")))
        ));
        lines.push(String::new());
    }
    lines.push("## Near misses".to_string());
    lines.push(String::new());
    let near_misses = match report.get("near_misses") {
        Some(JsonValue::List(items)) => items.clone(),
        _ => Vec::new(),
    };
    if near_misses.is_empty() {
        lines.push("None.".to_string());
        lines.push(String::new());
    }
    for miss in &near_misses {
        lines.push(format!(
            "- Hook {} missed a {} subject at {} in {} on line {}. {}.",
            fstring(miss.get("hook_id")),
            fstring(miss.get("event")),
            fstring(miss.get("timestamp")),
            fstring(miss.get("source")),
            int_of(miss.get("line")),
            fstring(miss.get("detail")),
        ));
    }
    if !near_misses.is_empty() {
        lines.push(String::new());
    }
    let mut text = lines.join("\n");
    while text.ends_with('\n') {
        text.pop();
    }
    text.push('\n');
    text
}
