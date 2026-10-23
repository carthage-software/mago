//! Type aliases and templates that take precedence over PHPDoc class names.

use crate::Range;
use crate::lookup::is_trivia;
use foldhash::{HashMap, HashSet};
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::{FunctionLikeKind, FunctionLikeMetadata};
use mago_database::file::FileId;
use mago_phpdoc_syntax::cst::{Document, TagValue};
use mago_span::HasSpan;
use mago_syntax::token::{Token, TokenKind};
use mago_word::word;

use super::{Name, NameKind};

#[derive(Default)]
pub(super) struct Bindings<'codebase> {
    classes: Vec<(Range, &'codebase ClassLikeMetadata)>,
    functions: Vec<(Range, &'codebase FunctionLikeMetadata)>,
}

pub(super) fn by_file<'codebase>(
    codebase: &'codebase CodebaseMetadata,
    candidates: &HashSet<FileId>,
) -> HashMap<FileId, Bindings<'codebase>> {
    let mut files: HashMap<FileId, Bindings<'_>> = HashMap::default();
    for class in codebase.class_likes.values().filter(|class| candidates.contains(&class.span.file_id)) {
        let span = class.span;
        files.entry(span.file_id).or_default().classes.push((Range::new(span.start.offset, span.end.offset), class));
    }
    // Magic and synthesized methods use class or doc spans, not a source function scope.
    for function in codebase.function_likes.values().filter(|function| {
        candidates.contains(&function.span.file_id)
            && !function.flags.is_magic_method()
            && (function.kind != FunctionLikeKind::Method
                || function.name_span.is_some_and(|span| span != function.span))
    }) {
        let span = function.span;
        files
            .entry(span.file_id)
            .or_default()
            .functions
            .push((Range::new(span.start.offset, span.end.offset), function));
    }
    files
}

impl Bindings<'_> {
    pub(super) fn include_docblocks(&mut self, tokens: &[Token<'_>]) {
        for range in
            self.classes.iter_mut().map(|(range, _)| range).chain(self.functions.iter_mut().map(|(range, _)| range))
        {
            let end = tokens.partition_point(|token| token.start.offset < range.start);
            let mut covered = range.start;
            for token in tokens[..end].iter().rev() {
                if token.start.offset + token.value.len() as u32 != covered || !is_trivia(token.kind) {
                    break;
                }
                covered = token.start.offset;
                if token.kind == TokenKind::DocBlockComment {
                    range.start = covered;
                    break;
                }
            }
        }
    }

    pub(super) fn is_type_bound(&self, name: &Name<'_>, value: &[u8]) -> bool {
        let NameKind::Type { generic } = name.kind else { return false };
        let offset = name.identifier.span.start.offset;
        let class = self
            .classes
            .iter()
            .filter(|(range, _)| range.contains(offset))
            .min_by_key(|(range, _)| range.end - range.start);
        let function = self
            .functions
            .iter()
            .filter(|(range, _)| {
                range.contains(offset) && class.is_none_or(|(class_range, _)| range.start >= class_range.start)
            })
            .min_by_key(|(range, _)| range.end - range.start);
        if let Some((_, function)) = function {
            return function.type_resolution_context.as_ref().is_some_and(|context| {
                let name = word(value);
                context.get_imported_type_alias(name).is_some()
                    || class.is_some() && context.has_type_alias(name)
                    || !generic && context.has_template_definition(name)
            });
        }
        // Class declarations and properties resolve aliases from their own class only.
        class.is_some_and(|(_, class)| {
            class.imported_type_aliases.keys().any(|alias| alias.as_bytes() == value)
                || class.type_aliases.keys().any(|alias| alias.as_bytes() == value)
                || !generic && class.template_types.keys().any(|template| template.as_bytes() == value)
        })
    }
}

pub(super) fn is_method_template(name: &Name<'_>, value: &[u8], document: &Document<'_>) -> bool {
    matches!(name.kind, NameKind::Type { generic: false })
        && method_templates(document, name.identifier.span.start.offset).any(|template| template == value)
}

pub(super) fn uncertain_template_bound(name: &Name<'_>, document: &Document<'_>) -> bool {
    if !matches!(name.kind, NameKind::Type { .. }) {
        return false;
    }
    let mut templates = Vec::new();
    for tag in document.tags() {
        match &tag.value {
            TagValue::Template(template) => templates.push(template),
            TagValue::Method(method) => {
                if let Some(parameters) = method.templates {
                    templates.extend(parameters.entries.iter().map(|entry| &entry.template));
                }
            }
            _ => {}
        }
    }
    // A bound may use an outer binding before its own template enters the type context.
    templates.iter().any(|template| template.name.value == name.identifier.value)
        && templates.iter().any(|template| {
            template
                .bound
                .map(|bound| bound.r#type)
                .into_iter()
                .chain(template.default.map(|default| default.r#type))
                .any(|ty| ty.span().contains(&name.identifier.span.start))
        })
}

fn method_templates<'document, 'arena>(
    document: &'document Document<'arena>,
    offset: u32,
) -> impl Iterator<Item = &'arena [u8]> + 'document
where
    'arena: 'document,
{
    document
        .tags()
        .filter(move |tag| {
            let span = tag.span();
            span.start.offset <= offset && offset < span.end.offset
        })
        .filter_map(|tag| match tag.value {
            TagValue::Method(method) => method.templates,
            _ => None,
        })
        .flat_map(|templates| templates.entries.iter().map(|entry| entry.template.name.value))
}
