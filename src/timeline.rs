//! Timeline query functions.

use crate::error::MemvidError;
use crate::handle::MemvidHandle;
use crate::util::{cstr_to_option_string, set_error_null, set_ok, string_to_cstr};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use std::os::raw::c_char;

/// JSON schema for TimelineQuery.
#[derive(Debug, Default, Deserialize)]
struct TimelineQueryJson {
    /// Maximum number of entries to return
    #[serde(default)]
    limit: Option<u64>,
    /// Timestamp lower bound (inclusive)
    #[serde(default)]
    since: Option<i64>,
    /// Timestamp upper bound (inclusive)
    #[serde(default)]
    until: Option<i64>,
    /// Return in reverse chronological order
    #[serde(default)]
    reverse: bool,
}

impl TimelineQueryJson {
    fn into_query(self) -> memvid_core::TimelineQuery {
        let mut builder = memvid_core::TimelineQueryBuilder::default();

        if let Some(limit) = self.limit {
            if let Some(nz) = NonZeroU64::new(limit) {
                builder = builder.limit(nz);
            }
        }
        if let Some(since) = self.since {
            builder = builder.since(since);
        }
        if let Some(until) = self.until {
            builder = builder.until(until);
        }
        builder = builder.reverse(self.reverse);

        builder.build()
    }
}

/// Timeline entry for JSON serialization.
#[derive(Debug, Serialize)]
struct TimelineEntryJson {
    frame_id: u64,
    timestamp: i64,
    preview: String,
    uri: Option<String>,
    child_frames: Vec<u64>,
}

impl From<&memvid_core::TimelineEntry> for TimelineEntryJson {
    fn from(e: &memvid_core::TimelineEntry) -> Self {
        Self {
            frame_id: e.frame_id,
            timestamp: e.timestamp,
            preview: e.preview.clone(),
            uri: e.uri.clone(),
            child_frames: e.child_frames.clone(),
        }
    }
}

/// Timeline response for JSON serialization.
#[derive(Debug, Serialize)]
struct TimelineResponseJson {
    entries: Vec<TimelineEntryJson>,
    count: usize,
}

/// Query the timeline (chronological frame list).
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `query_json`: JSON string with query parameters (NULL for defaults)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with timeline entries on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Query JSON Schema
///
/// ```json
/// {
///   "limit": 100,
///   "since": 1234567890,
///   "until": 1234567899,
///   "reverse": false
/// }
/// ```
///
/// # Response JSON Schema
///
/// ```json
/// {
///   "entries": [
///     {
///       "frame_id": 1,
///       "timestamp": 1234567890,
///       "preview": "First 120 chars...",
///       "uri": "file://doc.txt",
///       "child_frames": [2, 3, 4]
///     }
///   ],
///   "count": 1
/// }
/// ```
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `query_json` must be a valid null-terminated UTF-8 string or NULL
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_timeline(
    handle: *mut MemvidHandle,
    query_json: *const c_char,
    error: *mut MemvidError,
) -> *mut c_char {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error_null(error, MemvidError::invalid_handle()) },
    };

    // Parse query JSON
    let query = match unsafe { cstr_to_option_string(query_json, "query_json") } {
        Ok(Some(json_str)) => match serde_json::from_str::<TimelineQueryJson>(&json_str) {
            Ok(q) => q.into_query(),
            Err(e) => return unsafe { set_error_null(error, MemvidError::json_parse(e)) },
        },
        Ok(None) => memvid_core::TimelineQuery::default(),
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    match handle.as_mut().timeline(query) {
        Ok(entries) => {
            let response = TimelineResponseJson {
                count: entries.len(),
                entries: entries.iter().map(TimelineEntryJson::from).collect(),
            };
            match serde_json::to_string(&response) {
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
