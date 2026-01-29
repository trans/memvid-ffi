//! C FFI bindings for memvid-core.
//!
//! This crate provides a C-compatible interface to the memvid-core library,
//! enabling use from Crystal, Go, Ruby, and other languages with C FFI support.
//!
//! # Thread Safety
//!
//! `MemvidHandle` is NOT `Send` or `Sync`. All operations on a handle must
//! occur from the same thread that created it, or external synchronization
//! must be provided.
//!
//! # Memory Management
//!
//! - Handles returned by `memvid_create`/`memvid_open` must be freed with `memvid_close`
//! - Strings returned by functions must be freed with `memvid_string_free`
//! - Error messages must be freed with `memvid_error_free`
//!
//! # Feature Flags
//!
//! The FFI library respects the same feature flags as memvid-core:
//!
//! - `lex` (default): Lexical/full-text search via Tantivy
//! - `vec`: Vector similarity search via HNSW
//! - `clip`: CLIP visual embeddings (requires `vec`)
//! - `full`: All features enabled

#![allow(clippy::missing_safety_doc)]

mod ask;
mod doctor;
mod error;
mod frame;
mod handle;
mod lifecycle;
mod mutation;
mod search;
mod state;
mod timeline;
mod util;
mod verify;

// Re-export all public FFI types and functions
pub use ask::memvid_ask;
pub use doctor::{memvid_doctor, memvid_doctor_apply, memvid_doctor_plan};
pub use error::{memvid_error_free, MemvidError, MemvidErrorCode};
pub use frame::{memvid_delete_frame, memvid_frame_by_id, memvid_frame_by_uri, memvid_frame_content};
pub use handle::MemvidHandle;
pub use lifecycle::{memvid_close, memvid_create, memvid_open};
pub use mutation::{memvid_commit, memvid_put_bytes, memvid_put_bytes_with_options};
pub use search::{memvid_search, memvid_string_free};
pub use state::{memvid_frame_count, memvid_stats, MemvidStats};
pub use timeline::memvid_timeline;
pub use verify::memvid_verify;

use std::os::raw::c_char;

