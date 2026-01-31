use crate::T;
use crate::ast::ast::Hint;
use crate::ast::ast::IntersectionHint;
use crate::ast::ast::NullableHint;
use crate::ast::ast::ParenthesizedHint;
use crate::ast::ast::UnionHint;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn is_at_type_hint(&self, stream: &mut TokenStream<'_, '_>) -> Result<bool, ParseError> {
        Ok(matches!(
            stream.lookahead(0)?.map(|t| t.kind),
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

    pub(crate) fn parse_optional_type_hint(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<Hint<'arena>>, ParseError> {
        if self.is_at_type_hint(stream)? { Ok(Some(self.parse_type_hint(stream)?)) } else { Ok(None) }
    }

    pub(crate) fn parse_type_hint(&self, stream: &mut TokenStream<'_, 'arena>) -> Result<Hint<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        let hint = match &token.kind {
            T!["?"] => Hint::Nullable(self.parse_nullable_type_hint(stream)?),
            T!["("] => Hint::Parenthesized(self.parse_parenthesized_type_hint(stream)?),
            T!["array"] => Hint::Array(self.expect_any_keyword(stream)?),
            T!["callable"] => Hint::Callable(self.expect_any_keyword(stream)?),
            T!["null"] => Hint::Null(self.expect_any_keyword(stream)?),
            T!["true"] => Hint::True(self.expect_any_keyword(stream)?),
            T!["false"] => Hint::False(self.expect_any_keyword(stream)?),
            T!["static"] => Hint::Static(self.expect_any_keyword(stream)?),
            T!["self"] => Hint::Self_(self.expect_any_keyword(stream)?),
            T!["parent"] => Hint::Parent(self.expect_any_keyword(stream)?),
            T!["enum" | "from" | QualifiedIdentifier | FullyQualifiedIdentifier] => {
                Hint::Identifier(self.parse_identifier(stream)?)
            }
            T![Identifier] => match token.value {
                val if val.eq_ignore_ascii_case("void") => Hint::Void(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("never") => Hint::Never(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("float") => Hint::Float(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("bool") => Hint::Bool(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("int") => Hint::Integer(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("string") => Hint::String(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("object") => Hint::Object(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("mixed") => Hint::Mixed(self.parse_local_identifier(stream)?),
                val if val.eq_ignore_ascii_case("iterable") => Hint::Iterable(self.parse_local_identifier(stream)?),
                _ => Hint::Identifier(self.parse_identifier(stream)?),
            },
            _ => {
                return Err(stream.unexpected(
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

        let next = stream.lookahead(0)?;
        Ok(match next.map(|t| t.kind) {
            Some(T!["|"]) => {
                let left = hint;
                let pipe = stream.eat(T!["|"])?.span;
                let right = self.parse_type_hint(stream)?;

                Hint::Union(UnionHint { left: self.arena.alloc(left), pipe, right: self.arena.alloc(right) })
            }
            Some(T!["&"])
                if !matches!(stream.lookahead(1)?.map(|t| t.kind), Some(T!["$variable"] | T!["..."] | T!["&"])) =>
            {
                let left = hint;
                let ampersand = stream.eat(T!["&"])?.span;
                let right = self.parse_type_hint(stream)?;

                Hint::Intersection(IntersectionHint {
                    left: self.arena.alloc(left),
                    ampersand,
                    right: self.arena.alloc(right),
                })
            }
            _ => hint,
        })
    }

    pub(crate) fn parse_nullable_type_hint(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<NullableHint<'arena>, ParseError> {
        let question_mark = stream.eat(T!["?"])?.span;
        let hint = self.parse_type_hint(stream)?;

        Ok(NullableHint { question_mark, hint: self.arena.alloc(hint) })
    }

    pub(crate) fn parse_parenthesized_type_hint(
        &self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ParenthesizedHint<'arena>, ParseError> {
        let left_parenthesis = stream.eat(T!["("])?.span;
        let hint = self.parse_type_hint(stream)?;
        let right_parenthesis = stream.eat(T![")"])?.span;

        Ok(ParenthesizedHint { left_parenthesis, hint: self.arena.alloc(hint), right_parenthesis })
    }
}
