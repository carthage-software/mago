mod benchmarks;
mod config;
mod content;
mod i18n;
mod nav;
mod render;
mod rules;
mod versions;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

use crate::render::build_site;

#[derive(Debug, Parser)]
#[command(name = "mago-docs", about = "Build the Mago documentation site")]
struct Cli {}

fn main() -> Result<()> {
    init_tracing();
    let _ = Cli::parse();
    // Anchor every relative path to the crate's source tree so the binary
    // works whether you invoke it from the repo root, the docs/ directory,
    // or out of `target/release` in CI.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    build_site(&root)
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("MAGO_DOCS_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).without_time().init();
}
