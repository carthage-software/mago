//! Global and qualified symbol candidates, imports, and namespace ranking.
//!
//! Candidates borrow their metadata while they are ranked. Entries are built
//! only for the best [`MAX_RESULTS`] of them.

use std::cmp::Ordering;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::constant::ConstantMetadata;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_database::Database;
use mago_names::is_never_namespaced_function;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_php_version::PHPVersion;

use crate::domain::CompletionDocumentation;
use crate::domain::CompletionEntry;
use crate::domain::CompletionInsert;
use crate::domain::CompletionKind;
use crate::matcher;
use crate::semantic_index::ImportBinding;
use crate::semantic_index::SemanticIndex;
use crate::semantic_index::SymbolKind;
use crate::symbol_resolution::is_hidden_class_like;

use super::MAX_RESULTS;
use super::context::NameIntent;
use super::items::class_item;
use super::items::finish;
use super::items::function_item;
use super::items::qualified_class_item;
use super::rank::Origin;
use super::rank::Rank;
use super::rank::RankContext;
use super::rank::compare_names;
use super::rank::preselects;

#[derive(Clone, Copy)]
enum Source<'a> {
    Class(&'a ClassLikeMetadata),
    Function(&'a FunctionLikeMetadata),
    Constant(&'a ConstantMetadata),
}

impl Source<'_> {
    const fn order(self) -> u8 {
        match self {
            Self::Class(_) => 0,
            Self::Function(_) => 1,
            Self::Constant(_) => 2,
        }
    }
}

struct Candidate<'a> {
    rank: Rank,
    name: &'a [u8],
    fqn: &'a [u8],
    source: Source<'a>,
}

impl Candidate<'_> {
    fn order(&self, other: &Self) -> Ordering {
        self.rank
            .cmp(&other.rank)
            .then_with(|| self.name.len().cmp(&other.name.len()))
            .then_with(|| compare_names(self.name, other.name))
            .then_with(|| self.fqn.cmp(other.fqn))
            .then_with(|| self.source.order().cmp(&other.source.order()))
    }
}

pub(super) fn bare_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    version: PHPVersion,
    scope: &NamespaceScope,
    index: &SemanticIndex,
    offset: u32,
    prefix: &[u8],
    context: NameIntent,
    access_class: Option<&ClassLikeMetadata>,
    ranking: &RankContext<'_>,
) -> Vec<CompletionEntry> {
    let imports: Vec<&ImportBinding> =
        index.imports.iter().filter(|binding| binding.namespace_range.contains(offset)).collect();

    let mut candidates = Vec::new();
    for binding in &imports {
        let Some(matched) = matcher::score(prefix, &binding.alias) else { continue };
        let source = match binding.kind {
            SymbolKind::ClassLike if context.includes(NameKind::Default) => codebase
                .get_class_like(&binding.target)
                .filter(|meta| {
                    meta.is_available_in_version(version) && accepts_class(codebase, context, meta, access_class)
                })
                .map(|meta| (Source::Class(meta), meta.flags)),
            SymbolKind::Function if context.includes(NameKind::Function) => codebase
                .get_function(&binding.target)
                .filter(|meta| meta.is_available_in_version(version))
                .map(|meta| (Source::Function(meta), meta.flags)),
            SymbolKind::Constant if context.includes(NameKind::Constant) => codebase
                .get_constant(&binding.target)
                .filter(|meta| meta.is_available_in_version(version))
                .map(|meta| (Source::Constant(meta), meta.flags)),
            _ => None,
        };
        if let Some((source, flags)) = source {
            candidates.push(Candidate {
                rank: Rank::imported(matched, flags),
                name: &binding.alias,
                fqn: &binding.target,
                source,
            });
        }
    }

    if context.includes(NameKind::Default) {
        for meta in codebase.class_likes.values() {
            if !meta.is_available_in_version(version) || is_hidden_class_like(meta.name.as_bytes()) {
                continue;
            }

            let fqcn = meta.original_name.as_bytes();
            let short = local_name(fqcn);
            let Some(matched) = matcher::score(prefix, short) else {
                continue;
            };
            if !accepts_class(codebase, context, meta, access_class)
                || offered_by_import(&imports, SymbolKind::ClassLike, short, fqcn)
            {
                continue;
            }

            let rank = ranking.symbol(matched, meta.flags, fqcn);
            candidates.push(Candidate { rank, name: short, fqn: fqcn, source: Source::Class(meta) });
        }
    }

    if context.includes(NameKind::Function) {
        for meta in codebase.function_likes.values() {
            if !matches!(meta.kind, FunctionLikeKind::Function) || !meta.is_available_in_version(version) {
                continue;
            }

            let fqn = meta.original_name.as_bytes();
            let local = local_name(fqn);
            let Some(matched) = matcher::score(prefix, local) else {
                continue;
            };
            if offered_by_import(&imports, SymbolKind::Function, local, fqn) {
                continue;
            }

            let rank = ranking.symbol(matched, meta.flags, fqn);
            candidates.push(Candidate { rank, name: local, fqn, source: Source::Function(meta) });
        }
    }

    if context.includes(NameKind::Constant) {
        for meta in codebase.constants.values() {
            if !meta.is_available_in_version(version) {
                continue;
            }
            let fqn = meta.name.as_bytes();
            let local = local_name(fqn);
            let Some(matched) = matcher::score(prefix, local) else {
                continue;
            };
            if offered_by_import(&imports, SymbolKind::Constant, local, fqn) {
                continue;
            }

            let rank = ranking.symbol(matched, meta.flags, fqn);
            candidates.push(Candidate { rank, name: local, fqn, source: Source::Constant(meta) });
        }
    }

    let candidates = best(candidates);
    let mut entries: Vec<_> =
        candidates.iter().map(|candidate| bare_entry(database, codebase, scope, &imports, candidate)).collect();
    finish(&mut entries, preselected(&candidates));
    entries
}

