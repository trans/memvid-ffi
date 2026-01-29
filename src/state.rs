//! State query functions (stats, frame_count).

use crate::error::MemvidError;
use crate::handle::MemvidHandle;
use crate::util::{set_error, set_ok};

/// Memory statistics.
///
/// All fields are value types that can be safely copied.
#[repr(C)]
#[derive(Debug, Default)]
pub struct MemvidStats {
    /// Total number of frames
    pub frame_count: u64,
    /// Number of active (non-deleted) frames
    pub active_frame_count: u64,
    /// Total file size in bytes
    pub size_bytes: u64,
    /// Total payload bytes (uncompressed)
    pub payload_bytes: u64,
    /// Logical bytes (after compression)
    pub logical_bytes: u64,
    /// Capacity limit in bytes
    pub capacity_bytes: u64,
    /// Whether lexical search index exists
    pub has_lex_index: u8,
    /// Whether vector search index exists
    pub has_vec_index: u8,
    /// Whether CLIP index exists
    pub has_clip_index: u8,
    /// Whether time index exists
    pub has_time_index: u8,
    /// Padding for alignment
    pub _padding: [u8; 4],
    /// WAL size in bytes
    pub wal_bytes: u64,
    /// Lexical index size in bytes
    pub lex_index_bytes: u64,
    /// Vector index size in bytes
    pub vec_index_bytes: u64,
    /// Time index size in bytes
    pub time_index_bytes: u64,
    /// Number of vectors stored
    pub vector_count: u64,
    /// Number of CLIP images
    pub clip_image_count: u64,
    /// Compression ratio (percentage, 0-100+)
    pub compression_ratio_percent: f64,
    /// Storage savings percentage
    pub savings_percent: f64,
    /// Storage utilization percentage
    pub storage_utilisation_percent: f64,
    /// Remaining capacity in bytes
    pub remaining_capacity_bytes: u64,
}

impl From<&memvid_core::Stats> for MemvidStats {
    fn from(s: &memvid_core::Stats) -> Self {
        Self {
            frame_count: s.frame_count,
            active_frame_count: s.active_frame_count,
            size_bytes: s.size_bytes,
            payload_bytes: s.payload_bytes,
            logical_bytes: s.logical_bytes,
            capacity_bytes: s.capacity_bytes,
            has_lex_index: s.has_lex_index as u8,
            has_vec_index: s.has_vec_index as u8,
            has_clip_index: s.has_clip_index as u8,
            has_time_index: s.has_time_index as u8,
            _padding: [0; 4],
            wal_bytes: s.wal_bytes,
            lex_index_bytes: s.lex_index_bytes,
            vec_index_bytes: s.vec_index_bytes,
            time_index_bytes: s.time_index_bytes,
            vector_count: s.vector_count,
            clip_image_count: s.clip_image_count,
            compression_ratio_percent: s.compression_ratio_percent,
            savings_percent: s.savings_percent,
            storage_utilisation_percent: s.storage_utilisation_percent,
            remaining_capacity_bytes: s.remaining_capacity_bytes,
        }
    }
}

/// Get memory statistics.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `stats`: Out-parameter for statistics (must not be NULL)
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// 1 on success, 0 on failure.
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `stats` must be a valid pointer
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_stats(
    handle: *mut MemvidHandle,
    stats: *mut MemvidStats,
    error: *mut MemvidError,
) -> i32 {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error(error, MemvidError::invalid_handle()) },
    };

    if stats.is_null() {
        return unsafe { set_error(error, MemvidError::null_pointer("stats")) };
    }

    match handle.as_ref().stats() {
        Ok(s) => {
            unsafe { *stats = MemvidStats::from(&s) };
            unsafe { set_ok(error) };
            1
        }
        Err(e) => unsafe { set_error(error, MemvidError::from_core_error(e)) },
    }
}

/// Get the number of frames in the memory.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// Frame count on success, 0 on error (check error->code to distinguish
/// from an empty memory).
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_frame_count(
    handle: *mut MemvidHandle,
    error: *mut MemvidError,
) -> u64 {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error(error, MemvidError::invalid_handle()) },
    };

    unsafe { set_ok(error) };
    handle.as_ref().frame_count() as u64
}
