use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::statement::Block;
use mago_hir::ir::statement::Namespace;
use mago_hir::ir::statement::NamespaceBody;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::Terminator;
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
        terminator: Option<Terminator>,
        namespace: &'source Namespace<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;
        let previous = self.namespace;
        self.namespace = match namespace.name {
            Some(name) => name.value,
            None => b"",
        };

        let (body, exit) = match namespace.body {
            NamespaceBody::BraceDelimited(block) => {
                let (statements, exit) = self.infer_block(block.statements)?;

                (NamespaceBody::BraceDelimited(self.arena.alloc(Block { span: block.span, statements })), exit)
            }
            NamespaceBody::Implicit { terminator: body_terminator, statements } => {
                let (statements, exit) = self.infer_block(statements)?;

                (NamespaceBody::Implicit { terminator: body_terminator, statements }, exit)
            }
        };

        self.namespace = previous;

        let meta = Flow { reachable, exit };
        let name = match namespace.name {
            Some(name) => {
                let name = name.copy_into(self.arena);
                Some(&*self.arena.alloc(name))
            }
            None => None,
        };

        let namespace = Namespace { span: namespace.span, name, body };

        Ok(Statement { meta, span, kind: StatementKind::Namespace(self.arena.alloc(namespace)), terminator })
    }
}
