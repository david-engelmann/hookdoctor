//! The hookdoctor command line over the Rust library.
//!
//! This shell is the closing slice of the port under obligations
//! PORT2-1 to PORT2-3. It carries the one flat door of the Python
//! command line, with the argument reading the contract probe
//! recorded, replacing the slice 1 shell and its own wordings
//! whole. A reading the argument reader refuses exits with code
//! 2 and the short usage line of this shell on standard error,
//! and a refusal of the tool exits with code 2 and the one
//! refusal line, the hookdoctor error prefix and the wording, on
//! standard error, and a door that succeeds exits with code 0 in
//! silence. The arguments stand as the operating system hands
//! them over, their bytes whole, so a file name carrying a byte
//! that is not UTF-8 names the file it names.

use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use hookdoctor::errors::HookdoctorError;
use hookdoctor::examine::{build_report, validate_report};
use hookdoctor::ingest::{parse_config, parse_session_text, TraceRecord};
use hookdoctor::reader::{read_file_bytes, read_json_file};
use hookdoctor::render::render_json;
use hookdoctor::report::render_markdown;

const USAGE: &str = "usage: hookdoctor --config PATH --session PATH [--session PATH ...] --output PATH --report PATH [--config-label TEXT] [--session-label TEXT ...] [--window-start TIMESTAMP] [--window-end TIMESTAMP]";

/// The help text the help flag prints. The usage line stands
/// once in the shell, in the constant above, and the help text
/// builds on it, so the two outputs can never drift apart on
/// the line they share.
fn help() -> String {
    format!("{USAGE}\n\nexamine configured hooks against session execution traces\n")
}

struct FlagSpec {
    name: &'static str,
    required: bool,
    append: bool,
}

const FLAGS: &[FlagSpec] = &[
    FlagSpec {
        name: "config",
        required: true,
        append: false,
    },
    FlagSpec {
        name: "session",
        required: true,
        append: true,
    },
    FlagSpec {
        name: "output",
        required: true,
        append: false,
    },
    FlagSpec {
        name: "report",
        required: true,
        append: false,
    },
    FlagSpec {
        name: "config-label",
        required: false,
        append: false,
    },
    FlagSpec {
        name: "session-label",
        required: false,
        append: true,
    },
    FlagSpec {
        name: "window-start",
        required: false,
        append: false,
    },
    FlagSpec {
        name: "window-end",
        required: false,
        append: false,
    },
];

/// The values of the door reading, each flag standing with every
/// value it took in the order it took them, each standing as the
/// operating system handed it over. A flag that stands once reads
/// its last value.
type Values = HashMap<&'static str, Vec<OsString>>;

fn token_bytes(token: &OsString) -> &[u8] {
    token.as_os_str().as_bytes()
}

/// The negative number shapes the Python argument reader does
/// not treat as option strings, a minus then digits or a minus
/// then a dotted fraction, with the digit reading of the runtime
/// the crate matcher carries. A token carrying bytes that are
/// not text reads as no number, as the surrogate standing in the
/// runtime text is no digit either.
fn is_negative_number(token: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(token) else {
        return false;
    };
    hookdoctor::regexlite::fullmatch(r"-\d+|-\d*[.]\d+", text).unwrap_or(false)
}

/// Whether a token standing after a flag reads as a further
/// option string to the argument reader, in which case the flag
/// stands without its value. A token carrying a space never
/// reads as an option string to that reader.
fn looks_like_option(token: &[u8]) -> bool {
    token.len() > 1 && token[0] == b'-' && !is_negative_number(token) && !token.contains(&b' ')
}

/// Whether a single dash token is the help flag standing first
/// in a cluster of short flags. The Python argument reader
/// expands a cluster letter by letter, so the help flag fires
/// where it stands first, ahead of any letter it does not know,
/// and a cluster that carries an equals sign directly after the
/// help letter is the help flag carrying an explicit value,
/// which that reader refuses at the point it stands rather than
/// answers.
fn is_help_cluster(token: &[u8]) -> bool {
    token.len() > 1
        && token[0] == b'-'
        && token[1] != b'-'
        && !is_negative_number(token)
        && token[1] == b'h'
        && (token.len() == 2 || token[2] != b'=')
}

/// Whether a single dash token is the help flag carrying an
/// equals value, the one cluster shape the Python argument
/// reader refuses outright where it stands.
fn is_help_with_equals(token: &[u8]) -> bool {
    token.len() > 2 && token[0] == b'-' && token[1] == b'h' && token[2] == b'='
}

enum LongMatch {
    Flag(usize),
    Help,
    Ambiguous,
    Unknown,
}

