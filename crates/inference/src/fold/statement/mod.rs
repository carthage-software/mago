use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::statement::Namespace;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_span::Span;

use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

mod branch;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_statement(
        &mut self,
        statement: &'source Statement<'source, SymbolId, S, E>,
    ) -> Statement<'arena, SymbolId, Flow, Type<'arena>> {
        match statement.kind {
            StatementKind::Expression(expression) => self.infer_expression_statement(statement.span, expression),
            StatementKind::Namespace(namespace) => self.infer_namespace(statement.span, namespace),
            StatementKind::Sequence(statements) => self.infer_sequence(statement.span, statements),
            StatementKind::Return(value) => self.infer_return(statement.span, value),
            StatementKind::If(conditional) => self.infer_if(statement.span, conditional),
            StatementKind::Noop => Statement {
                meta: Flow { reachable: true, exit: ControlFlow::Fallthrough },
                span: statement.span,
                kind: StatementKind::Noop,
            },
            _ => todo!(),
        }
    }

    /// Folds a run of statements, threading reachability: once a statement exits
    /// non-locally (`return`, `break`, `continue`, or a diverging expression),
    /// every following statement is marked unreachable. The returned [`Flow`] is
    /// the block's own: `reachable` is whether control falls off the end, and
    /// `exit` is how it left when it did not.
    pub fn infer_block(
        &mut self,
        statements: &'source [Statement<'source, SymbolId, S, E>],
    ) -> (&'arena [Statement<'arena, SymbolId, Flow, Type<'arena>>], Flow) {
        let mut items = Vec::new_in(self.arena);
        let mut reachable = true;
        let mut exit = ControlFlow::Fallthrough;

        for statement in statements {
            let mut typed = self.infer_statement(statement);
            if reachable {
                if !matches!(typed.meta.exit, ControlFlow::Fallthrough) {
                    exit = typed.meta.exit;
                    reachable = false;
                }
            } else {
                typed.meta.reachable = false;
            }

            items.push(typed);
        }

        (items.leak(), Flow { reachable, exit })
    }

    pub fn infer_namespace(
        &mut self,
        span: Span,
        namespace: &'source Namespace<'source, SymbolId, S, E>,
    ) -> Statement<'arena, SymbolId, Flow, Type<'arena>> {
        let previous = self.namespace;
        self.namespace = match namespace.name {
            Some(name) => name.value,
            None => b"",
        };

        let body = self.infer_statement(namespace.statement);
        self.namespace = previous;

        let meta = body.meta;
        let statement = self.arena.alloc(body);
        let name = match namespace.name {
            Some(name) => {
                let name = name.copy_into(self.arena);
                Some(&*self.arena.alloc(name))
            }
            None => None,
        };

        let namespace = Namespace { span: namespace.span, name, statement };

        Statement { meta, span, kind: StatementKind::Namespace(self.arena.alloc(namespace)) }
    }

    pub fn infer_sequence(
        &mut self,
        span: Span,
        statements: &'source [Statement<'source, SymbolId, S, E>],
    ) -> Statement<'arena, SymbolId, Flow, Type<'arena>> {
        let (items, meta) = self.infer_block(statements);

        Statement { meta, span, kind: StatementKind::Sequence(items) }
    }

    pub fn infer_return(
        &mut self,
        span: Span,
        value: Option<&'source Expression<'source, SymbolId, S, E>>,
    ) -> Statement<'arena, SymbolId, Flow, Type<'arena>> {
        let value = match value {
            Some(value) => {
                let value = self.infer_expression(value);

                Some(&*self.arena.alloc(value))
            }
            None => None,
        };

        Statement {
            meta: Flow { reachable: true, exit: ControlFlow::Return },
            span,
            kind: StatementKind::Return(value),
        }
    }

    pub fn infer_expression_statement(
        &mut self,
        span: Span,
        expression: &'source Expression<'source, SymbolId, S, E>,
    ) -> Statement<'arena, SymbolId, Flow, Type<'arena>> {
        let inferred_expression = self.infer_expression(expression);

        let exit = if inferred_expression.meta.is_never() { ControlFlow::Diverge } else { ControlFlow::Fallthrough };

        Statement {
            meta: Flow { reachable: true, exit },
            span,
            kind: StatementKind::Expression(self.arena.alloc(inferred_expression)),
        }
    }
}
