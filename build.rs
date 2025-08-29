use std::fs;
use std::io;
use std::path::Path;

use mago_prelude::Prelude;

pub fn main() -> io::Result<()> {
    println!("cargo:rustc-env=TARGET={}", std::env::var("TARGET").unwrap());
    println!("cargo:rerun-if-changed=crates/prelude/assets");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let prelude_file = Path::new(&out_dir).join("prelude.bin");

    let prelude_bin = std::thread::Builder::new()
        .stack_size(36 * 1024 * 1024)
        .name("prelude_builder".into())
        .spawn(|| {
            let prelude = Prelude::build();

            prelude.encode().expect("Failed to encode the prelude")
        })?
        .join()
        .expect("Failed to join prelude thread");

    fs::write(prelude_file, prelude_bin)?;

    Ok(())
}
