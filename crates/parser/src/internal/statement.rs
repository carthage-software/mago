use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::attribute::parse_attribute_list_sequence;
use crate::internal::block::parse_block;
use crate::internal::class_like::parse_class_with_attributes;
use crate::internal::class_like::parse_enum_with_attributes;
use crate::internal::class_like::parse_interface_with_attributes;
use crate::internal::class_like::parse_trait_with_attributes;
use crate::internal::constant::parse_constant_with_attributes;
use crate::internal::control_flow::r#if::parse_if;
use crate::internal::control_flow::switch::parse_switch;
use crate::internal::declare::parse_declare;
use crate::internal::echo::parse_echo;
use crate::internal::expression::parse_expression;
use crate::internal::function_like::arrow_function::parse_arrow_function_with_attributes;
use crate::internal::function_like::closure::parse_closure_with_attributes;
use crate::internal::function_like::function::parse_function_with_attributes;
use crate::internal::global::parse_global;
use crate::internal::goto::parse_goto;
use crate::internal::goto::parse_label;
use crate::internal::halt_compiler::parse_halt_compiler;
use crate::internal::inline::parse_inline;
use crate::internal::namespace::parse_namespace;
use crate::internal::r#loop::do_while::parse_do_while;
use crate::internal::r#loop::foreach::parse_foreach;
use crate::internal::r#loop::parse_break;
use crate::internal::r#loop::parse_continue;
use crate::internal::r#loop::r#for::parse_for;
use crate::internal::r#loop::r#while::parse_while;
use crate::internal::r#return::parse_return;
use crate::internal::r#static::parse_static;
use crate::internal::r#try::parse_try;
use crate::internal::r#use::parse_use;
use crate::internal::tag::parse_closing_tag;
use crate::internal::tag::parse_opening_tag;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::unset::parse_unset;
use crate::internal::utils;

