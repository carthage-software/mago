//! Capability logic for the [`Server`](crate::Server).
//!
//! Each submodule adds one `get_*` method returning a domain value; byte
//! offsets and plain structs, never a wire-protocol type. The protocol layer
//! translates these to LSP responses.

pub mod code_action;
pub mod code_lens;
pub mod completion;
pub mod definition;
pub mod document_link;
pub mod folding_range;
pub mod formatting;
pub mod hover;
pub mod inlay_hint;
pub mod references;
pub mod selection_range;
pub mod semantic_tokens;
pub mod signature_help;
pub mod workspace_symbol;
