use crate::T;
use crate::ast::ast::UnaryPrefix;
use crate::ast::ast::UnaryPrefixOperator;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::GetPrecedence;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_unary_prefix_operation(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<UnaryPrefix<'arena>, ParseError> {
        let token = stream.consume()?;

        let operator = match token.kind {
            T!["(array)"] => UnaryPrefixOperator::ArrayCast(token.span, token.value),
            T!["(bool)"] => UnaryPrefixOperator::BoolCast(token.span, token.value),
            T!["(boolean)"] => UnaryPrefixOperator::BooleanCast(token.span, token.value),
            T!["(double)"] => UnaryPrefixOperator::DoubleCast(token.span, token.value),
            T!["(real)"] => UnaryPrefixOperator::RealCast(token.span, token.value),
            T!["(float)"] => UnaryPrefixOperator::FloatCast(token.span, token.value),
            T!["(int)"] => UnaryPrefixOperator::IntCast(token.span, token.value),
            T!["(integer)"] => UnaryPrefixOperator::IntegerCast(token.span, token.value),
            T!["(object)"] => UnaryPrefixOperator::ObjectCast(token.span, token.value),
            T!["(unset)"] => UnaryPrefixOperator::UnsetCast(token.span, token.value),
            T!["(binary)"] => UnaryPrefixOperator::BinaryCast(token.span, token.value),
            T!["(string)"] => UnaryPrefixOperator::StringCast(token.span, token.value),
            T!["(void)"] => UnaryPrefixOperator::VoidCast(token.span, token.value),
            T!["@"] => UnaryPrefixOperator::ErrorControl(token.span),
            T!["!"] => UnaryPrefixOperator::Not(token.span),
            T!["~"] => UnaryPrefixOperator::BitwiseNot(token.span),
            T!["-"] => UnaryPrefixOperator::Negation(token.span),
            T!["+"] => UnaryPrefixOperator::Plus(token.span),
            T!["++"] => UnaryPrefixOperator::PreIncrement(token.span),
            T!["--"] => UnaryPrefixOperator::PreDecrement(token.span),
            T!["&"] => UnaryPrefixOperator::Reference(token.span),
            _ => {
                return Err(stream.unexpected(
                    Some(token),
                    T![
                        "(array)",
                        "(bool)",
                        "(boolean)",
                        "(double)",
                        "(real)",
                        "(float)",
                        "(int)",
                        "(integer)",
                        "(object)",
                        "(unset)",
                        "(binary)",
                        "(string)",
                        "@",
                        "!",
                        "~",
                        "-",
                        "+",
                        "++",
                        "--",
                        "&"
                    ],
                ));
            }
        };

        let operand = self.parse_expression_with_precedence(stream, operator.precedence())?;

        Ok(UnaryPrefix { operator, operand: self.arena.alloc(operand) })
    }
}
