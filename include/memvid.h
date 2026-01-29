/**
 * memvid - Single-file AI Memory Library
 * C FFI Bindings
 *
 * Thread Safety: MemvidHandle is NOT thread-safe. Use from a single thread
 * or provide external synchronization.
 *
 * Memory Ownership:
 * - Handles: Caller owns, must call memvid_close()
 * - Returned strings: Caller owns, must call memvid_string_free()
 * - MemvidError.message: FFI owns, call memvid_error_free()
 */

#ifndef MEMVID_FFI_H
#define MEMVID_FFI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Error codes for FFI functions.
 *
 * Codes 1-99 map to memvid-core error variants.
 * Codes 100+ are FFI-specific errors.
 */
typedef enum MemvidErrorCode {
    /** No error */
    MemvidErrorCode_Ok = 0,
    /** General I/O error */
    MemvidErrorCode_Io = 1,
    /** File encoding error */
    MemvidErrorCode_Encode = 2,
    /** File decoding error */
    MemvidErrorCode_Decode = 3,
    /** Lock acquisition failed */
    MemvidErrorCode_Lock = 4,
    /** File is locked by another process */
    MemvidErrorCode_Locked = 5,
    /** Checksum mismatch */
    MemvidErrorCode_ChecksumMismatch = 6,
    /** Invalid header */
    MemvidErrorCode_InvalidHeader = 7,
    /** File is encrypted */
    MemvidErrorCode_EncryptedFile = 8,
    /** Invalid table of contents */
    MemvidErrorCode_InvalidToc = 9,
    /** Invalid time index */
    MemvidErrorCode_InvalidTimeIndex = 10,
    /** Lexical index not enabled */
    MemvidErrorCode_LexNotEnabled = 11,
    /** Vector index not enabled */
    MemvidErrorCode_VecNotEnabled = 12,
    /** CLIP index not enabled */
    MemvidErrorCode_ClipNotEnabled = 13,
    /** Vector dimension mismatch */
    MemvidErrorCode_VecDimensionMismatch = 14,
    /** Invalid sketch track */
    MemvidErrorCode_InvalidSketchTrack = 15,
    /** Invalid logic mesh */
    MemvidErrorCode_InvalidLogicMesh = 16,
    /** Logic mesh not enabled */
    MemvidErrorCode_LogicMeshNotEnabled = 17,
    /** NER model not available */
    MemvidErrorCode_NerModelNotAvailable = 18,
    /** Invalid tier */
    MemvidErrorCode_InvalidTier = 21,
    /** Ticket sequence error */
    MemvidErrorCode_TicketSequence = 22,
    /** Ticket required */
    MemvidErrorCode_TicketRequired = 23,
    /** Capacity exceeded */
    MemvidErrorCode_CapacityExceeded = 24,
    /** API key required */
    MemvidErrorCode_ApiKeyRequired = 25,
    /** Memory already bound */
    MemvidErrorCode_MemoryAlreadyBound = 26,
    /** Requires sealed memory */
    MemvidErrorCode_RequiresSealed = 31,
    /** Requires open memory */
    MemvidErrorCode_RequiresOpen = 32,
    /** Doctor no operation */
    MemvidErrorCode_DoctorNoOp = 33,
    /** Doctor error */
    MemvidErrorCode_Doctor = 34,
    /** Feature unavailable */
    MemvidErrorCode_FeatureUnavailable = 41,
    /** Invalid cursor */
    MemvidErrorCode_InvalidCursor = 42,
    /** Invalid frame */
    MemvidErrorCode_InvalidFrame = 43,
    /** Frame not found */
    MemvidErrorCode_FrameNotFound = 44,
    /** Frame not found by URI */
    MemvidErrorCode_FrameNotFoundByUri = 45,
    /** Invalid query */
    MemvidErrorCode_InvalidQuery = 46,
    /** Ticket signature invalid */
    MemvidErrorCode_TicketSignatureInvalid = 51,
    /** Model signature invalid */
    MemvidErrorCode_ModelSignatureInvalid = 52,
    /** Model manifest invalid */
    MemvidErrorCode_ModelManifestInvalid = 53,
    /** Model integrity error */
    MemvidErrorCode_ModelIntegrity = 54,
    /** Extraction failed */
    MemvidErrorCode_ExtractionFailed = 61,
    /** Embedding failed */
    MemvidErrorCode_EmbeddingFailed = 62,
    /** Rerank failed */
    MemvidErrorCode_RerankFailed = 63,
    /** Tantivy error */
    MemvidErrorCode_Tantivy = 64,
    /** Table extraction error */
    MemvidErrorCode_TableExtraction = 65,
    /** Schema validation error */
    MemvidErrorCode_SchemaValidation = 66,
    /** WAL corruption */
    MemvidErrorCode_WalCorruption = 71,
    /** Manifest WAL corrupted */
    MemvidErrorCode_ManifestWalCorrupted = 72,
    /** Checkpoint failed */
    MemvidErrorCode_CheckpointFailed = 73,
    /** Auxiliary file detected */
    MemvidErrorCode_AuxiliaryFileDetected = 74,
    /** Null pointer passed (FFI-specific) */
    MemvidErrorCode_NullPointer = 100,
    /** Invalid UTF-8 string (FFI-specific) */
    MemvidErrorCode_InvalidUtf8 = 101,
    /** JSON parse error (FFI-specific) */
    MemvidErrorCode_JsonParse = 102,
    /** Invalid handle (FFI-specific) */
    MemvidErrorCode_InvalidHandle = 103,
    /** Unknown error */
    MemvidErrorCode_Unknown = 255,
} MemvidErrorCode;

