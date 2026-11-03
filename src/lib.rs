//! Hookdoctor examines hooks configured for a coding harness
//! against session execution traces and classifies each hook as
//! firing, silent, or firing without effect, with the evidence for
//! the classification and the near misses for the silent hooks.

pub mod dates;
pub mod errors;
pub mod examine;
pub mod ingest;
pub mod jsonvalue;
pub mod reader;
pub mod regexlite;
pub mod render;
pub mod report;

pub use errors::{HookdoctorError, ReportValidationError};
pub use examine::{build_report, validate_report, REPORT_FORMAT};
pub use ingest::{parse_config, parse_session_text, HookConfig, TraceRecord};
pub use jsonvalue::JsonValue;
pub use render::render_json;
pub use report::render_markdown;
