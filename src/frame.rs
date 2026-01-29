//! Frame retrieval and content functions.

use crate::error::MemvidError;
use crate::handle::MemvidHandle;
use crate::util::{set_error, set_error_null, set_ok, string_to_cstr};
use serde::Serialize;
use std::os::raw::c_char;

/// Frame data serialized for FFI.
///
/// This mirrors the core Frame struct but with FFI-friendly types.
#[derive(Debug, Serialize)]
struct FrameJson {
    id: u64,
    timestamp: i64,
    kind: Option<String>,
    uri: Option<String>,
    title: Option<String>,
    status: String,
    payload_length: u64,
    tags: Vec<String>,
    labels: Vec<String>,
    parent_id: Option<u64>,
    chunk_index: Option<u32>,
    chunk_count: Option<u32>,
}

impl From<&memvid_core::Frame> for FrameJson {
    fn from(f: &memvid_core::Frame) -> Self {
        Self {
            id: f.id,
            timestamp: f.timestamp,
            kind: f.kind.clone(),
            uri: f.uri.clone(),
            title: f.title.clone(),
            status: format!("{:?}", f.status),
            payload_length: f.payload_length,
            tags: f.tags.clone(),
            labels: f.labels.clone(),
            parent_id: f.parent_id,
            chunk_index: f.chunk_index,
            chunk_count: f.chunk_count,
        }
    }
}

/// Get frame metadata by ID.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `frame_id`: Frame identifier
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with frame metadata on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # JSON Schema
///
/// ```json
/// {
///   "id": 1,
///   "timestamp": 1234567890,
///   "kind": "text/plain",
///   "uri": "file://doc.txt",
///   "title": "Document Title",
///   "status": "Active",
///   "payload_length": 1024,
///   "tags": ["tag1", "tag2"],
///   "labels": ["label1"],
///   "parent_id": null,
///   "chunk_index": null,
///   "chunk_count": null
/// }
/// ```
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_frame_by_id(
    handle: *mut MemvidHandle,
    frame_id: u64,
    error: *mut MemvidError,
) -> *mut c_char {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error_null(error, MemvidError::invalid_handle()) },
    };

    match handle.as_mut().frame_by_id(frame_id) {
        Ok(frame) => {
            let json_frame = FrameJson::from(&frame);
            match serde_json::to_string(&json_frame) {
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

/// Get frame metadata by URI.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `uri`: Frame URI (null-terminated UTF-8 string)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with frame metadata on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `uri` must be a valid null-terminated UTF-8 string
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_frame_by_uri(
    handle: *mut MemvidHandle,
    uri: *const c_char,
    error: *mut MemvidError,
) -> *mut c_char {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error_null(error, MemvidError::invalid_handle()) },
    };

    let uri_str = match unsafe { crate::util::cstr_to_string(uri, "uri") } {
        Ok(s) => s,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    match handle.as_mut().frame_by_uri(&uri_str) {
        Ok(frame) => {
            let json_frame = FrameJson::from(&frame);
            match serde_json::to_string(&json_frame) {
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

/// Get frame text content by ID.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `frame_id`: Frame identifier
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// Frame text content on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_frame_content(
    handle: *mut MemvidHandle,
    frame_id: u64,
    error: *mut MemvidError,
) -> *mut c_char {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error_null(error, MemvidError::invalid_handle()) },
    };

    match handle.as_mut().frame_text_by_id(frame_id) {
        Ok(content) => {
            unsafe { set_ok(error) };
            string_to_cstr(content)
        }
        Err(e) => unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    }
}

/// Soft-delete a frame.
///
/// Creates a tombstone entry; the frame data is not immediately removed.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `frame_id`: Frame identifier to delete
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// WAL sequence number on success, 0 on failure.
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_delete_frame(
    handle: *mut MemvidHandle,
    frame_id: u64,
    error: *mut MemvidError,
) -> u64 {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error(error, MemvidError::invalid_handle()) },
    };

    match handle.as_mut().delete_frame(frame_id) {
        Ok(seq) => {
            unsafe { set_ok(error) };
            seq
        }
        Err(e) => unsafe { set_error(error, MemvidError::from_core_error(e)) },
    }
}