/// Match a long flag name against the flags of the door and the
/// help flag, exactly first and then by a prefix that names one
/// alone.
fn match_long(name: &[u8]) -> LongMatch {
    if name.is_empty() {
        return LongMatch::Unknown;
    }
    if name == b"help" {
        return LongMatch::Help;
    }
    for (index, spec) in FLAGS.iter().enumerate() {
        if spec.name.as_bytes() == name {
            return LongMatch::Flag(index);
        }
    }
    let mut found: Option<LongMatch> = None;
    let mut ambiguous = false;
    if b"help".starts_with(name) {
        found = Some(LongMatch::Help);
    }
    for (index, spec) in FLAGS.iter().enumerate() {
        if spec.name.as_bytes().starts_with(name) {
            if found.is_some() {
                ambiguous = true;
            }
            found = Some(LongMatch::Flag(index));
        }
    }
    if ambiguous {
        LongMatch::Ambiguous
    } else {
        found.unwrap_or(LongMatch::Unknown)
    }
}

enum Scan {
    Help,
    UsageError,
    Parsed(Values),
}

/// Read the arguments of the door the way the Python argument
/// reader reads them. An ambiguous prefix and a flag standing
/// without its value refuse the reading at the point they stand,
/// and so does the help flag carrying an equals value. A word or
/// flag the door does not take is set aside and refuses the
/// reading once the scan ends, so a help flag standing after one
/// still answers. The separator word ends the option reading and
/// every word after it is set aside the same way, and the
/// separator standing at all refuses the reading on this
/// runtime, as the probe recorded. A flag given more than once
/// keeps its last value, and a repeating flag keeps every
/// standing in the order given. A single dash token reads as a
/// cluster of short flags, the help flag firing where it stands
/// first in one.
fn scan(args: &[OsString]) -> Scan {
    let mut values: Values = HashMap::new();
    let mut refused_at_end = false;
    let mut after_separator = false;
    let mut position = 0;
    while position < args.len() {
        let token = &args[position];
        let tok = token_bytes(token);
        if after_separator {
            refused_at_end = true;
            position += 1;
            continue;
        }
        if tok == b"--" {
            after_separator = true;
            refused_at_end = true;
            position += 1;
            continue;
        }
        if tok == b"-h" || is_help_cluster(tok) {
            return Scan::Help;
        }
        if is_help_with_equals(tok) {
            return Scan::UsageError;
        }
        if let Some(long) = tok.strip_prefix(b"--") {
            let (name, inline) = match long.iter().position(|byte| *byte == b'=') {
                Some(equal) => (
                    &long[..equal],
                    Some(OsString::from_vec(long[equal + 1..].to_vec())),
                ),
                None => (long, None),
            };
            match match_long(name) {
                LongMatch::Help => {
                    if inline.is_some() {
                        return Scan::UsageError;
                    }
                    return Scan::Help;
                }
                LongMatch::Ambiguous => return Scan::UsageError,
                LongMatch::Unknown => {
                    refused_at_end = true;
                    position += 1;
                }
                LongMatch::Flag(index) => {
                    let spec = &FLAGS[index];
                    let value = match inline {
                        Some(value) => {
                            position += 1;
                            value
                        }
                        None => match args.get(position + 1) {
                            Some(value) if !looks_like_option(token_bytes(value)) => {
                                let value = value.clone();
                                position += 2;
                                value
                            }
                            _ => return Scan::UsageError,
                        },
                    };
                    if spec.append {
                        values.entry(spec.name).or_default().push(value);
                    } else {
                        values.insert(spec.name, vec![value]);
                    }
                }
            }
            continue;
        }
        refused_at_end = true;
        position += 1;
    }
    if refused_at_end {
        return Scan::UsageError;
    }
    for spec in FLAGS {
        if spec.required {
            match values.get(spec.name) {
                Some(standings) if !standings.is_empty() => {}
                _ => return Scan::UsageError,
            }
        }
    }
    Scan::Parsed(values)
}

fn single<'a>(values: &'a Values, name: &str) -> Option<&'a OsString> {
    values.get(name).and_then(|standings| standings.last())
}

fn standings<'a>(values: &'a Values, name: &str) -> &'a [OsString] {
    values.get(name).map(Vec::as_slice).unwrap_or(&[])
}

fn need<'a>(values: &'a Values, name: &str) -> &'a OsString {
    single(values, name).expect("a required flag stands once the reading accepts")
}

fn path_of(values: &Values, name: &str) -> PathBuf {
    PathBuf::from(need(values, name))
}

/// The text of a window word. The shell reads three classes of
/// word and this helper carries only the window words. The path
/// words never pass through it, standing as bytes for the
/// filesystem end to end, and the label words stand apart in the
/// door, refused there where they are not UTF-8 text, because a
/// label flows into the outputs as data. A window word a
/// timestamp grammar reads, and no timestamp that grammar
/// accepts can contain the replacement character, so a word
/// carrying a byte that is not UTF-8 reads here with that
/// character standing in it and refuses as no timestamp under
/// the window wording, while a word that stands whole as UTF-8
/// passes through unchanged.
fn word_text(value: &OsString) -> String {
    value.to_string_lossy().into_owned()
}

fn refusal(error: impl Into<HookdoctorError>) -> String {
    error.into().message().to_string()
}

