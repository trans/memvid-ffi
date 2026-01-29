//! Doctor (file repair/maintenance) functions.

use crate::error::MemvidError;
use crate::util::{cstr_to_path, cstr_to_string, set_error_null, set_ok, string_to_cstr};
use std::os::raw::c_char;

/// Run doctor diagnostics and optionally repair a memory file.
///
/// This is a static function that does not require an open handle.
/// The file should NOT be open when running doctor.
///
/// # Parameters
///
/// - `path`: Path to the .mv2 file (null-terminated UTF-8 string)
/// - `options_json`: JSON string with doctor options (NULL for defaults)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with doctor report on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Options JSON Schema
///
/// ```json
/// {
///   "rebuild_time_index": false,
///   "rebuild_lex_index": false,
///   "rebuild_vec_index": false,
///   "vacuum": false,
///   "dry_run": false,
///   "quiet": false
/// }
/// ```
///
/// # Response JSON Schema
///
/// ```json
/// {
///   "plan": { ... },
///   "status": "clean|healed|partial|failed|plan_only",
///   "phases": [...],
///   "findings": [...],
///   "metrics": { "total_duration_ms": 123, ... },
///   "verification": { ... }
/// }
/// ```
///
/// # Safety
///
/// - `path` must be a valid null-terminated UTF-8 string
/// - `options_json` must be a valid null-terminated UTF-8 string or NULL
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_doctor(
    path: *const c_char,
    options_json: *const c_char,
    error: *mut MemvidError,
) -> *mut c_char {
    let path = match unsafe { cstr_to_path(path) } {
        Ok(p) => p,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    let options: memvid_core::DoctorOptions = if options_json.is_null() {
        Default::default()
    } else {
        match unsafe { cstr_to_string(options_json, "options_json") } {
            Ok(json_str) => match serde_json::from_str(&json_str) {
                Ok(opts) => opts,
                Err(e) => return unsafe { set_error_null(error, MemvidError::json_parse(e)) },
            },
            Err(e) => return unsafe { set_error_null(error, e) },
        }
    };

    match memvid_core::Memvid::doctor(&path, options) {
        Ok(report) => match serde_json::to_string(&report) {
            Ok(json) => {
                unsafe { set_ok(error) };
                string_to_cstr(json)
            }
            Err(e) => unsafe { set_error_null(error, MemvidError::json_serialize(e)) },
        },
        Err(e) => unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    }
}

/// Create a doctor repair plan without executing it.
///
/// Use this to preview what repairs would be made, then optionally
/// call `memvid_doctor_apply` to execute the plan.
///
/// # Parameters
///
/// - `path`: Path to the .mv2 file (null-terminated UTF-8 string)
/// - `options_json`: JSON string with doctor options (NULL for defaults)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with doctor plan on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Safety
///
/// - `path` must be a valid null-terminated UTF-8 string
/// - `options_json` must be a valid null-terminated UTF-8 string or NULL
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_doctor_plan(
    path: *const c_char,
    options_json: *const c_char,
    error: *mut MemvidError,
) -> *mut c_char {
    let path = match unsafe { cstr_to_path(path) } {
        Ok(p) => p,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    let options: memvid_core::DoctorOptions = if options_json.is_null() {
        Default::default()
    } else {
        match unsafe { cstr_to_string(options_json, "options_json") } {
            Ok(json_str) => match serde_json::from_str(&json_str) {
                Ok(opts) => opts,
                Err(e) => return unsafe { set_error_null(error, MemvidError::json_parse(e)) },
            },
            Err(e) => return unsafe { set_error_null(error, e) },
        }
    };

    match memvid_core::Memvid::doctor_plan(&path, options) {
        Ok(plan) => match serde_json::to_string(&plan) {
            Ok(json) => {
                unsafe { set_ok(error) };
                string_to_cstr(json)
            }
            Err(e) => unsafe { set_error_null(error, MemvidError::json_serialize(e)) },
        },
        Err(e) => unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    }
}

/// Apply a previously created doctor plan.
///
/// # Parameters
///
/// - `path`: Path to the .mv2 file (null-terminated UTF-8 string)
/// - `plan_json`: JSON string with doctor plan (from `memvid_doctor_plan`)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with doctor report on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Safety
///
/// - `path` must be a valid null-terminated UTF-8 string
/// - `plan_json` must be a valid null-terminated UTF-8 string
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_doctor_apply(
    path: *const c_char,
    plan_json: *const c_char,
    error: *mut MemvidError,
) -> *mut c_char {
    let path = match unsafe { cstr_to_path(path) } {
        Ok(p) => p,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    let plan: memvid_core::DoctorPlan = match unsafe { cstr_to_string(plan_json, "plan_json") } {
        Ok(json_str) => match serde_json::from_str(&json_str) {
            Ok(p) => p,
            Err(e) => return unsafe { set_error_null(error, MemvidError::json_parse(e)) },
        },
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    match memvid_core::Memvid::doctor_apply(&path, plan) {
        Ok(report) => match serde_json::to_string(&report) {
            Ok(json) => {
                unsafe { set_ok(error) };
                string_to_cstr(json)
            }
            Err(e) => unsafe { set_error_null(error, MemvidError::json_serialize(e)) },
        },
        Err(e) => unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    }
}
