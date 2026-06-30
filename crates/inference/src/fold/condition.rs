use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Binary;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::UnaryPrefix;
use mago_hir::ir::expression::operator::BinaryOperator;
use mago_hir::ir::expression::operator::UnaryPrefixOperator;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_BOOL;
use mago_oracle::ty::well_known::TYPE_FALSE;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_oracle::ty::well_known::TYPE_TRUE;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::extension::AssertionTiming;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;
use crate::reconciler::reconcile;
use crate::semantics::truthiness;

/// A condition analyzed into its two truth-paths: the environment as it would be
/// if the condition holds, and if it does not. `None` means that path is
/// provably impossible (e.g. the `else` of an always-true condition).
type Continuations<'arena, Env> = (Expression<'arena, SymbolId, Flow, Type<'arena>>, Option<Env>, Option<Env>);

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// Infers a condition expression and computes the environment on each of its
    /// truth-paths, threading both per-path narrowing and per-path side effects
    /// (`&&`/`||` short-circuit, assignments). `None` for a path means it is
    /// unreachable. The returned expression is the fully-typed condition node.
    pub fn analyze_condition(
        &mut self,
        expr: &'source Expression<'source, SymbolId, S, E>,
    ) -> InferenceResult<Continuations<'arena, Environment<'source, 'arena, A>>> {
        let result = match &expr.kind {
            ExpressionKind::Parenthesized(inner) => {
                let (inner, when_true, when_false) = self.analyze_condition(inner)?;
                let meta = inner.meta;

                let node =
                    Expression { meta, span: expr.span, kind: ExpressionKind::Parenthesized(self.arena.alloc(inner)) };

                (node, when_true, when_false)
            }
            ExpressionKind::UnaryPrefix(unary) if matches!(unary.operator, UnaryPrefixOperator::Not) => {
                let (operand, operand_true, operand_false) = self.analyze_condition(unary.operand)?;

                let when_true = operand_false;
                let when_false = operand_true;
                let meta = reachability_meta(when_true.is_some(), when_false.is_some());

                let node =
                    UnaryPrefix { span: unary.span, operator: unary.operator, operand: self.arena.alloc(operand) };

                (
                    Expression { meta, span: expr.span, kind: ExpressionKind::UnaryPrefix(self.arena.alloc(node)) },
                    when_true,
                    when_false,
                )
            }
            ExpressionKind::Binary(binary) if matches!(binary.operator, BinaryOperator::And) => {
                self.analyze_and(expr.span, binary)?
            }
            ExpressionKind::Binary(binary) if matches!(binary.operator, BinaryOperator::Or) => {
                self.analyze_or(expr.span, binary)?
            }
            _ => {
                let typed = self.infer_expression(expr)?;
                let when_true = match truthiness(typed.meta) {
                    Some(false) => None,
                    _ => self.narrowed_environment(&typed, true),
                };
                let when_false = match truthiness(typed.meta) {
                    Some(true) => None,
                    _ => self.narrowed_environment(&typed, false),
                };

                (typed, when_true, when_false)
            }
        };

        Ok(result)
    }

    fn analyze_and(
        &mut self,
        span: Span,
        binary: &'source Binary<'source, SymbolId, S, E>,
    ) -> InferenceResult<Continuations<'arena, Environment<'source, 'arena, A>>> {
        let (left, left_true, left_false) = self.analyze_condition(binary.left)?;

        let (right, when_true, when_false) = match left_true {
            Some(left_true) => {
                self.environment = left_true;
                let (right, right_true, right_false) = self.analyze_condition(binary.right)?;
                let when_false = Environment::merge_options(left_false, right_false, &mut self.ty);

                (right, right_true, when_false)
            }
            None => {
                let (right, _, _) = self.analyze_condition(binary.right)?;

                (right, None, left_false)
            }
        };

        let meta = reachability_meta(when_true.is_some(), when_false.is_some());
        let node = Binary {
            span: binary.span,
            left: self.arena.alloc(left),
            operator: binary.operator,
            right: self.arena.alloc(right),
        };

        Ok((Expression { meta, span, kind: ExpressionKind::Binary(self.arena.alloc(node)) }, when_true, when_false))
    }

    fn analyze_or(
        &mut self,
        span: Span,
        binary: &'source Binary<'source, SymbolId, S, E>,
    ) -> InferenceResult<Continuations<'arena, Environment<'source, 'arena, A>>> {
        let (left, left_true, left_false) = self.analyze_condition(binary.left)?;

        let (right, when_true, when_false) = match left_false {
            Some(left_false) => {
                self.environment = left_false;
                let (right, right_true, right_false) = self.analyze_condition(binary.right)?;
                let when_true = Environment::merge_options(left_true, right_true, &mut self.ty);

                (right, when_true, right_false)
            }
            None => {
                let (right, _, _) = self.analyze_condition(binary.right)?;

                (right, left_true, None)
            }
        };

        let meta = reachability_meta(when_true.is_some(), when_false.is_some());
        let node = Binary {
            span: binary.span,
            left: self.arena.alloc(left),
            operator: binary.operator,
            right: self.arena.alloc(right),
        };

        Ok((Expression { meta, span, kind: ExpressionKind::Binary(self.arena.alloc(node)) }, when_true, when_false))
    }

    /// A clone of the current environment narrowed by the condition's `polarity`
    /// assertions. `None` if a narrowing has no inhabitants (an impossible path).
    fn narrowed_environment(
        &mut self,
        condition: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        polarity: bool,
    ) -> Option<Environment<'source, 'arena, A>> {
        let mut environment = self.environment.clone();

        let mut assertions = Vec::new_in(self.source);
        self.narrowing_assertions(condition, polarity, &mut assertions);
        for (place, base, assertion) in &assertions {
            let current = environment.lookup(*place).unwrap_or(*base);
            let narrowed = reconcile(&mut self.ty, self.symbols, *assertion, current);
            if narrowed.is_never() {
                return None;
            }

            environment.set(*place, narrowed);
        }

        if !self.extensions.assertion.is_empty() {
            for (variable, assertion, timing) in &self.extension_assertions(condition) {
                let applies = match timing {
                    AssertionTiming::Always => true,
                    AssertionTiming::WhenTrue => polarity,
                    AssertionTiming::WhenFalse => !polarity,
                };

                if !applies {
                    continue;
                }

                let base = environment.get(*variable);
                let narrowed = reconcile(&mut self.ty, self.symbols, *assertion, base);
                if narrowed.is_never() {
                    return None;
                }

                environment.set(*variable, narrowed);
            }
        }

        Some(environment)
    }
}

fn reachability_meta<'arena>(when_true: bool, when_false: bool) -> Type<'arena> {
    match (when_true, when_false) {
        (true, false) => TYPE_TRUE,
        (false, true) => TYPE_FALSE,
        (false, false) => TYPE_NEVER,
        (true, true) => TYPE_BOOL,
    }
}
