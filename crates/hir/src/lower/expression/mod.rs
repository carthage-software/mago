use mago_phpdoc_syntax::cst::expression::ConstantExpression;
use mago_phpdoc_syntax::cst::expression::UnaryPrefixConstantOperator;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::argument::Argument;
use crate::ir::expression::Access;
use crate::ir::expression::ArrayElement;
use crate::ir::expression::Assignment;
use crate::ir::expression::Binary;
use crate::ir::expression::Call;
use crate::ir::expression::Callee;
use crate::ir::expression::CompositeStringPart;
use crate::ir::expression::Conditional;
use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::Instantiation;
use crate::ir::expression::MagicConstant;
use crate::ir::expression::Match;
use crate::ir::expression::MatchArm;
use crate::ir::expression::PartialApplication;
use crate::ir::expression::UnaryPostfix;
use crate::ir::expression::UnaryPrefix;
use crate::ir::expression::Yield;
use crate::ir::expression::definition::DefinitionExpression;
use crate::ir::expression::definition::DefinitionExpressionKind;
use crate::ir::expression::operator::BinaryOperator;
use crate::ir::expression::operator::UnaryPrefixOperator;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::literal::Literal;
use crate::ir::literal::LiteralFloat;
use crate::ir::literal::LiteralInteger;
use crate::ir::literal::LiteralKind;
use crate::ir::literal::LiteralString;
use crate::ir::literal::LiteralStringKind;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