pub fn parse_statement(stream: &mut TokenStream<'_, '_>) -> Result<Statement, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T![InlineText | InlineShebang] => Statement::Inline(parse_inline(stream)?),
        T!["<?php"] | T!["<?="] | T!["<?"] => Statement::OpeningTag(parse_opening_tag(stream)?),
        T!["?>"] => Statement::ClosingTag(parse_closing_tag(stream)?),
        T!["declare"] => Statement::Declare(parse_declare(stream)?),
        T!["namespace"] => Statement::Namespace(parse_namespace(stream)?),
        T!["use"] => Statement::Use(parse_use(stream)?),
        T!["return"] => Statement::Return(parse_return(stream)?),
        T!["#["] => {
            let attributes = parse_attribute_list_sequence(stream)?;
            let next = utils::peek(stream)?;
            let maybe_after = utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind);

            match next.kind {
                T!["interface"] => Statement::Interface(parse_interface_with_attributes(stream, attributes)?),
                T!["trait"] => Statement::Trait(parse_trait_with_attributes(stream, attributes)?),
                T!["enum"] => Statement::Enum(parse_enum_with_attributes(stream, attributes)?),
                T!["class"] => Statement::Class(parse_class_with_attributes(stream, attributes)?),
                T!["const"] => Statement::Constant(parse_constant_with_attributes(stream, attributes)?),
                T!["function"] => {
                    // unlike when we have modifiers, here, we don't know if this is meant to be a closure or a function
                    parse_closure_or_function(stream, attributes)?
                }
                T!["fn"] => Statement::Expression(ExpressionStatement {
                    expression: Box::new(Expression::ArrowFunction(parse_arrow_function_with_attributes(
                        stream, attributes,
                    )?)),
                    terminator: parse_terminator(stream)?,
                }),
                T!["static"] if maybe_after == Some(T!["fn"]) => Statement::Expression(ExpressionStatement {
                    expression: Box::new(Expression::ArrowFunction(parse_arrow_function_with_attributes(
                        stream, attributes,
                    )?)),
                    terminator: parse_terminator(stream)?,
                }),
                T!["static"] if maybe_after == Some(T!["function"]) => Statement::Expression(ExpressionStatement {
                    expression: Box::new(Expression::Closure(parse_closure_with_attributes(stream, attributes)?)),
                    terminator: parse_terminator(stream)?,
                }),
                kind if kind.is_modifier() => Statement::Class(parse_class_with_attributes(stream, attributes)?),
                _ => {
                    return Err(utils::unexpected(
                        stream,
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
        T!["interface"] => Statement::Interface(parse_interface_with_attributes(stream, Sequence::empty())?),
        T!["trait"] => Statement::Trait(parse_trait_with_attributes(stream, Sequence::empty())?),
        T!["enum"] => Statement::Enum(parse_enum_with_attributes(stream, Sequence::empty())?),
        T!["class"] => Statement::Class(parse_class_with_attributes(stream, Sequence::empty())?),
        T!["function"] => {
            // just like when we have attributes, we don't know if this is meant to be a closure or a function
            parse_closure_or_function(stream, Sequence::empty())?
        }
        T!["global"] => Statement::Global(parse_global(stream)?),
        T!["static"] if matches!(utils::peek_nth(stream, 1)?.kind, T!["$variable"]) => {
            Statement::Static(parse_static(stream)?)
        }
        kind if kind.is_modifier()
            && !matches!(utils::peek_nth(stream, 1)?.kind, T!["::" | "(" | "->" | "?->" | "[" | "fn" | "function"]) =>
        {
            Statement::Class(parse_class_with_attributes(stream, Sequence::empty())?)
        }
        T!["__halt_compiler"] => Statement::HaltCompiler(parse_halt_compiler(stream)?),
        T![";"] => Statement::Noop(utils::expect(stream, T![";"])?.span),
        T!["const"] => Statement::Constant(parse_constant_with_attributes(stream, Sequence::empty())?),
        T!["if"] => Statement::If(parse_if(stream)?),
        T!["switch"] => Statement::Switch(parse_switch(stream)?),
        T!["foreach"] => Statement::Foreach(parse_foreach(stream)?),
        T!["for"] => Statement::For(parse_for(stream)?),
        T!["while"] => Statement::While(parse_while(stream)?),
        T!["do"] => Statement::DoWhile(parse_do_while(stream)?),
        T!["continue"] => Statement::Continue(parse_continue(stream)?),
        T!["break"] => Statement::Break(parse_break(stream)?),
        T!["unset"] => Statement::Unset(parse_unset(stream)?),
        T!["{"] => Statement::Block(parse_block(stream)?),
        T!["try"] => Statement::Try(parse_try(stream)?),
        T!["echo"] => Statement::Echo(parse_echo(stream)?),
        T!["goto"] => Statement::Goto(parse_goto(stream)?),
        kind if kind.is_identifier_maybe_reserved() && matches!(utils::peek_nth(stream, 1)?.kind, T![":"]) => {
            Statement::Label(parse_label(stream)?)
        }
        _ => Statement::Expression(ExpressionStatement {
            expression: Box::new(parse_expression(stream)?),
            terminator: parse_terminator(stream)?,
        }),
    })
}

fn parse_closure_or_function(
    stream: &mut TokenStream<'_, '_>,
    attributes: Sequence<AttributeList>,
) -> Result<Statement, ParseError> {
    Ok(match (utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind), utils::maybe_peek_nth(stream, 2)?.map(|t| t.kind)) {
        // if the next token is `(` or `&` followed by `(`, then we know this is a closure
        (Some(T!["("]), _) | (Some(T!["&"]), Some(T!["("])) => Statement::Expression(ExpressionStatement {
            expression: Box::new(Expression::Closure(parse_closure_with_attributes(stream, attributes)?)),
            terminator: parse_terminator(stream)?,
        }),
        _ => {
            // otherwise, we know this is a function
            Statement::Function(parse_function_with_attributes(stream, attributes)?)
        }
    })
}
