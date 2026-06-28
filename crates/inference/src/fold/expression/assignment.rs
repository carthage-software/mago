use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::expression::Assignment;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::annotation::Annotation;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::lower_type_annotation;
use mago_oracle::ty::Type;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_assignment(
        &mut self,
        span: Span,
        assignment: &'source Assignment<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let value = self.infer_expression(assignment.right);

        let value_meta = match assignment.operator {
            None => value.meta,
            Some(_) => todo!(),
        };

        let target = self.bind_target(assignment.left, value_meta);
        let meta = target.meta;

        let assignment = Assignment {
            span: assignment.span,
            left: self.arena.alloc(target),
            operator: assignment.operator,
            right: self.arena.alloc(value),
        };

        Expression { meta, span, kind: ExpressionKind::Assignment(self.arena.alloc(assignment)) }
    }

    fn bind_target(
        &mut self,
        target: &'source Expression<'source, SymbolId, S, E>,
        ty: Type<'arena>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        match &target.kind {
            ExpressionKind::Variable(variable) => {
                let variable = match variable {
                    Variable::Direct(direct) => {
                        let direct = direct.copy_into(self.arena);
                        self.environment.insert(Var::new(direct.name), ty);

                        Variable::Direct(direct)
                    }
                    Variable::Indirect(expression) => {
                        let expression = self.infer_expression(expression);

                        Variable::Indirect(self.arena.alloc(expression))
                    }
                    Variable::Nested(_) => todo!(),
                };

                Expression { meta: ty, span: target.span, kind: ExpressionKind::Variable(variable) }
            }
            ExpressionKind::Annotation(annotation) => {
                // `/** @var T $x */` on an assignment target: bind the inner
                // target with the annotated type instead of the assigned value.
                let type_annotation = annotation.annotation.type_annotation.copy_into(self.arena);
                let annotated = lower_type_annotation(&mut self.ty, &type_annotation).unwrap_or(ty);

                let inner = self.bind_target(annotation.expression, annotated);

                let variable_annotation = annotation.annotation.copy_into(self.arena);
                let node = Annotation {
                    annotation: self.arena.alloc(variable_annotation),
                    expression: self.arena.alloc(inner),
                };

                Expression {
                    meta: annotated,
                    span: target.span,
                    kind: ExpressionKind::Annotation(self.arena.alloc(node)),
                }
            }
            _ => todo!(),
        }
    }
}
