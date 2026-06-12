use crate::ir::delimited::Delimited;
use crate::ir::expression::ArrayElement;
use crate::ir::expression::ArrayElementKind;
use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::annotation::Annotation;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::Variable;
use crate::ir::variable::annotation::VariableAnnotation;
use crate::lower::Lowering;
use crate::lower::statement::annotation::VariableBindings;
use mago_allocator::Arena;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn annotate_expression(
        &self,
        expression: &'arena Expression<'arena, (), (), ()>,
        type_annotation: &'arena TypeAnnotation<'arena>,
    ) -> &'arena Expression<'arena, (), (), ()> {
        let annotation = self.arena.alloc(VariableAnnotation {
            span: type_annotation.span,
            type_annotation,
            variable: None,
            errors: &[],
        });

        self.arena.alloc(Expression {
            meta: (),
            span: expression.span,
            kind: ExpressionKind::Annotation(self.arena.alloc(Annotation { annotation, expression })),
        })
    }

    pub(crate) fn fold_assignment_statement(
        &self,
        expression: &'arena Expression<'arena, (), (), ()>,
        bindings: &mut VariableBindings<'scratch, 'arena, S>,
    ) -> &'arena Expression<'arena, (), (), ()> {
        let ExpressionKind::Assignment(assignment) = expression.kind else {
            return expression;
        };

        let mut folded = *assignment;
        folded.left = self.fold_assignment_target(folded.left, bindings);
        if let Some(type_annotation) = bindings.take_unnamed() {
            folded.right = self.annotate_expression(folded.right, type_annotation);
        }

        self.arena.alloc(Expression {
            meta: (),
            span: expression.span,
            kind: ExpressionKind::Assignment(self.arena.alloc(folded)),
        })
    }

    pub(crate) fn fold_returned_expression(
        &self,
        expression: Option<&'arena Expression<'arena, (), (), ()>>,
        bindings: &mut VariableBindings<'scratch, 'arena, S>,
    ) -> Option<&'arena Expression<'arena, (), (), ()>> {
        let expression = expression?;

        match bindings.take_unnamed() {
            Some(type_annotation) => Some(self.annotate_expression(expression, type_annotation)),
            None => Some(expression),
        }
    }

    pub(crate) fn fold_assignment_target(
        &self,
        target: &'arena Expression<'arena, (), (), ()>,
        bindings: &mut VariableBindings<'scratch, 'arena, S>,
    ) -> &'arena Expression<'arena, (), (), ()> {
        match target.kind {
            ExpressionKind::Variable(Variable::Direct(variable)) => match bindings.take_named(variable.name) {
                Some(type_annotation) => self.annotate_expression(target, type_annotation),
                None => target,
            },
            ExpressionKind::Array(elements) => {
                let elements = self.fold_assignment_target_elements(elements, bindings);

                self.arena.alloc(Expression { meta: (), span: target.span, kind: ExpressionKind::Array(elements) })
            }
            ExpressionKind::List(elements) => {
                let elements = self.fold_assignment_target_elements(elements, bindings);

                self.arena.alloc(Expression { meta: (), span: target.span, kind: ExpressionKind::List(elements) })
            }
            _ => target,
        }
    }

    fn fold_assignment_target_elements(
        &self,
        elements: Delimited<'arena, ArrayElement<'arena, (), (), ()>>,
        bindings: &mut VariableBindings<'scratch, 'arena, S>,
    ) -> Delimited<'arena, ArrayElement<'arena, (), (), ()>> {
        Delimited {
            span: elements.span,
            items: self.arena.alloc_slice_fill_iter(elements.iter().map(|element| {
                let kind = match element.kind {
                    ArrayElementKind::Value(value) => {
                        ArrayElementKind::Value(self.fold_assignment_target(value, bindings))
                    }
                    ArrayElementKind::KeyValue(key, value) => {
                        ArrayElementKind::KeyValue(key, self.fold_assignment_target(value, bindings))
                    }
                    other => other,
                };

                ArrayElement { span: element.span, kind }
            })),
        }
    }
}
