// Build script for memvid-ffi
//
// Note: cbindgen header generation is disabled because cbindgen 0.27.0
// does not recognize Rust 2024's #[unsafe(no_mangle)] attribute syntax.
// The header file (include/memvid.h) is maintained manually.
//
// To regenerate types only, run:
//   cbindgen --config cbindgen.toml --crate memvid-ffi --output include/memvid_types.h

fn main() {
    println!("cargo:rerun-if-changed=src/");
}
