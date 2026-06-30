use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::argument::Argument;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_BOOL;
use mago_oracle::ty::well_known::TYPE_FALSE;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_oracle::ty::well_known::TYPE_TRUE;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::semantics::truthiness;

/// Which of the four file-inclusion constructs is being inferred.
pub enum RequireKind {
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
}

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_empty(
        &mut self,
        span: Span,
        operand: &'source Expression<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let operand = self.infer_expression(operand)?;

        let meta = if operand.meta.is_never() {
            TYPE_NEVER
        } else {
            match truthiness(operand.meta) {
                Some(true) => TYPE_FALSE,
                Some(false) => TYPE_TRUE,
                None => TYPE_BOOL,
            }
        };

        Ok(Expression { meta, span, kind: ExpressionKind::Empty(self.arena.alloc(operand)) })
    }

    pub fn infer_eval(
        &mut self,
        span: Span,
        operand: &'source Expression<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let operand = self.infer_expression(operand)?;
        let meta = if operand.meta.is_never() { TYPE_NEVER } else { TYPE_MIXED };

        Ok(Expression { meta, span, kind: ExpressionKind::Eval(self.arena.alloc(operand)) })
    }

    pub fn infer_print(
        &mut self,
        span: Span,
        operand: &'source Expression<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let operand = self.infer_expression(operand)?;
        let meta = if operand.meta.is_never() { TYPE_NEVER } else { self.ty.int_literal_type(1) };

        Ok(Expression { meta, span, kind: ExpressionKind::Print(self.arena.alloc(operand)) })
    }

    pub fn infer_require_like(
        &mut self,
        span: Span,
        operand: &'source Expression<'source, SymbolId, S, E>,
        kind: RequireKind,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let operand = self.infer_expression(operand)?;
        let meta = if operand.meta.is_never() { TYPE_NEVER } else { TYPE_MIXED };
        let operand = self.arena.alloc(operand);

        let kind = match kind {
            RequireKind::Include => ExpressionKind::Include(operand),
            RequireKind::IncludeOnce => ExpressionKind::IncludeOnce(operand),
            RequireKind::Require => ExpressionKind::Require(operand),
            RequireKind::RequireOnce => ExpressionKind::RequireOnce(operand),
        };

        Ok(Expression { meta, span, kind })
    }

    pub fn infer_isset(
        &mut self,
        span: Span,
        delimited: &Delimited<'source, Expression<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut items = Vec::new_in(self.arena);
        let mut any_unset = false;
        let mut all_set = true;
        let mut has_never = false;

        for operand in delimited.items {
            let operand = self.infer_expression(operand)?;
            let operand_type = operand.meta;
            if operand_type.is_never() {
                has_never = true;
            } else if matches!(operand_type.atoms, [Atom::Null]) {
                any_unset = true;
            } else if operand_type.atoms.iter().any(|atom| matches!(atom, Atom::Null | Atom::Mixed(_))) {
                all_set = false;
            }

            items.push(operand);
        }

        let meta = if has_never {
            TYPE_NEVER
        } else if any_unset {
            TYPE_FALSE
        } else if all_set {
            TYPE_TRUE
        } else {
            TYPE_BOOL
        };

        Ok(Expression {
            meta,
            span,
            kind: ExpressionKind::Isset(Delimited { span: delimited.span, items: items.leak() }),
        })
    }

    pub fn infer_exit(
        &mut self,
        span: Span,
        arguments: Option<&Delimited<'source, Argument<'source, SymbolId, S, E>>>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let arguments = match arguments {
            Some(delimited) => {
                let mut items = Vec::new_in(self.arena);
                for argument in delimited.items {
                    items.push(self.infer_argument(argument)?);
                }

                Some(Delimited { span: delimited.span, items: items.leak() })
            }
            None => None,
        };

        Ok(Expression { meta: TYPE_NEVER, span, kind: ExpressionKind::Exit(arguments) })
    }

    fn infer_argument(
        &mut self,
        argument: &'source Argument<'source, SymbolId, S, E>,
    ) -> InferenceResult<Argument<'arena, SymbolId, Flow, Type<'arena>>> {
        Ok(match argument {
            Argument::Value(expression) => {
                let expression = self.infer_expression(expression)?;

                Argument::Value(self.arena.alloc(expression))
            }
            Argument::Variadic(expression) => {
                let expression = self.infer_expression(expression)?;

                Argument::Variadic(self.arena.alloc(expression))
            }
            Argument::Named(name, expression) => {
                let expression = self.infer_expression(expression)?;

                Argument::Named(name.copy_into(self.arena), self.arena.alloc(expression))
            }
        })
    }
}
