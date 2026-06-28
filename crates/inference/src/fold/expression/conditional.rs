use mago_allocator::Arena;
use mago_hir::ir::expression::Conditional;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::assertion::Assertion;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::reconciler::reconcile;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_conditional(
        &mut self,
        span: Span,
        conditional: &'source Conditional<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let (condition, when_true, when_false) = self.analyze_condition(conditional.condition);
        let condition_meta = condition.meta;

        let (then_node, then_result, then_environment) = match conditional.then {
            Some(then) => match when_true {
                Some(environment) => {
                    self.environment = environment;
                    let typed = self.infer_expression(then);
                    let result = typed.meta;
                    let post = self.environment.clone();

                    (Some(&*self.arena.alloc(typed)), Some(result), Some(post))
                }
                None => {
                    let typed = self.infer_expression(then);

                    (Some(&*self.arena.alloc(typed)), None, None)
                }
            },
            None => match when_true {
                Some(environment) => {
                    let result = reconcile(&mut self.ty, self.symbols, Assertion::Truthy, condition_meta);

                    (None, Some(result), Some(environment))
                }
                None => (None, None, None),
            },
        };

        let (else_node, else_result, else_environment) = match when_false {
            Some(environment) => {
                self.environment = environment;
                let typed = self.infer_expression(conditional.r#else);
                let result = typed.meta;
                let post = self.environment.clone();

                (&*self.arena.alloc(typed), Some(result), Some(post))
            }
            None => {
                let typed = self.infer_expression(conditional.r#else);

                (&*self.arena.alloc(typed), None, None)
            }
        };

        let meta = match (then_result, else_result) {
            (Some(then), Some(otherwise)) => self.union(then, otherwise),
            (Some(then), None) => then,
            (None, Some(otherwise)) => otherwise,
            (None, None) => TYPE_NEVER,
        };

        if let Some(environment) = self.merge_condition_environments(then_environment, else_environment) {
            self.environment = environment;
        }

        let conditional = Conditional {
            span: conditional.span,
            condition: self.arena.alloc(condition),
            then: then_node,
            r#else: else_node,
        };

        Expression { meta, span, kind: ExpressionKind::Conditional(self.arena.alloc(conditional)) }
    }
}
