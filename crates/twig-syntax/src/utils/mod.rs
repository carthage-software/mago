//! Utility helpers for inspecting Twig AST nodes.

use crate::ast::Statement;

/// Returns `true` if the statement is a `{% block %}` declaration.
#[inline]
#[must_use]
pub const fn is_block(statement: &Statement<'_>) -> bool {
    matches!(statement, Statement::Block(_))
}

/// Returns `true` if the statement participates in template inheritance
/// (`extends`, `use`, `block`, `embed`, `import`, `from`, `include`,
/// `macro`).
#[inline]
#[must_use]
pub const fn is_inheritance(statement: &Statement<'_>) -> bool {
    matches!(
        statement,
        Statement::Extends(_)
            | Statement::Use(_)
            | Statement::Block(_)
            | Statement::Embed(_)
            | Statement::Import(_)
            | Statement::From(_)
            | Statement::Include(_)
            | Statement::Macro(_),
    )
}

/// Returns `true` if the statement is a control-flow construct (`if`,
/// `for`, `guard`).
#[inline]
#[must_use]
pub const fn is_control_flow(statement: &Statement<'_>) -> bool {
    matches!(statement, Statement::If(_) | Statement::For(_) | Statement::Guard(_))
}
