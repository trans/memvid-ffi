//! File verification functions.

use crate::error::MemvidError;
use crate::util::{cstr_to_path, set_error_null, set_ok, string_to_cstr};
use serde::Serialize;
use std::os::raw::c_char;

/// Verification status for JSON serialization.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum VerificationStatusJson {
    Passed,
    Failed,
    Skipped,
}

impl From<&memvid_core::VerificationStatus> for VerificationStatusJson {
    fn from(s: &memvid_core::VerificationStatus) -> Self {
        match s {
            memvid_core::VerificationStatus::Passed => Self::Passed,
            memvid_core::VerificationStatus::Failed => Self::Failed,
            memvid_core::VerificationStatus::Skipped => Self::Skipped,
        }
    }
}

/// Individual verification check for JSON serialization.
#[derive(Debug, Serialize)]
struct VerificationCheckJson {
    name: String,
    status: VerificationStatusJson,
    details: Option<String>,
}

impl From<&memvid_core::VerificationCheck> for VerificationCheckJson {
    fn from(c: &memvid_core::VerificationCheck) -> Self {
        Self {
            name: c.name.clone(),
            status: VerificationStatusJson::from(&c.status),
            details: c.details.clone(),
        }
    }
}

/// Verification report for JSON serialization.
#[derive(Debug, Serialize)]
struct VerificationReportJson {
    file_path: String,
    overall_status: VerificationStatusJson,
    checks: Vec<VerificationCheckJson>,
}

impl From<&memvid_core::VerificationReport> for VerificationReportJson {
    fn from(r: &memvid_core::VerificationReport) -> Self {
        Self {
            file_path: r.file_path.to_string_lossy().to_string(),
            overall_status: VerificationStatusJson::from(&r.overall_status),
            checks: r.checks.iter().map(VerificationCheckJson::from).collect(),
        }
    }
}

/// Verify file integrity.
///
/// This is a static function that does not require an open handle.
///
/// # Parameters
///
/// - `path`: Path to the .mv2 file (null-terminated UTF-8 string)
/// - `deep`: Perform deep verification (more thorough but slower)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with verification report on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Response JSON Schema
///
/// ```json
/// {
///   "file_path": "/path/to/file.mv2",
///   "overall_status": "passed",
///   "checks": [
///     {
///       "name": "TimeIndexEntryCount",
///       "status": "passed",
///       "details": null
///     },
///     {
///       "name": "LexIndexDecode",
///       "status": "passed",
///       "details": "Lexical index decoded successfully"
///     }
///   ]
/// }
/// ```
///
/// Status values: "passed", "failed", "skipped"
///
/// # Safety
///
/// - `path` must be a valid null-terminated UTF-8 string
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_verify(
    path: *const c_char,
    deep: i32,
    error: *mut MemvidError,
) -> *mut c_char {
    let path = match unsafe { cstr_to_path(path) } {
        Ok(p) => p,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    let deep = deep != 0;

    match memvid_core::Memvid::verify(&path, deep) {
        Ok(report) => {
            let json_report = VerificationReportJson::from(&report);
            match serde_json::to_string(&json_report) {
                Ok(json) => {
                    unsafe { set_ok(error) };
                    string_to_cstr(json)
                }
                Err(e) => unsafe { set_error_null(error, MemvidError::json_serialize(e)) },
            }
        }
        Err(e) => unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    }
}
