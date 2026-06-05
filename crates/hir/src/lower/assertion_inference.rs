use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst;

use crate::ir::effect::annotation::AssertAnnotation;
use crate::ir::effect::annotation::AssertAnnotationTarget;
use crate::ir::parameter::Parameter;
use crate::ir::r#type::annotation::NamedTypeAnnotation;
use crate::ir::r#type::annotation::ReferenceKind;
use crate::ir::r#type::annotation::StringTypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotationKind;
use crate::ir::variable::DirectVariable;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

type InferredAssertionPair<'arena> = (AssertAnnotation<'arena>, AssertAnnotation<'arena>);

impl<'arena> Lowering<'_, 'arena> {
    /// Infers a function-like's `@assert-if-true`/`@assert-if-false` annotations from a single boolean
    /// `return`/arrow expression over a parameter (e.g. `return $x !== null;`). Returns the inferred
    /// `(if_true, if_false)` annotation slices, or `None` when nothing is inferred.
    pub(crate) fn infer_function_like_assertions(
        &self,
        return_expression: Option<&'arena cst::Expression<'arena>>,
        parameters: &'arena [Parameter<'arena, (), (), ()>],
    ) -> Option<(&'arena [AssertAnnotation<'arena>], &'arena [AssertAnnotation<'arena>])> {
        if !self.settings.infer_assertions || parameters.is_empty() {
            return None;
        }

        let return_expression = return_expression?;
        let span = return_expression.span();
        let (if_true, if_false) = self.infer_assertion_pair(return_expression, parameters, false, span)?;

        Some((self.arena.alloc_slice_copy(&[if_true]), self.arena.alloc_slice_copy(&[if_false])))
    }

    /// The single `return expr;` of a block body, if the block is exactly that. Multi-statement or
    /// multi-path bodies are not reasoned about, keeping inference sound.
    pub(crate) fn single_return_expression(
        &self,
        block: &'arena cst::Block<'arena>,
    ) -> Option<&'arena cst::Expression<'arena>> {
        if block.statements.len() != 1 {
            return None;
        }

        let cst::Statement::Return(r#return) = &block.statements.as_slice()[0] else {
            return None;
        };

        r#return.value
    }

    fn infer_assertion_pair(
        &self,
        expression: &'arena cst::Expression<'arena>,
        parameters: &'arena [Parameter<'arena, (), (), ()>],
        negated: bool,
        span: Span,
    ) -> Option<InferredAssertionPair<'arena>> {
        match unwrap_parens(expression) {
            cst::Expression::UnaryPrefix(unary) if matches!(unary.operator, cst::UnaryPrefixOperator::Not(_)) => {
                self.infer_assertion_pair(unary.operand, parameters, !negated, span)
            }
            cst::Expression::Binary(binary) => match &binary.operator {
                cst::BinaryOperator::Instanceof(_) => {
                    let (variable, kind) = self.parse_instanceof(binary.lhs, binary.rhs, parameters)?;

                    Some(self.build_assertion_pair(variable, kind, span, negated))
                }
                cst::BinaryOperator::Identical(_) | cst::BinaryOperator::Equal(_) => {
                    let variable = parse_null_compare(binary.lhs, binary.rhs, parameters)?;

                    Some(self.build_assertion_pair(variable, TypeAnnotationKind::Null, span, negated))
                }
                cst::BinaryOperator::NotIdentical(_)
                | cst::BinaryOperator::NotEqual(_)
                | cst::BinaryOperator::AngledNotEqual(_) => {
                    let variable = parse_null_compare(binary.lhs, binary.rhs, parameters)?;

                    Some(self.build_assertion_pair(variable, TypeAnnotationKind::Null, span, !negated))
                }
                _ => None,
            },
            cst::Expression::Call(cst::Call::Function(call)) => {
                let (variable, kind) = self.parse_type_check_function(call, parameters)?;

                Some(self.build_assertion_pair(variable, kind, span, negated))
            }
            _ => None,
        }
    }

    fn build_assertion_pair(
        &self,
        variable: DirectVariable<'arena>,
        kind: TypeAnnotationKind<'arena>,
        span: Span,
        negated: bool,
    ) -> InferredAssertionPair<'arena> {
        let r#type = &*self.arena.alloc(TypeAnnotation { span, kind });
        let target = AssertAnnotationTarget::Variable(variable);

        let if_true = AssertAnnotation { span, negated, equality: false, r#type, target };
        let if_false = AssertAnnotation { span, negated: !negated, equality: false, r#type, target };

        (if_true, if_false)
    }

    fn parse_instanceof(
        &self,
        lhs: &'arena cst::Expression<'arena>,
        rhs: &'arena cst::Expression<'arena>,
        parameters: &'arena [Parameter<'arena, (), (), ()>],
    ) -> Option<(DirectVariable<'arena>, TypeAnnotationKind<'arena>)> {
        let variable = parameter_variable(lhs, parameters)?;
        let cst::Expression::Identifier(identifier) = unwrap_parens(rhs) else {
            return None;
        };

        let kind = TypeAnnotationKind::Named(NamedTypeAnnotation {
            kind: ReferenceKind::Identifier(self.lower_identifier(identifier, Some(NameResolutionKind::Default))),
            type_arguments: &[],
        });

        Some((variable, kind))
    }

    fn parse_type_check_function(
        &self,
        call: &'arena cst::FunctionCall<'arena>,
        parameters: &'arena [Parameter<'arena, (), (), ()>],
    ) -> Option<(DirectVariable<'arena>, TypeAnnotationKind<'arena>)> {
        let cst::Expression::Identifier(function) = call.function else {
            return None;
        };

        let resolved = self.lower_identifier(function, Some(NameResolutionKind::Function));
        let kind = type_for_check_function(resolved.value)?;

        if call.argument_list.arguments.len() != 1 {
            return None;
        }

        let argument = call.argument_list.arguments.iter().next()?;

        parameter_variable(argument.value(), parameters).map(|variable| (variable, kind))
    }
}