/// The source name a path reads as where no label stands, the
/// final component after the last separator of this platform,
/// as the Python path reading takes it. A path ending at the
/// root or at a parent step names nothing and reads as the
/// empty label, and a name carrying a byte that is not UTF-8
/// text refuses under the wording of the label it would stand
/// as, where the Python dies uncaught writing such a label.
fn source_name(path: &Path, wording: &str) -> Result<String, String> {
    match path.file_name() {
        Some(name) => name
            .to_str()
            .map(str::to_string)
            .ok_or_else(|| wording.to_string()),
        None => Ok(String::new()),
    }
}

/// Resolve the configuration source. A label word that stands
/// and is not UTF-8 text refuses here where the labels are
/// resolved, ahead of every write, and an empty label and an
/// absent label fall back to the file name of the configuration
/// path, as the Python truthiness reading does.
fn config_source(values: &Values, config_path: &Path) -> Result<String, String> {
    const WORDING: &str = "the configuration label is not UTF-8 text";
    match single(values, "config-label") {
        Some(word) => match word.to_str() {
            Some(text) if !text.is_empty() => Ok(text.to_string()),
            Some(_) => source_name(config_path, WORDING),
            None => Err(WORDING.to_string()),
        },
        None => source_name(config_path, WORDING),
    }
}

/// Resolve the session sources. The labels that stand are taken
/// in order, and one carrying a byte that is not UTF-8 text
/// refuses here where the labels are resolved, ahead of every
/// write. Where no labels stand, each source reads as the file
/// name of its session path under the same refusal.
fn session_sources(values: &Values, session_paths: &[PathBuf]) -> Result<Vec<String>, String> {
    const WORDING: &str = "a session label is not UTF-8 text";
    let labels = standings(values, "session-label");
    if labels.is_empty() {
        return session_paths
            .iter()
            .map(|path| source_name(path, WORDING))
            .collect();
    }
    let mut sources = Vec::new();
    for word in labels {
        match word.to_str() {
            Some(text) => sources.push(text.to_string()),
            None => return Err(WORDING.to_string()),
        }
    }
    Ok(sources)
}

/// Write one output file the way the Python write helper does,
/// creating the directory path where it does not stand, with the
/// one refusal wording every write failure carries.
fn write_text(path: &Path, text: &str) -> Result<(), String> {
    let outcome = (|| -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, text)
    })();
    outcome.map_err(|_| "an output could not be written".to_string())
}

fn door(values: &Values) -> Result<(), String> {
    let config_path = path_of(values, "config");
    let document = read_json_file(&config_path, "configuration").map_err(refusal)?;
    let hooks = parse_config(&document).map_err(refusal)?;

    let session_paths: Vec<PathBuf> = standings(values, "session")
        .iter()
        .map(PathBuf::from)
        .collect();
    let mut records: Vec<TraceRecord> = Vec::new();
    for (index, session_path) in session_paths.iter().enumerate() {
        let bytes = read_file_bytes(session_path, "session file").map_err(refusal)?;
        let text = String::from_utf8(bytes)
            .map_err(|_| "the session file is not UTF-8 text".to_string())?;
        let mut built = parse_session_text(&text, index as i64).map_err(refusal)?;
        records.append(&mut built);
    }

    let label_count = standings(values, "session-label").len();
    if label_count != 0 && label_count != session_paths.len() {
        return Err("session labels do not match the session files".to_string());
    }
    let config_source = config_source(values, &config_path)?;
    let session_sources = session_sources(values, &session_paths)?;

    let window_start = single(values, "window-start").map(word_text);
    let window_end = single(values, "window-end").map(word_text);
    let report = build_report(
        &hooks,
        &records,
        &config_source,
        &session_sources,
        window_start.as_deref(),
        window_end.as_deref(),
    )
    .map_err(refusal)?;
    validate_report(
        &report,
        &hooks,
        &records,
        &config_source,
        &session_sources,
        window_start.as_deref(),
        window_end.as_deref(),
    )
    .map_err(refusal)?;

    write_text(&path_of(values, "output"), &render_json(&report))?;
    write_text(&path_of(values, "report"), &render_markdown(&report))?;
    Ok(())
}

fn print_stdout(text: &str) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(text.as_bytes());
    let _ = handle.flush();
}

fn print_stderr(bytes: &[u8]) {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    let _ = handle.write_all(bytes);
    let _ = handle.flush();
}

fn run(args: &[OsString]) -> u8 {
    match scan(args) {
        Scan::Help => {
            print_stdout(&help());
            0
        }
        Scan::UsageError => {
            print_stderr(USAGE.as_bytes());
            print_stderr(b"\n");
            2
        }
        Scan::Parsed(values) => match door(&values) {
            Ok(()) => 0,
            Err(message) => {
                print_stderr(format!("hookdoctor error: {message}\n").as_bytes());
                2
            }
        },
    }
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    ExitCode::from(run(&args))
}
