//! Lifecycle management functions (create, open, close).

use crate::error::MemvidError;
use crate::handle::MemvidHandle;
use crate::util::{cstr_to_path, set_error_null, set_ok};
use std::os::raw::c_char;

/// Create a new Memvid memory at the specified path.
///
/// # Parameters
///
/// - `path`: Filesystem path for the memory (UTF-8 encoded, null-terminated)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// Handle on success, NULL on failure.
///
/// # Ownership
///
/// Caller owns the returned handle. Must call `memvid_close()` to free.
///
/// # Safety
///
/// - `path` must be a valid null-terminated UTF-8 string or NULL
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_create(
    path: *const c_char,
    error: *mut MemvidError,
) -> *mut MemvidHandle {
    // Validate path parameter
    let path = match unsafe { cstr_to_path(path) } {
        Ok(p) => p,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    // Create the memvid instance
    match memvid_core::Memvid::create(&path) {
        Ok(memvid) => {
            unsafe { set_ok(error) };
            Box::into_raw(MemvidHandle::new(memvid))
        }
        Err(e) => unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    }
}

/// Open an existing Memvid memory.
///
/// # Parameters
///
/// - `path`: Filesystem path to existing memory (UTF-8 encoded, null-terminated)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// Handle on success, NULL on failure.
///
/// # Ownership
///
/// Caller owns the returned handle. Must call `memvid_close()` to free.
///
/// # Safety
///
/// - `path` must be a valid null-terminated UTF-8 string or NULL
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_open(
    path: *const c_char,
    error: *mut MemvidError,
) -> *mut MemvidHandle {
    // Validate path parameter
    let path = match unsafe { cstr_to_path(path) } {
        Ok(p) => p,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    // Open the memvid instance
    match memvid_core::Memvid::open(&path) {
        Ok(memvid) => {
            unsafe { set_ok(error) };
            Box::into_raw(MemvidHandle::new(memvid))
        }
        Err(e) => unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    }
}

/// Close and free a Memvid handle.
///
/// After this call, the handle is invalid and must not be used.
///
/// # Parameters
///
/// - `handle`: Handle to close (safe to pass NULL)
///
/// # Safety
///
/// - `handle` must be a valid handle returned by `memvid_create` or `memvid_open`,
///   or NULL
/// - The handle must not be used after this call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_close(handle: *mut MemvidHandle) {
    if handle.is_null() {
        return;
    }

    // Take ownership and drop
    unsafe {
        drop(Box::from_raw(handle));
    }
}