fn unwrap_parens<'arena>(mut expression: &'arena cst::Expression<'arena>) -> &'arena cst::Expression<'arena> {
    while let cst::Expression::Parenthesized(parenthesized) = expression {
        expression = parenthesized.expression;
    }

    expression
}

fn parameter_variable<'arena>(
    expression: &'arena cst::Expression<'arena>,
    parameters: &[Parameter<'arena, (), (), ()>],
) -> Option<DirectVariable<'arena>> {
    let cst::Expression::Variable(cst::Variable::Direct(direct)) = unwrap_parens(expression) else {
        return None;
    };

    if parameters.iter().any(|parameter| parameter.variable.name == direct.name) {
        Some(DirectVariable { span: direct.span, name: direct.name })
    } else {
        None
    }
}

fn parse_null_compare<'arena>(
    lhs: &'arena cst::Expression<'arena>,
    rhs: &'arena cst::Expression<'arena>,
    parameters: &[Parameter<'arena, (), (), ()>],
) -> Option<DirectVariable<'arena>> {
    if let Some(variable) = parameter_variable(lhs, parameters)
        && is_null_literal(rhs)
    {
        return Some(variable);
    }

    if let Some(variable) = parameter_variable(rhs, parameters)
        && is_null_literal(lhs)
    {
        return Some(variable);
    }

    None
}

fn is_null_literal(expression: &cst::Expression<'_>) -> bool {
    matches!(unwrap_parens(expression), cst::Expression::Literal(cst::Literal::Null(_)))
}

fn type_for_check_function<'arena>(name: &[u8]) -> Option<TypeAnnotationKind<'arena>> {
    if name.eq_ignore_ascii_case(b"is_int")
        || name.eq_ignore_ascii_case(b"is_integer")
        || name.eq_ignore_ascii_case(b"is_long")
    {
        return Some(TypeAnnotationKind::Int(None));
    }

    if name.eq_ignore_ascii_case(b"is_string") {
        return Some(TypeAnnotationKind::String(StringTypeAnnotation {
            casing: None,
            literal: None,
            non_empty: false,
            numeric: false,
            truthy: false,
            callable: false,
        }));
    }

    if name.eq_ignore_ascii_case(b"is_float")
        || name.eq_ignore_ascii_case(b"is_double")
        || name.eq_ignore_ascii_case(b"is_real")
    {
        return Some(TypeAnnotationKind::Float(None));
    }

    if name.eq_ignore_ascii_case(b"is_bool") {
        return Some(TypeAnnotationKind::Bool(None));
    }

    if name.eq_ignore_ascii_case(b"is_null") {
        return Some(TypeAnnotationKind::Null);
    }

    if name.eq_ignore_ascii_case(b"is_object") {
        return Some(TypeAnnotationKind::Object);
    }

    if name.eq_ignore_ascii_case(b"is_numeric") {
        return Some(TypeAnnotationKind::Numeric);
    }

    None
}
