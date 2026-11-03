//! Reading JSON documents under the tool's refusal discipline.
//!
//! Every refusal names the document by the label the reader was
//! given, matching the Python command line reader.

use std::path::Path;

use crate::errors::HookdoctorError;
use crate::jsonvalue::{parse_json, parse_json_lenient, JsonValue, ParseFailure};

fn refusal(failure: ParseFailure, label: &str) -> HookdoctorError {
    match failure {
        ParseFailure::Invalid => HookdoctorError::new(format!("the {label} is not valid JSON")),
        ParseFailure::DuplicateKey(key) => {
            HookdoctorError::new(format!("the {label} carries the key {key} more than once"))
        }
    }
}

/// Decode a JSON document from text under the given label.
pub fn read_json_document(text: &str, label: &str) -> Result<JsonValue, HookdoctorError> {
    parse_json(text).map_err(|failure| refusal(failure, label))
}

/// Read and decode a JSON document from a local file.
pub fn read_json_file(path: &Path, label: &str) -> Result<JsonValue, HookdoctorError> {
    let data = std::fs::read(path)
        .map_err(|_| HookdoctorError::new(format!("the {label} could not be read")))?;
    let text = String::from_utf8(data)
        .map_err(|_| HookdoctorError::new(format!("the {label} is not valid JSON")))?;
    read_json_document(&text, label)
}

/// Read bytes from a local file under the given label.
pub fn read_file_bytes(path: &Path, label: &str) -> Result<Vec<u8>, HookdoctorError> {
    std::fs::read(path).map_err(|_| HookdoctorError::new(format!("the {label} could not be read")))
}

/// Sample hook output text for an object, the standing reading of
/// the examination. A text that does not read as an object samples
/// as no output, and a repeated key leaves its last value standing.
pub fn parse_output_text(text: &str) -> Option<JsonValue> {
    if text.is_empty() {
        return None;
    }
    match parse_json_lenient(text) {
        Ok(value) if value.is_object() => Some(value),
        _ => None,
    }
}
