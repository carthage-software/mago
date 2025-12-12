//! Hook traits for the analyzer plugin system.
//!
//! Hooks allow plugins to intercept and modify the analysis process at various points:
//!
//! - [`ProgramHook`]: Program-level events (before/after program analysis)
//! - [`StatementHook`]: Statement analysis (before/after each statement)
//! - [`ExpressionHook`]: Expression analysis (before/after each expression)
//! - [`FunctionCallHook`]: Function call analysis
//! - [`MethodCallHook`]: Method call analysis
//! - [`StaticMethodCallHook`]: Static method call analysis
//! - [`NullSafeMethodCallHook`]: Nullsafe method call analysis
//! - [`ClassDeclarationHook`]: Class declaration analysis
//! - [`InterfaceDeclarationHook`]: Interface declaration analysis
//! - [`TraitDeclarationHook`]: Trait declaration analysis
//! - [`EnumDeclarationHook`]: Enum declaration analysis
//! - [`FunctionDeclarationHook`]: Function declaration analysis
//! - [`IssueFilterHook`]: Filter issues at the end of analysis
//!
//! All hooks receive real AST references and a [`HookContext`](crate::plugin::context::HookContext)
//! that provides mutable access to the analysis state.

mod action;
mod call;
mod declaration;
mod expression;
mod filter;
mod program;
mod statement;

pub use action::*;
pub use call::*;
pub use declaration::*;
pub use expression::*;
pub use filter::*;
pub use program::*;
pub use statement::*;
