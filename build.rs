use std::fs;
use std::io;
use std::path::Path;

use mago_prelude::Prelude;

const PRELUDE_FRAMES: usize = 4;

pub fn main() -> io::Result<()> {
    println!("cargo:rustc-env=TARGET={}", std::env::var("TARGET").unwrap());
    println!("cargo:rerun-if-changed=crates/prelude/assets");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo:rustc-check-cfg=cfg(mago_instrumented)");

    let rustflags = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    if rustflags.split('\x1f').any(|flag| {
        let flag = flag.strip_prefix("-C").or_else(|| flag.strip_prefix("--codegen=")).unwrap_or(flag);
        let name = flag.split('=').next().unwrap_or_default();

        matches!(name, "profile-generate" | "instrument-coverage")
    }) {
        println!("cargo:rustc-cfg=mago_instrumented");
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let out_dir = Path::new(&out_dir);

    let prelude = std::thread::Builder::new()
        .stack_size(36 * 1024 * 1024)
        .name("prelude_builder".into())
        .spawn(Prelude::build)?
        .join()
        .expect("Failed to join prelude thread");

    let encoded = prelude.encode().expect("Failed to encode the prelude");
    let chunks: Vec<&[u8]> = encoded.chunks(encoded.len().div_ceil(PRELUDE_FRAMES)).collect();
    let compressed: Vec<io::Result<Vec<u8>>> = std::thread::scope(|scope| {
        let handles: Vec<_> = chunks.iter().map(|chunk| scope.spawn(|| zstd::bulk::compress(chunk, 19))).collect();
        handles.into_iter().map(|handle| handle.join().expect("Failed to join compression thread")).collect()
    });

    let mut stream = Vec::new();
    let mut frames = Vec::with_capacity(chunks.len());
    for (chunk, frame) in chunks.iter().zip(compressed) {
        let frame = frame?;
        frames.push(format!("({}, {})", frame.len(), chunk.len()));
        stream.extend(frame);
    }
    fs::write(out_dir.join("prelude.zst"), stream)?;
    fs::write(
        out_dir.join("prelude_frames.rs"),
        format!("const FRAMES: [(usize, usize); {}] = [{}];\n", frames.len(), frames.join(", ")),
    )?;

    let plain =
        bincode::serde::encode_to_vec(&prelude, bincode::config::standard()).expect("Failed to encode the prelude");
    fs::write(out_dir.join("prelude.bin"), plain)?;

    Ok(())
}
