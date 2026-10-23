//! Capability logic for the [`Server`](crate::Server).
//!
//! Each submodule adds one `get_*` method returning a domain value; byte
//! offsets and plain structs, never a wire-protocol type. The protocol layer
//! translates these to LSP responses.

pub mod call_hierarchy;
mod class_rename;
pub mod code_action;
pub mod code_lens;
pub mod completion;
pub mod definition;
pub mod document_link;
pub mod document_symbol;
pub mod folding_range;
pub mod formatting;
mod function_rename;
pub mod highlights;
pub mod hover;
pub mod inlay_hint;
pub mod member_rename;
mod refactor_constant;
mod refactor_edits;
mod refactor_extract;
mod refactor_extract_method;
mod refactor_generate;
mod refactor_implement;
mod refactor_import;
mod refactor_literal;
pub mod refactor_methods;
mod refactor_stub;
pub mod references;
pub mod rename;
pub mod selection_range;
mod signature;
pub mod signature_help;
pub mod type_definition;
pub mod type_hierarchy;
pub mod workspace_symbol;