/**
 * Opaque handle to a Memvid instance.
 *
 * The handle must be freed with memvid_close().
 */
typedef struct MemvidHandle MemvidHandle;

/**
 * Error structure returned via out-parameter.
 *
 * The message field is owned by the FFI layer when non-null.
 * Call memvid_error_free() to release the message memory.
 */
typedef struct MemvidError {
    /** Error code */
    MemvidErrorCode code;
    /** Error message (NULL if code == Ok) */
    char *message;
} MemvidError;

/**
 * Memory statistics.
 *
 * All fields are value types that can be safely copied.
 */
typedef struct MemvidStats {
    /** Total number of frames */
    uint64_t frame_count;
    /** Number of active (non-deleted) frames */
    uint64_t active_frame_count;
    /** Total file size in bytes */
    uint64_t size_bytes;
    /** Total payload bytes (uncompressed) */
    uint64_t payload_bytes;
    /** Logical bytes (after compression) */
    uint64_t logical_bytes;
    /** Capacity limit in bytes */
    uint64_t capacity_bytes;
    /** Whether lexical search index exists */
    uint8_t has_lex_index;
    /** Whether vector search index exists */
    uint8_t has_vec_index;
    /** Whether CLIP index exists */
    uint8_t has_clip_index;
    /** Whether time index exists */
    uint8_t has_time_index;
    /** Padding for alignment */
    uint8_t _padding[4];
    /** WAL size in bytes */
    uint64_t wal_bytes;
    /** Lexical index size in bytes */
    uint64_t lex_index_bytes;
    /** Vector index size in bytes */
    uint64_t vec_index_bytes;
    /** Time index size in bytes */
    uint64_t time_index_bytes;
    /** Number of vectors stored */
    uint64_t vector_count;
    /** Number of CLIP images */
    uint64_t clip_image_count;
    /** Compression ratio (percentage, 0-100+) */
    double compression_ratio_percent;
    /** Storage savings percentage */
    double savings_percent;
    /** Storage utilization percentage */
    double storage_utilisation_percent;
    /** Remaining capacity in bytes */
    uint64_t remaining_capacity_bytes;
} MemvidStats;

/* ============================================================================
 * Version and Feature Functions
 * ============================================================================ */

/**
 * Get the library version string.
 *
 * @return Static string containing the version (e.g., "0.1.0").
 *         Do not free this string.
 */
const char *memvid_version(void);

/**
 * Get feature flags bitmask.
 *
 * @return Bitmask indicating which features are compiled in:
 *         - Bit 0 (0x01): lex - Lexical search
 *         - Bit 1 (0x02): vec - Vector search
 *         - Bit 2 (0x04): clip - CLIP embeddings
 */
uint32_t memvid_features(void);

/* ============================================================================
 * Lifecycle Functions
 * ============================================================================ */

/**
 * Create a new Memvid memory at the specified path.
 *
 * @param path   Filesystem path for the memory (UTF-8 encoded, null-terminated)
 * @param error  Out-parameter for error information (may be NULL)
 *
 * @return Handle on success, NULL on failure.
 *         Caller owns the returned handle. Must call memvid_close() to free.
 */
MemvidHandle *memvid_create(const char *path, MemvidError *error);

/**
 * Open an existing Memvid memory.
 *
 * @param path   Filesystem path to existing memory (UTF-8 encoded, null-terminated)
 * @param error  Out-parameter for error information (may be NULL)
 *
 * @return Handle on success, NULL on failure.
 *         Caller owns the returned handle. Must call memvid_close() to free.
 */
