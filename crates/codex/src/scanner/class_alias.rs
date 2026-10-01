use mago_allocator::Arena;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Access;
use mago_syntax::cst::Argument;
use mago_syntax::cst::ClassLikeConstantSelector;
use mago_syntax::cst::Expression;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::Literal;
use mago_word::Word;
use mago_word::ascii_lowercase_word;

use crate::scanner::Context;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_class_like_alias<'arena, A>(
    call: &'arena FunctionCall<'arena>,
    context: &Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    current_class: Option<Word>,
    parent_class: Option<Word>,
) -> Option<(Word, Word, Span)>
where
    A: Arena,
{
    let Expression::Identifier(identifier) = call.function.unparenthesized() else {
        return None;
    };

    let function = identifier.value();
    if !function.eq_ignore_ascii_case(b"class_alias") && !function.eq_ignore_ascii_case(b"\\class_alias") {
        return None;
    }

    let mut target = None;
    let mut alias = None;
    let mut positional = 0;

    for argument in &call.argument_list.arguments {
        match argument {
            Argument::Positional(argument) => {
                if argument.ellipsis.is_some() {
                    return None;
                }

                match positional {
                    0 => target = Some(argument.value),
                    1 => alias = Some(argument.value),
                    2 => {}
                    _ => return None,
                }

                positional += 1;
            }
            Argument::Named(argument) if argument.name.value.eq_ignore_ascii_case(b"class") => {
                target = Some(argument.value);
            }
            Argument::Named(argument) if argument.name.value.eq_ignore_ascii_case(b"alias") => {
                alias = Some(argument.value);
            }
            Argument::Named(argument) if argument.name.value.eq_ignore_ascii_case(b"autoload") => {}
            Argument::Named(_) => return None,
        }
    }

    let target = get_class_like_name(target?, context, scope, current_class, parent_class)?;
    let alias = get_class_like_name(alias?, context, scope, current_class, parent_class)?;

    Some((alias, target, call.span()))
}

#[inline]
fn get_class_like_name<A>(
    expression: &Expression<'_>,
    context: &Context<'_, '_, A>,
    scope: &NamespaceScope,
    current_class: Option<Word>,
    parent_class: Option<Word>,
) -> Option<Word>
where
    A: Arena,
{
    match expression.unparenthesized() {
        Expression::Literal(Literal::String(literal)) => {
            let value = literal.value?;
            normalize_class_like_name(value)
        }
        Expression::Access(Access::ClassConstant(access)) => {
            let ClassLikeConstantSelector::Identifier(constant) = &access.constant else {
                return None;
            };

            if !constant.value.eq_ignore_ascii_case(b"class") {
                return None;
            }

            match access.class.unparenthesized() {
                Expression::Identifier(identifier) => {
                    Some(ascii_lowercase_word(context.resolved_names.get(identifier)))
                }
                Expression::Self_(_) | Expression::Static(_) => current_class,
                Expression::Parent(_) => parent_class,
                _ => None,
            }
        }
        expression => {
            let inferred = infer(context, scope, expression, current_class)?;
            normalize_class_like_name(inferred.get_single_literal_string_value()?)
        }
    }
}

#[inline]
fn normalize_class_like_name(value: &[u8]) -> Option<Word> {
    let value = value.strip_prefix(b"\\").unwrap_or(value);
    (!value.is_empty()).then(|| ascii_lowercase_word(value))
}
