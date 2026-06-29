use mago_allocator::Arena;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::MagicConstant;
use mago_hir::ir::expression::MagicConstantKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::EMPTY_STRING;
use mago_oracle::ty::well_known::NON_EMPTY_LITERAL_STRING;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'arena, A, S, E> InferenceFolder<'_, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_magic_constant(
        &mut self,
        span: Span,
        magic_constant: &MagicConstant,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let meta = match magic_constant.kind {
            MagicConstantKind::Line => {
                let line = line_number(self.line_starts, span.start.offset) + 1;

                self.int_literal(i64::from(line))
            }
            MagicConstantKind::File | MagicConstantKind::Directory => self.ty.union_of(&[NON_EMPTY_LITERAL_STRING]),
            MagicConstantKind::Namespace => {
                let namespace = self.namespace;

                self.literal_string(namespace)
            }
            MagicConstantKind::Trait
            | MagicConstantKind::Method
            | MagicConstantKind::Function
            | MagicConstantKind::Property
            | MagicConstantKind::Class => self.ty.union_of(&[EMPTY_STRING]),
        };

        Ok(Expression { meta, span, kind: ExpressionKind::MagicConstant(*magic_constant) })
    }
}

/// The zero-based line for `offset`, mirroring `File::line_number` over the
/// precomputed line-start table.
fn line_number(line_starts: &[u32], offset: u32) -> u32 {
    line_starts.binary_search(&offset).unwrap_or_else(|next_line| next_line.saturating_sub(1)) as u32
}
