//! Source PHPDoc attached to a declaration.

use foldhash::HashMap;
use mago_allocator::LocalArena;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::property::PropertyMetadata;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::{ClassLikeMember, Node};
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_word::Word;

use crate::Server;
use crate::domain::SourceDocumentation;
use crate::lookup::DocblockAnchor;
use crate::lookup::DocblockSource;

mod tags;

/// Reuse source snapshots and tokens while documenting several symbols.
pub(crate) struct Documentation<'server> {
    server: &'server Server,
    first: Option<(FileId, DocblockSource)>,
    sources: HashMap<FileId, DocblockSource>,
    native_methods: HashMap<FileId, Vec<NativeMethod>>,
}

enum CallableSource {
    Declaration(Span),
    MagicMethod { owner: Span, name: Word },
}

struct NativeMethod {
    owner: Span,
    name: Span,
    declaration: Span,
}

pub(crate) fn for_declaration(server: &Server, span: Span, anchor: DocblockAnchor) -> Option<SourceDocumentation> {
    Documentation::new(server).for_declaration(span, anchor)
}

pub(crate) fn for_callable(server: &Server, key: FunctionLikeIdentifier) -> Option<SourceDocumentation> {
    Documentation::new(server).for_callable(key)
}

pub(crate) fn for_parameter(server: &Server, owner: Span, parameter: &[u8]) -> Option<SourceDocumentation> {
    Documentation::new(server).for_parameter(owner, parameter)
}

pub(crate) fn for_property(server: &Server, owner: Word, property: &PropertyMetadata) -> Option<SourceDocumentation> {
    Documentation::new(server).for_property(owner, property)
}

pub(crate) fn for_magic_property(
    server: &Server,
    owner: Word,
    property: &PropertyMetadata,
) -> Option<SourceDocumentation> {
    Documentation::new(server).for_magic_property(owner, property)
}

impl<'server> Documentation<'server> {
    pub(crate) fn new(server: &'server Server) -> Self {
        Self { server, first: None, sources: HashMap::default(), native_methods: HashMap::default() }
    }

    pub(crate) fn for_declaration(&mut self, span: Span, anchor: DocblockAnchor) -> Option<SourceDocumentation> {
        self.source_text(span, anchor).map(|text| tags::parse(text, span.file_id))
    }

    pub(crate) fn for_callable(&mut self, key: FunctionLikeIdentifier) -> Option<SourceDocumentation> {
        match self.callable_source(key)? {
            CallableSource::Declaration(span) => self.for_declaration(span, DocblockAnchor::DeclarationStart),
            CallableSource::MagicMethod { owner, name } => {
                let text = self.source_text(owner, DocblockAnchor::DeclarationStart)?;
                let text = tags::method_description(&text, owner.file_id, name.as_bytes())?;
                Some(SourceDocumentation { text, tags: vec![] })
            }
        }
    }

    pub(crate) fn for_callable_parameter(
        &mut self,
        key: FunctionLikeIdentifier,
        parameter: &[u8],
    ) -> Option<SourceDocumentation> {
        let CallableSource::Declaration(owner) = self.callable_source(key)? else { return None };
        self.for_parameter(owner, parameter)
    }

    fn callable_source(&mut self, key: FunctionLikeIdentifier) -> Option<CallableSource> {
        let codebase = self.server.codebase();
        let key =
            key.as_method_identifier().map_or(key, |method| codebase.get_declaring_method_identifier(&method).into());
        let callable = codebase.get_function_like(&key)?;
        if callable.flags.is_magic_method() {
            let FunctionLikeIdentifier::Method(class, _) = key else { return None };
            let owner = codebase.get_class_like_by_name(class)?.span;
            return Some(CallableSource::MagicMethod { owner, name: callable.original_name });
        }
        if callable.has_docblock {
            return Some(CallableSource::Declaration(callable.span));
        }
        let FunctionLikeIdentifier::Method(class, name) = key else { return None };
        let owner = codebase.get_class_like_by_name(class)?.span;
        if callable.span.file_id != owner.file_id || callable.span.end > owner.start {
            return None;
        }
        // A same-class @method can supply the signature while the native method owns its docs.
        self.native_method_span(owner, name.as_bytes()).map(CallableSource::Declaration)
    }

    fn native_method_span(&mut self, owner: Span, name: &[u8]) -> Option<Span> {
        let file = self.server.database().get(&owner.file_id).ok()?;
        let settings = self.server.parser_settings();
        let methods = self.native_methods.entry(owner.file_id).or_insert_with(|| {
            let arena = LocalArena::new();
            let program = parse_file_borrowed_with_settings(&arena, &file, settings);
            let mut methods = Vec::new();
            collect_native_methods(Node::Program(program), &mut methods);
            methods
        });
        methods.iter().find_map(|method| {
            (method.owner == owner
                && file
                    .contents
                    .get(method.name.start.offset as usize..method.name.end.offset as usize)?
                    .eq_ignore_ascii_case(name))
            .then_some(method.declaration)
        })
    }

