//! Shared completion item construction, ranking, and result limits.

use mago_bytes::BytesDisplay;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::display::NameDisplay;
use mago_codex::ttype::union::TUnion;
use mago_database::Database;

use crate::Server;
use crate::documentation::Documentation;
use crate::domain::CompletionAlternative;
use crate::domain::CompletionDetail;
use crate::domain::CompletionDocumentation;
use crate::domain::CompletionEntry;
use crate::domain::CompletionInsert;
use crate::domain::CompletionKind;
use crate::domain::SourceDocumentation;
use crate::lookup::DocblockAnchor;

use super::CANDIDATE_LIMIT;
use super::rank::Rank;
use super::rank::compare_names;
use super::rank::preselects;

pub(super) fn finalize(mut ranked: Vec<(Rank, CompletionEntry)>) -> Vec<CompletionEntry> {
    ranked.sort_unstable_by(|(left_rank, left), (right_rank, right)| {
        left_rank
            .cmp(right_rank)
            .then_with(|| compare_names(left.label.as_bytes(), right.label.as_bytes()))
            .then_with(|| (left.kind as u8).cmp(&(right.kind as u8)))
    });
    ranked.truncate(CANDIDATE_LIMIT);
    let preselect = preselects(ranked.first().map(|(rank, _)| rank), ranked.get(1).map(|(rank, _)| rank));
    let mut entries: Vec<_> = ranked.into_iter().map(|(_, entry)| entry).collect();
    finish(&mut entries, preselect);
    entries
}

pub(super) fn finish(entries: &mut [CompletionEntry], preselect: bool) {
    for entry in entries.iter_mut() {
        entry.filter_text = Some(entry.label.clone());
    }
    if let Some(first) = entries.first_mut() {
        first.preselect = preselect;
    }
}

