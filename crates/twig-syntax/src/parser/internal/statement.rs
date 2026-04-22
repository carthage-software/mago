use crate::ast::Print;
use crate::ast::Sequence;
use crate::ast::Statement;
use crate::ast::Text;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

/// Trait used by block parsers to recognise their closing `{% endX %}` tag.
pub trait Terminator {
    fn is_end(&self, name: &str) -> bool;
}

pub struct NoTerminator;

impl Terminator for NoTerminator {
    fn is_end(&self, _name: &str) -> bool {
        false
    }
}

pub struct BlockTerminator {
    pub names: &'static [&'static str],
}

impl Terminator for BlockTerminator {
    fn is_end(&self, name: &str) -> bool {
        self.names.contains(&name)
    }
}

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Parse a sequence of top-level statements until `terminator` matches
    /// or EOF is reached.
    pub(crate) fn parse_statements<T: Terminator>(
        &mut self,
        terminator: &T,
    ) -> Result<Sequence<'arena, Statement<'arena>>, ParseError> {
        let mut statements = self.new_vec();
        while let Some(token) = self.stream.lookahead(0)? {
            match token.kind {
                TwigTokenKind::RawText => {
                    let t = self.stream.consume()?;
                    statements.push(Statement::Text(Text { value: t.value, span: self.stream.span_of(&t) }));
                }
                kind if kind.is_open_variable() => {
                    let start_tok = self.stream.consume()?;
                    let open_variable = self.stream.span_of(&start_tok);
                    let expression = self.parse_expression()?;
                    let close_variable = self.stream.expect_variable_end()?;
                    statements.push(Statement::Print(Print { open_variable, expression, close_variable }));
                }
                kind if kind.is_open_block() => {
                    let Some(name_tok) = self.stream.lookahead(1)? else {
                        return Err(self.stream.unexpected(None, &[TwigTokenKind::Name]));
                    };
                    if name_tok.kind != TwigTokenKind::Name {
                        return Err(ParseError::UnexpectedToken(
                            "a block tag must start with a tag name".to_string(),
                            self.stream.span_of(&name_tok),
                        ));
                    }
                    if terminator.is_end(name_tok.value) {
                        // Leave the `{%` + keyword for the caller to consume.
                        break;
                    }

                    let open_tag = self.stream.consume()?;
                    let keyword_tok = self.stream.consume()?;
                    let statement = self.parse_tag(open_tag, keyword_tok)?;
                    statements.push(statement);
                }
                TwigTokenKind::VerbatimText => {
                    // Should never appear at the statement level outside verbatim handling.
                    self.stream.consume()?;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        format!("unexpected token {:?} at statement level", token.kind),
                        self.stream.span_of(&token),
                    ));
                }
            }
        }

        Ok(Sequence::new(statements))
    }
}
