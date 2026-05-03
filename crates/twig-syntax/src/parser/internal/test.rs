use crate::ast::Expression;
use crate::ast::Test;
use crate::ast::TestArguments;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    /// Parse a test expression suffix: `operand is [not] name [second_word] [arguments]`.
    /// The leading `is` token must already have been consumed.
    pub(crate) fn parse_test(
        &mut self,
        operand: Expression<'arena>,
        is_keyword: crate::ast::Keyword<'arena>,
    ) -> Result<Expression<'arena>, ParseError> {
        let mut not_keyword = None;
        if let Some(token) = self.stream.lookahead(0)?
            && matches!(token.kind, TwigTokenKind::Not | TwigTokenKind::Name)
            && token.value == "not"
        {
            let not_tok = self.stream.consume()?;
            not_keyword = Some(self.keyword_from(&not_tok));
        }

        let name = self.expect_flexible_identifier("expected test name")?;

        let second_word = match self.stream.lookahead(0)? {
            Some(token) if token.kind == TwigTokenKind::Name && !matches!(token.value, "and" | "or" | "xor") => {
                let tok = self.stream.consume()?;
                Some(self.keyword_from(&tok))
            }
            _ => None,
        };

        let arguments = if let Some(list) = self.parse_optional_argument_list()? {
            TestArguments::Parenthesised(list)
        } else if self.stream.is_start_of_primary()? {
            let value = self.parse_expression_with_precedence(100)?;
            TestArguments::Bare(self.alloc(value))
        } else {
            TestArguments::None
        };

        Ok(Expression::Test(Test {
            operand: self.alloc(operand),
            is_keyword,
            not_keyword,
            name,
            second_word,
            arguments,
        }))
    }
}
