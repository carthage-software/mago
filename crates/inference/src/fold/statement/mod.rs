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

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

mod branch;

/// The typed statements of a block paired with how the block exits.
type InferredBlock<'arena> = (&'arena [Statement<'arena, SymbolId, Flow, Type<'arena>>], ControlFlow);

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_statement(
        &mut self,
        statement: &Statement<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let reachable = self.reachable;

        let mut typed = match statement.kind {
            StatementKind::Expression(expression) => self.infer_expression_statement(statement.span, expression)?,
            StatementKind::Namespace(namespace) => self.infer_namespace(statement.span, namespace)?,
            StatementKind::Sequence(statements) => self.infer_sequence(statement.span, statements)?,
            StatementKind::Return(value) => self.infer_return(statement.span, value)?,
            StatementKind::If(conditional) => self.infer_if(statement.span, conditional)?,
            StatementKind::Noop => Statement {
                meta: Flow { reachable, exit: ControlFlow::Fallthrough },
                span: statement.span,
                kind: StatementKind::Noop,
            },
            _ => return Err(InferenceError::Unsupported { span: statement.span, construct: "this statement" }),
        };

        typed.meta.reachable = reachable;
        self.reachable = reachable && matches!(typed.meta.exit, ControlFlow::Fallthrough);

        Ok(typed)
    }

    /// Folds a run of statements, returning the typed statements and the block's
    /// own `exit` (how the block leaves: `Fallthrough` if control can fall off the
    /// end, otherwise the divergence of the first statement that left non-locally).
    /// Per-statement reachability is threaded through [`Self::reachable`] in
    /// [`Self::infer_statement`], so divergence here also propagates into the
    /// statements that follow and into any nested bodies.
    pub fn infer_block(
        &mut self,
        statements: &'source [Statement<'source, SymbolId, S, E>],
    ) -> InferenceResult<InferredBlock<'arena>> {
        let mut items = Vec::new_in(self.arena);
        let mut exit = ControlFlow::Fallthrough;

        for statement in statements {
            let was_reachable = self.reachable;
            let typed = self.infer_statement(statement)?;
            if was_reachable && !matches!(typed.meta.exit, ControlFlow::Fallthrough) {
                exit = typed.meta.exit;
            }

            items.push(typed);
        }

        Ok((items.leak(), exit))
    }

    pub fn infer_namespace(
        &mut self,
        span: Span,
        namespace: &'source Namespace<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let previous = self.namespace;
        self.namespace = match namespace.name {
            Some(name) => name.value,
            None => b"",
        };

        let body = self.infer_statement(namespace.statement)?;
        self.namespace = previous;

        let meta = Flow { reachable: true, exit: body.meta.exit };
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

    pub fn infer_sequence(
        &mut self,
        span: Span,
        statements: &'source [Statement<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let (items, exit) = self.infer_block(statements)?;

        Ok(Statement { meta: Flow { reachable: true, exit }, span, kind: StatementKind::Sequence(items) })
    }

    pub fn infer_return(
        &mut self,
        span: Span,
        value: Option<&'source Expression<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let value = match value {
            Some(value) => {
                let value = self.infer_expression(value)?;

                Some(&*self.arena.alloc(value))
            }
            None => None,
        };

        let exit = match value {
            Some(value) if value.meta.is_never() => ControlFlow::Diverge,
            _ => ControlFlow::Return,
        };

        Ok(Statement { meta: Flow { reachable: true, exit }, span, kind: StatementKind::Return(value) })
    }

    pub fn infer_expression_statement(
        &mut self,
        span: Span,
        expression: &'source Expression<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let inferred_expression = self.infer_expression(expression)?;

        let exit = if inferred_expression.meta.is_never() { ControlFlow::Diverge } else { ControlFlow::Fallthrough };

        Ok(Statement {
            meta: Flow { reachable: true, exit },
            span,
            kind: StatementKind::Expression(self.arena.alloc(inferred_expression)),
        })
    }
}
