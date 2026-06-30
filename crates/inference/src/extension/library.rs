use mago_allocator::Arena;
use mago_hir::ir::expression::CalleeKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::NON_NEGATIVE_INT;

use crate::extension::ExtensionContext;
use crate::extension::ExtensionInference;
use crate::flow::Flow;

#[derive(Debug, Default, Clone, Copy)]
pub struct StdlibInference;

impl<A: Arena> ExtensionInference<A> for StdlibInference {
    fn infer<'arena>(
        &self,
        context: &mut ExtensionContext<'_, '_, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<Type<'arena>> {
        let ExpressionKind::Call(call) = &expression.kind else {
            return None;
        };
        let CalleeKind::Function(callee) = &call.callee.kind else {
            return None;
        };
        let ExpressionKind::Identifier(identifier) = &callee.kind else {
            return None;
        };

        match identifier.last_segment() {
            b"strlen" | b"mb_strlen" | b"grapheme_strlen" | b"iconv_strlen" | b"count" | b"sizeof" => {
                Some(context.union(&[NON_NEGATIVE_INT]))
            }
            _ => None,
        }
    }
}