fn bare_entry(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    scope: &NamespaceScope,
    imports: &[&ImportBinding],
    candidate: &Candidate<'_>,
) -> CompletionEntry {
    let name = candidate.name;
    let fqn = candidate.fqn;
    let imported = candidate.rank.origin == Origin::Scope;
    match candidate.source {
        Source::Class(meta) => {
            let resolves_here = imported || scope.resolve_str(NameKind::Default, name).0.eq_ignore_ascii_case(fqn);
            class_item(meta, name, resolves_here)
        }
        Source::Function(meta) if imported => function_item(database, codebase, meta, name, name),
        Source::Function(meta) => {
            let (resolved, aliased) = scope.resolve_str(NameKind::Function, name);
            let resolves_here = resolved.eq_ignore_ascii_case(fqn)
                || (!aliased && !fqn.contains(&b'\\') && codebase.get_function(&resolved).is_none());
            if resolves_here {
                function_item(database, codebase, meta, name, name)
            } else {
                function_item(database, codebase, meta, name, &[b"\\".as_slice(), fqn].concat())
            }
        }
        Source::Constant(meta) => {
            let mut entry = CompletionEntry {
                documentation_source: Some(CompletionDocumentation::DeclarationName(meta.span)),
                deprecated: meta.flags.is_deprecated(),
                ..CompletionEntry::new(String::from_utf8_lossy(name).into_owned(), CompletionKind::Constant)
            };
            if !imported {
                let (resolved, aliased) = scope.resolve_str(NameKind::Constant, name);
                let valid_import = !aliased
                    || imports
                        .iter()
                        .any(|binding| binding.kind == SymbolKind::Constant && binding.alias.as_ref() == name);
                let resolves_here = (valid_import && same_constant(&resolved, fqn))
                    || (!aliased && !fqn.contains(&b'\\') && codebase.get_constant(&resolved).is_none());
                let fqn = String::from_utf8_lossy(fqn);
                entry.insert = (!resolves_here).then(|| CompletionInsert::Text(format!("\\{fqn}")));
                entry.detail = Some(fqn.into_owned());
            }
            entry
        }
    }
}

pub(super) fn qualified_items(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    version: PHPVersion,
    scope: &NamespaceScope,
    qualifier: &[u8],
    prefix: &[u8],
    context: NameIntent,
    access_class: Option<&ClassLikeMetadata>,
    ranking: &RankContext<'_>,
) -> Vec<CompletionEntry> {
    let import = matches!(context, NameIntent::Import(_));
    let want_prefix = if qualifier.is_empty() {
        Vec::new()
    } else {
        let mut want = if import {
            qualifier.strip_prefix(b"\\").unwrap_or(qualifier).to_ascii_lowercase()
        } else {
            scope.resolve_str(NameKind::Default, qualifier).0.to_ascii_lowercase()
        };
        want.push(b'\\');
        want
    };

    let mut candidates = Vec::new();
    if context.includes(NameKind::Default) {
        for (key, meta) in &codebase.class_likes {
            if !meta.is_available_in_version(version)
                || is_hidden_class_like(key.as_bytes())
                || !key.as_bytes().starts_with(&want_prefix)
            {
                continue;
            }

            let fqcn = meta.original_name.as_bytes();
            let relative = &fqcn[want_prefix.len().min(fqcn.len())..];
            let Some(matched) = matcher::score(prefix, relative) else {
                continue;
            };
            if !accepts_class(codebase, context, meta, access_class) {
                continue;
            }

            let rank = ranking.qualified(matched, meta.flags, fqcn, relative);
            candidates.push(Candidate { rank, name: relative, fqn: fqcn, source: Source::Class(meta) });
        }
    }
    if context.includes(NameKind::Function) {
        for meta in codebase.function_likes.values() {
            if !matches!(meta.kind, FunctionLikeKind::Function) || !meta.is_available_in_version(version) {
                continue;
            }
            let fqn = meta.original_name.as_bytes();
            if (import && is_never_namespaced_function(fqn))
                || !fqn.get(..want_prefix.len()).is_some_and(|head| head.eq_ignore_ascii_case(&want_prefix))
            {
                continue;
            }
            let relative = &fqn[want_prefix.len()..];
            if let Some(matched) = matcher::score(prefix, relative) {
                let rank = ranking.qualified(matched, meta.flags, fqn, relative);
                candidates.push(Candidate { rank, name: relative, fqn, source: Source::Function(meta) });
            }
        }
    }
    if context.includes(NameKind::Constant) {
        for meta in codebase.constants.values() {
            if !meta.is_available_in_version(version) {
                continue;
            }
            let fqn = meta.name.as_bytes();
            if !fqn.get(..want_prefix.len()).is_some_and(|head| head.eq_ignore_ascii_case(&want_prefix)) {
                continue;
            }
            let relative = &fqn[want_prefix.len()..];
            if let Some(matched) = matcher::score(prefix, relative) {
                let rank = ranking.qualified(matched, meta.flags, fqn, relative);
                candidates.push(Candidate { rank, name: relative, fqn, source: Source::Constant(meta) });
            }
        }
    }

    let candidates = best(candidates);
    let mut entries: Vec<_> = candidates
        .iter()
        .map(|candidate| match candidate.source {
            Source::Class(meta) => qualified_class_item(meta, candidate.name),
            Source::Function(meta) => {
                let mut item = function_item(database, codebase, meta, candidate.name, candidate.name);
                if import {
                    item.insert = Some(CompletionInsert::Text(String::from_utf8_lossy(candidate.name).into_owned()));
                }
                item
            }
            Source::Constant(meta) => CompletionEntry {
                documentation_source: Some(CompletionDocumentation::DeclarationName(meta.span)),
                deprecated: meta.flags.is_deprecated(),
                ..CompletionEntry::new(String::from_utf8_lossy(candidate.name).into_owned(), CompletionKind::Constant)
            },
        })
        .collect();
    finish(&mut entries, preselected(&candidates));
    entries
}

