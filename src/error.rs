//! Error handling for the FFI layer.
//!
//! This module provides C-compatible error types and conversion from memvid-core errors.

use std::ffi::CString;
use std::os::raw::c_char;

/// Error codes for FFI functions.
///
/// These codes are stable and can be matched in C/Crystal code.
/// Codes 1-99 map to memvid-core error variants.
/// Codes 100+ are FFI-specific errors.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemvidErrorCode {
    /// No error
    Ok = 0,

    // I/O and file errors (1-10)
    /// General I/O error
    Io = 1,
    /// File encoding/decoding error
    Encode = 2,
    /// File decoding error
    Decode = 3,
    /// Lock acquisition failed
    Lock = 4,
    /// File is locked by another process
    Locked = 5,
    /// Checksum mismatch
    ChecksumMismatch = 6,
    /// Invalid header
    InvalidHeader = 7,
    /// File is encrypted
    EncryptedFile = 8,
    /// Invalid table of contents
    InvalidToc = 9,
    /// Invalid time index
    InvalidTimeIndex = 10,

    // Index errors (11-20)
    /// Lexical index not enabled
    LexNotEnabled = 11,
    /// Vector index not enabled
    VecNotEnabled = 12,
    /// CLIP index not enabled
    ClipNotEnabled = 13,
    /// Vector dimension mismatch
    VecDimensionMismatch = 14,
    /// Invalid sketch track
    InvalidSketchTrack = 15,
    /// Invalid logic mesh
    InvalidLogicMesh = 16,
    /// Logic mesh not enabled
    LogicMeshNotEnabled = 17,
    /// NER model not available
    NerModelNotAvailable = 18,

    // Capacity and tier errors (21-30)
    /// Invalid tier
    InvalidTier = 21,
    /// Ticket sequence error
    TicketSequence = 22,
    /// Ticket required
    TicketRequired = 23,
    /// Capacity exceeded
    CapacityExceeded = 24,
    /// API key required
    ApiKeyRequired = 25,
    /// Memory already bound
    MemoryAlreadyBound = 26,

    // State errors (31-40)
    /// Requires sealed memory
    RequiresSealed = 31,
    /// Requires open memory
    RequiresOpen = 32,
    /// Doctor no operation
    DoctorNoOp = 33,
    /// Doctor error
    Doctor = 34,

    // Feature and query errors (41-50)
    /// Feature unavailable
    FeatureUnavailable = 41,
    /// Invalid cursor
    InvalidCursor = 42,
    /// Invalid frame
    InvalidFrame = 43,
    /// Frame not found
    FrameNotFound = 44,
    /// Frame not found by URI
    FrameNotFoundByUri = 45,
    /// Invalid query
    InvalidQuery = 46,

    // Signature and verification errors (51-60)
    /// Ticket signature invalid
    TicketSignatureInvalid = 51,
    /// Model signature invalid
    ModelSignatureInvalid = 52,
    /// Model manifest invalid
    ModelManifestInvalid = 53,
    /// Model integrity error
    ModelIntegrity = 54,

    // Processing errors (61-70)
    /// Extraction failed
    ExtractionFailed = 61,
    /// Embedding failed
    EmbeddingFailed = 62,
    /// Rerank failed
    RerankFailed = 63,
    /// Tantivy error
    Tantivy = 64,
    /// Table extraction error
    TableExtraction = 65,
    /// Schema validation error
    SchemaValidation = 66,

    // WAL errors (71-80)
    /// WAL corruption
    WalCorruption = 71,
    /// Manifest WAL corrupted
    ManifestWalCorrupted = 72,
    /// Checkpoint failed
    CheckpointFailed = 73,
    /// Auxiliary file detected
    AuxiliaryFileDetected = 74,

    // FFI-specific errors (100+)
    /// Null pointer passed
    NullPointer = 100,
    /// Invalid UTF-8 string
    InvalidUtf8 = 101,
    /// JSON parse error
    JsonParse = 102,
    /// Invalid handle
    InvalidHandle = 103,
    /// Unknown error
    Unknown = 255,
}

/// Error structure returned via out-parameter.
///
/// # Memory Ownership
///
/// The `message` field is owned by the FFI layer when non-null.
/// Call `memvid_error_free()` to release the message memory.
#[repr(C)]
pub struct MemvidError {
    /// Error code
    pub code: MemvidErrorCode,
    /// Error message (NULL if code == Ok)
    pub message: *mut c_char,
}

impl MemvidError {
    /// Create a success result (no error).
    pub fn ok() -> Self {
        Self {
            code: MemvidErrorCode::Ok,
            message: std::ptr::null_mut(),
        }
    }

    /// Create an error from a memvid-core error.
    pub fn from_core_error(e: memvid_core::MemvidError) -> Self {
        let code = error_code_from_core(&e);
        let message = CString::new(e.to_string())
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut());
        Self { code, message }
    }

    /// Create a null pointer error.
    pub fn null_pointer(param: &str) -> Self {
        let msg = format!("null pointer passed for parameter: {param}");
        Self {
            code: MemvidErrorCode::NullPointer,
            message: CString::new(msg)
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut()),
        }
    }

    /// Create an invalid UTF-8 error.
    pub fn invalid_utf8(context: &str) -> Self {
        let msg = format!("invalid UTF-8 in {context}");
        Self {
            code: MemvidErrorCode::InvalidUtf8,
            message: CString::new(msg)
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut()),
        }
    }

    /// Create a JSON parse error.
    pub fn json_parse(e: serde_json::Error) -> Self {
        let msg = format!("JSON parse error: {e}");
        Self {
            code: MemvidErrorCode::JsonParse,
            message: CString::new(msg)
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut()),
        }
    }

    /// Create a JSON serialization error.
    pub fn json_serialize(e: serde_json::Error) -> Self {
        let msg = format!("JSON serialization error: {e}");
        Self {
            code: MemvidErrorCode::JsonParse,
            message: CString::new(msg)
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut()),
        }
    }

    /// Create an invalid handle error.
    pub fn invalid_handle() -> Self {
        Self {
            code: MemvidErrorCode::InvalidHandle,
            message: CString::new("invalid or null handle")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut()),
        }
    }
}

