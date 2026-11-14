use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_span::HasSpan;
use mago_twig_syntax::cst::{
    ArrowFunction, BlockBody, Cache, Expression, Filter, InterpolatedString, MethodCall, Name, StringConcatenation,
    Test, WithExpressionClause,
};
use mago_twig_syntax::walker::{Walker, walk_expression};

use crate::tag::TagContents;

const MAX_EXPRESSION_DEPTH: usize = 128;

pub(super) struct ExpressionSafety<'arena, A: Arena> {
    pending: Vec<'arena, (&'arena Expression<'arena>, usize), A>,
    depth: usize,
    safe: bool,
}

impl<'arena, A: Arena> ExpressionSafety<'arena, A> {
    pub fn new(arena: &'arena A) -> Self {
        Self { pending: Vec::new_in(arena), depth: 0, safe: true }
    }

    pub fn can_format(&mut self, expression: &'arena Expression<'arena>) -> bool {
        self.pending.clear();
        self.pending.push((expression, 0));
        self.safe = true;
        while let Some((expression, depth)) = self.pending.pop() {
            if depth >= MAX_EXPRESSION_DEPTH || !self.safe {
                return false;
            }

            self.depth = depth;
            walk_expression(&Inspect, expression, self);
        }

        self.safe
    }
}

struct Inspect;

impl<'arena, A: Arena> Walker<'arena, 'arena, ExpressionSafety<'arena, A>> for Inspect {
    fn walk_expression(&self, expression: &'arena Expression<'arena>, context: &mut ExpressionSafety<'arena, A>) {
        context.pending.push((expression, context.depth + 1));
    }

    fn walk_in_name(&self, name: &'arena Name<'arena>, context: &mut ExpressionSafety<'arena, A>) {
        context.safe &= !ambiguous_name(name.name);
    }

    fn walk_in_test(&self, test: &'arena Test<'arena>, context: &mut ExpressionSafety<'arena, A>) {
        context.safe &= !ambiguous_name(test.name.value)
            && test.second_word.is_none_or(|word| {
                !ambiguous_name(word.value)
                    && !starts_compound_operator(word.value)
                    && !ambiguous_words(test.name.value, word.value)
            });
    }

    fn walk_in_filter(&self, filter: &'arena Filter<'arena>, context: &mut ExpressionSafety<'arena, A>) {
        context.safe &= !ambiguous_name(filter.name.value);
    }

    fn walk_in_method_call(&self, call: &'arena MethodCall<'arena>, context: &mut ExpressionSafety<'arena, A>) {
        context.safe &= !ambiguous_name(call.method.value);
    }

    fn walk_in_arrow_function(&self, arrow: &'arena ArrowFunction<'arena>, context: &mut ExpressionSafety<'arena, A>) {
        context.safe &= !arrow.parameters.iter().any(|parameter| ambiguous_name(parameter.value));
    }

    fn walk_interpolated_string(&self, _: &'arena InterpolatedString<'arena>, _: &mut ExpressionSafety<'arena, A>) {}

    fn walk_string_concatenation(&self, _: &'arena StringConcatenation<'arena>, _: &mut ExpressionSafety<'arena, A>) {}
}

pub(super) fn ambiguous_name(name: &[u8]) -> bool {
    matches!(name, b"not" | b"in" | b"is" | b"and" | b"or" | b"xor" | b"matches")
}

fn ambiguous_words(first: &[u8], second: &[u8]) -> bool {
    matches!((first, second), (b"starts" | b"ends", b"with") | (b"has", b"some" | b"every"))
}

fn starts_compound_operator(name: &[u8]) -> bool {
    matches!(name, b"starts" | b"ends" | b"has")
}

fn ends_with_word(source: &[u8], word: &[u8]) -> bool {
    source.strip_suffix(word).is_some_and(|prefix| {
        prefix.last().is_none_or(|byte| byte.is_ascii() && !byte.is_ascii_alphanumeric() && *byte != b'_')
    })
}

fn ambiguous_with(template: &Expression<'_>, clause: Option<&WithExpressionClause<'_>>, source: &[u8]) -> bool {
    clause.is_some_and(|clause| {
        if clause.with_keyword.span.end != clause.variables.span().start {
            return false;
        }

        let prefix = &source[..template.span().end.offset as usize];
        ends_with_word(prefix, b"starts") || ends_with_word(prefix, b"ends")
    })
}

pub(super) fn preserves_header_names(kind: TagContents<'_>, source: &[u8]) -> bool {
    let ambiguous = match kind {
        TagContents::For(node) => node.targets.iter().any(|target| ambiguous_name(target.value)),
        TagContents::Set(node) => node.names.iter().any(|name| ambiguous_name(name.value)),
        TagContents::Block(node) => {
            ambiguous_name(node.name.value)
                || matches!(node.body, BlockBody::Short(_)) && starts_compound_operator(node.name.value)
        }
        TagContents::Include(node) => ambiguous_with(&node.template, node.with_clause.as_ref(), source),
        TagContents::Embed(node) => ambiguous_with(&node.template, node.with_clause.as_ref(), source),
        TagContents::Use(node) => node
            .aliases
            .iter()
            .any(|alias| ambiguous_name(alias.from.value) || alias.to.is_some_and(|name| ambiguous_name(name.value))),
        TagContents::Import(node) => ambiguous_name(node.alias.value),
        TagContents::From(node) => node
            .names
            .iter()
            .any(|name| ambiguous_name(name.from.value) || name.to.is_some_and(|name| ambiguous_name(name.value))),
        TagContents::Macro(node) => {
            ambiguous_name(node.name.value) || node.arguments.iter().any(|argument| ambiguous_name(argument.name.value))
        }
        TagContents::Apply(node) => node.filters.iter().any(|filter| ambiguous_name(filter.name.value)),
        TagContents::Deprecated(node) => node.options.iter().any(|option| ambiguous_name(option.name.value)),
        TagContents::Types(node) => node.declarations.iter().any(|declaration| ambiguous_name(declaration.name.value)),
        TagContents::Guard(node) => {
            ambiguous_name(node.name.value)
                || node
                    .second_word
                    .is_some_and(|word| ambiguous_name(word.value) || ambiguous_words(node.name.value, word.value))
        }
        _ => false,
    };

    !ambiguous
}

pub(super) fn preserves_cache_options(source: &[u8], cache: &Cache<'_>) -> bool {
    let mut end = cache.key.span().end.offset as usize;
    for option in cache.options() {
        if !source[end..option.keyword.span.start.offset as usize].iter().all(u8::is_ascii_whitespace) {
            return false;
        }
        end = option.right_parenthesis.end.offset as usize;
    }

    source[end..cache.close_tag.start.offset as usize].iter().all(u8::is_ascii_whitespace)
}
