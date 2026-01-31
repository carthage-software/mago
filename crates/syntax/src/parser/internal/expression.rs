use either::Either;

use crate::T;
use crate::ast::ast::Access;
use crate::ast::ast::ArrayAccess;
use crate::ast::ast::ArrayAppend;
use crate::ast::ast::ArrowFunction;
use crate::ast::ast::Assignment;
use crate::ast::ast::AssignmentOperator;
use crate::ast::ast::Binary;
use crate::ast::ast::BinaryOperator;
use crate::ast::ast::Call;
use crate::ast::ast::ClassConstantAccess;
use crate::ast::ast::ClassLikeConstantSelector;
use crate::ast::ast::ClassLikeMemberSelector;
use crate::ast::ast::Closure;
use crate::ast::ast::Conditional;
use crate::ast::ast::ConstantAccess;
use crate::ast::ast::Expression;
use crate::ast::ast::FunctionCall;
use crate::ast::ast::FunctionPartialApplication;
use crate::ast::ast::MethodCall;
use crate::ast::ast::MethodPartialApplication;
use crate::ast::ast::NullSafeMethodCall;
use crate::ast::ast::NullSafePropertyAccess;
use crate::ast::ast::Parenthesized;
use crate::ast::ast::PartialApplication;
use crate::ast::ast::Pipe;
use crate::ast::ast::PropertyAccess;
use crate::ast::ast::StaticMethodCall;
use crate::ast::ast::StaticMethodPartialApplication;
use crate::ast::ast::StaticPropertyAccess;
use crate::ast::ast::UnaryPostfix;
use crate::ast::ast::UnaryPostfixOperator;
use crate::ast::ast::UnaryPrefix;
use crate::ast::ast::UnaryPrefixOperator;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::Associativity;
use crate::token::GetPrecedence;
use crate::token::Precedence;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_expression(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Expression<'arena>, ParseError> {
        self.parse_expression_with_precedence(stream, Precedence::Lowest)
    }

    pub(crate) fn parse_expression_with_precedence(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        precedence: Precedence,
    ) -> Result<Expression<'arena>, ParseError> {
        let mut left = self.parse_lhs_expression(stream, precedence)?;

        while let Some(next) = stream.lookahead(0)? {
            if !self.state.within_indirect_variable
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

                left = self.parse_postfix_expression(stream, left, precedence)?;
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

                left = self.parse_infix_expression(stream, left)?;
            } else {
                break;
            }
        }

        Ok(left)
    }

    #[inline]
    fn parse_lhs_expression(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        precedence: Precedence,
    ) -> Result<Expression<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        let next = stream.lookahead(1)?.map(|t| t.kind);

        let is_call = precedence != Precedence::New && matches!(next, Some(T!["("]));
        let is_call_or_access = is_call
            || matches!(
                next,
                Some(
                    crate::token::TokenKind::LeftBracket
                        | crate::token::TokenKind::ColonColon
                        | crate::token::TokenKind::MinusGreaterThan
                        | crate::token::TokenKind::QuestionMinusGreaterThan
                )
            );

        if token.kind.is_literal() && (!token.kind.is_keyword() || !is_call_or_access) {
            return self.parse_literal(stream).map(Expression::Literal);
        }

        if token.kind.is_unary_prefix() {
            return self.parse_unary_prefix_operation(stream).map(Expression::UnaryPrefix);
        }

        if matches!(token.kind, T!["#["]) {
            return self.parse_arrow_function_or_closure(stream).map(|e| match e {
                Either::Left(arrow_function) => Expression::ArrowFunction(arrow_function),
                Either::Right(closure) => Expression::Closure(closure),
            });
        }

        if matches!(token.kind, T!["clone"]) {
            return self.parse_ambiguous_clone_expression(stream);
        }

        if !self.state.within_string_interpolation
            && (matches!((token.kind, next), (T!["function" | "fn"], _))
                || matches!((token.kind, next), (T!["static"], Some(T!["function" | "fn"]))))
        {
            return self.parse_arrow_function_or_closure(stream).map(|e| match e {
                Either::Left(arrow_function) => Expression::ArrowFunction(arrow_function),
                Either::Right(closure) => Expression::Closure(closure),
            });
        }

        Ok(match (token.kind, next) {
            (T!["static"], _) => Expression::Static(self.expect_any_keyword(stream)?),
            (T!["self"], _) if !is_call => Expression::Self_(self.expect_any_keyword(stream)?),
            (T!["parent"], _) if !is_call => Expression::Parent(self.expect_any_keyword(stream)?),
            (kind, _) if kind.is_construct() => Expression::Construct(self.parse_construct(stream)?),
            (T!["list"], Some(T!["("])) => Expression::List(self.parse_list(stream)?),
            (T!["new"], Some(T!["class" | "#["])) => Expression::AnonymousClass(self.parse_anonymous_class(stream)?),
            (T!["new"], Some(T!["static"])) => Expression::Instantiation(self.parse_instantiation(stream)?),
            (T!["new"], Some(kind)) if kind.is_modifier() => {
                Expression::AnonymousClass(self.parse_anonymous_class(stream)?)
            }
            (T!["new"], _) => Expression::Instantiation(self.parse_instantiation(stream)?),
            (T!["throw"], _) => Expression::Throw(self.parse_throw(stream)?),
            (T!["yield"], _) => Expression::Yield(self.parse_yield(stream)?),
            (T!["\""] | T!["<<<"] | T!["`"], ..) => Expression::CompositeString(self.parse_string(stream)?),
            (T!["("], _) => Expression::Parenthesized(Parenthesized {
                left_parenthesis: stream.eat(T!["("])?.span,
                expression: self.arena.alloc(self.parse_expression(stream)?),
                right_parenthesis: stream.eat(T![")"])?.span,
            }),
            (T!["match"], Some(T!["("])) => Expression::Match(self.parse_match(stream)?),
            (T!["array"], Some(T!["("])) => Expression::LegacyArray(self.parse_legacy_array(stream)?),
            (T!["["], _) => Expression::Array(self.parse_array(stream)?),
            (
                crate::token::TokenKind::Dollar
                | crate::token::TokenKind::DollarLeftBrace
                | crate::token::TokenKind::Variable,
                _,
            ) => self.parse_variable(stream).map(Expression::Variable)?,
            (kind, _) if kind.is_magic_constant() => Expression::MagicConstant(self.parse_magic_constant(stream)?),
            (kind, ..)
                if matches!(kind, T![Identifier | QualifiedIdentifier | FullyQualifiedIdentifier | "clone"])
                    || kind.is_soft_reserved_identifier()
                    || (self.state.within_string_interpolation && kind.is_reserved_identifier()) =>
            {
                Expression::Identifier(self.parse_identifier(stream)?)
            }
            _ => return Err(stream.unexpected(Some(token), &[])),
        })
    }

    fn parse_arrow_function_or_closure(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Either<ArrowFunction<'arena>, Closure<'arena>>, ParseError> {
        let attributes = self.parse_attribute_list_sequence(stream)?;

        let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        let after = stream.lookahead(1)?;

        Ok(match (next.kind, after.map(|t| t.kind)) {
            (T!["function"], _) | (T!["static"], Some(T!["function"])) => {
                Either::Right(self.parse_closure_with_attributes(stream, attributes)?)
            }
            (T!["fn"], _) | (T!["static"], Some(T!["fn"])) => {
                Either::Left(self.parse_arrow_function_with_attributes(stream, attributes)?)
            }
            _ => return Err(stream.unexpected(Some(next), &[T!["function"], T!["fn"], T!["static"]])),
        })
    }

    fn parse_postfix_expression(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        lhs: Expression<'arena>,
        precedence: Precedence,
    ) -> Result<Expression<'arena>, ParseError> {
        let operator = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match operator.kind {
            T!["("] => {
                let partial_args = self.parse_partial_argument_list(stream)?;

                if partial_args.has_placeholders() {
                    Expression::PartialApplication(PartialApplication::Function(FunctionPartialApplication {
                        function: self.arena.alloc(lhs),
                        argument_list: partial_args,
                    }))
                } else {
                    Expression::Call(Call::Function(FunctionCall {
                        function: self.arena.alloc(lhs),
                        argument_list: partial_args.into_argument_list(self.arena),
                    }))
                }
            }
            T!["["] => {
                let left_bracket = stream.consume()?.span;
                let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
                if matches!(next.kind, T!["]"]) {
                    Expression::ArrayAppend(ArrayAppend {
                        array: self.arena.alloc(lhs),
                        left_bracket,
                        right_bracket: stream.consume()?.span,
                    })
                } else {
                    Expression::ArrayAccess(ArrayAccess {
                        array: self.arena.alloc(lhs),
                        left_bracket,
                        index: self.arena.alloc(self.parse_expression(stream)?),
                        right_bracket: stream.eat(T!["]"])?.span,
                    })
                }
            }
            T!["::"] => {
                let double_colon = stream.consume()?.span;
                let selector_or_variable = self.parse_classlike_constant_selector_or_variable(stream)?;
                let current = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

                if Precedence::CallDim > precedence && matches!(current.kind, T!["("]) {
                    let method = match selector_or_variable {
                        Either::Left(selector) => match selector {
                            ClassLikeConstantSelector::Identifier(i) => ClassLikeMemberSelector::Identifier(i),
                            ClassLikeConstantSelector::Expression(c) => ClassLikeMemberSelector::Expression(c),
                        },
                        Either::Right(variable) => ClassLikeMemberSelector::Variable(variable),
                    };

                    let partial_args = self.parse_partial_argument_list(stream)?;

                    if partial_args.has_placeholders() {
                        Expression::PartialApplication(PartialApplication::StaticMethod(
                            StaticMethodPartialApplication {
                                class: self.arena.alloc(lhs),
                                double_colon,
                                method,
                                argument_list: partial_args,
                            },
                        ))
                    } else {
                        Expression::Call(Call::StaticMethod(StaticMethodCall {
                            class: self.arena.alloc(lhs),
                            double_colon,
                            method,
                            argument_list: partial_args.into_argument_list(self.arena),
                        }))
                    }
                } else {
                    match selector_or_variable {
                        Either::Left(selector) => Expression::Access(Access::ClassConstant(ClassConstantAccess {
                            class: self.arena.alloc(lhs),
                            double_colon,
                            constant: selector,
                        })),
                        Either::Right(variable) => Expression::Access(Access::StaticProperty(StaticPropertyAccess {
                            class: self.arena.alloc(lhs),
                            double_colon,
                            property: variable,
                        })),
                    }
                }
            }
            T!["->"] => {
                let arrow = stream.consume()?.span;
                let selector = self.parse_classlike_member_selector(stream)?;

                if Precedence::CallDim > precedence && matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["("])) {
                    let partial_args = self.parse_partial_argument_list(stream)?;

                    if partial_args.has_placeholders() {
                        Expression::PartialApplication(PartialApplication::Method(MethodPartialApplication {
                            object: self.arena.alloc(lhs),
                            arrow,
                            method: selector,
                            argument_list: partial_args,
                        }))
                    } else {
                        Expression::Call(Call::Method(MethodCall {
                            object: self.arena.alloc(lhs),
                            arrow,
                            method: selector,
                            argument_list: partial_args.into_argument_list(self.arena),
                        }))
                    }
                } else {
                    Expression::Access(Access::Property(PropertyAccess {
                        object: self.arena.alloc(lhs),
                        arrow,
                        property: selector,
                    }))
                }
            }
            T!["?->"] => {
                let question_mark_arrow = stream.consume()?.span;
                let selector = self.parse_classlike_member_selector(stream)?;

                if Precedence::CallDim > precedence && matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["("])) {
                    Expression::Call(Call::NullSafeMethod(NullSafeMethodCall {
                        object: self.arena.alloc(lhs),
                        question_mark_arrow,
                        method: selector,
                        argument_list: self.parse_argument_list(stream)?,
                    }))
                } else {
                    Expression::Access(Access::NullSafeProperty(NullSafePropertyAccess {
                        object: self.arena.alloc(lhs),
                        question_mark_arrow,
                        property: selector,
                    }))
                }
            }
            T!["++"] => Expression::UnaryPostfix(UnaryPostfix {
                operand: self.arena.alloc(lhs),
                operator: UnaryPostfixOperator::PostIncrement(stream.consume()?.span),
            }),
            T!["--"] => Expression::UnaryPostfix(UnaryPostfix {
                operand: self.arena.alloc(lhs),
                operator: UnaryPostfixOperator::PostDecrement(stream.consume()?.span),
            }),
            _ => unreachable!(),
        })
    }

    fn parse_infix_expression(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        lhs: Expression<'arena>,
    ) -> Result<Expression<'arena>, ParseError> {
        let operator = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match operator.kind {
            T!["??"] => {
                let qq = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::NullCoalesce)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::NullCoalesce(qq),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["?"] => {
                if matches!(stream.lookahead(1)?.map(|t| t.kind), Some(T![":"])) {
                    Expression::Conditional(Conditional {
                        condition: self.arena.alloc(lhs),
                        question_mark: stream.consume()?.span,
                        then: None,
                        colon: stream.consume()?.span,
                        r#else: self
                            .arena
                            .alloc(self.parse_expression_with_precedence(stream, Precedence::ElvisOrConditional)?),
                    })
                } else {
                    Expression::Conditional(Conditional {
                        condition: self.arena.alloc(lhs),
                        question_mark: stream.consume()?.span,
                        then: Some(self.arena.alloc(self.parse_expression(stream)?)),
                        colon: stream.eat(T![":"])?.span,
                        r#else: self
                            .arena
                            .alloc(self.parse_expression_with_precedence(stream, Precedence::ElvisOrConditional)?),
                    })
                }
            }
            T!["+"] => Expression::Binary(Binary {
                lhs: self.arena.alloc(lhs),
                operator: BinaryOperator::Addition(stream.consume()?.span),
                rhs: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::AddSub)?),
            }),
            T!["-"] => Expression::Binary(Binary {
                lhs: self.arena.alloc(lhs),
                operator: BinaryOperator::Subtraction(stream.consume()?.span),
                rhs: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::AddSub)?),
            }),
            T!["*"] => Expression::Binary(Binary {
                lhs: self.arena.alloc(lhs),
                operator: BinaryOperator::Multiplication(stream.consume()?.span),
                rhs: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::MulDivMod)?),
            }),
            T!["/"] => Expression::Binary(Binary {
                lhs: self.arena.alloc(lhs),
                operator: BinaryOperator::Division(stream.consume()?.span),
                rhs: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::MulDivMod)?),
            }),
            T!["%"] => Expression::Binary(Binary {
                lhs: self.arena.alloc(lhs),
                operator: BinaryOperator::Modulo(stream.consume()?.span),
                rhs: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::MulDivMod)?),
            }),
            T!["**"] => Expression::Binary(Binary {
                lhs: self.arena.alloc(lhs),
                operator: BinaryOperator::Exponentiation(stream.consume()?.span),
                rhs: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::Pow)?),
            }),
            T!["="] => {
                let operator = AssignmentOperator::Assign(stream.consume()?.span);

                let by_ref = matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["&"]));

                let rhs = if by_ref {
                    let ampersand = stream.eat(T!["&"])?;
                    let referenced_expr = self.parse_expression_with_precedence(stream, Precedence::Reference)?;

                    Expression::UnaryPrefix(UnaryPrefix {
                        operator: UnaryPrefixOperator::Reference(ampersand.span),
                        operand: self.arena.alloc(referenced_expr),
                    })
                } else {
                    self.parse_expression_with_precedence(stream, Precedence::Assignment)?
                };

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["+="] => {
                let operator = AssignmentOperator::Addition(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["-="] => {
                let operator = AssignmentOperator::Subtraction(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["*="] => {
                let operator = AssignmentOperator::Multiplication(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["/="] => {
                let operator = AssignmentOperator::Division(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["%="] => {
                let operator = AssignmentOperator::Modulo(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["**="] => {
                let operator = AssignmentOperator::Exponentiation(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["&="] => {
                let operator = AssignmentOperator::BitwiseAnd(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["|="] => {
                let operator = AssignmentOperator::BitwiseOr(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["^="] => {
                let operator = AssignmentOperator::BitwiseXor(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["<<="] => {
                let operator = AssignmentOperator::LeftShift(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T![">>="] => {
                let operator = AssignmentOperator::RightShift(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["??="] => {
                let operator = AssignmentOperator::Coalesce(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T![".="] => {
                let operator = AssignmentOperator::Concat(stream.consume()?.span);
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Assignment)?;

                self.create_assignment_expression(lhs, operator, rhs)
            }
            T!["&"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::BitwiseAnd)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::BitwiseAnd(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["|"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::BitwiseOr)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::BitwiseOr(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["^"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::BitwiseXor)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::BitwiseXor(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["<<"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::BitShift)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::LeftShift(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T![">>"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::BitShift)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::RightShift(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["=="] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Equality)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::Equal(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["==="] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Equality)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::Identical(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["!="] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Equality)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::NotEqual(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["!=="] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Equality)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::NotIdentical(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["<>"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Equality)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::AngledNotEqual(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["<"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Comparison)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::LessThan(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T![">"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Comparison)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::GreaterThan(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["<="] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Comparison)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::LessThanOrEqual(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T![">="] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Comparison)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::GreaterThanOrEqual(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["<=>"] => {
                let operator = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Equality)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::Spaceship(operator),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["&&"] => {
                let and = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::And)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::And(and),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["||"] => {
                let or = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Or)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::Or(or),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["and"] => {
                let and = self.expect_any_keyword(stream)?;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::KeyAnd)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::LowAnd(and),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["or"] => {
                let or = self.expect_any_keyword(stream)?;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::KeyOr)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::LowOr(or),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["xor"] => {
                let xor = self.expect_any_keyword(stream)?;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::KeyXor)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::LowXor(xor),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["."] => {
                let dot = stream.consume()?.span;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Concat)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::StringConcat(dot),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["instanceof"] => {
                let instanceof = self.expect_any_keyword(stream)?;
                let rhs = self.parse_expression_with_precedence(stream, Precedence::Instanceof)?;

                Expression::Binary(Binary {
                    lhs: self.arena.alloc(lhs),
                    operator: BinaryOperator::Instanceof(instanceof),
                    rhs: self.arena.alloc(rhs),
                })
            }
            T!["|>"] => {
                let operator = stream.consume()?.span;
                let callable = self.parse_expression_with_precedence(stream, Precedence::Pipe)?;

                Expression::Pipe(Pipe { input: self.arena.alloc(lhs), operator, callable: self.arena.alloc(callable) })
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
    fn create_assignment_expression(
        &mut self,
        lhs: Expression<'arena>,
        operator: AssignmentOperator,
        rhs: Expression<'arena>,
    ) -> Expression<'arena> {
        match lhs {
            Expression::UnaryPrefix(prefix) => {
                if !prefix.operator.is_increment_or_decrement() && Precedence::Assignment < prefix.operator.precedence()
                {
                    // make `(--$x) = $y` into `--($x = $y)`
                    let UnaryPrefix { operator: prefix_operator, operand } = prefix;

                    Expression::UnaryPrefix(UnaryPrefix {
                        operator: prefix_operator,
                        operand: self.arena.alloc(self.create_assignment_expression(operand.clone(), operator, rhs)),
                    })
                } else {
                    Expression::Assignment(Assignment {
                        lhs: self.arena.alloc(Expression::UnaryPrefix(prefix)),
                        operator,
                        rhs: self.arena.alloc(rhs),
                    })
                }
            }
            Expression::Binary(operation) => {
                let assignment_precedence = Precedence::Assignment;
                let binary_precedence = operation.operator.precedence();

                if assignment_precedence < binary_precedence {
                    // make `($x == $y) = $z` into `$x == ($y = $z)`
                    let Binary { lhs: binary_lhs, operator: binary_operator, rhs: binary_rhs } = operation;

                    Expression::Binary(Binary {
                        lhs: binary_lhs,
                        operator: binary_operator,
                        rhs: self.arena.alloc(self.create_assignment_expression(binary_rhs.clone(), operator, rhs)),
                    })
                } else {
                    Expression::Assignment(Assignment {
                        lhs: self.arena.alloc(Expression::Binary(operation)),
                        operator,
                        rhs: self.arena.alloc(rhs),
                    })
                }
            }
            Expression::Conditional(conditional) => {
                let Conditional { condition, question_mark, then, colon, r#else } = conditional;

                Expression::Conditional(Conditional {
                    condition,
                    question_mark,
                    then,
                    colon,
                    r#else: self.arena.alloc(self.create_assignment_expression(r#else.clone(), operator, rhs)),
                })
            }
            _ => {
                Expression::Assignment(Assignment { lhs: self.arena.alloc(lhs), operator, rhs: self.arena.alloc(rhs) })
            }
        }
    }
}
