use either::Either;

use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::argument;
use crate::parser::internal::array::parse_array;
use crate::parser::internal::array::parse_legacy_array;
use crate::parser::internal::array::parse_list;
use crate::parser::internal::attribute;
use crate::parser::internal::class_like::member;
use crate::parser::internal::class_like::parse_anonymous_class;
use crate::parser::internal::clone::parse_ambiguous_clone_expression;
use crate::parser::internal::construct::parse_construct;
use crate::parser::internal::control_flow::r#match::parse_match;
use crate::parser::internal::function_like::arrow_function::parse_arrow_function_with_attributes;
use crate::parser::internal::function_like::closure::parse_closure_with_attributes;
use crate::parser::internal::identifier;
use crate::parser::internal::instantiation::parse_instantiation;
use crate::parser::internal::literal;
use crate::parser::internal::magic_constant::parse_magic_constant;
use crate::parser::internal::operation::unary;
use crate::parser::internal::string::parse_string;
use crate::parser::internal::throw::parse_throw;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::parser::internal::variable;
use crate::parser::internal::r#yield::parse_yield;
use crate::token::Associativity;
use crate::token::Precedence;

pub fn parse_expression<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Expression<'arena>, ParseError> {
    parse_expression_with_precedence(stream, Precedence::Lowest)
}

pub fn parse_expression_with_precedence<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    precedence: Precedence,
) -> Result<Expression<'arena>, ParseError> {
    let mut left = parse_lhs_expression(stream, precedence)?;

    while let Some(next) = utils::maybe_peek(stream)? {
        if !stream.state.within_indirect_variable
            && !matches!(precedence, Precedence::Instanceof | Precedence::New)
            && !matches!(next.kind, T!["(" | "::"])
            && let Expression::Identifier(identifier) = left
        {
            left = Expression::ConstantAccess(ConstantAccess { name: identifier });
        }

        // Stop parsing if the next token is a terminator.
        if matches!(next.kind, T![";" | "?>"]) {
            break;
        }

        if next.kind.is_postfix() {
            let postfix_precedence = Precedence::postfix(&next.kind);
            if postfix_precedence < precedence {
                break;
            }

            left = parse_postfix_expression(stream, left, precedence)?;
        } else if next.kind.is_infix() {
            let infix_precedence = Precedence::infix(&next.kind);

            if infix_precedence < precedence {
                break;
            }

            if infix_precedence == precedence
                && let Some(Associativity::Left) = infix_precedence.associativity()
            {
                break;
            }

            left = parse_infix_expression(stream, left)?;
        } else {
            break;
        }
    }

    Ok(left)
}

#[inline]
fn parse_lhs_expression<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    precedence: Precedence,
) -> Result<Expression<'arena>, ParseError> {
    let token = utils::peek(stream)?;
    let next = utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind);

    let is_call = precedence != Precedence::New && matches!(next, Some(T!["("]));
    let is_call_or_access = is_call || matches!(next, Some(T!["[" | "::" | "->" | "?->"]));

    if token.kind.is_literal() && (!token.kind.is_keyword() || !is_call_or_access) {
        return literal::parse_literal(stream).map(Expression::Literal);
    }

    if token.kind.is_unary_prefix() {
        return unary::parse_unary_prefix_operation(stream).map(Expression::UnaryPrefix);
    }

    if matches!(token.kind, T!["#["]) {
        return parse_arrow_function_or_closure(stream).map(|e| match e {
            Either::Left(arrow_function) => Expression::ArrowFunction(arrow_function),
            Either::Right(closure) => Expression::Closure(closure),
        });
    }

    if matches!(token.kind, T!["clone"]) {
        return parse_ambiguous_clone_expression(stream);
    }

    if matches!((token.kind, next), (T!["function" | "fn"], _))
        || matches!((token.kind, next), (T!["static"], Some(T!["function" | "fn"])))
    {
        return parse_arrow_function_or_closure(stream).map(|e| match e {
            Either::Left(arrow_function) => Expression::ArrowFunction(arrow_function),
            Either::Right(closure) => Expression::Closure(closure),
        });
    }

    Ok(match (token.kind, next) {
        (T!["static"], _) => Expression::Static(utils::expect_any_keyword(stream)?),
        (T!["self"], _) if !is_call => Expression::Self_(utils::expect_any_keyword(stream)?),
        (T!["parent"], _) if !is_call => Expression::Parent(utils::expect_any_keyword(stream)?),
        (kind, _) if kind.is_construct() => Expression::Construct(parse_construct(stream)?),
        (T!["list"], Some(T!["("])) => Expression::List(parse_list(stream)?),
        (T!["new"], Some(T!["class" | "#["])) => Expression::AnonymousClass(parse_anonymous_class(stream)?),
        (T!["new"], Some(T!["static"])) => Expression::Instantiation(parse_instantiation(stream)?),
        (T!["new"], Some(kind)) if kind.is_modifier() => Expression::AnonymousClass(parse_anonymous_class(stream)?),
        (T!["new"], _) => Expression::Instantiation(parse_instantiation(stream)?),
        (T!["throw"], _) => Expression::Throw(parse_throw(stream)?),
        (T!["yield"], _) => Expression::Yield(parse_yield(stream)?),
        (T!["\""] | T!["<<<"] | T!["`"], ..) => Expression::CompositeString(parse_string(stream)?),
        (T!["("], _) => Expression::Parenthesized(Parenthesized {
            left_parenthesis: utils::expect_span(stream, T!["("])?,
            expression: {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            },
            right_parenthesis: utils::expect_span(stream, T![")"])?,
        }),
        (T!["match"], Some(T!["("])) => Expression::Match(parse_match(stream)?),
        (T!["array"], Some(T!["("])) => Expression::LegacyArray(parse_legacy_array(stream)?),
        (T!["["], _) => Expression::Array(parse_array(stream)?),
        (T!["$" | "${" | "$variable"], _) => variable::parse_variable(stream).map(Expression::Variable)?,
        (kind, _) if kind.is_magic_constant() => Expression::MagicConstant(parse_magic_constant(stream)?),
        (kind, ..)
            if matches!(kind, T![Identifier | QualifiedIdentifier | FullyQualifiedIdentifier | "clone"])
                || kind.is_soft_reserved_identifier() =>
        {
            Expression::Identifier(identifier::parse_identifier(stream)?)
        }
        _ => return Err(utils::unexpected(stream, Some(token), &[])),
    })
}

