use crate::T;
use crate::ast::ast::Hint;
use crate::ast::ast::IntersectionHint;
use crate::ast::ast::NullableHint;
use crate::ast::ast::ParenthesizedHint;
use crate::ast::ast::UnionHint;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn is_at_type_hint(&mut self) -> Result<bool, ParseError> {
        Ok(matches!(
            self.stream.peek_kind(0)?,
            Some(T!["?"
                | "("
                | "array"
                | "callable"
                | "null"
                | "true"
                | "false"
                | "static"
                | "self"
                | "parent"
                | "enum"
                | "from"
                | Identifier
                | QualifiedIdentifier
                | FullyQualifiedIdentifier])
        ))
    }

    pub(crate) fn parse_optional_type_hint(&mut self) -> Result<Option<Hint<'arena>>, ParseError> {
        if self.is_at_type_hint()? { Ok(Some(self.parse_type_hint()?)) } else { Ok(None) }
    }

    pub(crate) fn parse_type_hint(&mut self) -> Result<Hint<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        let hint = match &token.kind {
            T!["?"] => Hint::Nullable(self.parse_nullable_type_hint()?),
            T!["("] => Hint::Parenthesized(self.parse_parenthesized_type_hint()?),
            T!["array"] => Hint::Array(self.expect_any_keyword()?),
            T!["callable"] => Hint::Callable(self.expect_any_keyword()?),
            T!["null"] => Hint::Null(self.expect_any_keyword()?),
            T!["true"] => Hint::True(self.expect_any_keyword()?),
            T!["false"] => Hint::False(self.expect_any_keyword()?),
            T!["static"] => Hint::Static(self.expect_any_keyword()?),
            T!["self"] => Hint::Self_(self.expect_any_keyword()?),
            T!["parent"] => Hint::Parent(self.expect_any_keyword()?),
            T!["enum" | "from" | QualifiedIdentifier | FullyQualifiedIdentifier] => {
                Hint::Identifier(self.parse_identifier()?)
            }
            T![Identifier] => match token.value {
                val if val.eq_ignore_ascii_case("void") => Hint::Void(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("never") => Hint::Never(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("float") => Hint::Float(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("bool") => Hint::Bool(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("int") => Hint::Integer(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("string") => Hint::String(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("object") => Hint::Object(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("mixed") => Hint::Mixed(self.parse_local_identifier()?),
                val if val.eq_ignore_ascii_case("iterable") => Hint::Iterable(self.parse_local_identifier()?),
                _ => Hint::Identifier(self.parse_identifier()?),
            },
            _ => {
                return Err(self.stream.unexpected(
                    Some(token),
                    T![
                        "?",
                        "(",
                        "array",
                        "callable",
                        "null",
                        "true",
                        "false",
                        "static",
                        "self",
                        "parent",
                        "enum",
                        "from",
                        Identifier,
                        QualifiedIdentifier,
                        FullyQualifiedIdentifier,
                    ],
                ));
            }
        };

        let next = self.stream.lookahead(0)?;
        Ok(match next.map(|t| t.kind) {
            Some(T!["|"]) => {
                let left = hint;
                let pipe = self.stream.eat_span(T!["|"])?;
                let right = self.parse_type_hint()?;

                Hint::Union(UnionHint { left: self.arena.alloc(left), pipe, right: self.arena.alloc(right) })
            }
            Some(T!["&"]) if !matches!(self.stream.peek_kind(1)?, Some(T!["$variable"] | T!["..."] | T!["&"])) => {
                let left = hint;
                let ampersand = self.stream.eat_span(T!["&"])?;
                let right = self.parse_type_hint()?;

                Hint::Intersection(IntersectionHint {
                    left: self.arena.alloc(left),
                    ampersand,
                    right: self.arena.alloc(right),
                })
            }
            _ => hint,
        })
    }

    pub(crate) fn parse_nullable_type_hint(&mut self) -> Result<NullableHint<'arena>, ParseError> {
        let question_mark = self.stream.eat_span(T!["?"])?;
        let hint = self.parse_type_hint()?;

        Ok(NullableHint { question_mark, hint: self.arena.alloc(hint) })
    }

    pub(crate) fn parse_parenthesized_type_hint(&mut self) -> Result<ParenthesizedHint<'arena>, ParseError> {
        let left_parenthesis = self.stream.eat_span(T!["("])?;
        let hint = self.parse_type_hint()?;
        let right_parenthesis = self.stream.eat_span(T![")"])?;

        Ok(ParenthesizedHint { left_parenthesis, hint: self.arena.alloc(hint), right_parenthesis })
    }
}