    pub(crate) fn for_parameter(&mut self, owner: Span, parameter: &[u8]) -> Option<SourceDocumentation> {
        let text = self.source_text(owner, DocblockAnchor::DeclarationStart)?;
        let text = tags::parameter_description(&text, owner.file_id, parameter)?;
        Some(SourceDocumentation { text, tags: vec![] })
    }

    pub(crate) fn for_property(&mut self, owner: Word, property: &PropertyMetadata) -> Option<SourceDocumentation> {
        let anchor =
            if property.span.is_some() { DocblockAnchor::DeclarationStart } else { DocblockAnchor::DeclarationName };
        let direct = property.span.or(property.name_span).and_then(|span| self.for_declaration(span, anchor));
        if direct.is_some() || !property.flags.is_promoted_property() {
            return direct;
        }
        let constructor = self.server.codebase().get_declaring_method(owner.as_bytes(), b"__construct")?;
        let parameter = constructor
            .parameters
            .iter()
            .find(|parameter| Some(parameter.name_span) == property.name_span && parameter.name == property.name)?;
        self.for_parameter(constructor.span, parameter.name.0.as_bytes())
    }

    pub(crate) fn for_magic_property(
        &mut self,
        owner: Word,
        property: &PropertyMetadata,
    ) -> Option<SourceDocumentation> {
        let owner = self.server.codebase().get_class_like_by_name(owner)?.span;
        let text = self.source_text(owner, DocblockAnchor::DeclarationStart)?;
        let text = tags::property_description(
            &text,
            owner.file_id,
            property.name.0.as_bytes(),
            property.flags.is_writeonly(),
        )?;
        Some(SourceDocumentation { text, tags: vec![] })
    }

    fn source_text(&mut self, span: Span, anchor: DocblockAnchor) -> Option<String> {
        if self.first.is_none() {
            let file = self.server.database().get(&span.file_id).ok()?;
            self.first = Some((span.file_id, self.server.tokens.docblocks(file)));
        }
        let source = if let Some((id, source)) = &self.first
            && *id == span.file_id
        {
            source
        } else {
            match self.sources.entry(span.file_id) {
                std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let file = self.server.database().get(&span.file_id).ok()?;
                    entry.insert(self.server.tokens.docblocks(file))
                }
            }
        };
        let comment = source.before(span.start.offset, anchor)?;
        let body = comment.strip_prefix(b"/**")?.strip_suffix(b"*/")?;
        clean_comment(body)
    }
}

fn collect_native_methods(node: Node<'_, '_>, methods: &mut Vec<NativeMethod>) {
    let members = match node {
        Node::Class(class) => Some(&class.members),
        Node::AnonymousClass(class) => Some(&class.members),
        Node::Interface(class) => Some(&class.members),
        Node::Trait(class) => Some(&class.members),
        Node::Enum(class) => Some(&class.members),
        _ => None,
    };
    if let Some(members) = members {
        methods.extend(members.iter().filter_map(|member| match member {
            ClassLikeMember::Method(method) => {
                Some(NativeMethod { owner: node.span(), name: method.name.span, declaration: method.span() })
            }
            _ => None,
        }));
    }
    node.visit_children(|child| collect_native_methods(child, methods));
}

fn clean_comment(body: &[u8]) -> Option<String> {
    let body = String::from_utf8_lossy(body);
    let mut lines: Vec<_> = body
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                line.strip_prefix(' ').unwrap_or(line)
            } else {
                line.trim_start().strip_prefix('*').map_or(line, |rest| rest.strip_prefix(' ').unwrap_or(rest))
            }
        })
        .collect();
    if let Some(last) = lines.last_mut() {
        *last = last.strip_suffix(' ').unwrap_or(last);
    }
    let first = lines.iter().position(|line| !line.trim().is_empty())?;
    let last = lines.iter().rposition(|line| !line.trim().is_empty())?;
    Some(lines[first..=last].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_indented_example_stays_code() {
        let text = clean_comment(b"\n *     @return int\n ");
        assert_eq!(text.as_deref(), Some("    @return int"));
        assert!(text.is_some_and(|text| tags::parse(text, FileId::zero()).tags.is_empty()));
        assert_eq!(clean_comment(b" One line. ").as_deref(), Some("One line."));
    }
}
