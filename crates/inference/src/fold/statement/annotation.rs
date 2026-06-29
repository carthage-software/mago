use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::annotation::VariableBindingAnnotation;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::lower_type_annotation;
use mago_oracle::ty::Type;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// `/** @var T $x */` as a statement: binds `$x` to the annotated type in the
    /// environment.
    pub(crate) fn infer_variable_binding_annotation(
        &mut self,
        span: Span,
        annotation: &'source VariableBindingAnnotation<'source>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let type_annotation = annotation.type_annotation.copy_into(self.arena);
        if let Some(ty) = lower_type_annotation(&mut self.ty, &type_annotation) {
            self.environment.set(Var::new(self.arena.alloc_slice_copy(annotation.variable.name)), ty);
        }

        let node = annotation.copy_into(self.arena);

        Ok(Statement {
            meta: Flow { reachable: true, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::VariableBindingAnnotation(self.arena.alloc(node)),
        })
    }
}