pub(super) fn document(server: &Server, items: &mut [CompletionEntry]) {
    let codebase = server.codebase();
    let mut documentation = Documentation::new(server);
    for item in items {
        if let Some(alternatives) = item.member_alternatives.take() {
            let alternatives: Vec<_> = alternatives
                .into_vec()
                .into_iter()
                .map(|alternative| {
                    let target = alternative.detail_source.and_then(|source| DetailTarget::resolve(codebase, source));
                    (alternative, target)
                })
                .collect();

            let names = NameDisplay::for_types(
                codebase,
                alternatives.iter().filter_map(|(_, target)| *target).flat_map(DetailTarget::types),
            );

            let mut rendered = None;
            let mut distinct = Vec::new();
            for (mut alternative, target) in alternatives {
                if let Some(source) = alternative.detail_source.take() {
                    let detail = target.map(|target| render(server.database(), &names, &source, target, &item.label));
                    alternative.description.declaration = declaration(source, detail.as_deref());
                    rendered.get_or_insert(detail);
                }
                if let Some(source) = alternative.documentation_source.take() {
                    alternative.description.documentation = source_document(server, &mut documentation, source);
                }
                if !distinct.iter().any(|entry: &CompletionAlternative| entry.description == alternative.description) {
                    distinct.push(alternative);
                }
            }
            distinct.sort_by(|a, b| {
                a.description
                    .scope
                    .cmp(&b.description.scope)
                    .then_with(|| a.description.declaration.cmp(&b.description.declaration))
            });
            item.documentation_source = None;
            if distinct.len() == 1 {
                if let Some(detail) = rendered {
                    item.detail = detail;
                    item.detail_source = None;
                }

                item.documentation = distinct.pop().and_then(|entry| entry.description.documentation);
            } else {
                item.detail_source = None;
                item.detail = Some(
                    distinct
                        .iter()
                        .map(|entry| {
                            format!(
                                "{}: {}",
                                entry.description.scope.as_deref().unwrap_or_default(),
                                entry.description.declaration
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" | "),
                );
                item.documentation = None;
                item.member_alternatives = Some(distinct.into_boxed_slice());
            }
        }

        if let Some(source) = item.detail_source.take() {
            item.detail = DetailTarget::resolve(codebase, source).map(|target| {
                let names = NameDisplay::for_types(codebase, target.types());
                render(server.database(), &names, &source, target, &item.label)
            });
        }
        let Some(source) = item.documentation_source else { continue };
        let document = source_document(server, &mut documentation, source);
        if document.is_some() {
            item.documentation = document;
        }
    }
}

#[derive(Clone, Copy)]
enum DetailTarget<'code> {
    Signature(&'code FunctionLikeMetadata),
    Type(&'code TUnion),
}

impl<'code> DetailTarget<'code> {
    fn resolve(codebase: &'code CodebaseMetadata, source: CompletionDetail) -> Option<Self> {
        let property = match source {
            CompletionDetail::Function { name } => {
                return codebase.get_function_like(&FunctionLikeIdentifier::Function(name)).map(Self::Signature);
            }
            CompletionDetail::Method { method, .. } => return codebase.get_method_by_id(&method).map(Self::Signature),
            CompletionDetail::Property { class, name } => {
                codebase.get_declaring_property(class.as_bytes(), name.as_bytes())
            }
            CompletionDetail::MagicProperty { class, name } => {
                codebase.get_declaring_magic_property(class.as_bytes(), name.as_bytes())
            }
        }?;

        property.type_metadata.as_ref().map(|ty| Self::Type(&ty.type_union))
    }

    fn types(self) -> impl Iterator<Item = &'code TUnion> {
        let (function, ty) = match self {
            Self::Signature(function) => (Some(function), None),
            Self::Type(ty) => (None, Some(ty)),
        };

        function.into_iter().flat_map(super::super::signature::signature_types).chain(ty)
    }
}

fn render(
    database: &Database<'_>,
    names: &NameDisplay<'_>,
    source: &CompletionDetail,
    target: DetailTarget<'_>,
    label: &str,
) -> String {
    match target {
        DetailTarget::Signature(function) => {
            let name = match source {
                CompletionDetail::Method { name, .. } => name.as_bytes(),
                _ => label.as_bytes(),
            };

            render_signature(database, names, function, name)
        }
        DetailTarget::Type(ty) => names.union(ty),
    }
}

fn declaration(source: CompletionDetail, detail: Option<&str>) -> String {
    match source {
        CompletionDetail::Function { name } | CompletionDetail::Method { name, .. } => {
            detail.map_or_else(|| format!("function {name}"), |detail| format!("function {detail}"))
        }
        CompletionDetail::Property { name, .. } | CompletionDetail::MagicProperty { name, .. } => {
            detail.map_or_else(|| name.to_string(), |ty| format!("{ty} {name}"))
        }
    }
}

fn source_document(
    server: &Server,
    documentation: &mut Documentation<'_>,
    source: CompletionDocumentation,
) -> Option<SourceDocumentation> {
    match source {
        CompletionDocumentation::Callable(key) => documentation.for_callable(key),
        CompletionDocumentation::Declaration(span) => {
            documentation.for_declaration(span, DocblockAnchor::DeclarationStart)
        }
        CompletionDocumentation::DeclarationName(span) => {
            documentation.for_declaration(span, DocblockAnchor::DeclarationName)
        }
        CompletionDocumentation::Parameter { callable, name } => {
            documentation.for_callable_parameter(callable, name.as_bytes())
        }
        CompletionDocumentation::MagicProperty { class, name } => {
            let property = server.codebase().get_declaring_magic_property(class.as_bytes(), name.as_bytes())?;
            documentation.for_magic_property(class, property)
        }
        CompletionDocumentation::Property { class, name } => {
            let codebase = server.codebase();
            codebase
                .get_declaring_property_class(class.as_bytes(), name.as_bytes())
                .zip(codebase.get_declaring_property(class.as_bytes(), name.as_bytes()))
                .and_then(|(owner, property)| documentation.for_property(owner, property))
        }
    }
}

pub(super) fn function_item(meta: &FunctionLikeMetadata, display: &[u8], insert: &[u8]) -> CompletionEntry {
    CompletionEntry {
        detail_source: Some(CompletionDetail::Function { name: meta.name }),
        documentation_source: meta
            .has_docblock
            .then_some(CompletionDocumentation::Callable(FunctionLikeIdentifier::Function(meta.name))),
        insert: Some(CompletionInsert::Call(String::from_utf8_lossy(insert).into_owned())),
        deprecated: meta.flags.is_deprecated(),
        ..CompletionEntry::new(String::from_utf8_lossy(display).into_owned(), CompletionKind::Function)
    }
}

fn render_signature(
    database: &Database<'_>,
    names: &NameDisplay<'_>,
    meta: &FunctionLikeMetadata,
    name: &[u8],
) -> String {
    use std::fmt::Write;
    let mut sig = String::with_capacity(64);
    if meta.flags.is_by_reference() {
        sig.push('&');
    }
    let _ = write!(sig, "{}", BytesDisplay(name));
    sig.push('(');
    let mut first = true;
    for p in &meta.parameters {
        if !first {
            sig.push_str(", ");
        }

        first = false;
        let displayed_type = p.type_metadata.as_ref().map(|ty| names.union(&ty.type_union));
        super::super::signature::append_parameter(&mut sig, database, p, displayed_type.as_deref().map(str::as_bytes));
    }

    sig.push(')');
    if let Some(rt) = &meta.return_type_metadata {
        sig.push_str(": ");
        sig.push_str(&names.union(&rt.type_union));
    }

    sig
}

/// Build a class completion. `resolves_here` is true when typing the short name
/// at the cursor already resolves to this class (it is imported or lives in the
/// current namespace); otherwise the item inserts the leading-`\` FQCN.
pub(super) fn class_item(meta: &ClassLikeMetadata, short: &[u8], resolves_here: bool) -> CompletionEntry {
    let fqcn = meta.original_name.as_bytes();
    let insert_text = (!resolves_here).then(|| {
        let mut text = String::with_capacity(fqcn.len() + 1);
        text.push('\\');
        text.push_str(&String::from_utf8_lossy(fqcn));
        text
    });

    CompletionEntry {
        insert: insert_text.map(CompletionInsert::Text),
        namespace: Some(class_namespace(meta)),
        documentation: Some(class_documentation(meta)),
        documentation_source: Some(CompletionDocumentation::Declaration(meta.span)),
        deprecated: meta.flags.is_deprecated(),
        ..CompletionEntry::new(String::from_utf8_lossy(short).into_owned(), class_item_kind(meta))
    }
}

pub(super) fn qualified_class_item(meta: &ClassLikeMetadata, display: &[u8]) -> CompletionEntry {
    CompletionEntry {
        namespace: Some(class_namespace(meta)),
        documentation: Some(class_documentation(meta)),
        documentation_source: Some(CompletionDocumentation::Declaration(meta.span)),
        deprecated: meta.flags.is_deprecated(),
        ..CompletionEntry::new(String::from_utf8_lossy(display).into_owned(), class_item_kind(meta))
    }
}

fn class_namespace(meta: &ClassLikeMetadata) -> String {
    let name = meta.original_name.as_bytes();
    let namespace = memchr::memrchr(b'\\', name).map_or(b"\\".as_slice(), |index| &name[..index]);
    String::from_utf8_lossy(namespace).into_owned()
}

fn class_item_kind(meta: &ClassLikeMetadata) -> CompletionKind {
    use mago_codex::symbol::SymbolKind as M;
    match meta.kind {
        M::Class => CompletionKind::Class,
        M::Interface => CompletionKind::Interface,
        M::Trait => CompletionKind::Class,
        M::Enum => CompletionKind::Enum,
    }
}

fn class_documentation(meta: &ClassLikeMetadata) -> SourceDocumentation {
    SourceDocumentation { text: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(), tags: vec![] }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::path::Path;
    use std::sync::Arc;

    use mago_codex::identifier::method::MethodIdentifier;
    use mago_codex::reference::SymbolReferences;
    use mago_database::DatabaseConfiguration;
    use mago_database::file::File;
    use mago_php_version::PHPVersion;
    use mago_word::word;

    use super::*;
    use crate::domain::HoverEntry;

    fn server(source: &str) -> Server {
        let mut database = Database::new(DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]));
        database.add(File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into()));
        let mut server = Server::new(
            database,
            CodebaseMetadata::new(),
            SymbolReferences::new(),
            crate::Settings {
                php_version: PHPVersion::PHP85,
                features: crate::Features { analyzer: true, linter: false, formatter: false },
                parser: mago_syntax::settings::ParserSettings::default(),
                analyzer: mago_analyzer::settings::Settings::default(),
                linter: mago_linter::settings::Settings::default(),
                formatter: mago_formatter::settings::FormatSettings::default(),
                plugin_registry: Arc::new(mago_analyzer::plugin::PluginRegistry::new()),
            },
        );
        server.analyze().unwrap();
        server
    }

    #[test]
    fn function_details_wait_for_the_final_items() {
        let server = server("<?php namespace App; function take(int $value): int { return $value; }");
        let function = server.codebase().get_function(br"App\take").unwrap();
        let mut items = [function_item(function, b"take", br"\App\take")];
        assert_eq!(items[0].detail, None);
        document(&server, &mut items);
        assert_eq!(items[0].detail.as_deref(), Some("take(int $value): int"));
    }

    #[test]
    fn collapsed_alternatives_supply_their_rendered_detail() {
        let server = server(
            "<?php abstract class Base { public function take(int $value): int { return $value; } }
            final class Left extends Base {} final class Right extends Base {}",
        );
        let alternative = CompletionAlternative {
            description: HoverEntry::in_class(b"Base", String::new()),
            documentation_source: None,
            detail_source: Some(CompletionDetail::Method {
                method: MethodIdentifier::new(word("Base"), word("take")),
                name: word("take"),
            }),
        };
        let mut items = [CompletionEntry {
            member_alternatives: Some(vec![alternative.clone(), alternative].into_boxed_slice()),
            ..CompletionEntry::new("take".into(), CompletionKind::Method)
        }];
        document(&server, &mut items);
        assert_eq!(items[0].detail.as_deref(), Some("take(int $value): int"));
        assert!(items[0].member_alternatives.is_none());
    }
}
