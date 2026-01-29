//! Utility functions for FFI operations.

use crate::error::MemvidError;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;

/// Convert a C string to a PathBuf.
///
/// Returns an error if the pointer is null or contains invalid UTF-8.
///
/// # Safety
///
/// The caller must ensure `ptr` is either null or points to a valid
/// null-terminated C string.
pub unsafe fn cstr_to_path(ptr: *const c_char) -> Result<PathBuf, MemvidError> {
    if ptr.is_null() {
        return Err(MemvidError::null_pointer("path"));
    }

    let cstr = unsafe { CStr::from_ptr(ptr) };
    match cstr.to_str() {
        Ok(s) => Ok(PathBuf::from(s)),
        Err(_) => Err(MemvidError::invalid_utf8("path")),
    }
}

/// Convert a C string to a Rust String.
///
/// Returns an error if the pointer is null or contains invalid UTF-8.
///
/// # Safety
///
/// The caller must ensure `ptr` is either null or points to a valid
/// null-terminated C string.
pub unsafe fn cstr_to_string(ptr: *const c_char, param_name: &str) -> Result<String, MemvidError> {
    if ptr.is_null() {
        return Err(MemvidError::null_pointer(param_name));
    }

    let cstr = unsafe { CStr::from_ptr(ptr) };
    match cstr.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(MemvidError::invalid_utf8(param_name)),
    }
}

/// Convert an optional C string to an Option<String>.
///
/// Returns None if the pointer is null, Ok(Some(String)) if valid,
/// or an error if the string contains invalid UTF-8.
///
/// # Safety
///
/// The caller must ensure `ptr` is either null or points to a valid
/// null-terminated C string.
pub unsafe fn cstr_to_option_string(
    ptr: *const c_char,
    param_name: &str,
) -> Result<Option<String>, MemvidError> {
    if ptr.is_null() {
        return Ok(None);
    }

    let cstr = unsafe { CStr::from_ptr(ptr) };
    match cstr.to_str() {
        Ok(s) => Ok(Some(s.to_string())),
        Err(_) => Err(MemvidError::invalid_utf8(param_name)),
    }
}

/// Convert a Rust string to a C string, returning an owned pointer.
///
/// The caller is responsible for freeing the returned pointer with `memvid_string_free`.
/// Returns null if the string contains internal null bytes.
pub fn string_to_cstr(s: String) -> *mut c_char {
    CString::new(s)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// Set an error in the out-parameter and return a default value.
///
/// # Safety
///
/// The caller must ensure `error` is either null or a valid pointer.
pub unsafe fn set_error<T: Default>(error: *mut MemvidError, err: MemvidError) -> T {
    if let Some(e) = unsafe { error.as_mut() } {
        *e = err;
    }
    T::default()
}

/// Set an error in the out-parameter and return null.
///
/// # Safety
///
/// The caller must ensure `error` is either null or a valid pointer.
pub unsafe fn set_error_null<T>(error: *mut MemvidError, err: MemvidError) -> *mut T {
    if let Some(e) = unsafe { error.as_mut() } {
        *e = err;
    }
    std::ptr::null_mut()
}

/// Set success in the out-parameter error.
///
/// # Safety
///
/// The caller must ensure `error` is either null or a valid pointer.
pub unsafe fn set_ok(error: *mut MemvidError) {
    if let Some(e) = unsafe { error.as_mut() } {
        *e = MemvidError::ok();
    }
}