MemvidHandle *memvid_open(const char *path, MemvidError *error);

/**
 * Close and free a Memvid handle.
 *
 * After this call, the handle is invalid and must not be used.
 *
 * @param handle  Handle to close (safe to pass NULL)
 */
void memvid_close(MemvidHandle *handle);

/* ============================================================================
 * Mutation Functions
 * ============================================================================ */

/**
 * Add content to the memory.
 *
 * @param handle  Valid Memvid handle
 * @param data    Pointer to content bytes
 * @param len     Length of content in bytes
 * @param error   Out-parameter for error information (may be NULL)
 *
 * @return Frame ID on success, 0 on failure (check error->code).
 */
uint64_t memvid_put_bytes(MemvidHandle *handle,
                          const uint8_t *data,
                          size_t len,
                          MemvidError *error);

/**
 * Add content with options (JSON configuration).
 *
 * @param handle       Valid Memvid handle
 * @param data         Pointer to content bytes
 * @param len          Length of content in bytes
 * @param options_json JSON string with PutOptions (NULL for defaults)
 * @param error        Out-parameter for error information (may be NULL)
 *
 * @return Frame ID on success, 0 on failure.
 *
 * Options JSON Schema:
 * {
 *   "uri": "string",
 *   "title": "string",
 *   "timestamp": 1234567890,
 *   "track": "string",
 *   "kind": "string",
 *   "tags": {"key": "value"},
 *   "labels": ["label1", "label2"],
 *   "search_text": "override text",
 *   "auto_tag": true,
 *   "extract_dates": true,
 *   "extract_triplets": true,
 *   "no_raw": false,
 *   "dedup": false
 * }
 */
uint64_t memvid_put_bytes_with_options(MemvidHandle *handle,
                                       const uint8_t *data,
                                       size_t len,
                                       const char *options_json,
                                       MemvidError *error);

/**
 * Commit pending changes to disk.
 *
 * @param handle  Valid Memvid handle
 * @param error   Out-parameter for error information (may be NULL)
 *
 * @return 1 on success, 0 on failure.
 */
int memvid_commit(MemvidHandle *handle, MemvidError *error);

/* ============================================================================
 * Search Functions
 * ============================================================================ */

/**
 * Search the memory.
 *
 * @param handle        Valid Memvid handle
 * @param request_json  JSON string with search parameters
 * @param error         Out-parameter for error information (may be NULL)
 *
 * @return JSON string with search results on success, NULL on failure.
 *         Caller must free with memvid_string_free().
 *
 * Request JSON Schema:
 * {
 *   "query": "search text",
 *   "top_k": 10,
 *   "offset": 0,
 *   "track": "optional-track",
 *   "mode": "lex|vec|hybrid"
 * }
 *
 * Response JSON Schema:
 * {
 *   "hits": [
 *     {
 *       "frame_id": 1,
 *       "score": 0.95,
 *       "snippet": "matched text...",
 *       "uri": "optional-uri",
 *       "title": "optional-title"
 *     }
 *   ],
 *   "total": 100
 * }
 */
char *memvid_search(MemvidHandle *handle,
                    const char *request_json,
                    MemvidError *error);

/**
 * Free a string returned by memvid functions.
 *
 * @param str  String to free (safe to pass NULL)
 */
void memvid_string_free(char *str);

/* ============================================================================
 * State Query Functions
 * ============================================================================ */

/**
 * Get memory statistics.
 *
 * @param handle  Valid Memvid handle
 * @param stats   Out-parameter for statistics (must not be NULL)
 * @param error   Out-parameter for error information (may be NULL)
 *
 * @return 1 on success, 0 on failure.
 */
int memvid_stats(MemvidHandle *handle, MemvidStats *stats, MemvidError *error);

/**
 * Get the number of frames in the memory.
 *
 * @param handle  Valid Memvid handle
 * @param error   Out-parameter for error information (may be NULL)
 *
 * @return Frame count on success, 0 on error
 *         (check error->code to distinguish from an empty memory).
 */
uint64_t memvid_frame_count(MemvidHandle *handle, MemvidError *error);

/* ============================================================================
 * Frame Retrieval Functions
 * ============================================================================ */

/**
 * Get frame metadata by ID.
 *
 * @param handle    Valid Memvid handle
 * @param frame_id  Frame identifier (0-indexed)
 * @param error     Out-parameter for error information (may be NULL)
 *
 * @return JSON string with frame metadata on success, NULL on failure.
 *         Caller must free with memvid_string_free().
 */
