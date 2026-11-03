//! The hookdoctor command line. Its argument surface is its own,
//! as obligation PORT-1 states, carrying the examination end to
//! end over local files.

use std::process::ExitCode;

use hookdoctor::errors::HookdoctorError;
use hookdoctor::examine::{build_report, validate_report};
use hookdoctor::ingest::{parse_config, parse_session_text, TraceRecord};
use hookdoctor::reader::{read_file_bytes, read_json_file};
use hookdoctor::render::render_json;
use hookdoctor::report::render_markdown;

struct Args {
    config: String,
    sessions: Vec<String>,
    output: String,
    report: String,
    config_label: Option<String>,
    session_labels: Vec<String>,
    window_start: Option<String>,
    window_end: Option<String>,
}

fn fail(message: &str) -> ExitCode {
    eprintln!("hookdoctor error: {message}");
    ExitCode::from(2)
}

fn file_name(path: &str) -> String {
    let trimmed = path.trim_end_matches(['/', '\\']);
    match trimmed.rsplit(['/', '\\']).next() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => path.to_string(),
    }
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut config: Option<String> = None;
    let mut sessions: Vec<String> = Vec::new();
    let mut output: Option<String> = None;
    let mut report: Option<String> = None;
    let mut config_label: Option<String> = None;
    let mut session_labels: Vec<String> = Vec::new();
    let mut window_start: Option<String> = None;
    let mut window_end: Option<String> = None;
    let mut index = 0;
    while index < argv.len() {
        let flag = argv[index].as_str();
        let take = |index: &mut usize| -> Result<String, String> {
            *index += 1;
            argv.get(*index)
                .cloned()
                .ok_or_else(|| format!("the {flag} option carries no value"))
        };
        match flag {
            "--config" => config = Some(take(&mut index)?),
            "--session" => sessions.push(take(&mut index)?),
            "--output" => output = Some(take(&mut index)?),
            "--report" => report = Some(take(&mut index)?),
            "--config-label" => config_label = Some(take(&mut index)?),
            "--session-label" => session_labels.push(take(&mut index)?),
            "--window-start" => window_start = Some(take(&mut index)?),
            "--window-end" => window_end = Some(take(&mut index)?),
            other => return Err(format!("an unknown option stands in the call, {other}")),
        }
        index += 1;
    }
    let config = config.ok_or_else(|| "the call carries no configuration".to_string())?;
    if sessions.is_empty() {
        return Err("the call carries no session file".to_string());
    }
    let output = output.ok_or_else(|| "the call carries no output path".to_string())?;
    let report = report.ok_or_else(|| "the call carries no report path".to_string())?;
    Ok(Args {
        config,
        sessions,
        output,
        report,
        config_label,
        session_labels,
        window_start,
        window_end,
    })
}

fn read_session_text(path: &str) -> Result<String, HookdoctorError> {
    let bytes = read_file_bytes(std::path::Path::new(path), "session file")?;
    String::from_utf8(bytes).map_err(|_| HookdoctorError::new("the session file is not UTF-8 text"))
}

fn write_text(path: &str, text: &str) -> Result<(), HookdoctorError> {
    let destination = std::path::Path::new(path);
    let write = (|| -> std::io::Result<()> {
        if let Some(parent) = destination.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(destination, text)
    })();
    write.map_err(|_| HookdoctorError::new("an output could not be written"))
}

fn run(args: &Args) -> Result<(), HookdoctorError> {
    let document = read_json_file(std::path::Path::new(&args.config), "configuration")?;
    let hooks = parse_config(&document)?;
    let mut records: Vec<TraceRecord> = Vec::new();
    for (index, session_path) in args.sessions.iter().enumerate() {
        let text = read_session_text(session_path)?;
        let mut built = parse_session_text(&text, index as i64)?;
        records.append(&mut built);
    }
    if !args.session_labels.is_empty() && args.session_labels.len() != args.sessions.len() {
        return Err(HookdoctorError::new(
            "session labels do not match the session files",
        ));
    }
    let config_source = args
        .config_label
        .clone()
        .unwrap_or_else(|| file_name(&args.config));
    let session_sources: Vec<String> = if args.session_labels.is_empty() {
        args.sessions.iter().map(|path| file_name(path)).collect()
    } else {
        args.session_labels.clone()
    };
    let report = build_report(
        &hooks,
        &records,
        &config_source,
        &session_sources,
        args.window_start.as_deref(),
        args.window_end.as_deref(),
    )?;
    validate_report(
        &report,
        &hooks,
        &records,
        &config_source,
        &session_sources,
        args.window_start.as_deref(),
        args.window_end.as_deref(),
    )
    .map_err(HookdoctorError::from)?;
    write_text(&args.output, &render_json(&report))?;
    write_text(&args.report, &render_markdown(&report))?;
    Ok(())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(args) => args,
        Err(message) => return fail(&message),
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(error.message()),
    }
}
