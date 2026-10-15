//! Shared codebase queries and edits for Mago frontends.
//!
//! `mago-server` provides completion, hover, diagnostics, navigation, rename,
//! code actions, and formatting without a wire protocol. Callers pass a
//! [`FileId`] and byte offsets and receive plain domain values. Each frontend
//! maps those inputs and results to its own protocol types.
//!
//! # Loaded files and native workspaces
//!
//! [`Server`] never touches the filesystem. It does not discover or load
//! configuration, it does not walk directories, and it does not read files
//! from disk. The caller builds a [`Database`](mago_database::Database) of
//! files, decodes the prelude metadata and symbol references, assembles
//! [`Settings`], and passes them to [`Server::new`]. File edits arrive as
//! in-memory byte buffers. This loaded-file layer also supports WebAssembly.
//!
//! Native callers can use `workspace::Workspace` and
//! `workspace::registry::Workspaces` to route paths, track accepted buffers,
//! and apply disk changes. That layer may read the filesystem. Buffer keys
//! belong to the caller; the registry never parses them.
//!
//! # Offsets
//!
//! Every offset and span the server emits or accepts is a zero-based **byte**
//! offset (`u32`) into a file's raw contents; spans are half-open `[start,
//! end)`. The server never speaks lines, columns, or UTF-16; converting to an
//! editor's position model is the protocol layer's job.
//!
//! # Multiple workspaces
//!
//! Each [`Server`] owns one loaded workspace. The native registry routes
//! files to the most specific project root or a configured external source.
//! It shares accepted buffers across all workspaces that load the same file.

#![allow(clippy::wildcard_imports)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::pub_use)]
#![allow(clippy::similar_names)]
#![allow(clippy::single_char_lifetime_names)]

mod call_analysis;
pub mod capabilities;
pub mod diagnostics;
mod documentation;
pub mod domain;
pub mod error;
pub mod file_analysis;
pub mod linter;
pub mod lookup;
mod matcher;
mod method_presentation;
pub mod resolved_names;
pub mod semantic_index;
pub mod server;
pub mod settings;
mod symbol_resolution;
mod variables;
#[cfg(not(target_arch = "wasm32"))]
pub mod workspace;

pub use domain::CodeActionItem;
pub use domain::CodeActionKind;
pub use domain::CodeLensItem;
pub use domain::CompletionAlternative;
pub use domain::CompletionEntry;
pub use domain::CompletionInsert;
pub use domain::CompletionKind;
pub use domain::CompletionList;
pub use domain::DiagnosticData;
pub use domain::DocProblem;
pub use domain::DocumentLinkItem;
pub use domain::DocumentSymbolItem;
pub use domain::DocumentationTag;
pub use domain::DocumentationTagKind;
pub use domain::FoldKind;
pub use domain::FoldRange;
pub use domain::FormattedDocument;
pub use domain::HintKind;
pub use domain::HoverEntry;
pub use domain::HoverInfo;
pub use domain::InlayHintItem;
pub use domain::NameProblem;
pub use domain::Range;
pub use domain::RefusalReason;
pub use domain::RenameRefusal;
pub use domain::SelectionRangeItem;
pub use domain::Severity;
pub use domain::SignatureInfo;
pub use domain::SourceDocumentation;
pub use domain::SymbolKind;
pub use domain::SymbolLocation;
pub use domain::SymbolRank;
pub use domain::TextReplacement;
pub use domain::WorkspaceSymbolItem;
pub use error::ServerError;
pub use file_analysis::FileAnalysis;
pub use linter::LinterContext;
pub use server::Server;
pub use settings::Features;
pub use settings::Settings;
pub use symbol_resolution::resolved_symbol_name;

pub use mago_database::file::FileId;