char *memvid_frame_by_id(MemvidHandle *handle, uint64_t frame_id, MemvidError *error);

/**
 * Get frame metadata by URI.
 *
 * @param handle  Valid Memvid handle
 * @param uri     Frame URI (null-terminated UTF-8 string)
 * @param error   Out-parameter for error information (may be NULL)
 *
 * @return JSON string with frame metadata on success, NULL on failure.
 *         Caller must free with memvid_string_free().
 */
char *memvid_frame_by_uri(MemvidHandle *handle, const char *uri, MemvidError *error);

/**
 * Get frame text content by ID.
 *
 * @param handle    Valid Memvid handle
 * @param frame_id  Frame identifier (0-indexed)
 * @param error     Out-parameter for error information (may be NULL)
 *
 * @return Frame text content on success, NULL on failure.
 *         Caller must free with memvid_string_free().
 */
char *memvid_frame_content(MemvidHandle *handle, uint64_t frame_id, MemvidError *error);

/**
 * Soft-delete a frame.
 *
 * @param handle    Valid Memvid handle
 * @param frame_id  Frame identifier (0-indexed)
 * @param error     Out-parameter for error information (may be NULL)
 *
 * @return WAL sequence number on success, 0 on failure.
 */
uint64_t memvid_delete_frame(MemvidHandle *handle, uint64_t frame_id, MemvidError *error);

/* ============================================================================
 * Timeline Functions
 * ============================================================================ */

/**
 * Query the timeline (chronological frame list).
 *
 * @param handle      Valid Memvid handle
 * @param query_json  JSON string with query parameters (NULL for defaults)
 * @param error       Out-parameter for error information (may be NULL)
 *
 * @return JSON string with timeline entries on success, NULL on failure.
 *         Caller must free with memvid_string_free().
 *
 * Query JSON Schema:
 * {
 *   "limit": 100,
 *   "since": 1234567890,
 *   "until": 1234567899,
 *   "reverse": false
 * }
 */
char *memvid_timeline(MemvidHandle *handle, const char *query_json, MemvidError *error);

/* ============================================================================
 * Verification Functions
 * ============================================================================ */

/**
 * Verify file integrity.
 *
 * @param path   Path to the .mv2 file (null-terminated UTF-8 string)
 * @param deep   Perform deep verification (non-zero for true)
 * @param error  Out-parameter for error information (may be NULL)
 *
 * @return JSON string with verification report on success, NULL on failure.
 *         Caller must free with memvid_string_free().
 */
char *memvid_verify(const char *path, int deep, MemvidError *error);

/* ============================================================================
 * RAG/Ask Functions
 * ============================================================================ */

/**
 * Ask a question using RAG (Retrieval-Augmented Generation).
 *
 * Performs context retrieval based on the question. When context_only is true
 * (the default), returns retrieved context without synthesis. Answer synthesis
 * requires an external LLM.
 *
 * @param handle        Valid Memvid handle
 * @param request_json  JSON string with ask parameters
 * @param error         Out-parameter for error information (may be NULL)
 *
 * @return JSON string with ask response on success, NULL on failure.
 *         Caller must free with memvid_string_free().
 *
 * Request JSON Schema:
 * {
 *   "question": "What is the capital of France?",
 *   "top_k": 10,
 *   "snippet_chars": 200,
 *   "uri": null,
 *   "scope": null,
 *   "context_only": true,
 *   "mode": "hybrid"
 * }
 *
 * Mode values: "lex", "sem", "hybrid" (default: "hybrid")
 *
 * Response JSON Schema:
 * {
 *   "question": "...",
 *   "mode": "hybrid",
 *   "retriever": "lex",
 *   "context_only": true,
 *   "retrieval": { "query": "...", "hits": [...], ... },
 *   "answer": null,
 *   "citations": [...],
 *   "context_fragments": [...],
 *   "stats": { "retrieval_ms": 5, "synthesis_ms": 0, "latency_ms": 5 }
 * }
 */
char *memvid_ask(MemvidHandle *handle, const char *request_json, MemvidError *error);

/* ============================================================================
 * Memory Management Functions
 * ============================================================================ */

/**
 * Free an error's message field.
 *
 * After this call, error->message is set to NULL.
 *
 * @param error  Error to free (safe to pass NULL, or if message is NULL)
 */
void memvid_error_free(MemvidError *error);

#ifdef __cplusplus
}
#endif

#endif /* MEMVID_FFI_H */
