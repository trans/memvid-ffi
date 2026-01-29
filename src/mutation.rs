//! Content mutation functions (put, commit).

use crate::error::MemvidError;
use crate::handle::MemvidHandle;
use crate::util::{cstr_to_option_string, set_error, set_ok};
use libc::size_t;
use memvid_core::PutOptions;
use serde::Deserialize;
use std::os::raw::c_char;

/// JSON schema for PutOptions.
///
/// This allows callers to pass options as a JSON string rather than
/// requiring complex struct marshalling.
#[derive(Debug, Default, Deserialize)]
struct PutOptionsJson {
    /// Document URI
    #[serde(default)]
    uri: Option<String>,
    /// Document title
    #[serde(default)]
    title: Option<String>,
    /// Unix timestamp (seconds)
    #[serde(default)]
    timestamp: Option<i64>,
    /// Track/collection name
    #[serde(default)]
    track: Option<String>,
    /// Document kind/type
    #[serde(default)]
    kind: Option<String>,
    /// Tags as key-value pairs
    #[serde(default)]
    tags: Option<std::collections::HashMap<String, String>>,
    /// Simple labels
    #[serde(default)]
    labels: Option<Vec<String>>,
    /// Override search text
    #[serde(default)]
    search_text: Option<String>,
    /// Enable auto-tagging
    #[serde(default)]
    auto_tag: Option<bool>,
    /// Enable date extraction
    #[serde(default)]
    extract_dates: Option<bool>,
    /// Enable triplet extraction
    #[serde(default)]
    extract_triplets: Option<bool>,
    /// Skip storing raw content (hash only)
    #[serde(default)]
    no_raw: Option<bool>,
    /// Deduplicate by hash
    #[serde(default)]
    dedup: Option<bool>,
}

impl PutOptionsJson {
    fn into_put_options(self) -> PutOptions {
        let mut builder = PutOptions::builder();

        if let Some(uri) = self.uri {
            builder = builder.uri(uri);
        }
        if let Some(title) = self.title {
            builder = builder.title(title);
        }
        if let Some(ts) = self.timestamp {
            builder = builder.timestamp(ts);
        }
        if let Some(track) = self.track {
            builder = builder.track(track);
        }
        if let Some(kind) = self.kind {
            builder = builder.kind(kind);
        }
        if let Some(tags) = self.tags {
            for (k, v) in tags {
                builder = builder.tag(&k, &v);
            }
        }
        if let Some(labels) = self.labels {
            for label in labels {
                builder = builder.label(label);
            }
        }
        if let Some(text) = self.search_text {
            builder = builder.search_text(text);
        }
        if let Some(auto_tag) = self.auto_tag {
            builder = builder.auto_tag(auto_tag);
        }
        if let Some(extract_dates) = self.extract_dates {
            builder = builder.extract_dates(extract_dates);
        }
        if let Some(extract_triplets) = self.extract_triplets {
            builder = builder.extract_triplets(extract_triplets);
        }
        if let Some(no_raw) = self.no_raw {
            builder = builder.no_raw(no_raw);
        }
        if let Some(dedup) = self.dedup {
            builder = builder.dedup(dedup);
        }

        builder.build()
    }
}

/// Add content to the memory.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `data`: Pointer to content bytes
/// - `len`: Length of content in bytes
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// Frame ID on success, 0 on failure (check error->code).
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `data` must point to at least `len` bytes, or be NULL if `len` is 0
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_put_bytes(
    handle: *mut MemvidHandle,
    data: *const u8,
    len: size_t,
    error: *mut MemvidError,
) -> u64 {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error(error, MemvidError::invalid_handle()) },
    };

    if data.is_null() && len > 0 {
        return unsafe { set_error(error, MemvidError::null_pointer("data")) };
    }

    let slice = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, len) }
    };

    match handle.as_mut().put_bytes(slice) {
        Ok(frame_id) => {
            unsafe { set_ok(error) };
            frame_id
        }
        Err(e) => unsafe { set_error(error, MemvidError::from_core_error(e)) },
    }
}

/// Add content with options (JSON configuration).
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `data`: Pointer to content bytes
/// - `len`: Length of content in bytes
/// - `options_json`: JSON string with PutOptions (NULL for defaults)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// Frame ID on success, 0 on failure.
///
/// # Options JSON Schema
///
/// ```json
/// {
///   "uri": "string",
///   "title": "string",
///   "timestamp": 1234567890,
///   "track": "string",
///   "kind": "string",
///   "tags": {"key": "value"},
///   "labels": ["label1", "label2"],
///   "search_text": "override text",
///   "auto_tag": true,
///   "extract_dates": true,
///   "extract_triplets": true,
///   "no_raw": false,
///   "dedup": false
/// }
/// ```
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `data` must point to at least `len` bytes
/// - `options_json` must be a valid UTF-8 string or NULL
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_put_bytes_with_options(
    handle: *mut MemvidHandle,
    data: *const u8,
    len: size_t,
    options_json: *const c_char,
    error: *mut MemvidError,
) -> u64 {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error(error, MemvidError::invalid_handle()) },
    };

    if data.is_null() && len > 0 {
        return unsafe { set_error(error, MemvidError::null_pointer("data")) };
    }

    let slice = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, len) }
    };

    // Parse options JSON
    let options = match unsafe { cstr_to_option_string(options_json, "options_json") } {
        Ok(Some(json_str)) => match serde_json::from_str::<PutOptionsJson>(&json_str) {
            Ok(opts) => opts.into_put_options(),
            Err(e) => return unsafe { set_error(error, MemvidError::json_parse(e)) },
        },
        Ok(None) => PutOptions::default(),
        Err(e) => return unsafe { set_error(error, e) },
    };

    match handle.as_mut().put_bytes_with_options(slice, options) {
        Ok(frame_id) => {
            unsafe { set_ok(error) };
            frame_id
        }
        Err(e) => unsafe { set_error(error, MemvidError::from_core_error(e)) },
    }
}

/// Commit pending changes to disk.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// 1 on success, 0 on failure.
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_commit(handle: *mut MemvidHandle, error: *mut MemvidError) -> i32 {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error(error, MemvidError::invalid_handle()) },
    };

    match handle.as_mut().commit() {
        Ok(()) => {
            unsafe { set_ok(error) };
            1
        }
        Err(e) => unsafe { set_error(error, MemvidError::from_core_error(e)) },
    }
}
