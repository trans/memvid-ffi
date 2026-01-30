# memvid-ffi

C FFI bindings for [memvid-core](https://github.com/memvid/memvid), enabling language bindings for the memvid single-file AI memory format.

## Status

**Core API complete. Development paused.**

This library covers memvid's core local-only API. Development has been paused due to memvid's trajectory toward a SaaS model with API key requirements for advanced features.

### What's Implemented

| Category | Functions |
|----------|-----------|
| Lifecycle | `memvid_create`, `memvid_open`, `memvid_close` |
| Mutations | `memvid_put_bytes`, `memvid_put_bytes_with_options`, `memvid_commit`, `memvid_delete_frame` |
| Search | `memvid_search` |
| Frames | `memvid_frame_by_id`, `memvid_frame_by_uri`, `memvid_frame_content` |
| State | `memvid_stats`, `memvid_frame_count` |
| Timeline | `memvid_timeline` |
| RAG | `memvid_ask` |
| Maintenance | `memvid_verify`, `memvid_doctor`, `memvid_doctor_plan`, `memvid_doctor_apply` |
| Utilities | `memvid_version`, `memvid_features`, `memvid_string_free`, `memvid_error_free` |

**22 FFI functions, 26 tests**

### Not Implemented

- Memory Cards / enrichment (misaligned with external systems)
- Sessions / replay (CLI-only feature)
- Models management (manual download, not SDK)
- CLIP image embeddings
- Vector search mode

## Building

```bash
cargo build --release
```

The library outputs `libmemvid.so` (Linux), `libmemvid.dylib` (macOS), or `memvid.dll` (Windows).

Header file is generated at `include/memvid.h`.

## Usage

See the [Crystal bindings](https://github.com/trans/memvid.cr) for a complete example of using this FFI layer.

## License

MIT
