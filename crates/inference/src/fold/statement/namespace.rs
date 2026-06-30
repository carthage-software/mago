use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::statement::Namespace;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_namespace(
        &mut self,
        span: Span,
        namespace: &'source Namespace<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;
        let previous = self.namespace;
        self.namespace = match namespace.name {
            Some(name) => name.value,
            None => b"",
        };

        let body = self.infer_statement(namespace.statement)?;
        self.namespace = previous;

        let meta = Flow { reachable, exit: body.meta.exit };
        let statement = self.arena.alloc(body);
        let name = match namespace.name {
            Some(name) => {
                let name = name.copy_into(self.arena);
                Some(&*self.arena.alloc(name))
            }
            None => None,
        };

        let namespace = Namespace { span: namespace.span, name, statement };

        Ok(Statement { meta, span, kind: StatementKind::Namespace(self.arena.alloc(namespace)) })
    }
}
