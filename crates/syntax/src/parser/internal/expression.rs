use either::Either;
use mago_database::file::HasFileId;
use mago_span::HasSpan;
use mago_span::Span;

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
use crate::parser::MAX_RECURSION_DEPTH;
use crate::parser::Parser;
use crate::token::Associativity;
use crate::token::GetPrecedence;
use crate::token::Precedence;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_expression(&mut self) -> Result<&'arena Expression<'arena>, ParseError> {
        self.parse_expression_with_precedence(Precedence::Lowest)
    }

    /// Internal expression parsing that uses arena-allocated references to reduce stack usage.
    /// Returns `&'arena Expression<'arena>` (8 bytes) instead of `Expression<'arena>` (488 bytes).
    pub(crate) fn parse_expression_with_precedence(
        &mut self,
        precedence: Precedence,
    ) -> Result<&'arena Expression<'arena>, ParseError> {
        self.state.recursion_depth += 1;
        if self.state.recursion_depth > MAX_RECURSION_DEPTH {
            self.state.recursion_depth -= 1;
            let file_id = self.stream.file_id();
            let span =
                self.stream.lookahead(0)?.map(|t| t.span_for(file_id)).unwrap_or_else(|| {
                    Span::new(file_id, self.stream.current_position(), self.stream.current_position())
                });

            return Err(ParseError::RecursionLimitExceeded(span));
        }

        let result = self.parse_expression_with_precedence_inner(precedence);
        self.state.recursion_depth -= 1;
        result
    }

    fn parse_expression_with_precedence_inner(
        &mut self,
        precedence: Precedence,
    ) -> Result<&'arena Expression<'arena>, ParseError> {
        let mut left = self.parse_lhs_expression(precedence)?;

        while let Some(next) = self.stream.lookahead(0)? {
            if !self.state.within_indirect_variable
                && !matches!(precedence, Precedence::Instanceof | Precedence::New)
                && !matches!(next.kind, T!["(" | "::"])
                && let Expression::Identifier(identifier) = left
            {
                left = self.arena.alloc(Expression::ConstantAccess(ConstantAccess { name: *identifier }));
            }

            // Stop parsing if the next token is a terminator.
            if matches!(next.kind, T![";" | "?>"]) {
                break;
            }

            // Don't allow function calls on error expressions.
            // This prevents `if(...)` from being parsed as a function call when `if` is an unexpected token.
            if matches!(left, Expression::Error(_)) && matches!(next.kind, T!["("]) {
                break;
            }

            if next.kind.is_postfix() {
                let postfix_precedence = Precedence::postfix(&next.kind);
                if postfix_precedence < precedence {
                    break;
                }

                left = self.parse_postfix_expression(left, precedence)?;
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

                left = self.parse_infix_expression(left)?;
            } else {
                break;
            }
        }

        Ok(left)
    }

    #[inline]
    fn parse_lhs_expression(&mut self, precedence: Precedence) -> Result<&'arena Expression<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        let next = self.stream.peek_kind(1)?;

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
            return Ok(self.arena.alloc(Expression::Literal(self.parse_literal()?)));
        }

        if token.kind.is_unary_prefix() {
            return Ok(self.arena.alloc(Expression::UnaryPrefix(self.parse_unary_prefix_operation()?)));
        }

        if matches!(token.kind, T!["#["]) {
            return Ok(self.arena.alloc(match self.parse_arrow_function_or_closure()? {
                Either::Left(arrow_function) => Expression::ArrowFunction(arrow_function),
                Either::Right(closure) => Expression::Closure(closure),
            }));
        }

        if matches!(token.kind, T!["clone"]) {
            return Ok(self.arena.alloc(self.parse_ambiguous_clone_expression()?));
        }

        if !self.state.within_string_interpolation
            && (matches!((token.kind, next), (T!["function" | "fn"], _))
                || matches!((token.kind, next), (T!["static"], Some(T!["function" | "fn"]))))
        {
            return Ok(self.arena.alloc(match self.parse_arrow_function_or_closure()? {
                Either::Left(arrow_function) => Expression::ArrowFunction(arrow_function),
                Either::Right(closure) => Expression::Closure(closure),
            }));
        }

        Ok(self.arena.alloc(match (token.kind, next) {
            (T!["static"], _) => Expression::Static(self.expect_any_keyword()?),
            (T!["self"], _) if !is_call => Expression::Self_(self.expect_any_keyword()?),
            (T!["parent"], _) if !is_call => Expression::Parent(self.expect_any_keyword()?),
            (kind, _) if kind.is_construct() => Expression::Construct(self.parse_construct()?),
            (T!["list"], Some(T!["("])) => Expression::List(self.parse_list()?),
            (T!["new"], Some(T!["class" | "#["])) => Expression::AnonymousClass(self.parse_anonymous_class()?),
            (T!["new"], Some(T!["static"])) => Expression::Instantiation(self.parse_instantiation()?),
            (T!["new"], Some(kind)) if kind.is_modifier() => Expression::AnonymousClass(self.parse_anonymous_class()?),
            (T!["new"], _) => Expression::Instantiation(self.parse_instantiation()?),
            (T!["throw"], _) => Expression::Throw(self.parse_throw()?),
            (T!["yield"], _) => Expression::Yield(self.parse_yield()?),
            (T!["\""] | T!["<<<"] | T!["`"], ..) => Expression::CompositeString(self.parse_string()?),
            (T!["("], _) => Expression::Parenthesized(Parenthesized {
                left_parenthesis: self.stream.eat_span(T!["("])?,
                expression: self.parse_expression_with_precedence(Precedence::Lowest)?,
                right_parenthesis: self.stream.eat_span(T![")"])?,
            }),
            (T!["match"], Some(T!["("])) => Expression::Match(self.parse_match()?),
            (T!["array"], Some(T!["("])) => Expression::LegacyArray(self.parse_legacy_array()?),
            (T!["["], _) => Expression::Array(self.parse_array()?),
            (
                crate::token::TokenKind::Dollar
                | crate::token::TokenKind::DollarLeftBrace
                | crate::token::TokenKind::Variable,
                _,
            ) => Expression::Variable(self.parse_variable()?),
            (kind, _) if kind.is_magic_constant() => Expression::MagicConstant(self.parse_magic_constant()?),
            (kind, ..)
                if matches!(kind, T![Identifier | QualifiedIdentifier | FullyQualifiedIdentifier | "clone"])
                    || kind.is_soft_reserved_identifier()
                    || (self.state.within_string_interpolation && kind.is_reserved_identifier()) =>
            {
                Expression::Identifier(self.parse_identifier()?)
            }
            _ => {
                // Check if this is a token that we should NOT consume.
                // 1. Statement-starting keywords should always be left for the statement parser
                // 2. Closing delimiters (`)`, `]`, `}`, `,`) should be preserved ONLY when we're
                //    inside a sub-expression (precedence > Lowest), as they might be part of an
                //    outer context. At the top level (Lowest precedence), consume them to avoid
                //    infinite loops.
                let is_statement_keyword = matches!(
                    token.kind,
                    T!["if"
                        | "else"
                        | "elseif"
                        | "while"
                        | "do"
                        | "for"
                        | "foreach"
                        | "switch"
                        | "try"
                        | "catch"
                        | "finally"
                        | "class"
                        | "interface"
                        | "trait"
                        | "enum"
                        | "function"
                        | "fn"
                        | "return"
                        | "break"
                        | "continue"
                        | "goto"
                        | "declare"
                        | "namespace"
                        | "use"
                        | "const"
                        | "global"
                        | "abstract"
                        | "final"]
                );
                let is_closing_delimiter = matches!(token.kind, T![")" | "]" | "}" | ","]);
                let should_preserve =
                    is_statement_keyword || (is_closing_delimiter && precedence != Precedence::Lowest);

                if should_preserve {
                    let pos = self.stream.current_position();
                    let span = Span::new(self.stream.file_id(), pos, pos);
                    let err = self.stream.unexpected(Some(token), &[]);
                    self.errors.push(err);
                    Expression::Error(span)
                } else {
                    // Consume the unexpected token to prevent infinite loops
                    let consumed = self.stream.consume()?;
                    let err = self.stream.unexpected(Some(consumed), &[]);
                    let span = err.span();
                    self.errors.push(err);
                    Expression::Error(span)
                }
            }
        }))
    }

    fn parse_arrow_function_or_closure(
        &mut self,
    ) -> Result<Either<ArrowFunction<'arena>, Closure<'arena>>, ParseError> {
        let attributes = self.parse_attribute_list_sequence()?;

        let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        let after = self.stream.lookahead(1)?;

        Ok(match (next.kind, after.map(|t| t.kind)) {
            (T!["function"], _) | (T!["static"], Some(T!["function"])) => {
                Either::Right(self.parse_closure_with_attributes(attributes)?)
            }
            (T!["fn"], _) | (T!["static"], Some(T!["fn"])) => {
                Either::Left(self.parse_arrow_function_with_attributes(attributes)?)
            }
            _ => return Err(self.stream.unexpected(Some(next), &[T!["function"], T!["fn"], T!["static"]])),
        })
    }

    fn parse_postfix_expression(
        &mut self,
        lhs: &'arena Expression<'arena>,
        precedence: Precedence,
    ) -> Result<&'arena Expression<'arena>, ParseError> {
        let operator = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(self.arena.alloc(match operator.kind {
            T!["("] => {
                let partial_args = self.parse_partial_argument_list()?;

                if partial_args.has_placeholders() {
                    Expression::PartialApplication(PartialApplication::Function(FunctionPartialApplication {
                        function: lhs,
                        argument_list: partial_args,
                    }))
                } else {
                    Expression::Call(Call::Function(FunctionCall {
                        function: lhs,
                        argument_list: partial_args.into_argument_list(self.arena),
                    }))
                }
            }
            T!["["] => {
                let left_bracket = self.stream.consume_span()?;
                let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
                if matches!(next.kind, T!["]"]) {
                    Expression::ArrayAppend(ArrayAppend {
                        array: lhs,
                        left_bracket,
                        right_bracket: self.stream.consume_span()?,
                    })
                } else {
                    Expression::ArrayAccess(ArrayAccess {
                        array: lhs,
                        left_bracket,
                        index: self.parse_expression_with_precedence(Precedence::Lowest)?,
                        right_bracket: self.stream.eat_span(T!["]"])?,
                    })
                }
            }
            T!["::"] => {
                let double_colon = self.stream.consume_span()?;
                let selector_or_variable = self.parse_classlike_constant_selector_or_variable()?;
                let current = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

                if Precedence::CallDim > precedence && matches!(current.kind, T!["("]) {
                    let method = match selector_or_variable {
                        Either::Left(selector) => match selector {
                            ClassLikeConstantSelector::Identifier(i) => ClassLikeMemberSelector::Identifier(i),
                            ClassLikeConstantSelector::Expression(c) => ClassLikeMemberSelector::Expression(c),
                            ClassLikeConstantSelector::Missing(span) => ClassLikeMemberSelector::Missing(span),
                        },
                        Either::Right(variable) => ClassLikeMemberSelector::Variable(variable),
                    };

                    let partial_args = self.parse_partial_argument_list()?;

                    if partial_args.has_placeholders() {
                        Expression::PartialApplication(PartialApplication::StaticMethod(
                            StaticMethodPartialApplication {
                                class: lhs,
                                double_colon,
                                method,
                                argument_list: partial_args,
                            },
                        ))
                    } else {
                        Expression::Call(Call::StaticMethod(StaticMethodCall {
                            class: lhs,
                            double_colon,
                            method,
                            argument_list: partial_args.into_argument_list(self.arena),
                        }))
                    }
                } else {
                    match selector_or_variable {
                        Either::Left(selector) => Expression::Access(Access::ClassConstant(ClassConstantAccess {
                            class: lhs,
                            double_colon,
                            constant: selector,
                        })),
                        Either::Right(variable) => Expression::Access(Access::StaticProperty(StaticPropertyAccess {
                            class: lhs,
                            double_colon,
                            property: variable,
                        })),
                    }
                }
            }
            T!["->"] => {
                let arrow = self.stream.consume_span()?;
                let selector = self.parse_classlike_member_selector()?;

                if Precedence::CallDim > precedence && matches!(self.stream.peek_kind(0)?, Some(T!["("])) {
                    let partial_args = self.parse_partial_argument_list()?;

                    if partial_args.has_placeholders() {
                        Expression::PartialApplication(PartialApplication::Method(MethodPartialApplication {
                            object: lhs,
                            arrow,
                            method: selector,
                            argument_list: partial_args,
                        }))
                    } else {
                        Expression::Call(Call::Method(MethodCall {
                            object: lhs,
                            arrow,
                            method: selector,
                            argument_list: partial_args.into_argument_list(self.arena),
                        }))
                    }
                } else {
                    Expression::Access(Access::Property(PropertyAccess { object: lhs, arrow, property: selector }))
                }
            }
            T!["?->"] => {
                let question_mark_arrow = self.stream.consume_span()?;
                let selector = self.parse_classlike_member_selector()?;

                if Precedence::CallDim > precedence && matches!(self.stream.peek_kind(0)?, Some(T!["("])) {
                    Expression::Call(Call::NullSafeMethod(NullSafeMethodCall {
                        object: lhs,
                        question_mark_arrow,
                        method: selector,
                        argument_list: self.parse_argument_list()?,
                    }))
                } else {
                    Expression::Access(Access::NullSafeProperty(NullSafePropertyAccess {
                        object: lhs,
                        question_mark_arrow,
                        property: selector,
                    }))
                }
            }
            T!["++"] => Expression::UnaryPostfix(UnaryPostfix {
                operand: lhs,
                operator: UnaryPostfixOperator::PostIncrement(self.stream.consume_span()?),
            }),
            T!["--"] => Expression::UnaryPostfix(UnaryPostfix {
                operand: lhs,
                operator: UnaryPostfixOperator::PostDecrement(self.stream.consume_span()?),
            }),
            _ => unreachable!(),
        }))
    }

    fn parse_infix_expression(
        &mut self,
        lhs: &'arena Expression<'arena>,
    ) -> Result<&'arena Expression<'arena>, ParseError> {
        let operator = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(self.arena.alloc(match operator.kind {
            T!["??"] => {
                let qq = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::NullCoalesce)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::NullCoalesce(qq), rhs })
            }
            T!["?"] => {
                if matches!(self.stream.peek_kind(1)?, Some(T![":"])) {
                    Expression::Conditional(Conditional {
                        condition: lhs,
                        question_mark: self.stream.consume_span()?,
                        then: None,
                        colon: self.stream.consume_span()?,
                        r#else: self.parse_expression_with_precedence(Precedence::ElvisOrConditional)?,
                    })
                } else {
                    Expression::Conditional(Conditional {
                        condition: lhs,
                        question_mark: self.stream.consume_span()?,
                        then: Some(self.parse_expression_with_precedence(Precedence::Lowest)?),
                        colon: self.stream.eat_span(T![":"])?,
                        r#else: self.parse_expression_with_precedence(Precedence::ElvisOrConditional)?,
                    })
                }
            }
            T!["+"] => Expression::Binary(Binary {
                lhs,
                operator: BinaryOperator::Addition(self.stream.consume_span()?),
                rhs: self.parse_expression_with_precedence(Precedence::AddSub)?,
            }),
            T!["-"] => Expression::Binary(Binary {
                lhs,
                operator: BinaryOperator::Subtraction(self.stream.consume_span()?),
                rhs: self.parse_expression_with_precedence(Precedence::AddSub)?,
            }),
            T!["*"] => Expression::Binary(Binary {
                lhs,
                operator: BinaryOperator::Multiplication(self.stream.consume_span()?),
                rhs: self.parse_expression_with_precedence(Precedence::MulDivMod)?,
            }),
            T!["/"] => Expression::Binary(Binary {
                lhs,
                operator: BinaryOperator::Division(self.stream.consume_span()?),
                rhs: self.parse_expression_with_precedence(Precedence::MulDivMod)?,
            }),
            T!["%"] => Expression::Binary(Binary {
                lhs,
                operator: BinaryOperator::Modulo(self.stream.consume_span()?),
                rhs: self.parse_expression_with_precedence(Precedence::MulDivMod)?,
            }),
            T!["**"] => Expression::Binary(Binary {
                lhs,
                operator: BinaryOperator::Exponentiation(self.stream.consume_span()?),
                rhs: self.parse_expression_with_precedence(Precedence::Pow)?,
            }),
            T!["="] => {
                let operator = AssignmentOperator::Assign(self.stream.consume_span()?);

                let by_ref = matches!(self.stream.peek_kind(0)?, Some(T!["&"]));

                let rhs = if by_ref {
                    let ampersand_span = self.stream.eat_span(T!["&"])?;
                    let referenced_expr = self.parse_expression_with_precedence(Precedence::Reference)?;

                    self.arena.alloc(Expression::UnaryPrefix(UnaryPrefix {
                        operator: UnaryPrefixOperator::Reference(ampersand_span),
                        operand: referenced_expr,
                    }))
                } else {
                    self.parse_expression_with_precedence(Precedence::Assignment)?
                };

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["+="] => {
                let operator = AssignmentOperator::Addition(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["-="] => {
                let operator = AssignmentOperator::Subtraction(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["*="] => {
                let operator = AssignmentOperator::Multiplication(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["/="] => {
                let operator = AssignmentOperator::Division(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["%="] => {
                let operator = AssignmentOperator::Modulo(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["**="] => {
                let operator = AssignmentOperator::Exponentiation(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["&="] => {
                let operator = AssignmentOperator::BitwiseAnd(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["|="] => {
                let operator = AssignmentOperator::BitwiseOr(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["^="] => {
                let operator = AssignmentOperator::BitwiseXor(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["<<="] => {
                let operator = AssignmentOperator::LeftShift(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T![">>="] => {
                let operator = AssignmentOperator::RightShift(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["??="] => {
                let operator = AssignmentOperator::Coalesce(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T![".="] => {
                let operator = AssignmentOperator::Concat(self.stream.consume_span()?);
                let rhs = self.parse_expression_with_precedence(Precedence::Assignment)?;

                return Ok(self.create_assignment_expression(lhs, operator, rhs));
            }
            T!["&"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::BitwiseAnd)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::BitwiseAnd(operator), rhs })
            }
            T!["|"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::BitwiseOr)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::BitwiseOr(operator), rhs })
            }
            T!["^"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::BitwiseXor)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::BitwiseXor(operator), rhs })
            }
            T!["<<"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::BitShift)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::LeftShift(operator), rhs })
            }
            T![">>"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::BitShift)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::RightShift(operator), rhs })
            }
            T!["=="] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Equality)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::Equal(operator), rhs })
            }
            T!["==="] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Equality)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::Identical(operator), rhs })
            }
            T!["!="] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Equality)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::NotEqual(operator), rhs })
            }
            T!["!=="] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Equality)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::NotIdentical(operator), rhs })
            }
            T!["<>"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Equality)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::AngledNotEqual(operator), rhs })
            }
            T!["<"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Comparison)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::LessThan(operator), rhs })
            }
            T![">"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Comparison)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::GreaterThan(operator), rhs })
            }
            T!["<="] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Comparison)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::LessThanOrEqual(operator), rhs })
            }
            T![">="] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Comparison)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::GreaterThanOrEqual(operator), rhs })
            }
            T!["<=>"] => {
                let operator = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Equality)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::Spaceship(operator), rhs })
            }
            T!["&&"] => {
                let and = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::And)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::And(and), rhs })
            }
            T!["||"] => {
                let or = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Or)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::Or(or), rhs })
            }
            T!["and"] => {
                let and = self.expect_any_keyword()?;
                let rhs = self.parse_expression_with_precedence(Precedence::KeyAnd)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::LowAnd(and), rhs })
            }
            T!["or"] => {
                let or = self.expect_any_keyword()?;
                let rhs = self.parse_expression_with_precedence(Precedence::KeyOr)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::LowOr(or), rhs })
            }
            T!["xor"] => {
                let xor = self.expect_any_keyword()?;
                let rhs = self.parse_expression_with_precedence(Precedence::KeyXor)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::LowXor(xor), rhs })
            }
            T!["."] => {
                let dot = self.stream.consume_span()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Concat)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::StringConcat(dot), rhs })
            }
            T!["instanceof"] => {
                let instanceof = self.expect_any_keyword()?;
                let rhs = self.parse_expression_with_precedence(Precedence::Instanceof)?;

                Expression::Binary(Binary { lhs, operator: BinaryOperator::Instanceof(instanceof), rhs })
            }
            T!["|>"] => {
                let operator = self.stream.consume_span()?;
                let callable = self.parse_expression_with_precedence(Precedence::Pipe)?;

                Expression::Pipe(Pipe { input: lhs, operator, callable })
            }
            _ => unreachable!(),
        }))
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
        lhs: &'arena Expression<'arena>,
        operator: AssignmentOperator,
        rhs: &'arena Expression<'arena>,
    ) -> &'arena Expression<'arena> {
        match lhs {
            Expression::UnaryPrefix(prefix) => {
                if !prefix.operator.is_increment_or_decrement() && Precedence::Assignment < prefix.operator.precedence()
                {
                    // make `(--$x) = $y` into `--($x = $y)`
                    self.arena.alloc(Expression::UnaryPrefix(UnaryPrefix {
                        operator: prefix.operator.clone(),
                        operand: self.create_assignment_expression(prefix.operand, operator, rhs),
                    }))
                } else {
                    self.arena.alloc(Expression::Assignment(Assignment { lhs, operator, rhs }))
                }
            }
            Expression::Binary(operation) => {
                let assignment_precedence = Precedence::Assignment;
                let binary_precedence = operation.operator.precedence();

                if assignment_precedence < binary_precedence {
                    // make `($x == $y) = $z` into `$x == ($y = $z)`
                    self.arena.alloc(Expression::Binary(Binary {
                        lhs: operation.lhs,
                        operator: operation.operator,
                        rhs: self.create_assignment_expression(operation.rhs, operator, rhs),
                    }))
                } else {
                    self.arena.alloc(Expression::Assignment(Assignment { lhs, operator, rhs }))
                }
            }
            Expression::Conditional(conditional) => self.arena.alloc(Expression::Conditional(Conditional {
                condition: conditional.condition,
                question_mark: conditional.question_mark,
                then: conditional.then,
                colon: conditional.colon,
                r#else: self.create_assignment_expression(conditional.r#else, operator, rhs),
            })),
            _ => self.arena.alloc(Expression::Assignment(Assignment { lhs, operator, rhs })),
        }
    }
}