pub mod annotation;
pub mod definition;
pub mod operator;
pub mod selector;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_expression(
        &mut self,
        expression: &'arena cst::Expression<'arena>,
    ) -> Expression<'arena, (), (), ()> {
        Expression {
            meta: (),
            span: expression.span(),
            kind: match expression {
                cst::Expression::Parenthesized(expression) => self.lower_expression(expression.expression).kind,
                cst::Expression::Literal(literal) => ExpressionKind::Literal(self.lower_literal(literal)),
                cst::Expression::Binary(expression) => ExpressionKind::Binary(self.lower_binary(expression)),
                cst::Expression::Pipe(pipe) => ExpressionKind::Binary(self.lower_pipe(pipe)),
                cst::Expression::UnaryPrefix(unary) => ExpressionKind::UnaryPrefix(self.lower_unary_prefix(unary)),
                cst::Expression::UnaryPostfix(unary) => ExpressionKind::UnaryPostfix(self.lower_unary_postfix(unary)),
                cst::Expression::Assignment(assignment) => {
                    ExpressionKind::Assignment(self.lower_assignment(assignment))
                }
                cst::Expression::Conditional(conditional) => {
                    ExpressionKind::Conditional(self.lower_conditional(conditional))
                }
                cst::Expression::Array(array) => ExpressionKind::Array(self.lower_array_elements(&array.elements)),
                cst::Expression::LegacyArray(array) => {
                    ExpressionKind::Array(self.lower_array_elements(&array.elements))
                }
                cst::Expression::List(list) => ExpressionKind::List(self.lower_array_elements(&list.elements)),
                cst::Expression::ArrayAccess(access) => ExpressionKind::Access(self.lower_array_access(access)),
                cst::Expression::ArrayAppend(append) => {
                    ExpressionKind::ArrayAppend(self.arena.alloc(self.lower_expression(append.array)))
                }
                cst::Expression::Variable(variable) => ExpressionKind::Variable(self.lower_variable(variable)),
                cst::Expression::Match(r#match) => ExpressionKind::Match(self.lower_match(r#match)),
                cst::Expression::Yield(r#yield) => ExpressionKind::Yield(self.lower_yield(r#yield)),
                cst::Expression::Throw(throw) => {
                    ExpressionKind::Throw(self.arena.alloc(self.lower_expression(throw.exception)))
                }
                cst::Expression::Clone(clone) => {
                    ExpressionKind::Clone(self.arena.alloc(self.lower_expression(clone.object)))
                }
                cst::Expression::Construct(construct) => self.lower_construct(construct),
                cst::Expression::CompositeString(string) => self.lower_composite_string(string),
                cst::Expression::MagicConstant(magic_constant) => {
                    ExpressionKind::MagicConstant(self.lower_magic_constant(magic_constant))
                }
                cst::Expression::Parent(_) => ExpressionKind::Parent,
                cst::Expression::Static(_) => ExpressionKind::Static,
                cst::Expression::Self_(_) => ExpressionKind::Self_,
                cst::Expression::Identifier(identifier) => {
                    ExpressionKind::Identifier(self.lower_identifier(identifier, Some(NameResolutionKind::Default)))
                }
                cst::Expression::ConstantAccess(constant_access) => ExpressionKind::Constant(
                    self.lower_identifier(&constant_access.name, Some(NameResolutionKind::Constant)),
                ),
                cst::Expression::Instantiation(instantiation) => {
                    ExpressionKind::Instantiation(self.lower_instantiation(instantiation))
                }
                cst::Expression::Call(call) => ExpressionKind::Call(self.lower_call(call)),
                cst::Expression::PartialApplication(partial_application) => {
                    ExpressionKind::PartialApplication(self.lower_partial_application(partial_application))
                }
                cst::Expression::Access(access) => ExpressionKind::Access(self.lower_access(access)),
                cst::Expression::AnonymousClass(anonymous_class) => {
                    ExpressionKind::Definition(self.arena.alloc(DefinitionExpression {
                        meta: (),
                        kind: DefinitionExpressionKind::AnonymousClass(self.lower_anonymous_class(anonymous_class)),
                    }))
                }
                cst::Expression::Closure(closure) => {
                    ExpressionKind::Definition(self.arena.alloc(DefinitionExpression {
                        meta: (),
                        kind: DefinitionExpressionKind::Closure(self.lower_closure(closure)),
                    }))
                }
                cst::Expression::ArrowFunction(arrow_function) => {
                    ExpressionKind::Definition(self.arena.alloc(DefinitionExpression {
                        meta: (),
                        kind: DefinitionExpressionKind::ArrowFunction(self.lower_arrow_function(arrow_function)),
                    }))
                }
                cst::Expression::Error(_) => ExpressionKind::SyntaxError,
                _ => {
                    debug_assert!(false, "unhandled expression kind: {:?}", expression);

                    // SAFETY: This code is unreachable because all possible expression kinds have been handled in the match arms above.
                    // The debug assertion ensures that if an unhandled expression kind is encountered during development, it will be caught and fixed.
                    unsafe { std::hint::unreachable_unchecked() }
                }
            },
        }
    }

    pub(crate) fn lower_constant_expression(
        &self,
        expression: &'arena ConstantExpression<'arena>,
    ) -> Expression<'arena, (), (), ()> {
        let kind = match expression {
            ConstantExpression::Integer(integer) => ExpressionKind::Literal(self.arena.alloc(Literal {
                span: integer.span,
                kind: LiteralKind::Integer(LiteralInteger { raw: integer.raw, value: Some(integer.value) }),
            })),
            ConstantExpression::Float(float) => ExpressionKind::Literal(self.arena.alloc(Literal {
                span: float.span,
                kind: LiteralKind::Float(LiteralFloat { raw: float.raw, value: float.value }),
            })),
            ConstantExpression::String(string) => ExpressionKind::Literal(self.arena.alloc(Literal {
                span: string.span,
                kind: LiteralKind::String(LiteralString {
                    kind: if string.raw.starts_with(b"'") {
                        LiteralStringKind::SingleQuoted
                    } else {
                        LiteralStringKind::DoubleQuoted
                    },
                    raw: string.raw,
                    value: Some(string.value),
                }),
            })),
            ConstantExpression::True(keyword) => {
                ExpressionKind::Literal(self.arena.alloc(Literal { span: keyword.span, kind: LiteralKind::True }))
            }
            ConstantExpression::False(keyword) => {
                ExpressionKind::Literal(self.arena.alloc(Literal { span: keyword.span, kind: LiteralKind::False }))
            }
            ConstantExpression::Null(keyword) => {
                ExpressionKind::Literal(self.arena.alloc(Literal { span: keyword.span, kind: LiteralKind::Null }))
            }
            ConstantExpression::UnaryPrefix(unary) => ExpressionKind::UnaryPrefix(self.arena.alloc(UnaryPrefix {
                operator: match unary.operator {
                    UnaryPrefixConstantOperator::Plus(_) => UnaryPrefixOperator::Plus,
                    UnaryPrefixConstantOperator::Negation(_) => UnaryPrefixOperator::Negation,
                },
                operand: self.arena.alloc(self.lower_constant_expression(unary.operand)),
            })),
            ConstantExpression::ConstantAccess(constant) => {
                ExpressionKind::Constant(self.resolve_phpdoc_identifier(&constant.name, NameResolutionKind::Constant))
            }
            ConstantExpression::ClassLikeConstantAccess(access) => {
                ExpressionKind::Access(self.arena.alloc(Access::ClassConstant(
                    self.arena.alloc(Expression {
                        meta: (),
                        span: access.class.span,
                        kind: ExpressionKind::Identifier(self.resolve_phpdoc_class(&access.class)),
                    }),
                    ConstantSelector::Name(self.phpdoc_name(&access.constant)),
                )))
            }
            ConstantExpression::Array(array) => {
                ExpressionKind::Array(self.arena.alloc_slice_fill_iter(array.items.iter().map(|item| match item.key {
                    Some(key) => ArrayElement::KeyValue(
                        self.arena.alloc(self.lower_constant_expression(key)),
                        self.arena.alloc(self.lower_constant_expression(item.value)),
                    ),
                    None => ArrayElement::Value(self.arena.alloc(self.lower_constant_expression(item.value))),
                })))
            }
        };

        Expression { meta: (), span: expression.span(), kind }
    }

    pub(crate) fn lower_binary(&mut self, binary: &'arena cst::Binary<'arena>) -> &'arena Binary<'arena, (), (), ()> {
        self.arena.alloc(Binary {
            left: self.arena.alloc(self.lower_expression(binary.lhs)),
            operator: self.lower_binary_operator(&binary.operator),
            right: self.arena.alloc(self.lower_expression(binary.rhs)),
        })
    }

    pub(crate) fn lower_pipe(&mut self, pipe: &'arena cst::Pipe<'arena>) -> &'arena Binary<'arena, (), (), ()> {
        self.arena.alloc(Binary {
            left: self.arena.alloc(self.lower_expression(pipe.input)),
            operator: BinaryOperator::Pipe,
            right: self.arena.alloc(self.lower_expression(pipe.callable)),
        })
    }

    pub(crate) fn lower_unary_prefix(
        &mut self,
        unary: &'arena cst::UnaryPrefix<'arena>,
    ) -> &'arena UnaryPrefix<'arena, (), (), ()> {
        let operand = self.arena.alloc(self.lower_expression(unary.operand));

        self.arena.alloc(UnaryPrefix { operator: self.lower_unary_prefix_operator(&unary.operator), operand })
    }

    pub(crate) fn lower_unary_postfix(
        &mut self,
        unary: &'arena cst::UnaryPostfix<'arena>,
    ) -> &'arena UnaryPostfix<'arena, (), (), ()> {
        let operand = self.arena.alloc(self.lower_expression(unary.operand));

        self.arena.alloc(UnaryPostfix { operand, operator: self.lower_unary_postfix_operator(&unary.operator) })
    }

    pub(crate) fn lower_assignment(
        &mut self,
        assignment: &'arena cst::Assignment<'arena>,
    ) -> &'arena Assignment<'arena, (), (), ()> {
        let left = self.arena.alloc(self.lower_expression(assignment.lhs));
        let right = self.arena.alloc(self.lower_expression(assignment.rhs));

        self.arena.alloc(Assignment { left, operator: self.lower_assignment_operator(&assignment.operator), right })
    }

    pub(crate) fn lower_conditional(
        &mut self,
        conditional: &'arena cst::Conditional<'arena>,
    ) -> &'arena Conditional<'arena, (), (), ()> {
        let condition = self.arena.alloc(self.lower_expression(conditional.condition));
        let then = self.lower_optional_expression(conditional.then);
        let r#else = self.arena.alloc(self.lower_expression(conditional.r#else));

        self.arena.alloc(Conditional { condition, then, r#else })
    }

    pub(crate) fn lower_array_access(
        &mut self,
        access: &'arena cst::ArrayAccess<'arena>,
    ) -> &'arena Access<'arena, (), (), ()> {
        let array = self.arena.alloc(self.lower_expression(access.array));
        let index = self.arena.alloc(self.lower_expression(access.index));

        self.arena.alloc(Access::Array(array, index))
    }

    pub(crate) fn lower_array_elements(
        &mut self,
        elements: &'arena cst::TokenSeparatedSequence<'arena, cst::ArrayElement<'arena>>,
    ) -> &'arena [ArrayElement<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(elements.iter().map(|element| self.lower_array_element(element)))
    }

    fn lower_array_element(&mut self, element: &'arena cst::ArrayElement<'arena>) -> ArrayElement<'arena, (), (), ()> {
        match element {
            cst::ArrayElement::KeyValue(element) => {
                let key = self.arena.alloc(self.lower_expression(element.key));
                let value = self.arena.alloc(self.lower_expression(element.value));

                ArrayElement::KeyValue(key, value)
            }
            cst::ArrayElement::Value(element) => {
                ArrayElement::Value(self.arena.alloc(self.lower_expression(element.value)))
            }
            cst::ArrayElement::Variadic(element) => {
                ArrayElement::Variadic(self.arena.alloc(self.lower_expression(element.value)))
            }
            cst::ArrayElement::Missing(_) => ArrayElement::Missing,
        }
    }

    pub(crate) fn lower_match(&mut self, r#match: &'arena cst::Match<'arena>) -> &'arena Match<'arena, (), (), ()> {
        let subject = self.arena.alloc(self.lower_expression(r#match.expression));
        let arms = self.arena.alloc_slice_fill_iter(r#match.arms.iter().map(|arm| self.lower_match_arm(arm)));

        self.arena.alloc(Match { subject, arms })
    }

    fn lower_match_arm(&mut self, arm: &'arena cst::MatchArm<'arena>) -> MatchArm<'arena, (), (), ()> {
        match arm {
            cst::MatchArm::Expression(arm) => {
                let conditions = self.lower_expression_list(&arm.conditions);
                let expression = self.arena.alloc(self.lower_expression(arm.expression));

                MatchArm::Expression(conditions, expression)
            }
            cst::MatchArm::Default(arm) => MatchArm::Default(self.arena.alloc(self.lower_expression(arm.expression))),
        }
    }

    pub(crate) fn lower_yield(&mut self, r#yield: &'arena cst::Yield<'arena>) -> &'arena Yield<'arena, (), (), ()> {
        let lowered = match r#yield {
            cst::Yield::Value(value) => match self.lower_optional_expression(value.value) {
                Some(expression) => Yield::Expression(expression),
                None => Yield::Nothing,
            },
            cst::Yield::Pair(pair) => {
                let key = self.arena.alloc(self.lower_expression(pair.key));
                let value = self.arena.alloc(self.lower_expression(pair.value));

                Yield::Pair(key, value)
            }
            cst::Yield::From(from) => Yield::From(self.arena.alloc(self.lower_expression(from.iterator))),
        };

        self.arena.alloc(lowered)
    }

    fn lower_construct(&mut self, construct: &'arena cst::Construct<'arena>) -> ExpressionKind<'arena, (), (), ()> {
        match construct {
            cst::Construct::Isset(construct) => ExpressionKind::Isset(self.lower_expression_list(&construct.values)),
            cst::Construct::Empty(construct) => {
                ExpressionKind::Empty(self.arena.alloc(self.lower_expression(construct.value)))
            }
            cst::Construct::Eval(construct) => {
                ExpressionKind::Eval(self.arena.alloc(self.lower_expression(construct.value)))
            }
            cst::Construct::Include(construct) => {
                ExpressionKind::Include(self.arena.alloc(self.lower_expression(construct.value)))
            }
            cst::Construct::IncludeOnce(construct) => {
                ExpressionKind::IncludeOnce(self.arena.alloc(self.lower_expression(construct.value)))
            }
            cst::Construct::Require(construct) => {
                ExpressionKind::Require(self.arena.alloc(self.lower_expression(construct.value)))
            }
            cst::Construct::RequireOnce(construct) => {
                ExpressionKind::RequireOnce(self.arena.alloc(self.lower_expression(construct.value)))
            }
            cst::Construct::Print(construct) => {
                ExpressionKind::Print(self.arena.alloc(self.lower_expression(construct.value)))
            }
            cst::Construct::Exit(construct) => {
                ExpressionKind::Exit(self.lower_exit_arguments(construct.arguments.as_ref()))
            }
            cst::Construct::Die(construct) => {
                ExpressionKind::Exit(self.lower_exit_arguments(construct.arguments.as_ref()))
            }
        }
    }

    fn lower_exit_arguments(
        &mut self,
        arguments: Option<&'arena cst::ArgumentList<'arena>>,
    ) -> &'arena [Argument<'arena, (), (), ()>] {
        match arguments {
            Some(arguments) => self.lower_argument_list(arguments),
            None => &[],
        }
    }

    fn lower_composite_string(
        &mut self,
        string: &'arena cst::CompositeString<'arena>,
    ) -> ExpressionKind<'arena, (), (), ()> {
        let parts = self.arena.alloc_slice_fill_iter(string.parts().iter().map(|part| self.lower_string_part(part)));

        match string {
            cst::CompositeString::ShellExecute(_) => ExpressionKind::ShellExecute(parts),
            _ => ExpressionKind::CompositeString(parts),
        }
    }

    fn lower_string_part(&mut self, part: &'arena cst::StringPart<'arena>) -> CompositeStringPart<'arena, (), (), ()> {
        match part {
            cst::StringPart::Literal(literal) => CompositeStringPart::Literal(literal.value),
            cst::StringPart::Expression(expression) => {
                CompositeStringPart::Expression(self.arena.alloc(self.lower_expression(expression)))
            }
            cst::StringPart::BracedExpression(braced) => {
                CompositeStringPart::Expression(self.arena.alloc(self.lower_expression(braced.expression)))
            }
        }
    }

    fn lower_magic_constant(&self, magic_constant: &'arena cst::MagicConstant<'arena>) -> MagicConstant {
        match magic_constant {
            cst::MagicConstant::Line(_) => MagicConstant::Line,
            cst::MagicConstant::File(_) => MagicConstant::File,
            cst::MagicConstant::Directory(_) => MagicConstant::Directory,
            cst::MagicConstant::Trait(_) => MagicConstant::Trait,
            cst::MagicConstant::Method(_) => MagicConstant::Method,
            cst::MagicConstant::Function(_) => MagicConstant::Function,
            cst::MagicConstant::Property(_) => MagicConstant::Property,
            cst::MagicConstant::Namespace(_) => MagicConstant::Namespace,
            cst::MagicConstant::Class(_) => MagicConstant::Class,
        }
    }

    fn lower_instantiation(
        &mut self,
        instantiation: &'arena cst::Instantiation<'arena>,
    ) -> &'arena Instantiation<'arena, (), (), ()> {
        let class = self.arena.alloc(self.lower_expression(instantiation.class));
        let arguments = match &instantiation.argument_list {
            Some(argument_list) => self.lower_argument_list(argument_list),
            None => &[],
        };

        self.arena.alloc(Instantiation { class, arguments })
    }

    fn lower_call(&mut self, call: &'arena cst::Call<'arena>) -> &'arena Call<'arena, (), (), ()> {
        let (callee, argument_list) = match call {
            cst::Call::Function(function_call) => {
                (Callee::Function(self.lower_callee(function_call.function)), &function_call.argument_list)
            }
            cst::Call::Method(method_call) => {
                let object = self.arena.alloc(self.lower_expression(method_call.object));
                let method = self.lower_member_selector(&method_call.method);

                (Callee::Method(object, method), &method_call.argument_list)
            }
            cst::Call::NullSafeMethod(method_call) => {
                let object = self.arena.alloc(self.lower_expression(method_call.object));
                let method = self.lower_member_selector(&method_call.method);

                (Callee::NullsafeMethod(object, method), &method_call.argument_list)
            }
            cst::Call::StaticMethod(static_method_call) => {
                let class = self.arena.alloc(self.lower_expression(static_method_call.class));
                let method = self.lower_member_selector(&static_method_call.method);

                (Callee::StaticMethod(class, method), &static_method_call.argument_list)
            }
        };

        let arguments = self.lower_argument_list(argument_list);

        self.arena.alloc(Call { callee, arguments })
    }

    fn lower_partial_application(
        &mut self,
        partial_application: &'arena cst::PartialApplication<'arena>,
    ) -> &'arena PartialApplication<'arena, (), (), ()> {
        let (callee, argument_list) = match partial_application {
            cst::PartialApplication::Function(function) => {
                (Callee::Function(self.lower_callee(function.function)), &function.argument_list)
            }
            cst::PartialApplication::Method(method) => {
                let object = self.arena.alloc(self.lower_expression(method.object));
                let selector = self.lower_member_selector(&method.method);

                (Callee::Method(object, selector), &method.argument_list)
            }
            cst::PartialApplication::StaticMethod(static_method) => {
                let class = self.arena.alloc(self.lower_expression(static_method.class));
                let selector = self.lower_member_selector(&static_method.method);

                (Callee::StaticMethod(class, selector), &static_method.argument_list)
            }
        };

        let arguments = self.lower_partial_argument_list(argument_list);

        self.arena.alloc(PartialApplication { callee, arguments })
    }

    fn lower_callee(&mut self, callee: &'arena cst::Expression<'arena>) -> &'arena Expression<'arena, (), (), ()> {
        match callee {
            cst::Expression::Identifier(identifier) => self.arena.alloc(Expression {
                meta: (),
                span: identifier.span(),
                kind: ExpressionKind::Identifier(self.lower_identifier(identifier, Some(NameResolutionKind::Function))),
            }),
            _ => self.arena.alloc(self.lower_expression(callee)),
        }
    }

    fn lower_access(&mut self, access: &'arena cst::Access<'arena>) -> &'arena Access<'arena, (), (), ()> {
        let lowered = match access {
            cst::Access::Property(property_access) => {
                let object = self.arena.alloc(self.lower_expression(property_access.object));
                let selector = self.lower_member_selector(&property_access.property);

                Access::Property(object, selector)
            }
            cst::Access::NullSafeProperty(property_access) => {
                let object = self.arena.alloc(self.lower_expression(property_access.object));
                let selector = self.lower_member_selector(&property_access.property);

                Access::NullsafeProperty(object, selector)
            }
            cst::Access::StaticProperty(property_access) => {
                let class = self.arena.alloc(self.lower_expression(property_access.class));
                let property = self.lower_variable(&property_access.property);

                Access::StaticProperty(class, property)
            }
            cst::Access::ClassConstant(constant_access) => {
                let class = self.arena.alloc(self.lower_expression(constant_access.class));
                let selector = self.lower_constant_selector(&constant_access.constant);

                Access::ClassConstant(class, selector)
            }
        };

        self.arena.alloc(lowered)
    }
}
