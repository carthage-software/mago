use std::borrow::Cow;

use mago_allocator::Arena;

use mago_bytes::BytesDisplay;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_twig_syntax::cst::Argument;
use mago_twig_syntax::cst::Call;
use mago_twig_syntax::cst::Expression;
use mago_twig_syntax::cst::Name;
use mago_twig_syntax::cst::Node;

use crate::context::LintContext;

/// Removes the parentheses around an expression.
pub(crate) fn ungroup<'ast, 'arena>(mut expression: &'ast Expression<'arena>) -> &'ast Expression<'arena> {
    while let Expression::Parenthesized(group) = expression {
        expression = group.inner;
    }

    expression
}

/// Whether an ASCII name contains an uppercase letter, which snake_case forbids.
pub(crate) fn is_not_snake_case(name: &[u8]) -> bool {
    name.is_ascii() && name.iter().any(u8::is_ascii_uppercase)
}

/// The key Twig uses for a named argument of its own functions, filters, and tests: it
/// ignores underscores and case.
pub(crate) fn normalize_argument_name(name: &[u8]) -> Cow<'_, [u8]> {
    if name.iter().any(|byte| *byte == b'_' || byte.is_ascii_uppercase()) {
        Cow::Owned(name.iter().filter(|byte| **byte != b'_').map(u8::to_ascii_lowercase).collect())
    } else {
        Cow::Borrowed(name)
    }
}

/// The issue for a name that repeats an earlier one, with both places marked.
///
/// `verb` says what the template does with the name, such as "defined" or "passed".
pub(crate) fn duplicate_name_issue(
    level: Level,
    code: &'static str,
    span: Span,
    previous: Span,
    kind: &str,
    verb: &str,
    name: &[u8],
) -> Issue {
    Issue::new(level, format!("The {kind} `{}` is {verb} more than once.", BytesDisplay(name)))
        .with_code(code)
        .with_annotation(Annotation::primary(span).with_message("Repeated here"))
        .with_annotation(Annotation::secondary(previous).with_message(format!("First {verb} here")))
}

/// The source text a span covers.
pub(crate) fn source_text<'ctx, A>(ctx: &LintContext<'ctx, '_, A>, span: Span) -> &'ctx [u8]
where
    A: Arena,
{
    &ctx.source_file.contents[span.start_offset() as usize..span.end_offset() as usize]
}

/// The snake_case spelling of a name, for suggestions.
pub(crate) fn snake_case(name: &[u8]) -> String {
    String::from_utf8_lossy(&mago_casing::to_snake_case(name)).into_owned()
}

/// The Twig function a call refers to, unless a `{% from %}` import shadows it with a macro.
pub(crate) fn builtin_function<'ast, 'arena, A>(
    ctx: &LintContext<'_, 'arena, A>,
    call: &'ast Call<'arena>,
) -> Option<&'ast Name<'arena>>
where
    A: Arena,
{
    match call.callee {
        Expression::Name(name) if !ctx.is_imported(name.name) => Some(name),
        _ => None,
    }
}

/// The `include()` arguments that control context sharing and missing templates.
pub(crate) struct IncludeArguments<'ast, 'arena> {
    pub with_context: Option<&'ast Expression<'arena>>,
    pub ignore_missing: Option<&'ast Expression<'arena>>,
}

/// Finds the `with_context` and `ignore_missing` arguments of an `include()` call, by name
/// or by position. Positions after a spread argument are unknown.
pub(crate) fn include_arguments<'ast, 'arena>(call: &'ast Call<'arena>) -> IncludeArguments<'ast, 'arena> {
    let mut arguments = IncludeArguments { with_context: None, ignore_missing: None };
    let mut position = 0;
    let mut unknown_position = false;
    for argument in call.argument_list.arguments.iter() {
        let value = ungroup(argument.value());
        match argument {
            Argument::Named(argument) => match normalize_argument_name(argument.name.value).as_ref() {
                b"withcontext" => arguments.with_context = Some(value),
                b"ignoremissing" => arguments.ignore_missing = Some(value),
                _ => {}
            },
            Argument::Positional(argument) => {
                if argument.ellipsis.is_some() {
                    unknown_position = true;
                } else if !unknown_position {
                    match position {
                        2 => arguments.with_context = Some(value),
                        3 => arguments.ignore_missing = Some(value),
                        _ => {}
                    }

                    position += 1;
                }
            }
        }
    }

    arguments
}

/// The nodes that declare blocks and macros for a template or an `embed` body, in source order.
///
/// Expressions cannot declare anything and are skipped, as are nested `embed` bodies, which
/// declare their own blocks and macros.
pub(crate) fn declarations<'ast, 'arena>(root: Node<'ast, 'arena>) -> Vec<Node<'ast, 'arena>> {
    let mut declarations = Vec::new();
    let mut stack = Vec::new();
    root.visit_children(|child| stack.push(child));
    stack.reverse();

    while let Some(node) = stack.pop() {
        if matches!(node, Node::Expression(_) | Node::Embed(_)) {
            continue;
        }

        if matches!(node, Node::Block(_) | Node::Macro(_)) {
            declarations.push(node);
        }

        let start = stack.len();
        node.visit_children(|child| stack.push(child));
        stack[start..].reverse();
    }

    declarations
}
