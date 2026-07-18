use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::identifier::Identifier;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_constant(
        &self,
        span: Span,
        identifier: &Identifier<'source>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let meta = self.resolve_constant_type(identifier).unwrap_or(TYPE_MIXED);

        Ok(Expression { meta, span, kind: ExpressionKind::Constant(identifier.copy_into(self.arena)) })
    }

    /// The constant name arrives already qualified to the current namespace. An
    /// unqualified, non-imported reference additionally falls back to the global
    /// namespace, mirroring PHP's runtime constant lookup.
    fn resolve_constant_type(&self, identifier: &Identifier<'source>) -> Option<Type<'arena>> {
        if let Some(ty) = self.symbols.global_constant_type(SymbolId::constant(identifier.value)) {
            return Some(ty);
        }

        if identifier.is_local() && !identifier.imported {
            let short_name = identifier.last_segment();
            if short_name != identifier.value {
                return self.symbols.global_constant_type(SymbolId::constant(short_name));
            }
        }

        None
    }
}
