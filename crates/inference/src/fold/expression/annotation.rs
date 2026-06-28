use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::annotation::Annotation;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::lower_type_annotation;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// `/** @var T */ <expr>` in a value position (an assignment's right-hand
    /// side or a returned expression): the annotated type `T` overrides the
    /// expression's own inferred type.
    pub fn infer_annotation(
        &mut self,
        span: Span,
        annotation: &'source Annotation<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let type_annotation = annotation.annotation.type_annotation.copy_into(self.arena);
        let annotated = lower_type_annotation(&mut self.ty, &type_annotation);

        let inner = self.infer_expression(annotation.expression);
        let meta = annotated.unwrap_or(inner.meta);

        let variable_annotation = annotation.annotation.copy_into(self.arena);
        let node =
            Annotation { annotation: self.arena.alloc(variable_annotation), expression: self.arena.alloc(inner) };

        Expression { meta, span, kind: ExpressionKind::Annotation(self.arena.alloc(node)) }
    }
}
