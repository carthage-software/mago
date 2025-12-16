//! A crate for building and loading Mago's "prelude".
//!
//! The prelude contains the pre-compiled `Database` and `CodebaseMetadata` for all
//! of PHP's built-in symbols (functions, classes, constants, etc.). This is
//! essential for providing accurate analysis, especially in environments like WASM
//! where stubs cannot be loaded from the filesystem.
//!
//! ## Workflow
//!
//! This crate is split into two parts using a feature flag:
//!
//! 1.  **At Compile Time (with the `build` feature):**
//!     - A `build.rs` script enables the `build` feature for this crate.
//!     - It calls `Prelude::build()` to perform the expensive, one-time analysis of all stub files.
//!     - It then calls `prelude.encode()` to serialize the result into a byte slice.
//!     - The bytes are written to a file (e.g., `prelude.bin`) in the `OUT_DIR`.
//!
//! 2.  **At Runtime (without the `build` feature):**
//!     - The main application uses `include_bytes!` to embed the `prelude.bin` file.
//!     - It calls `Prelude::decode()` on the bytes to instantly reconstruct the prelude in memory.

use bincode::config::standard;
use serde::Deserialize;
use serde::Serialize;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::Database;

use crate::error::PreludeError;

pub mod error;

#[cfg(feature = "build")]
pub mod build;

/// A container for the pre-compiled database and metadata of PHP's built-in symbols.
///
/// This struct holds all the necessary, fully-analyzed data for PHP's core library,
/// allowing for instant startup without needing to parse and analyze stubs at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prelude {
    /// The database of all built-in PHP files.
    pub database: Database<'static>,
    /// The fully populated and analyzed metadata for all symbols in the database.
    pub metadata: CodebaseMetadata,
    /// The collected symbol references from the analysis.
    pub symbol_references: SymbolReferences,
}

impl Prelude {
    /// Decodes a prelude from a byte slice.
    ///
    /// This is the primary runtime function, used to load a pre-compiled prelude
    /// that was embedded in the binary. It is a very fast operation.
    ///
    /// # Errors
    ///
    /// Returns a [`PreludeError`] if deserialization fails due to corrupted or
    /// incompatible binary data.
    pub fn decode(bytes: &[u8]) -> Result<Self, PreludeError> {
        let (prelude, _) = bincode::serde::decode_from_slice(bytes, standard())?;

        Ok(prelude)
    }

    /// (Builder-only) Builds the prelude by parsing and analyzing all embedded PHP stub files.
    ///
    /// This is an expensive, one-time operation that should only be run at compile
    /// time within a `build.rs` script. It is only available when the `build`
    /// feature is enabled.
    #[cfg(feature = "build")]
    #[must_use]
    pub fn build() -> Self {
        build::build_prelude_internal()
    }

    /// (Builder-only) Encodes the prelude into a compact byte slice.
    ///
    /// The resulting `Vec<u8>` can be saved to a file for later loading.
    /// This is only available when the `build` feature is enabled.
    ///
    /// # Errors
    ///
    /// Returns a [`PreludeError`] if serialization fails.
    #[cfg(feature = "build")]
    pub fn encode(&self) -> Result<Vec<u8>, PreludeError> {
        Ok(bincode::serde::encode_to_vec(self, standard())?)
    }
}

impl Default for Prelude {
    fn default() -> Self {
        let configuration =
            mago_database::DatabaseConfiguration::new(std::path::Path::new("/"), vec![], vec![], vec![], vec![])
                .into_static();

        Self {
            database: mago_database::Database::new(configuration),
            metadata: CodebaseMetadata::default(),
            symbol_references: SymbolReferences::default(),
        }
    }
}