fn parse_arrow_function_or_closure<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Either<ArrowFunction<'arena>, Closure<'arena>>, ParseError> {
    let attributes = attribute::parse_attribute_list_sequence(stream)?;

    let next = utils::peek(stream)?;
    let after = utils::maybe_peek_nth(stream, 1)?;

    Ok(match (next.kind, after.map(|t| t.kind)) {
        (T!["function"], _) | (T!["static"], Some(T!["function"])) => {
            Either::Right(parse_closure_with_attributes(stream, attributes)?)
        }
        (T!["fn"], _) | (T!["static"], Some(T!["fn"])) => {
            Either::Left(parse_arrow_function_with_attributes(stream, attributes)?)
        }
        _ => return Err(utils::unexpected(stream, Some(next), &[T!["function"], T!["fn"], T!["static"]])),
    })
}

fn parse_postfix_expression<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    lhs: Expression<'arena>,
    precedence: Precedence,
) -> Result<Expression<'arena>, ParseError> {
    let operator = utils::peek(stream)?;

    Ok(match operator.kind {
        T!["("] => {
            if matches!(
                (utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind), utils::maybe_peek_nth(stream, 2)?.map(|t| t.kind)),
                (Some(T!["..."]), Some(T![")"])),
            ) {
                Expression::ClosureCreation(ClosureCreation::Function(FunctionClosureCreation {
                    function: stream.alloc(lhs),
                    left_parenthesis: utils::expect_any(stream)?.span,
                    ellipsis: utils::expect_any(stream)?.span,
                    right_parenthesis: utils::expect_any(stream)?.span,
                }))
            } else {
                Expression::Call(Call::Function(FunctionCall {
                    function: stream.alloc(lhs),
                    argument_list: argument::parse_argument_list(stream)?,
                }))
            }
        }
        T!["["] => {
            let left_bracket = utils::expect_any(stream)?.span;
            let next = utils::peek(stream)?;
            if matches!(next.kind, T!["]"]) {
                Expression::ArrayAppend(ArrayAppend {
                    array: stream.alloc(lhs),
                    left_bracket,
                    right_bracket: utils::expect_any(stream)?.span,
                })
            } else {
                Expression::ArrayAccess(ArrayAccess {
                    array: stream.alloc(lhs),
                    left_bracket,
                    index: {
                        let expression = parse_expression(stream)?;

                        stream.alloc(expression)
                    },
                    right_bracket: utils::expect(stream, T!["]"])?.span,
                })
            }
        }
        T!["::"] => {
            let double_colon = utils::expect_any(stream)?.span;
            let selector_or_variable = member::parse_classlike_constant_selector_or_variable(stream)?;
            let current = utils::peek(stream)?;

            if Precedence::CallDim > precedence && matches!(current.kind, T!["("]) {
                let method = match selector_or_variable {
                    Either::Left(selector) => match selector {
                        ClassLikeConstantSelector::Identifier(i) => ClassLikeMemberSelector::Identifier(i),
                        ClassLikeConstantSelector::Expression(c) => ClassLikeMemberSelector::Expression(c),
                    },
                    Either::Right(variable) => ClassLikeMemberSelector::Variable(variable),
                };

                if matches!(
                    (
                        utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind),
                        utils::maybe_peek_nth(stream, 2)?.map(|t| t.kind)
                    ),
                    (Some(T!["..."]), Some(T![")"]))
                ) {
                    Expression::ClosureCreation(ClosureCreation::StaticMethod(StaticMethodClosureCreation {
                        class: stream.alloc(lhs),
                        double_colon,
                        method,
                        left_parenthesis: utils::expect_any(stream)?.span,
                        ellipsis: utils::expect_any(stream)?.span,
                        right_parenthesis: utils::expect_any(stream)?.span,
                    }))
                } else {
                    let arguments = argument::parse_argument_list(stream)?;

                    Expression::Call(Call::StaticMethod(StaticMethodCall {
                        class: stream.alloc(lhs),
                        double_colon,
                        method,
                        argument_list: arguments,
                    }))
                }
            } else {
                match selector_or_variable {
                    Either::Left(selector) => Expression::Access(Access::ClassConstant(ClassConstantAccess {
                        class: stream.arena().alloc(lhs),
                        double_colon,
                        constant: selector,
                    })),
                    Either::Right(variable) => Expression::Access(Access::StaticProperty(StaticPropertyAccess {
                        class: stream.arena().alloc(lhs),
                        double_colon,
                        property: variable,
                    })),
                }
            }
        }
        T!["->"] => {
            let arrow = utils::expect_any(stream)?.span;
            let selector = member::parse_classlike_member_selector(stream)?;

            if Precedence::CallDim > precedence && matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["("])) {
                if matches!(
                    (
                        utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind),
                        utils::maybe_peek_nth(stream, 2)?.map(|t| t.kind)
                    ),
                    (Some(T!["..."]), Some(T![")"]))
                ) {
                    Expression::ClosureCreation(ClosureCreation::Method(MethodClosureCreation {
                        object: stream.alloc(lhs),
                        arrow,
                        method: selector,
                        left_parenthesis: utils::expect_any(stream)?.span,
                        ellipsis: utils::expect_any(stream)?.span,
                        right_parenthesis: utils::expect_any(stream)?.span,
                    }))
                } else {
                    Expression::Call(Call::Method(MethodCall {
                        object: stream.alloc(lhs),
                        arrow,
                        method: selector,
                        argument_list: argument::parse_argument_list(stream)?,
                    }))
                }
            } else {
                Expression::Access(Access::Property(PropertyAccess {
                    object: stream.arena().alloc(lhs),
                    arrow,
                    property: selector,
                }))
            }
        }
        T!["?->"] => {
            let question_mark_arrow = utils::expect_any(stream)?.span;
            let selector = member::parse_classlike_member_selector(stream)?;

            if Precedence::CallDim > precedence && matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["("])) {
                Expression::Call(Call::NullSafeMethod(NullSafeMethodCall {
                    object: stream.alloc(lhs),
                    question_mark_arrow,
                    method: selector,
                    argument_list: argument::parse_argument_list(stream)?,
                }))
            } else {
                Expression::Access(Access::NullSafeProperty(NullSafePropertyAccess {
                    object: stream.arena().alloc(lhs),
                    question_mark_arrow,
                    property: selector,
                }))
            }
        }
        T!["++"] => Expression::UnaryPostfix(UnaryPostfix {
            operand: stream.alloc(lhs),
            operator: UnaryPostfixOperator::PostIncrement(utils::expect_any(stream)?.span),
        }),
        T!["--"] => Expression::UnaryPostfix(UnaryPostfix {
            operand: stream.alloc(lhs),
            operator: UnaryPostfixOperator::PostDecrement(utils::expect_any(stream)?.span),
        }),
        _ => unreachable!(),
    })
}

