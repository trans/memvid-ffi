//! Search functions.

use crate::error::MemvidError;
use crate::handle::MemvidHandle;
use crate::util::{cstr_to_string, set_error_null, set_ok, string_to_cstr};
use serde::{Deserialize, Serialize};
use std::os::raw::c_char;

/// JSON schema for SearchRequest input.
#[derive(Debug, Deserialize)]
struct SearchRequestJson {
    /// Search query string
    query: String,
    /// Maximum number of results (default: 10)
    #[serde(default = "default_top_k")]
    top_k: usize,
    /// Characters of context around matches (default: 200)
    #[serde(default = "default_snippet_chars")]
    snippet_chars: usize,
    /// Filter to specific URI
    #[serde(default)]
    uri: Option<String>,
    /// Filter to URI scope/prefix
    #[serde(default)]
    scope: Option<String>,
    /// Pagination cursor
    #[serde(default)]
    cursor: Option<String>,
}

fn default_top_k() -> usize {
    10
}

fn default_snippet_chars() -> usize {
    200
}

impl SearchRequestJson {
    fn into_search_request(self) -> memvid_core::SearchRequest {
        memvid_core::SearchRequest {
            query: self.query,
            top_k: self.top_k,
            snippet_chars: self.snippet_chars,
            uri: self.uri,
            scope: self.scope,
            cursor: self.cursor,
            #[cfg(feature = "temporal_track")]
            temporal: None,
            as_of_frame: None,
            as_of_ts: None,
            no_sketch: false,
        }
    }
}

/// JSON schema for SearchResponse output.
#[derive(Debug, Serialize)]
struct SearchResponseJson {
    /// Original query
    query: String,
    /// Execution time in milliseconds
    elapsed_ms: u128,
    /// Total number of hits (may exceed returned hits due to pagination)
    total_hits: usize,
    /// Search hits
    hits: Vec<SearchHitJson>,
    /// Concatenated context from all hits
    context: String,
    /// Cursor for next page (null if no more results)
    next_cursor: Option<String>,
    /// Search engine used
    engine: String,
}

/// JSON schema for individual search hit.
#[derive(Debug, Serialize)]
struct SearchHitJson {
    /// Result rank (1-based)
    rank: usize,
    /// Frame ID
    frame_id: u64,
    /// Document URI
    uri: String,
    /// Document title
    title: Option<String>,
    /// Snippet text with context
    text: String,
    /// Character range in document (start, end)
    range: (usize, usize),
    /// Number of keyword matches
    matches: usize,
    /// Relevance score
    score: Option<f32>,
    /// Tags
    tags: Vec<String>,
    /// Labels
    labels: Vec<String>,
}

impl From<&memvid_core::SearchHit> for SearchHitJson {
    fn from(hit: &memvid_core::SearchHit) -> Self {
        let (tags, labels) = hit
            .metadata
            .as_ref()
            .map(|m| (m.tags.clone(), m.labels.clone()))
            .unwrap_or_default();

        Self {
            rank: hit.rank,
            frame_id: hit.frame_id,
            uri: hit.uri.clone(),
            title: hit.title.clone(),
            text: hit.text.clone(),
            range: hit.range,
            matches: hit.matches,
            score: hit.score,
            tags,
            labels,
        }
    }
}

impl From<&memvid_core::SearchResponse> for SearchResponseJson {
    fn from(resp: &memvid_core::SearchResponse) -> Self {
        Self {
            query: resp.query.clone(),
            elapsed_ms: resp.elapsed_ms,
            total_hits: resp.total_hits,
            hits: resp.hits.iter().map(SearchHitJson::from).collect(),
            context: resp.context.clone(),
            next_cursor: resp.next_cursor.clone(),
            engine: format!("{:?}", resp.engine),
        }
    }
}

/// Search the memory.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `request_json`: JSON string with SearchRequest
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with SearchResponse, NULL on failure.
///
/// # Ownership
///
/// Caller owns the returned string. Must call `memvid_string_free()`.
///
/// # Request JSON Schema
///
/// ```json
/// {
///   "query": "search terms",
///   "top_k": 10,
///   "snippet_chars": 200,
///   "uri": "mv2://optional/filter",
///   "scope": "mv2://scope/prefix",
///   "cursor": "pagination_token"
/// }
/// ```
///
/// # Response JSON Schema
///
/// ```json
/// {
///   "query": "search terms",
///   "elapsed_ms": 42,
///   "total_hits": 100,
///   "hits": [
///     {
///       "rank": 1,
///       "frame_id": 42,
///       "uri": "mv2://doc.txt",
///       "title": "Document Title",
///       "text": "...matching text...",
///       "range": [100, 150],
///       "matches": 3,
///       "score": 0.95,
///       "tags": ["tag1"],
///       "labels": ["label1"]
///     }
///   ],
///   "context": "combined context text",
///   "next_cursor": "token_or_null",
///   "engine": "Tantivy"
/// }
/// ```
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `request_json` must be a valid UTF-8 string
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_search(
    handle: *mut MemvidHandle,
    request_json: *const c_char,
    error: *mut MemvidError,
) -> *mut c_char {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error_null(error, MemvidError::invalid_handle()) },
    };

    // Parse request JSON
    let json_str = match unsafe { cstr_to_string(request_json, "request_json") } {
        Ok(s) => s,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    let request: SearchRequestJson = match serde_json::from_str(&json_str) {
        Ok(r) => r,
        Err(e) => return unsafe { set_error_null(error, MemvidError::json_parse(e)) },
    };

    // Perform search
    let response = match handle.as_mut().search(request.into_search_request()) {
        Ok(r) => r,
        Err(e) => return unsafe { set_error_null(error, MemvidError::from_core_error(e)) },
    };

    // Serialize response to JSON
    let response_json = SearchResponseJson::from(&response);
    match serde_json::to_string(&response_json) {
        Ok(s) => {
            unsafe { set_ok(error) };
            string_to_cstr(s)
        }
        Err(e) => unsafe { set_error_null(error, MemvidError::json_parse(e)) },
    }
}

/// Free a string returned by the FFI layer.
///
/// # Safety
///
/// - `str` must be a string returned by an FFI function, or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_string_free(str: *mut c_char) {
    if !str.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(str));
        }
    }
}