/// Convert a memvid-core error to an FFI error code.
fn error_code_from_core(e: &memvid_core::MemvidError) -> MemvidErrorCode {
    use memvid_core::MemvidError::*;

    match e {
        Io { .. } => MemvidErrorCode::Io,
        Encode(_) => MemvidErrorCode::Encode,
        Decode(_) => MemvidErrorCode::Decode,
        Lock(_) => MemvidErrorCode::Lock,
        Locked(_) => MemvidErrorCode::Locked,
        ChecksumMismatch { .. } => MemvidErrorCode::ChecksumMismatch,
        InvalidHeader { .. } => MemvidErrorCode::InvalidHeader,
        EncryptedFile { .. } => MemvidErrorCode::EncryptedFile,
        InvalidToc { .. } => MemvidErrorCode::InvalidToc,
        InvalidTimeIndex { .. } => MemvidErrorCode::InvalidTimeIndex,
        InvalidSketchTrack { .. } => MemvidErrorCode::InvalidSketchTrack,
        InvalidLogicMesh { .. } => MemvidErrorCode::InvalidLogicMesh,
        LogicMeshNotEnabled => MemvidErrorCode::LogicMeshNotEnabled,
        NerModelNotAvailable { .. } => MemvidErrorCode::NerModelNotAvailable,
        InvalidTier => MemvidErrorCode::InvalidTier,
        LexNotEnabled => MemvidErrorCode::LexNotEnabled,
        VecNotEnabled => MemvidErrorCode::VecNotEnabled,
        ClipNotEnabled => MemvidErrorCode::ClipNotEnabled,
        VecDimensionMismatch { .. } => MemvidErrorCode::VecDimensionMismatch,
        AuxiliaryFileDetected { .. } => MemvidErrorCode::AuxiliaryFileDetected,
        WalCorruption { .. } => MemvidErrorCode::WalCorruption,
        ManifestWalCorrupted { .. } => MemvidErrorCode::ManifestWalCorrupted,
        CheckpointFailed { .. } => MemvidErrorCode::CheckpointFailed,
        TicketSequence { .. } => MemvidErrorCode::TicketSequence,
        TicketRequired { .. } => MemvidErrorCode::TicketRequired,
        CapacityExceeded { .. } => MemvidErrorCode::CapacityExceeded,
        ApiKeyRequired { .. } => MemvidErrorCode::ApiKeyRequired,
        MemoryAlreadyBound { .. } => MemvidErrorCode::MemoryAlreadyBound,
        RequiresSealed => MemvidErrorCode::RequiresSealed,
        RequiresOpen => MemvidErrorCode::RequiresOpen,
        DoctorNoOp => MemvidErrorCode::DoctorNoOp,
        Doctor { .. } => MemvidErrorCode::Doctor,
        FeatureUnavailable { .. } => MemvidErrorCode::FeatureUnavailable,
        InvalidCursor { .. } => MemvidErrorCode::InvalidCursor,
        InvalidFrame { .. } => MemvidErrorCode::InvalidFrame,
        FrameNotFound { .. } => MemvidErrorCode::FrameNotFound,
        FrameNotFoundByUri { .. } => MemvidErrorCode::FrameNotFoundByUri,
        TicketSignatureInvalid { .. } => MemvidErrorCode::TicketSignatureInvalid,
        ModelSignatureInvalid { .. } => MemvidErrorCode::ModelSignatureInvalid,
        ModelManifestInvalid { .. } => MemvidErrorCode::ModelManifestInvalid,
        ModelIntegrity { .. } => MemvidErrorCode::ModelIntegrity,
        ExtractionFailed { .. } => MemvidErrorCode::ExtractionFailed,
        EmbeddingFailed { .. } => MemvidErrorCode::EmbeddingFailed,
        RerankFailed { .. } => MemvidErrorCode::RerankFailed,
        InvalidQuery { .. } => MemvidErrorCode::InvalidQuery,
        Tantivy { .. } => MemvidErrorCode::Tantivy,
        TableExtraction { .. } => MemvidErrorCode::TableExtraction,
        SchemaValidation { .. } => MemvidErrorCode::SchemaValidation,
        #[cfg(feature = "temporal_track")]
        InvalidTemporalTrack { .. } => MemvidErrorCode::InvalidTimeIndex,
    }
}

/// Free error message memory.
///
/// Safe to call with NULL error or NULL message.
///
/// # Safety
///
/// The error pointer must be valid or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_error_free(error: *mut MemvidError) {
    if error.is_null() {
        return;
    }
    unsafe {
        let err = &mut *error;
        if !err.message.is_null() {
            drop(CString::from_raw(err.message));
            err.message = std::ptr::null_mut();
        }
    }
}
