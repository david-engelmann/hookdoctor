//! Error types mirroring the Python package error classes.

use std::fmt;

/// Raised when an input or operation cannot be trusted.
///
/// Mirrors the Python `HookdoctorError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookdoctorError {
    message: String,
}

impl HookdoctorError {
    pub fn new(message: impl Into<String>) -> Self {
        HookdoctorError {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HookdoctorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HookdoctorError {}

/// Raised when a report disagrees with a rebuilt examination.
///
/// Mirrors the Python `ReportValidationError`, a subclass of the
/// hookdoctor error there and a sibling type here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportValidationError {
    message: String,
}

impl ReportValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        ReportValidationError {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ReportValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ReportValidationError {}

impl From<ReportValidationError> for HookdoctorError {
    fn from(error: ReportValidationError) -> Self {
        HookdoctorError::new(error.message)
    }
}
