use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::statement::GlobalItem;
use mago_hir::ir::statement::Namespace;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::StaticItem;
use mago_hir::ir::r#type::annotation::TypeAnnotation;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::lower_type_annotation;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

mod annotation;
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
            StatementKind::Echo(expressions) => self.infer_echo(statement.span, expressions)?,
            StatementKind::Global(items) => self.infer_global(statement.span, items)?,
            StatementKind::Static(items) => self.infer_static(statement.span, items)?,
            StatementKind::Unset(Delimited { span: operands_span, items }) => {
                self.infer_unset(statement.span, operands_span, items)?
            }
            StatementKind::VariableBindingAnnotation(annotation) => {
                self.infer_variable_binding_annotation(statement.span, annotation)?
            }
            StatementKind::HaltCompiler => Statement {
                meta: Flow { reachable, exit: ControlFlow::Diverge },
                span: statement.span,
                kind: StatementKind::HaltCompiler,
            },
            StatementKind::Inline(_)
            | StatementKind::Use(_)
            | StatementKind::Item(_)
            | StatementKind::Declare(_)
            | StatementKind::Goto(_)
            | StatementKind::Label(_)
            | StatementKind::Try(_)
            | StatementKind::Foreach(_)
            | StatementKind::For(_)
            | StatementKind::While(_)
            | StatementKind::DoWhile(_)
            | StatementKind::Continue(_)
            | StatementKind::Break(_)
            | StatementKind::Switch(_) => {
                return Err(InferenceError::Unsupported { span: statement.span, construct: "this statement" });
            }
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

    /// `echo $a, $b;`: every operand is inferred for its side effects; the
    /// statement diverges only if one of them does.
    fn infer_echo(
        &mut self,
        span: Span,
        expressions: &'source [Expression<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut items = Vec::new_in(self.arena);
        let mut diverges = false;
        for expression in expressions {
            let typed = self.infer_expression(expression)?;
            diverges |= typed.meta.is_never();
            items.push(typed);
        }

        let exit = if diverges { ControlFlow::Diverge } else { ControlFlow::Fallthrough };

        Ok(Statement { meta: Flow { reachable: true, exit }, span, kind: StatementKind::Echo(items.leak()) })
    }

    /// `global $a, $b;`: each named global enters the local scope with its
    /// declared `@var` type, or `mixed` when none is given.
    fn infer_global(
        &mut self,
        span: Span,
        items: &'source [GlobalItem<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut typed_items = Vec::new_in(self.arena);
        for item in items {
            let ty = self.lowered_annotation(item.type_annotation).unwrap_or(TYPE_MIXED);
            if let Variable::Direct(direct) = &item.variable {
                self.environment.set(Var::new(self.arena.alloc_slice_copy(direct.name)), ty);
            }

            let variable = self.infer_variable_node(&item.variable)?;
            let type_annotation = item.type_annotation.map(|annotation| copy_ref_into(annotation, self.arena));
            typed_items.push(GlobalItem { span: item.span, variable, type_annotation });
        }

        Ok(Statement {
            meta: Flow { reachable: true, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Global(typed_items.leak()),
        })
    }

    /// `static $x = …;`: each static local is bound to its declared `@var` type,
    /// else its initializer's type, else `mixed`.
    fn infer_static(
        &mut self,
        span: Span,
        items: &'source [StaticItem<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut typed_items = Vec::new_in(self.arena);
        let mut diverges = false;
        for item in items {
            let value = match item.value {
                Some(value) => {
                    let typed = self.infer_expression(value)?;
                    diverges |= typed.meta.is_never();

                    Some(&*self.arena.alloc(typed))
                }
                None => None,
            };

            let ty = self
                .lowered_annotation(item.type_annotation)
                .or_else(|| value.map(|value| value.meta))
                .unwrap_or(TYPE_MIXED);
            self.environment.set(Var::new(self.arena.alloc_slice_copy(item.variable.name)), ty);

            let type_annotation = item.type_annotation.map(|annotation| copy_ref_into(annotation, self.arena));
            typed_items.push(StaticItem {
                span: item.span,
                variable: item.variable.copy_into(self.arena),
                type_annotation,
                value,
            });
        }

        let exit = if diverges { ControlFlow::Diverge } else { ControlFlow::Fallthrough };

        Ok(Statement { meta: Flow { reachable: true, exit }, span, kind: StatementKind::Static(typed_items.leak()) })
    }

    /// `unset($a, $b);`: each operand is inferred, and a directly-named variable
    /// is forgotten so a later read sees it as undefined (`mixed`).
    fn infer_unset(
        &mut self,
        span: Span,
        operands_span: Span,
        operands: &'source [Expression<'source, SymbolId, S, E>],
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut items = Vec::new_in(self.arena);
        for operand in operands {
            let typed = self.infer_expression(operand)?;
            if let ExpressionKind::Variable(Variable::Direct(direct)) = typed.kind {
                self.environment.unset(Var::new(direct.name));
            }

            items.push(typed);
        }

        Ok(Statement {
            meta: Flow { reachable: true, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Unset(Delimited { span: operands_span, items: items.leak() }),
        })
    }

    /// Lowers an optional `@var`-style type annotation to its type.
    fn lowered_annotation(&mut self, annotation: Option<&'source TypeAnnotation<'source>>) -> Option<Type<'arena>> {
        let annotation = annotation?.copy_into(self.arena);

        lower_type_annotation(&mut self.ty, &annotation)
    }
}
