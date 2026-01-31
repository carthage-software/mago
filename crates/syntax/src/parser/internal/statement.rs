use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Expression;
use crate::ast::ast::ExpressionStatement;
use crate::ast::ast::Statement;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_statement(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T![InlineText | InlineShebang] => Statement::Inline(self.parse_inline(stream)?),
            T!["<?php"] | T!["<?"] => Statement::OpeningTag(self.parse_opening_tag(stream)?),
            T!["<?="] => Statement::EchoTag(self.parse_echo_tag(stream)?),
            T!["?>"] => Statement::ClosingTag(self.parse_closing_tag(stream)?),
            T!["declare"] => Statement::Declare(self.parse_declare(stream)?),
            T!["namespace"] => Statement::Namespace(self.parse_namespace(stream)?),
            T!["use"] => Statement::Use(self.parse_use(stream)?),
            T!["return"] => Statement::Return(self.parse_return(stream)?),
            T!["#["] => {
                let attributes = self.parse_attribute_list_sequence(stream)?;
                let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
                let maybe_after = stream.lookahead(1)?.map(|t| t.kind);

                match next.kind {
                    T!["interface"] => Statement::Interface(self.parse_interface_with_attributes(stream, attributes)?),
                    T!["trait"] => Statement::Trait(self.parse_trait_with_attributes(stream, attributes)?),
                    T!["enum"] => Statement::Enum(self.parse_enum_with_attributes(stream, attributes)?),
                    T!["class"] => Statement::Class(self.parse_class_with_attributes(stream, attributes)?),
                    T!["const"] => Statement::Constant(self.parse_constant_with_attributes(stream, attributes)?),
                    T!["function"] => self.parse_closure_or_function(stream, attributes)?,
                    T!["fn"] => Statement::Expression(ExpressionStatement {
                        expression: {
                            self.arena.alloc(Expression::ArrowFunction(
                                self.parse_arrow_function_with_attributes(stream, attributes)?,
                            ))
                        },
                        terminator: self.parse_terminator(stream)?,
                    }),
                    T!["static"] if maybe_after == Some(T!["fn"]) => Statement::Expression(ExpressionStatement {
                        expression: {
                            self.arena.alloc(Expression::ArrowFunction(
                                self.parse_arrow_function_with_attributes(stream, attributes)?,
                            ))
                        },
                        terminator: self.parse_terminator(stream)?,
                    }),
                    T!["static"] if maybe_after == Some(T!["function"]) => Statement::Expression(ExpressionStatement {
                        expression: {
                            self.arena
                                .alloc(Expression::Closure(self.parse_closure_with_attributes(stream, attributes)?))
                        },
                        terminator: self.parse_terminator(stream)?,
                    }),
                    kind if kind.is_modifier() => {
                        Statement::Class(self.parse_class_with_attributes(stream, attributes)?)
                    }
                    _ => {
                        return Err(stream.unexpected(
                            Some(next),
                            T![
                                "interface",
                                "trait",
                                "enum",
                                "class",
                                "function",
                                "fn",
                                "readonly",
                                "abstract",
                                "final",
                                "static",
                            ],
                        ));
                    }
                }
            }
            T!["interface"] => {
                Statement::Interface(self.parse_interface_with_attributes(stream, Sequence::empty(self.arena))?)
            }
            T!["trait"] => Statement::Trait(self.parse_trait_with_attributes(stream, Sequence::empty(self.arena))?),
            T!["enum"] => Statement::Enum(self.parse_enum_with_attributes(stream, Sequence::empty(self.arena))?),
            T!["class"] => Statement::Class(self.parse_class_with_attributes(stream, Sequence::empty(self.arena))?),
            T!["function"] => self.parse_closure_or_function(stream, Sequence::empty(self.arena))?,
            T!["global"] => Statement::Global(self.parse_global(stream)?),
            T!["static"] if matches!(stream.lookahead(1)?.map(|t| t.kind), Some(T!["$variable"])) => {
                Statement::Static(self.parse_static(stream)?)
            }
            kind if kind.is_modifier()
                && !matches!(
                    stream.lookahead(1)?.map(|t| t.kind),
                    Some(T!["::" | "(" | "->" | "?->" | "[" | "fn" | "function"])
                ) =>
            {
                Statement::Class(self.parse_class_with_attributes(stream, Sequence::empty(self.arena))?)
            }
            T!["__halt_compiler"] => Statement::HaltCompiler(self.parse_halt_compiler(stream)?),
            T![";"] => Statement::Noop(stream.consume()?.span),
            T!["const"] => {
                Statement::Constant(self.parse_constant_with_attributes(stream, Sequence::empty(self.arena))?)
            }
            T!["if"] => Statement::If(self.parse_if(stream)?),
            T!["switch"] => Statement::Switch(self.parse_switch(stream)?),
            T!["foreach"] => Statement::Foreach(self.parse_foreach(stream)?),
            T!["for"] => Statement::For(self.parse_for(stream)?),
            T!["while"] => Statement::While(self.parse_while(stream)?),
            T!["do"] => Statement::DoWhile(self.parse_do_while(stream)?),
            T!["continue"] => Statement::Continue(self.parse_continue(stream)?),
            T!["break"] => Statement::Break(self.parse_break(stream)?),
            T!["unset"] => Statement::Unset(self.parse_unset(stream)?),
            T!["{"] => Statement::Block(self.parse_block(stream)?),
            T!["try"] => Statement::Try(self.parse_try(stream)?),
            T!["echo"] => Statement::Echo(self.parse_echo(stream)?),
            T!["goto"] => Statement::Goto(self.parse_goto(stream)?),
            kind if kind.is_identifier_maybe_reserved()
                && matches!(stream.lookahead(1)?.map(|t| t.kind), Some(T![":"])) =>
            {
                Statement::Label(self.parse_label(stream)?)
            }
            _ => Statement::Expression(ExpressionStatement {
                expression: self.arena.alloc(self.parse_expression(stream)?),
                terminator: self.parse_terminator(stream)?,
            }),
        })
    }

    fn parse_closure_or_function(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Statement<'arena>, ParseError> {
        Ok(match (stream.lookahead(1)?.map(|t| t.kind), stream.lookahead(2)?.map(|t| t.kind)) {
            // if the next token is `(` or `&` followed by `(`, then we know this is a closure
            (Some(T!["("]), _) | (Some(T!["&"]), Some(T!["("])) => Statement::Expression(ExpressionStatement {
                expression: {
                    self.arena.alloc(Expression::Closure(self.parse_closure_with_attributes(stream, attributes)?))
                },
                terminator: self.parse_terminator(stream)?,
            }),
            _ => {
                // otherwise, we know this is a function
                Statement::Function(self.parse_function_with_attributes(stream, attributes)?)
            }
        })
    }
}