/// Library version string.
///
/// # Returns
///
/// Static string containing the version (e.g., "0.1.0").
/// Do not free this string.
#[unsafe(no_mangle)]
pub extern "C" fn memvid_version() -> *const c_char {
    // Include null terminator in the static string
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

/// Feature flags bitmask.
///
/// # Returns
///
/// Bitmask indicating which features are compiled in:
/// - Bit 0 (0x01): `lex` - Lexical search
/// - Bit 1 (0x02): `vec` - Vector search
/// - Bit 2 (0x04): `clip` - CLIP embeddings
///
/// # Example
///
/// ```c
/// uint32_t features = memvid_features();
/// if (features & 0x01) { /* lex enabled */ }
/// if (features & 0x02) { /* vec enabled */ }
/// if (features & 0x04) { /* clip enabled */ }
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn memvid_features() -> u32 {
    let mut flags = 0u32;

    #[cfg(feature = "lex")]
    {
        flags |= 1 << 0;
    }

    #[cfg(feature = "vec")]
    {
        flags |= 1 << 1;
    }

    #[cfg(feature = "clip")]
    {
        flags |= 1 << 2;
    }

    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_version() {
        let version = memvid_version();
        assert!(!version.is_null());
        let version_str = unsafe { std::ffi::CStr::from_ptr(version) };
        assert!(!version_str.to_str().unwrap().is_empty());
    }

    #[test]
    fn test_features() {
        let features = memvid_features();
        // At minimum, lex should be enabled (default feature)
        #[cfg(feature = "lex")]
        assert!(features & 0x01 != 0);
    }

    #[test]
    fn test_create_and_close() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_create.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };

        assert!(!handle.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        unsafe { memvid_close(handle) };

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_put_and_search() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_put_search.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put some content
        let content = b"Hello, this is a test document about Rust FFI bindings.";
        let frame_id =
            unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        assert!(frame_id > 0 || error.code == MemvidErrorCode::Ok);

        // Commit
        let result = unsafe { memvid_commit(handle, &mut error) };
        assert_eq!(result, 1);

        // Search
        let search_json = CString::new(r#"{"query": "Rust FFI", "top_k": 5}"#).unwrap();
        let result_ptr = unsafe { memvid_search(handle, search_json.as_ptr(), &mut error) };

        if !result_ptr.is_null() {
            let result_str = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
            let result_json = result_str.to_str().unwrap();
            assert!(result_json.contains("hits"));
            unsafe { memvid_string_free(result_ptr) };
        }

        // Get stats
        let mut stats = MemvidStats::default();
        let result = unsafe { memvid_stats(handle, &mut stats, &mut error) };
        assert_eq!(result, 1);
        assert!(stats.frame_count >= 1);

        unsafe { memvid_close(handle) };

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_open() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_open.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();

        // Create and close a memory file
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());
        let content = b"Test content for open test.";
        unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Reopen it
        let handle = unsafe { memvid_open(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        // Verify content persisted
        let count = unsafe { memvid_frame_count(handle, &mut error) };
        assert_eq!(count, 1);

        unsafe { memvid_close(handle) };

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_put_bytes_with_options() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_put_options.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put with options
        let content = b"Document with metadata.";
        let options = CString::new(r#"{"title": "Test Doc", "uri": "test://doc1"}"#).unwrap();
        let frame_id = unsafe {
            memvid_put_bytes_with_options(
                handle,
                content.as_ptr(),
                content.len(),
                options.as_ptr(),
                &mut error,
            )
        };
        assert!(frame_id > 0);
        assert_eq!(error.code, MemvidErrorCode::Ok);

        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_frame_count() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_frame_count.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Initially empty
        let count = unsafe { memvid_frame_count(handle, &mut error) };
        assert_eq!(count, 0);
        assert_eq!(error.code, MemvidErrorCode::Ok);

        // Add frames
        let content1 = b"First document.";
        let content2 = b"Second document.";
        let content3 = b"Third document.";
        unsafe { memvid_put_bytes(handle, content1.as_ptr(), content1.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content2.as_ptr(), content2.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content3.as_ptr(), content3.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };

        // Should have 3 frames
        let count = unsafe { memvid_frame_count(handle, &mut error) };
        assert_eq!(count, 3);

        unsafe { memvid_close(handle) };

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_error_free() {
        let mut error = MemvidError::ok();

        // error_free should handle null message gracefully
        unsafe { memvid_error_free(&mut error) };
        assert!(error.message.is_null());

        // Trigger an actual error (null path)
        let handle = unsafe { memvid_create(std::ptr::null(), &mut error) };
        assert!(handle.is_null());
        assert_eq!(error.code, MemvidErrorCode::NullPointer);
        assert!(!error.message.is_null());

        // Free the error message
        unsafe { memvid_error_free(&mut error) };
        assert!(error.message.is_null());
    }

    #[test]
    fn test_frame_by_id() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_frame_by_id.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put content with options
        let content = b"Test document for frame_by_id.";
        let options = CString::new(r#"{"title": "Test Title", "uri": "test://frame1"}"#).unwrap();
        let _seq = unsafe {
            memvid_put_bytes_with_options(
                handle,
                content.as_ptr(),
                content.len(),
                options.as_ptr(),
                &mut error,
            )
        };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Reopen and get frame by ID (frame IDs are 0-indexed)
        let handle = unsafe { memvid_open(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // First frame has id=0
        let frame_json = unsafe { memvid_frame_by_id(handle, 0, &mut error) };
        if frame_json.is_null() {
            let msg = if !error.message.is_null() {
                unsafe { std::ffi::CStr::from_ptr(error.message) }
                    .to_str()
                    .unwrap_or("unknown")
            } else {
                "no message"
            };
            panic!("frame_by_id failed: {:?} - {}", error.code, msg);
        }
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let frame_str = unsafe { std::ffi::CStr::from_ptr(frame_json) };
        let json = frame_str.to_str().unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"title\":\"Test Title\""));
        assert!(json.contains("\"uri\":\"test://frame1\""));

        unsafe { memvid_string_free(frame_json) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_frame_by_uri() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_frame_by_uri.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put content with URI
        let content = b"Document with unique URI.";
        let options = CString::new(r#"{"uri": "test://unique-doc"}"#).unwrap();
        unsafe {
            memvid_put_bytes_with_options(
                handle,
                content.as_ptr(),
                content.len(),
                options.as_ptr(),
                &mut error,
            )
        };
        unsafe { memvid_commit(handle, &mut error) };

        // Get frame by URI
        let uri = CString::new("test://unique-doc").unwrap();
        let frame_json = unsafe { memvid_frame_by_uri(handle, uri.as_ptr(), &mut error) };
        assert!(!frame_json.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let frame_str = unsafe { std::ffi::CStr::from_ptr(frame_json) };
        let json = frame_str.to_str().unwrap();
        assert!(json.contains("\"uri\":\"test://unique-doc\""));

        unsafe { memvid_string_free(frame_json) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_frame_content() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_frame_content.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put content
        let content = b"This is the full content of the frame.";
        let _seq =
            unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Reopen and get content (frame IDs are 0-indexed)
        let handle = unsafe { memvid_open(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // First frame has id=0
        let content_ptr = unsafe { memvid_frame_content(handle, 0, &mut error) };
        assert!(!content_ptr.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let content_str = unsafe { std::ffi::CStr::from_ptr(content_ptr) };
        // Content includes auto-extracted metadata, so just check it starts with our text
        assert!(content_str
            .to_str()
            .unwrap()
            .starts_with("This is the full content of the frame."));

        unsafe { memvid_string_free(content_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_delete_frame() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_delete_frame.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put content
        let content = b"Document to be deleted.";
        let _seq =
            unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Reopen
        let handle = unsafe { memvid_open(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Verify we have 1 frame
        let mut stats = MemvidStats::default();
        unsafe { memvid_stats(handle, &mut stats, &mut error) };
        assert_eq!(stats.active_frame_count, 1);

        // Delete frame (frame IDs are 0-indexed, first frame has id=0)
        let seq = unsafe { memvid_delete_frame(handle, 0, &mut error) };
        if seq == 0 && error.code != MemvidErrorCode::Ok {
            let msg = if !error.message.is_null() {
                unsafe { std::ffi::CStr::from_ptr(error.message) }
                    .to_str()
                    .unwrap_or("unknown")
            } else {
                "no message"
            };
            panic!("delete_frame failed: {:?} - {}", error.code, msg);
        }
        assert!(seq > 0);
        unsafe { memvid_commit(handle, &mut error) };

        // Verify active count is now 0
        unsafe { memvid_stats(handle, &mut stats, &mut error) };
        assert_eq!(stats.active_frame_count, 0);

        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_timeline() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_timeline.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put multiple documents
        let content1 = b"First document in timeline.";
        let content2 = b"Second document in timeline.";
        let content3 = b"Third document in timeline.";
        unsafe { memvid_put_bytes(handle, content1.as_ptr(), content1.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content2.as_ptr(), content2.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content3.as_ptr(), content3.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };

        // Query timeline with default options
        let timeline_ptr = unsafe { memvid_timeline(handle, std::ptr::null(), &mut error) };
        assert!(!timeline_ptr.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let timeline_str = unsafe { std::ffi::CStr::from_ptr(timeline_ptr) };
        let json = timeline_str.to_str().unwrap();
        assert!(json.contains("\"entries\""));
        assert!(json.contains("\"count\":3"));

        unsafe { memvid_string_free(timeline_ptr) };

        // Query with limit
        let query = CString::new(r#"{"limit": 2}"#).unwrap();
        let timeline_ptr = unsafe { memvid_timeline(handle, query.as_ptr(), &mut error) };
        assert!(!timeline_ptr.is_null());

        let timeline_str = unsafe { std::ffi::CStr::from_ptr(timeline_ptr) };
        let json = timeline_str.to_str().unwrap();
        assert!(json.contains("\"count\":2"));

        unsafe { memvid_string_free(timeline_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_verify() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_verify.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();

        // Create a valid memory file
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());
        let content = b"Content for verification test.";
        unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Verify the file (shallow)
        let report_ptr = unsafe { memvid_verify(path_cstr.as_ptr(), 0, &mut error) };
        assert!(!report_ptr.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let report_str = unsafe { std::ffi::CStr::from_ptr(report_ptr) };
        let json = report_str.to_str().unwrap();
        assert!(json.contains("\"overall_status\":\"passed\""));
        assert!(json.contains("\"checks\""));

        unsafe { memvid_string_free(report_ptr) };

        // Verify the file (deep)
        let report_ptr = unsafe { memvid_verify(path_cstr.as_ptr(), 1, &mut error) };
        assert!(!report_ptr.is_null());

        let report_str = unsafe { std::ffi::CStr::from_ptr(report_ptr) };
        let json = report_str.to_str().unwrap();
        assert!(json.contains("\"overall_status\":\"passed\""));

        unsafe { memvid_string_free(report_ptr) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_ask() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_ask.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Add some content to search
        let content1 = b"The capital of France is Paris. It is known for the Eiffel Tower.";
        let content2 = b"Berlin is the capital of Germany. It has a famous wall.";
        let content3 = b"Tokyo is the capital of Japan. It is a very large city.";
        unsafe { memvid_put_bytes(handle, content1.as_ptr(), content1.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content2.as_ptr(), content2.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content3.as_ptr(), content3.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };

        // Ask a question (context_only mode - no LLM synthesis)
        let ask_json =
            CString::new(r#"{"question": "What is the capital of France?", "top_k": 5}"#).unwrap();
        let result_ptr = unsafe { memvid_ask(handle, ask_json.as_ptr(), &mut error) };

        if result_ptr.is_null() {
            let msg = if !error.message.is_null() {
                unsafe { std::ffi::CStr::from_ptr(error.message) }
                    .to_str()
                    .unwrap_or("unknown")
            } else {
                "no message"
            };
            panic!("memvid_ask failed: {:?} - {}", error.code, msg);
        }
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let result_str = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
        let json = result_str.to_str().unwrap();

        // Verify response structure
        assert!(json.contains("\"question\""));
        assert!(json.contains("\"context_only\":true"));
        assert!(json.contains("\"retrieval\""));
        assert!(json.contains("\"stats\""));
        assert!(json.contains("\"context_fragments\""));

        unsafe { memvid_string_free(result_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_doctor() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_doctor.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();

        // Create a valid memory file
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());
        let content = b"Content for doctor test.";
        unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Run doctor (should report clean)
        let report_ptr = unsafe { memvid_doctor(path_cstr.as_ptr(), std::ptr::null(), &mut error) };
        assert!(!report_ptr.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let report_str = unsafe { std::ffi::CStr::from_ptr(report_ptr) };
        let json = report_str.to_str().unwrap();
        assert!(json.contains("\"status\""));
        assert!(json.contains("\"plan\""));
        assert!(json.contains("\"metrics\""));

        unsafe { memvid_string_free(report_ptr) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_doctor_plan() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_doctor_plan.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();

        // Create a valid memory file
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());
        let content = b"Content for doctor plan test.";
        unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Get doctor plan
        let plan_ptr =
            unsafe { memvid_doctor_plan(path_cstr.as_ptr(), std::ptr::null(), &mut error) };
        assert!(!plan_ptr.is_null());
        assert_eq!(error.code, MemvidErrorCode::Ok);

        let plan_str = unsafe { std::ffi::CStr::from_ptr(plan_ptr) };
        let json = plan_str.to_str().unwrap();
        assert!(json.contains("\"version\""));
        assert!(json.contains("\"file_path\""));
        assert!(json.contains("\"phases\""));

        unsafe { memvid_string_free(plan_ptr) };
        let _ = std::fs::remove_file(&path);
    }

    // ==========================================================================
    // Edge Case Tests
    // ==========================================================================

    #[test]
    fn test_empty_content() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_empty_content.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put empty content
        let content = b"";
        let frame_id =
            unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        // Should succeed even with empty content
        assert!(frame_id > 0 || error.code == MemvidErrorCode::Ok);

        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_unicode_content() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_unicode.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Put unicode content
        let content = "Hello 世界! 🎉 Привет мир! مرحبا بالعالم";
        let content_bytes = content.as_bytes();
        let _frame_id = unsafe {
            memvid_put_bytes(handle, content_bytes.as_ptr(), content_bytes.len(), &mut error)
        };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Reopen and verify content
        let handle = unsafe { memvid_open(path_cstr.as_ptr(), &mut error) };
        let content_ptr = unsafe { memvid_frame_content(handle, 0, &mut error) };
        assert!(!content_ptr.is_null());

        let content_str = unsafe { std::ffi::CStr::from_ptr(content_ptr) };
        let retrieved = content_str.to_str().unwrap();
        assert!(retrieved.contains("世界"));
        assert!(retrieved.contains("🎉"));

        unsafe { memvid_string_free(content_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_search_no_results() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_search_no_results.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };

        let content = b"The quick brown fox jumps over the lazy dog.";
        unsafe { memvid_put_bytes(handle, content.as_ptr(), content.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };

        // Search for something not in the content
        let search_json = CString::new(r#"{"query": "elephant zebra giraffe", "top_k": 5}"#).unwrap();
        let result_ptr = unsafe { memvid_search(handle, search_json.as_ptr(), &mut error) };

        assert!(!result_ptr.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
        let json = result_str.to_str().unwrap();
        // Should have empty hits array
        assert!(json.contains("\"hits\":[]") || json.contains("\"hits\": []"));

        unsafe { memvid_string_free(result_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_search_empty_memory() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_search_empty.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };

        // Search empty memory
        let search_json = CString::new(r#"{"query": "anything", "top_k": 5}"#).unwrap();
        let result_ptr = unsafe { memvid_search(handle, search_json.as_ptr(), &mut error) };

        assert!(!result_ptr.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
        let json = result_str.to_str().unwrap();
        assert!(json.contains("hits"));

        unsafe { memvid_string_free(result_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_null_pointer_handling() {
        let mut error = MemvidError::ok();

        // Null path to create
        let handle = unsafe { memvid_create(std::ptr::null(), &mut error) };
        assert!(handle.is_null());
        assert_eq!(error.code, MemvidErrorCode::NullPointer);
        unsafe { memvid_error_free(&mut error) };

        // Null path to open
        let handle = unsafe { memvid_open(std::ptr::null(), &mut error) };
        assert!(handle.is_null());
        assert_eq!(error.code, MemvidErrorCode::NullPointer);
        unsafe { memvid_error_free(&mut error) };

        // Null handle to close (should not crash)
        unsafe { memvid_close(std::ptr::null_mut()) };
    }

    // Note: test_invalid_handle removed - using garbage pointers causes undefined behavior
    // The FFI layer validates handles but can't protect against completely invalid memory

    #[test]
    fn test_persistence_round_trip() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_round_trip.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();

        // Create memory, add content, close
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        let content1 = b"First document for persistence test.";
        let content2 = b"Second document for persistence test.";
        let content3 = b"Third document for persistence test.";
        unsafe { memvid_put_bytes(handle, content1.as_ptr(), content1.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content2.as_ptr(), content2.len(), &mut error) };
        unsafe { memvid_put_bytes(handle, content3.as_ptr(), content3.len(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };
        unsafe { memvid_close(handle) };

        // Reopen and verify everything persisted
        let handle = unsafe { memvid_open(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        let count = unsafe { memvid_frame_count(handle, &mut error) };
        assert_eq!(count, 3);

        // Verify content
        let content_ptr = unsafe { memvid_frame_content(handle, 0, &mut error) };
        assert!(!content_ptr.is_null());
        let content_str = unsafe { std::ffi::CStr::from_ptr(content_ptr) };
        assert!(content_str.to_str().unwrap().contains("First document"));
        unsafe { memvid_string_free(content_ptr) };

        // Search should still work
        let search_json = CString::new(r#"{"query": "persistence test", "top_k": 10}"#).unwrap();
        let result_ptr = unsafe { memvid_search(handle, search_json.as_ptr(), &mut error) };
        assert!(!result_ptr.is_null());

        let result_str = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
        let json = result_str.to_str().unwrap();
        assert!(json.contains("hits"));

        unsafe { memvid_string_free(result_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_many_frames() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_many_frames.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        assert!(!handle.is_null());

        // Add 50 frames
        for i in 0..50 {
            let content = format!("Document number {} with some searchable content.", i);
            let content_bytes = content.as_bytes();
            unsafe {
                memvid_put_bytes(handle, content_bytes.as_ptr(), content_bytes.len(), &mut error)
            };
        }
        unsafe { memvid_commit(handle, &mut error) };

        // Verify count
        let count = unsafe { memvid_frame_count(handle, &mut error) };
        assert_eq!(count, 50);

        // Search should still work
        let search_json = CString::new(r#"{"query": "Document number", "top_k": 100}"#).unwrap();
        let result_ptr = unsafe { memvid_search(handle, search_json.as_ptr(), &mut error) };
        assert!(!result_ptr.is_null());

        unsafe { memvid_string_free(result_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_timeline_empty_memory() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_ffi_timeline_empty.mv2");
        let path_cstr = CString::new(path.to_str().unwrap()).unwrap();

        let mut error = MemvidError::ok();
        let handle = unsafe { memvid_create(path_cstr.as_ptr(), &mut error) };
        unsafe { memvid_commit(handle, &mut error) };

        let timeline_ptr = unsafe { memvid_timeline(handle, std::ptr::null(), &mut error) };
        assert!(!timeline_ptr.is_null());

        let timeline_str = unsafe { std::ffi::CStr::from_ptr(timeline_ptr) };
        let json = timeline_str.to_str().unwrap();
        assert!(json.contains("\"count\":0"));

        unsafe { memvid_string_free(timeline_ptr) };
        unsafe { memvid_close(handle) };
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_verify_nonexistent_file() {
        let mut error = MemvidError::ok();
        let path = CString::new("/nonexistent/path/to/file.mv2").unwrap();

        let report_ptr = unsafe { memvid_verify(path.as_ptr(), 0, &mut error) };
        assert!(report_ptr.is_null());
        assert_ne!(error.code, MemvidErrorCode::Ok);

        unsafe { memvid_error_free(&mut error) };
    }

    // Note: test_double_close removed - double-free is undefined behavior
    // The Crystal wrapper handles this safely by tracking closed state
}
