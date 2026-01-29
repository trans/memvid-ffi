//! RAG/Ask query functions.

use crate::error::MemvidError;
use crate::handle::MemvidHandle;
use crate::util::{cstr_to_string, set_error_null, set_ok, string_to_cstr};
use memvid_core::types::{AskContextFragment, AskContextFragmentKind};
use serde::{Deserialize, Serialize};
use std::os::raw::c_char;

/// Ask mode for JSON serialization.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum AskModeJson {
    Lex,
    Sem,
    #[default]
    Hybrid,
}

impl From<&AskModeJson> for memvid_core::AskMode {
    fn from(m: &AskModeJson) -> Self {
        match m {
            AskModeJson::Lex => Self::Lex,
            AskModeJson::Sem => Self::Sem,
            AskModeJson::Hybrid => Self::Hybrid,
        }
    }
}

impl From<&memvid_core::AskMode> for AskModeJson {
    fn from(m: &memvid_core::AskMode) -> Self {
        match m {
            memvid_core::AskMode::Lex => Self::Lex,
            memvid_core::AskMode::Sem => Self::Sem,
            memvid_core::AskMode::Hybrid => Self::Hybrid,
        }
    }
}

/// Ask retriever for JSON serialization.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum AskRetrieverJson {
    Lex,
    Semantic,
    Hybrid,
    LexFallback,
    TimelineFallback,
}

impl From<&memvid_core::AskRetriever> for AskRetrieverJson {
    fn from(r: &memvid_core::AskRetriever) -> Self {
        match r {
            memvid_core::AskRetriever::Lex => Self::Lex,
            memvid_core::AskRetriever::Semantic => Self::Semantic,
            memvid_core::AskRetriever::Hybrid => Self::Hybrid,
            memvid_core::AskRetriever::LexFallback => Self::LexFallback,
            memvid_core::AskRetriever::TimelineFallback => Self::TimelineFallback,
        }
    }
}

/// Ask request from JSON.
#[derive(Debug, Default, Deserialize)]
struct AskRequestJson {
    question: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
    #[serde(default = "default_snippet_chars")]
    snippet_chars: usize,
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    start: Option<i64>,
    #[serde(default)]
    end: Option<i64>,
    #[serde(default = "default_context_only")]
    context_only: bool,
    #[serde(default)]
    mode: AskModeJson,
    #[serde(default)]
    as_of_frame: Option<u64>,
    #[serde(default)]
    as_of_ts: Option<i64>,
}

fn default_top_k() -> usize {
    10
}

fn default_snippet_chars() -> usize {
    200
}

fn default_context_only() -> bool {
    // Default to context_only since we don't have an LLM for synthesis
    true
}

impl AskRequestJson {
    fn into_request(self) -> memvid_core::AskRequest {
        memvid_core::AskRequest {
            question: self.question,
            top_k: self.top_k,
            snippet_chars: self.snippet_chars,
            uri: self.uri,
            scope: self.scope,
            cursor: self.cursor,
            start: self.start,
            end: self.end,
            context_only: self.context_only,
            mode: (&self.mode).into(),
            as_of_frame: self.as_of_frame,
            as_of_ts: self.as_of_ts,
            adaptive: None,
        }
    }
}

/// Ask stats for JSON serialization.
#[derive(Debug, Serialize)]
struct AskStatsJson {
    retrieval_ms: u128,
    synthesis_ms: u128,
    latency_ms: u128,
}

impl From<&memvid_core::AskStats> for AskStatsJson {
    fn from(s: &memvid_core::AskStats) -> Self {
        Self {
            retrieval_ms: s.retrieval_ms,
            synthesis_ms: s.synthesis_ms,
            latency_ms: s.latency_ms,
        }
    }
}

/// Ask citation for JSON serialization.
#[derive(Debug, Serialize)]
struct AskCitationJson {
    index: usize,
    frame_id: u64,
    uri: String,
    chunk_range: Option<(usize, usize)>,
    score: Option<f32>,
}

impl From<&memvid_core::AskCitation> for AskCitationJson {
    fn from(c: &memvid_core::AskCitation) -> Self {
        Self {
            index: c.index,
            frame_id: c.frame_id,
            uri: c.uri.clone(),
            chunk_range: c.chunk_range,
            score: c.score,
        }
    }
}

/// Context fragment kind for JSON serialization.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum AskContextFragmentKindJson {
    Full,
    Summary,
}

impl From<&AskContextFragmentKind> for AskContextFragmentKindJson {
    fn from(k: &AskContextFragmentKind) -> Self {
        match k {
            AskContextFragmentKind::Full => Self::Full,
            AskContextFragmentKind::Summary => Self::Summary,
        }
    }
}

/// Context fragment for JSON serialization.
#[derive(Debug, Serialize)]
struct AskContextFragmentJson {
    rank: usize,
    frame_id: u64,
    uri: String,
    title: Option<String>,
    score: Option<f32>,
    matches: usize,
    range: Option<(usize, usize)>,
    chunk_range: Option<(usize, usize)>,
    text: String,
    kind: Option<AskContextFragmentKindJson>,
}

impl From<&AskContextFragment> for AskContextFragmentJson {
    fn from(f: &AskContextFragment) -> Self {
        Self {
            rank: f.rank,
            frame_id: f.frame_id,
            uri: f.uri.clone(),
            title: f.title.clone(),
            score: f.score,
            matches: f.matches,
            range: f.range,
            chunk_range: f.chunk_range,
            text: f.text.clone(),
            kind: f.kind.as_ref().map(AskContextFragmentKindJson::from),
        }
    }
}

