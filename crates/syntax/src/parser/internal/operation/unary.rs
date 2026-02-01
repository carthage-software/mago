use mago_database::file::HasFileId;

use crate::T;
use crate::ast::ast::UnaryPrefix;
use crate::ast::ast::UnaryPrefixOperator;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::GetPrecedence;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_unary_prefix_operation(&mut self) -> Result<UnaryPrefix<'arena>, ParseError> {
        let token = self.stream.consume()?;

        let operator = match token.kind {
            T!["(array)"] => UnaryPrefixOperator::ArrayCast(token.span_for(self.stream.file_id()), token.value),
            T!["(bool)"] => UnaryPrefixOperator::BoolCast(token.span_for(self.stream.file_id()), token.value),
            T!["(boolean)"] => UnaryPrefixOperator::BooleanCast(token.span_for(self.stream.file_id()), token.value),
            T!["(double)"] => UnaryPrefixOperator::DoubleCast(token.span_for(self.stream.file_id()), token.value),
            T!["(real)"] => UnaryPrefixOperator::RealCast(token.span_for(self.stream.file_id()), token.value),
            T!["(float)"] => UnaryPrefixOperator::FloatCast(token.span_for(self.stream.file_id()), token.value),
            T!["(int)"] => UnaryPrefixOperator::IntCast(token.span_for(self.stream.file_id()), token.value),
            T!["(integer)"] => UnaryPrefixOperator::IntegerCast(token.span_for(self.stream.file_id()), token.value),
            T!["(object)"] => UnaryPrefixOperator::ObjectCast(token.span_for(self.stream.file_id()), token.value),
            T!["(unset)"] => UnaryPrefixOperator::UnsetCast(token.span_for(self.stream.file_id()), token.value),
            T!["(binary)"] => UnaryPrefixOperator::BinaryCast(token.span_for(self.stream.file_id()), token.value),
            T!["(string)"] => UnaryPrefixOperator::StringCast(token.span_for(self.stream.file_id()), token.value),
            T!["(void)"] => UnaryPrefixOperator::VoidCast(token.span_for(self.stream.file_id()), token.value),
            T!["@"] => UnaryPrefixOperator::ErrorControl(token.span_for(self.stream.file_id())),
            T!["!"] => UnaryPrefixOperator::Not(token.span_for(self.stream.file_id())),
            T!["~"] => UnaryPrefixOperator::BitwiseNot(token.span_for(self.stream.file_id())),
            T!["-"] => UnaryPrefixOperator::Negation(token.span_for(self.stream.file_id())),
            T!["+"] => UnaryPrefixOperator::Plus(token.span_for(self.stream.file_id())),
            T!["++"] => UnaryPrefixOperator::PreIncrement(token.span_for(self.stream.file_id())),
            T!["--"] => UnaryPrefixOperator::PreDecrement(token.span_for(self.stream.file_id())),
            T!["&"] => UnaryPrefixOperator::Reference(token.span_for(self.stream.file_id())),
            _ => {
                return Err(self.stream.unexpected(
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

        let operand = self.parse_expression_with_precedence(operator.precedence())?;

        Ok(UnaryPrefix { operator, operand: self.arena.alloc(operand) })
    }
}