fn parse_infix_expression<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    lhs: Expression<'arena>,
) -> Result<Expression<'arena>, ParseError> {
    let operator = utils::peek(stream)?;

    Ok(match operator.kind {
        T!["??"] => {
            let qq = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::NullCoalesce)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::NullCoalesce(qq),
                rhs: stream.alloc(rhs),
            })
        }
        T!["?"] => {
            if matches!(utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind), Some(T![":"])) {
                Expression::Conditional(Conditional {
                    condition: stream.alloc(lhs),
                    question_mark: utils::expect_any(stream)?.span,
                    then: None,
                    colon: utils::expect_any(stream)?.span,
                    r#else: {
                        let expression = parse_expression(stream)?;

                        stream.alloc(expression)
                    },
                })
            } else {
                Expression::Conditional(Conditional {
                    condition: stream.alloc(lhs),
                    question_mark: utils::expect_any(stream)?.span,
                    then: Some({
                        let expression = parse_expression(stream)?;

                        stream.alloc(expression)
                    }),
                    colon: utils::expect_span(stream, T![":"])?,
                    r#else: {
                        let expression = parse_expression(stream)?;

                        stream.alloc(expression)
                    },
                })
            }
        }
        T!["?:"] => {
            let question_colon = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::ElvisOrConditional)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::Elvis(question_colon),
                rhs: stream.alloc(rhs),
            })
        }
        T!["+"] => Expression::Binary(Binary {
            lhs: stream.alloc(lhs),
            operator: BinaryOperator::Addition(utils::expect_any(stream)?.span),
            rhs: {
                let expression = parse_expression_with_precedence(stream, Precedence::AddSub)?;
                stream.alloc(expression)
            },
        }),
        T!["-"] => Expression::Binary(Binary {
            lhs: stream.alloc(lhs),
            operator: BinaryOperator::Subtraction(utils::expect_any(stream)?.span),
            rhs: {
                let expression = parse_expression_with_precedence(stream, Precedence::AddSub)?;
                stream.alloc(expression)
            },
        }),
        T!["*"] => Expression::Binary(Binary {
            lhs: stream.alloc(lhs),
            operator: BinaryOperator::Multiplication(utils::expect_any(stream)?.span),
            rhs: {
                let expression = parse_expression_with_precedence(stream, Precedence::MulDivMod)?;
                stream.alloc(expression)
            },
        }),
        T!["/"] => Expression::Binary(Binary {
            lhs: stream.alloc(lhs),
            operator: BinaryOperator::Division(utils::expect_any(stream)?.span),
            rhs: {
                let expression = parse_expression_with_precedence(stream, Precedence::MulDivMod)?;
                stream.alloc(expression)
            },
        }),
        T!["%"] => Expression::Binary(Binary {
            lhs: stream.alloc(lhs),
            operator: BinaryOperator::Modulo(utils::expect_any(stream)?.span),
            rhs: {
                let expression = parse_expression_with_precedence(stream, Precedence::MulDivMod)?;
                stream.alloc(expression)
            },
        }),
        T!["**"] => Expression::Binary(Binary {
            lhs: stream.alloc(lhs),
            operator: BinaryOperator::Exponentiation(utils::expect_any(stream)?.span),
            rhs: {
                let expression = parse_expression_with_precedence(stream, Precedence::Pow)?;
                stream.alloc(expression)
            },
        }),
        T!["="] => {
            let operator = AssignmentOperator::Assign(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["+="] => {
            let operator = AssignmentOperator::Addition(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["-="] => {
            let operator = AssignmentOperator::Subtraction(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["*="] => {
            let operator = AssignmentOperator::Multiplication(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["/="] => {
            let operator = AssignmentOperator::Division(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["%="] => {
            let operator = AssignmentOperator::Modulo(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["**="] => {
            let operator = AssignmentOperator::Exponentiation(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["&="] => {
            let operator = AssignmentOperator::BitwiseAnd(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["|="] => {
            let operator = AssignmentOperator::BitwiseOr(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["^="] => {
            let operator = AssignmentOperator::BitwiseXor(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["<<="] => {
            let operator = AssignmentOperator::LeftShift(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T![">>="] => {
            let operator = AssignmentOperator::RightShift(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["??="] => {
            let operator = AssignmentOperator::Coalesce(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T![".="] => {
            let operator = AssignmentOperator::Concat(utils::expect_any(stream)?.span);
            let rhs = parse_expression_with_precedence(stream, Precedence::Assignment)?;

            create_assignment_expression(stream, lhs, operator, rhs)
        }
        T!["&"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::BitwiseAnd)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::BitwiseAnd(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["|"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::BitwiseOr)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::BitwiseOr(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["^"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::BitwiseXor)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::BitwiseXor(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["<<"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::BitShift)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::LeftShift(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T![">>"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::BitShift)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::RightShift(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["=="] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Equality)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::Equal(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["==="] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Equality)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::Identical(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["!="] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Equality)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::NotEqual(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["!=="] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Equality)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::NotIdentical(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["<>"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Equality)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::AngledNotEqual(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["<"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Comparison)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::LessThan(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T![">"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Comparison)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::GreaterThan(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["<="] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Comparison)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::LessThanOrEqual(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T![">="] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Comparison)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::GreaterThanOrEqual(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["<=>"] => {
            let operator = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Equality)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::Spaceship(operator),
                rhs: stream.alloc(rhs),
            })
        }
        T!["&&"] => {
            let and = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::And)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::And(and),
                rhs: stream.alloc(rhs),
            })
        }
        T!["||"] => {
            let or = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Or)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::Or(or),
                rhs: stream.alloc(rhs),
            })
        }
        T!["and"] => {
            let and = utils::expect_any_keyword(stream)?;
            let rhs = parse_expression_with_precedence(stream, Precedence::LowLogicalAnd)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::LowAnd(and),
                rhs: stream.alloc(rhs),
            })
        }
        T!["or"] => {
            let or = utils::expect_any_keyword(stream)?;
            let rhs = parse_expression_with_precedence(stream, Precedence::LowLogicalOr)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::LowOr(or),
                rhs: stream.alloc(rhs),
            })
        }
        T!["xor"] => {
            let xor = utils::expect_any_keyword(stream)?;
            let rhs = parse_expression_with_precedence(stream, Precedence::LowLogicalXor)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::LowXor(xor),
                rhs: stream.alloc(rhs),
            })
        }
        T!["."] => {
            let dot = utils::expect_any(stream)?.span;
            let rhs = parse_expression_with_precedence(stream, Precedence::Concat)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::StringConcat(dot),
                rhs: stream.alloc(rhs),
            })
        }
        T!["instanceof"] => {
            let instanceof = utils::expect_any_keyword(stream)?;
            let rhs = parse_expression_with_precedence(stream, Precedence::Instanceof)?;

            Expression::Binary(Binary {
                lhs: stream.alloc(lhs),
                operator: BinaryOperator::Instanceof(instanceof),
                rhs: stream.alloc(rhs),
            })
        }
        T!["|>"] => {
            let operator = utils::expect_any(stream)?.span;
            let callable = parse_expression_with_precedence(stream, Precedence::Pipe)?;

            Expression::Pipe(Pipe { input: stream.alloc(lhs), operator, callable: stream.alloc(callable) })
        }
        _ => unreachable!(),
    })
}

/// Creates an `Expression` representing an assignment operation while ensuring correct associativity.
///
/// In PHP, assignment operations have right-to-left associativity. This function
/// takes the left-hand side expression (`lhs`), the assignment operator, and the
/// right-hand side expression (`rhs`) and constructs an `Expression` that represents
/// the assignment while applying the correct associativity.
///
/// This ensures that when an assignment is nested within another expression, the assignment
/// is applied to the rightmost operand of the parent expression.
///
/// For example:
///
///  * `($x == $y) = $z` is transformed to `$x == ($y = $z)`
///  * `($x && $y) = $z` is transformed to `$x && ($y = $z)`
///  * `($x + $y) = $z` is transformed to `$x + ($y = $z)`
///  * `((string) $bar) = $foo` is transformed to `(string) ($bar = $foo)`
fn create_assignment_expression<'arena>(
    stream: &TokenStream<'_, 'arena>,
    lhs: Expression<'arena>,
    operator: AssignmentOperator,
    rhs: Expression<'arena>,
) -> Expression<'arena> {
    match lhs {
        Expression::UnaryPrefix(prefix) => Expression::UnaryPrefix(UnaryPrefix {
            operator: prefix.operator,
            operand: stream.alloc(create_assignment_expression(stream, prefix.operand.clone(), operator, rhs)),
        }),
        Expression::Binary(operation) => {
            if operation.operator.is_comparison()
                || operation.operator.is_logical()
                || operation.operator.is_bitwise()
                || operation.operator.is_arithmetic()
            {
                // make `($x == $y) = $z` into `$x == ($y = $z)`
                let Binary { lhs: binary_lhs, operator: binary_operator, rhs: binary_rhs } = operation;

                Expression::Binary(Binary {
                    lhs: binary_lhs,
                    operator: binary_operator,
                    rhs: stream.alloc(create_assignment_expression(stream, binary_rhs.clone(), operator, rhs)),
                })
            } else {
                Expression::Assignment(Assignment {
                    lhs: stream.alloc(Expression::Binary(operation)),
                    operator,
                    rhs: stream.alloc(rhs),
                })
            }
        }
        _ => Expression::Assignment(Assignment { lhs: stream.alloc(lhs), operator, rhs: stream.alloc(rhs) }),
    }
}