/// Search hit for JSON serialization (nested in response).
#[derive(Debug, Serialize)]
struct SearchHitJson {
    rank: usize,
    frame_id: u64,
    uri: String,
    title: Option<String>,
    range: (usize, usize),
    text: String,
    matches: usize,
    chunk_range: Option<(usize, usize)>,
    chunk_text: Option<String>,
    score: Option<f32>,
}

impl From<&memvid_core::SearchHit> for SearchHitJson {
    fn from(h: &memvid_core::SearchHit) -> Self {
        Self {
            rank: h.rank,
            frame_id: h.frame_id,
            uri: h.uri.clone(),
            title: h.title.clone(),
            range: h.range,
            text: h.text.clone(),
            matches: h.matches,
            chunk_range: h.chunk_range,
            chunk_text: h.chunk_text.clone(),
            score: h.score,
        }
    }
}

/// Search response for JSON serialization (nested in ask response).
#[derive(Debug, Serialize)]
struct SearchResponseJson {
    query: String,
    elapsed_ms: u128,
    total_hits: usize,
    hits: Vec<SearchHitJson>,
    context: String,
    next_cursor: Option<String>,
}

impl From<&memvid_core::SearchResponse> for SearchResponseJson {
    fn from(r: &memvid_core::SearchResponse) -> Self {
        Self {
            query: r.query.clone(),
            elapsed_ms: r.elapsed_ms,
            total_hits: r.total_hits,
            hits: r.hits.iter().map(SearchHitJson::from).collect(),
            context: r.context.clone(),
            next_cursor: r.next_cursor.clone(),
        }
    }
}

/// Ask response for JSON serialization.
#[derive(Debug, Serialize)]
struct AskResponseJson {
    question: String,
    mode: AskModeJson,
    retriever: AskRetrieverJson,
    context_only: bool,
    retrieval: SearchResponseJson,
    answer: Option<String>,
    citations: Vec<AskCitationJson>,
    context_fragments: Vec<AskContextFragmentJson>,
    stats: AskStatsJson,
}

impl From<&memvid_core::AskResponse> for AskResponseJson {
    fn from(r: &memvid_core::AskResponse) -> Self {
        Self {
            question: r.question.clone(),
            mode: AskModeJson::from(&r.mode),
            retriever: AskRetrieverJson::from(&r.retriever),
            context_only: r.context_only,
            retrieval: SearchResponseJson::from(&r.retrieval),
            answer: r.answer.clone(),
            citations: r.citations.iter().map(AskCitationJson::from).collect(),
            context_fragments: r
                .context_fragments
                .iter()
                .map(AskContextFragmentJson::from)
                .collect(),
            stats: AskStatsJson::from(&r.stats),
        }
    }
}

/// Ask a question using RAG (Retrieval-Augmented Generation).
///
/// This performs context retrieval based on the question. When `context_only`
/// is true (the default), it returns retrieved context without synthesis.
/// Answer synthesis requires an external LLM.
///
/// # Parameters
///
/// - `handle`: Valid Memvid handle
/// - `request_json`: JSON string with ask parameters
/// - `error`: Out-parameter for error information
///
/// # Returns
///
/// JSON string with ask response on success, NULL on failure.
/// Caller must free with `memvid_string_free()`.
///
/// # Request JSON Schema
///
/// ```json
/// {
///   "question": "What is the capital of France?",
///   "top_k": 10,
///   "snippet_chars": 200,
///   "uri": null,
///   "scope": null,
///   "cursor": null,
///   "start": null,
///   "end": null,
///   "context_only": true,
///   "mode": "hybrid",
///   "as_of_frame": null,
///   "as_of_ts": null
/// }
/// ```
///
/// Mode values: "lex", "sem", "hybrid" (default: "hybrid")
///
/// # Response JSON Schema
///
/// ```json
/// {
///   "question": "What is the capital of France?",
///   "mode": "hybrid",
///   "retriever": "lex",
///   "context_only": true,
///   "retrieval": {
///     "query": "capital France",
///     "elapsed_ms": 5,
///     "total_hits": 3,
///     "hits": [...],
///     "context": "...",
///     "next_cursor": null
///   },
///   "answer": null,
///   "citations": [...],
///   "context_fragments": [...],
///   "stats": {
///     "retrieval_ms": 5,
///     "synthesis_ms": 0,
///     "latency_ms": 5
///   }
/// }
/// ```
///
/// # Safety
///
/// - `handle` must be a valid handle
/// - `request_json` must be a valid null-terminated UTF-8 string
/// - `error` must be a valid pointer or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memvid_ask(
    handle: *mut MemvidHandle,
    request_json: *const c_char,
    error: *mut MemvidError,
) -> *mut c_char {
    let handle = match unsafe { MemvidHandle::from_ptr_mut(handle) } {
        Some(h) => h,
        None => return unsafe { set_error_null(error, MemvidError::invalid_handle()) },
    };

    let json_str = match unsafe { cstr_to_string(request_json, "request_json") } {
        Ok(s) => s,
        Err(e) => return unsafe { set_error_null(error, e) },
    };

    let request_json: AskRequestJson = match serde_json::from_str(&json_str) {
        Ok(r) => r,
        Err(e) => return unsafe { set_error_null(error, MemvidError::json_parse(e)) },
    };

    let request = request_json.into_request();

    // Call ask without an embedder (context_only mode or lex-only)
    match handle.as_mut().ask(request, None::<&dyn memvid_core::VecEmbedder>) {
        Ok(response) => {
            let json_response = AskResponseJson::from(&response);
            match serde_json::to_string(&json_response) {
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
