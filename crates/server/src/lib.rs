//! A transport-agnostic backend for Mago's editor features.
//!
//! `mago-server` answers editor questions; completion, hover, diagnostics,
//! go-to-definition, references, rename, lens, formatting; for a **single
//! workspace**. It is deliberately ignorant of any wire protocol: there is no
//! LSP, no JSON-RPC, no stdio, and no `tower-lsp` in sight. Callers speak in
//! plain inputs (a [`FileId`] and a byte offset) and receive plain domain
//! values back. A thin protocol layer (the CLI's language server) is
//! responsible for translating those domain values to and from LSP types.
//!
//! # No I/O
//!
//! The server never touches the filesystem. It does not discover or load
//! configuration, it does not walk directories, and it does not read files
//! from disk. The caller builds a [`Database`](mago_database::Database) of
//! files, decodes the prelude into a
//! [`CodebaseMetadata`](mago_codex::metadata::CodebaseMetadata), assembles a
//! [`Settings`], and hands all three to [`Server::new`]. File edits arrive as
//! in-memory byte buffers, not paths. This keeps the crate pure, easy to test,
//! and compilable to WebAssembly.
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
//! A `Server` owns exactly one workspace. An editor with several projects open
//! is modelled by the protocol layer holding one `Server` per workspace and
//! routing each request to the right one.

#![allow(clippy::wildcard_imports)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::pub_use)]
#![allow(clippy::similar_names)]
#![allow(clippy::single_char_lifetime_names)]

pub mod capabilities;
pub mod domain;
pub mod error;
pub mod file_analysis;
pub mod linter;
pub mod lookup;
pub mod server;
pub mod settings;

pub use domain::CodeActionItem;
pub use domain::CodeLensItem;
pub use domain::CompletionEntry;
pub use domain::CompletionKind;
pub use domain::CompletionList;
pub use domain::DiagnosticData;
pub use domain::DocumentLinkItem;
pub use domain::FoldKind;
pub use domain::FoldRange;
pub use domain::FormattedDocument;
pub use domain::HintKind;
pub use domain::HoverInfo;
pub use domain::InlayHintItem;
pub use domain::Range;
pub use domain::SelectionRangeItem;
pub use domain::SemanticTokenItem;
pub use domain::SemanticTokenKind;
pub use domain::Severity;
pub use domain::SignatureInfo;
pub use domain::SymbolKind;
pub use domain::SymbolLocation;
pub use domain::TextReplacement;
pub use domain::WorkspaceSymbolItem;
pub use error::ServerError;
pub use file_analysis::FileAnalysis;
pub use linter::LinterContext;
pub use server::ExpressionTypeIndex;
pub use server::Server;
pub use settings::Features;
pub use settings::Settings;

pub use mago_database::file::FileId;
