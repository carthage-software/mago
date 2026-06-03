use crate::ir::expression::ArrayElement;
use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::annotation::Annotation;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::Variable;
use crate::lower::Lowering;
use crate::lower::statement::annotation::VariableBindings;

impl<'arena> Lowering<'arena> {
    pub(crate) fn annotate_expression(
        &self,
        expression: &'arena Expression<'arena, (), (), ()>,
        type_annotation: &'arena TypeAnnotation<'arena>,
    ) -> &'arena Expression<'arena, (), (), ()> {
        self.arena.alloc(Expression {
            meta: (),
            span: expression.span,
            kind: ExpressionKind::Annotation(self.arena.alloc(Annotation { expression, type_annotation })),
        })
    }

    pub(crate) fn fold_assignment_statement(
        &self,
        expression: &'arena Expression<'arena, (), (), ()>,
        bindings: &mut VariableBindings<'arena>,
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
        bindings: &mut VariableBindings<'arena>,
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
        bindings: &mut VariableBindings<'arena>,
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
        elements: &'arena [ArrayElement<'arena, (), (), ()>],
        bindings: &mut VariableBindings<'arena>,
    ) -> &'arena [ArrayElement<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(elements.iter().map(|element| match *element {
            ArrayElement::Value(value) => ArrayElement::Value(self.fold_assignment_target(value, bindings)),
            ArrayElement::KeyValue(key, value) => {
                ArrayElement::KeyValue(key, self.fold_assignment_target(value, bindings))
            }
            other => other,
        }))
    }
}
