use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::walker::Walker;

use crate::artifacts::AnalysisArtifacts;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TypeCleaningWalker;

impl<'ast, 'arena> Walker<'ast, 'arena, AnalysisArtifacts> for TypeCleaningWalker {
    fn walk_in_expression(&self, expression: &'ast Expression<'arena>, artifacts: &mut AnalysisArtifacts) {
        let expression_span = expression.span();
        let expression_id = (expression_span.start.offset, expression_span.end.offset);

        artifacts.expression_types.remove(&expression_id);
    }
}

pub fn clean_nodes<'ast, 'arena>(stmts: &'ast [Statement<'arena>], artifacts: &mut AnalysisArtifacts) {
    for stmt in stmts {
        TypeCleaningWalker.walk_statement(stmt, artifacts);
    }
}