fn best(mut candidates: Vec<Candidate<'_>>) -> Vec<Candidate<'_>> {
    if candidates.len() > MAX_RESULTS {
        candidates.select_nth_unstable_by(MAX_RESULTS, Candidate::order);
        candidates.truncate(MAX_RESULTS);
    }
    candidates.sort_unstable_by(Candidate::order);
    candidates
}

fn preselected(candidates: &[Candidate<'_>]) -> bool {
    preselects(candidates.first().map(|candidate| &candidate.rank), candidates.get(1).map(|candidate| &candidate.rank))
}

fn offered_by_import(imports: &[&ImportBinding], kind: SymbolKind, local: &[u8], fqn: &[u8]) -> bool {
    imports.iter().any(|binding| {
        binding.kind == kind
            && binding.target.len() == fqn.len()
            && if kind == SymbolKind::Constant {
                binding.alias.as_ref() == local && same_constant(&binding.target, fqn)
            } else {
                binding.alias.eq_ignore_ascii_case(local) && binding.target.eq_ignore_ascii_case(fqn)
            }
    })
}

fn accepts_class(
    codebase: &CodebaseMetadata,
    intent: NameIntent,
    meta: &ClassLikeMetadata,
    access_class: Option<&ClassLikeMetadata>,
) -> bool {
    match intent {
        NameIntent::Declaration => false,
        NameIntent::Instantiation => {
            meta.kind.is_class() && !meta.flags.is_abstract() && constructor_visible(codebase, meta, access_class)
        }
        NameIntent::ClassExtends => meta.kind.is_class() && !meta.flags.is_final(),
        NameIntent::Interface => meta.kind.is_interface(),
        NameIntent::Trait => meta.kind.is_trait(),
        NameIntent::NativeType(context) if context.position == super::native_type_context::Position::Catch => {
            codebase.is_instance_of(meta.name.as_bytes(), b"Throwable")
        }
        NameIntent::Type | NameIntent::NativeType(_) => !meta.kind.is_trait(),
        NameIntent::Expression | NameIntent::Import(_) => true,
    }
}

fn constructor_visible(
    codebase: &CodebaseMetadata,
    class: &ClassLikeMetadata,
    access_class: Option<&ClassLikeMetadata>,
) -> bool {
    let constructor = mago_word::word("__construct");
    let Some(declaring) = class.declaring_method_ids.get(&constructor) else { return true };
    let Some(method) = codebase.get_method_by_id(declaring).and_then(|method| method.method_metadata.as_ref()) else {
        return true;
    };
    // Trait methods belong to the using class for access checks. An inherited
    // constructor keeps that owner, including its trait visibility changes.
    let owner = class.appearing_method_ids.get(&constructor).unwrap_or(declaring).get_class_name();
    let Some(owner) = codebase.get_class_like_by_name(owner) else { return false };
    let visibility = codebase.get_method_visibility(owner.name.as_bytes(), b"__construct").unwrap_or(method.visibility);
    super::members::visible(codebase, visibility, owner, access_class)
}

fn same_constant(left: &[u8], right: &[u8]) -> bool {
    local_name(left) == local_name(right) && namespace_of(left).eq_ignore_ascii_case(namespace_of(right))
}

fn local_name(full: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\\', full) {
        Some(i) => &full[i + 1..],
        None => full,
    }
}

fn namespace_of(fqcn: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\\', fqcn) {
        Some(i) => &fqcn[..i],
        None => &[],
    }
}
