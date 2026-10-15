//! `get_symbols`: ranked workspace-symbol search.
//!
//! Scores short names with the shared matcher and returns class-likes, free
//! functions, and global constants from project and library files, plus the
//! members of project class-likes. Builtins and synthetic names are skipped.
//! A query may qualify the name with a namespace (`Http\Client`, `\App\Kernel`)
//! or a class-like (`Client::send`, `Client::$timeout`), and `$name` searches
//! properties only.

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::symbol::SymbolKind as MagoSymbolKind;
use mago_span::Span as FileSpan;
use mago_word::empty_word;

use crate::Server;
use crate::domain::Range;
use crate::domain::SymbolKind;
use crate::domain::SymbolLocation;
use crate::domain::SymbolRank;
use crate::domain::WorkspaceSymbolItem;
use crate::matcher;
use crate::matcher::is_test_namespace;
use crate::symbol_resolution::is_hidden_class_like;

/// The most symbols one search returns.
pub const MAX_RESULTS: usize = 256;

const CLASS: u8 = 0;
const FUNCTION: u8 = 1;
const CONSTANT: u8 = 2;
const MEMBER: u8 = 3;

impl Server {
    /// Up to [`MAX_RESULTS`] matching symbols, best first. An empty query lists
    /// project top-level symbols alphabetically.
    #[must_use]
    pub fn get_symbols(&self, query: &str) -> Vec<WorkspaceSymbolItem> {
        let query = Query::parse(query);
        let codebase = self.codebase();
        let top_level = !query.member;
        let members = query.members();
        let mut hits = Hits::default();

        for meta in codebase.class_likes.values() {
            if !meta.flags.is_user_defined() || meta.flags.is_built_in() || is_hidden_class_like(meta.name.as_bytes()) {
                continue;
            }
            let fqcn = meta.original_name.as_bytes();
            if top_level {
                hits.add(query.top_level(meta.flags, fqcn, CLASS, || class_site(meta)));
            }
            if !members {
                continue;
            }
            let Some(container) = query.container(fqcn) else { continue };
            for (name, property) in &meta.properties {
                let Some(span) = property.name_span.or(property.span) else { continue };
                let Some(hit) = query.member(container, name.as_bytes(), SymbolKind::Property, span) else {
                    continue;
                };
                if meta.declaring_property_ids.get(name) == Some(&meta.name) {
                    hits.add(Some(hit));
                }
            }
            if query.property {
                continue;
            }
            for constant in meta.constants.values() {
                let mut span = constant.span;
                if span.file_id != meta.span.file_id
                    || span.start.offset < meta.span.start.offset
                    || span.end.offset > meta.span.end.offset
                {
                    continue;
                }
                span.end.offset = span.start.offset + constant.name.as_bytes().len() as u32;
                hits.add(query.member(container, constant.name.as_bytes(), SymbolKind::Constant, span));
            }
            for case in meta.enum_cases.values() {
                hits.add(query.member(container, case.name.as_bytes(), SymbolKind::EnumMember, case.name_span));
            }
            for name in &meta.methods {
                if !matcher::may_match(query.name, name.as_bytes()) {
                    continue;
                }
                let Some(method) = codebase.function_likes.get(&(meta.name, *name)) else { continue };
                let Some(span) = method.name_span else { continue };
                if !matches!(method.kind, FunctionLikeKind::Method) || !method.flags.is_user_defined() {
                    continue;
                }
                let kind = if name.as_bytes() == b"__construct" { SymbolKind::Constructor } else { SymbolKind::Method };
                hits.add(query.member(container, method.original_name.as_bytes(), kind, span));
            }
        }

        if !top_level {
            return hits.into_items();
        }

        let global = empty_word();
        for ((scope, name), meta) in &codebase.function_likes {
            if *scope == global
                && (hits.excludes(true) || query.may_name(name.as_bytes()))
                && matches!(meta.kind, FunctionLikeKind::Function)
                && !meta.flags.is_built_in()
                && !hits.excludes(!meta.flags.is_user_defined())
            {
                let site = || (SymbolKind::Function, meta.name_span.unwrap_or(meta.span));
                hits.add(query.top_level(meta.flags, meta.original_name.as_bytes(), FUNCTION, site));
            }
        }
        for meta in codebase.constants.values() {
            if !meta.flags.is_built_in() && !hits.excludes(!meta.flags.is_user_defined()) {
                let site = || (SymbolKind::Constant, meta.span);
                hits.add(query.top_level(meta.flags, meta.name.as_bytes(), CONSTANT, site));
            }
        }
        for (name, meta) in &codebase.class_likes {
            if hits.excludes(true) {
                break;
            }
            if query.may_name(name.as_bytes())
                && !is_hidden_class_like(name.as_bytes())
                && !meta.flags.is_user_defined()
                && !meta.flags.is_built_in()
            {
                hits.add(query.top_level(meta.flags, meta.original_name.as_bytes(), CLASS, || class_site(meta)));
            }
        }

        hits.into_items()
    }
}

struct Query<'a> {
    name: &'a [u8],
    qualifier: Option<&'a [u8]>,
    rooted: bool,
    member: bool,
    property: bool,
}

/// A class-like whose members a query searches.
#[derive(Clone, Copy)]
struct Container<'a> {
    name: &'a [u8],
    partial_qualifier: bool,
    test: bool,
}

impl<'a> Query<'a> {
    fn parse(query: &'a str) -> Self {
        let query = query.trim().as_bytes();
        let (rooted, query) = match query.strip_prefix(b"\\") {
            Some(rest) => (true, rest),
            None => (false, query),
        };
        if let Some(index) = memchr::memmem::rfind(query, b"::") {
            let name = &query[index + 2..];
            let property = name.strip_prefix(b"$");
            return Self {
                name: property.unwrap_or(name),
                qualifier: Some(&query[..index]),
                rooted,
                member: true,
                property: property.is_some(),
            };
        }
        if !rooted && let Some(name) = query.strip_prefix(b"$") {
            return Self { name, qualifier: None, rooted, member: true, property: true };
        }
        let (qualifier, name) = match memchr::memrchr(b'\\', query) {
            Some(index) => (Some(&query[..index]), &query[index + 1..]),
            None => (rooted.then_some(b"".as_slice()), query),
        };
        Self { name, qualifier, rooted, member: false, property: false }
    }

    fn browse(&self) -> bool {
        self.name.is_empty() && self.qualifier.is_none()
    }

    /// `false` when the short name of `fqn` cannot match, judged from the
    /// bytes of a lower-case key before its metadata is read.
    fn may_name(&self, fqn: &[u8]) -> bool {
        matcher::may_match(self.name, split_name(fqn).1)
    }

    fn members(&self) -> bool {
        self.member || (self.qualifier.is_none() && !self.name.is_empty())
    }

    fn top_level(
        &self,
        flags: MetadataFlags,
        fqn: &'a [u8],
        order: u8,
        site: impl FnOnce() -> (SymbolKind, FileSpan),
    ) -> Option<Hit<'a>> {
        let browse = self.browse();
        if browse && !flags.is_user_defined() {
            return None;
        }
        let (namespace, short) = split_name(fqn);
        let rank = if browse {
            SymbolRank {
                weak: false,
                library: false,
                test: false,
                partial_qualifier: false,
                matched: matcher::ANY,
                kind: CLASS,
            }
        } else {
            let partial_qualifier = match self.qualifier {
                Some(qualifier) => !qualifies(namespace, qualifier, self.rooted, false)?,
                None => false,
            };
            let matched = matcher::score(self.name, short)?;
            SymbolRank {
                weak: matched.is_weak(),
                library: !flags.is_user_defined(),
                test: is_test_namespace(namespace),
                partial_qualifier,
                matched,
                kind: order,
            }
        };
        let (kind, span) = site();
        Some(Hit { rank, short, container: None, name: fqn, kind, span })
    }

    /// The class-like `fqcn`, unless the qualifier names another one.
    fn container(&self, fqcn: &'a [u8]) -> Option<Container<'a>> {
        let partial_qualifier = match self.qualifier {
            Some(qualifier) => !qualifies(fqcn, qualifier, self.rooted, true)?,
            None => false,
        };
        Some(Container { name: fqcn, partial_qualifier, test: is_test_namespace(split_name(fqcn).0) })
    }

    fn member(&self, container: Container<'a>, name: &'a [u8], kind: SymbolKind, span: FileSpan) -> Option<Hit<'a>> {
        let short = name.strip_prefix(b"$").unwrap_or(name);
        let matched = matcher::score(self.name, short)?;
        let rank = SymbolRank {
            weak: matched.is_weak(),
            library: false,
            test: container.test,
            partial_qualifier: container.partial_qualifier,
            matched,
            kind: MEMBER,
        };
        Some(Hit { rank, short, container: Some(container.name), name, kind, span })
    }
}

/// The best hits so far. At most twice [`MAX_RESULTS`] are held; once that
/// many are seen, a hit that would sort after every kept one is dropped.
#[derive(Default)]
struct Hits<'a> {
    kept: Vec<Hit<'a>>,
    bounded: bool,
}

impl<'a> Hits<'a> {
    fn worst(&self) -> Option<&Hit<'a>> {
        self.bounded.then(|| &self.kept[MAX_RESULTS - 1])
    }

    /// `true` once no top-level symbol from a library (or from the project, when
    /// `library` is false) can be kept.
    fn excludes(&self, library: bool) -> bool {
        self.worst().is_some_and(|worst| (worst.rank.weak, worst.rank.library) < (false, library))
    }

    fn add(&mut self, hit: Option<Hit<'a>>) {
        let Some(hit) = hit else { return };
        if self.worst().is_some_and(|worst| hit.order(worst).is_ge()) {
            return;
        }
        self.kept.push(hit);
        if self.kept.len() == 2 * MAX_RESULTS {
            self.kept.select_nth_unstable_by(MAX_RESULTS - 1, Hit::order);
            self.kept.truncate(MAX_RESULTS);
            self.bounded = true;
        }
    }

    fn into_items(mut self) -> Vec<WorkspaceSymbolItem> {
        if self.kept.len() > MAX_RESULTS {
            self.kept.select_nth_unstable_by(MAX_RESULTS, Hit::order);
            self.kept.truncate(MAX_RESULTS);
        }
        self.kept.sort_unstable_by(Hit::order);
        self.kept.into_iter().map(Hit::into_item).collect()
    }
}

struct Hit<'a> {
    rank: SymbolRank,
    short: &'a [u8],
    container: Option<&'a [u8]>,
    name: &'a [u8],
    kind: SymbolKind,
    span: FileSpan,
}

impl Hit<'_> {
    fn order(&self, other: &Self) -> std::cmp::Ordering {
        self.rank
            .cmp(&other.rank)
            .then_with(|| {
                self.short.iter().map(u8::to_ascii_lowercase).cmp(other.short.iter().map(u8::to_ascii_lowercase))
            })
            .then_with(|| self.display().cmp(other.display()))
    }

    fn display(&self) -> impl Iterator<Item = &u8> {
        let separator: &[u8] = if self.container.is_some() { b"::" } else { b"" };
        self.container.unwrap_or_default().iter().chain(separator).chain(self.name)
    }

    fn into_item(self) -> WorkspaceSymbolItem {
        let name = String::from_utf8_lossy(self.name);
        let container = self.container.map(|container| String::from_utf8_lossy(container).into_owned());
        WorkspaceSymbolItem {
            name: match &container {
                Some(container) => format!("{container}::{name}"),
                None => name.into_owned(),
            },
            kind: self.kind,
            location: SymbolLocation {
                file: self.span.file_id,
                range: Range::new(self.span.start.offset, self.span.end.offset),
            },
            container,
            rank: self.rank,
        }
    }
}

/// `Some(exact)` when `qualifier` names whole segments of `scope`, ignoring
/// case. A rooted qualifier must start at the first segment, and a `class`
/// qualifier must end at the last one.
fn qualifies(scope: &[u8], qualifier: &[u8], rooted: bool, class: bool) -> Option<bool> {
    if scope.eq_ignore_ascii_case(qualifier) {
        return Some(true);
    }
    if qualifier.is_empty() {
        return (!rooted).then_some(false);
    }
    if class {
        let start = scope.len().checked_sub(qualifier.len())?;
        return (!rooted && start > 0 && scope[start - 1] == b'\\' && scope[start..].eq_ignore_ascii_case(qualifier))
            .then_some(false);
    }
    let mut start = 0;
    loop {
        let end = start + qualifier.len();
        if scope.get(start..end).is_some_and(|part| part.eq_ignore_ascii_case(qualifier))
            && scope.get(end).is_none_or(|byte| *byte == b'\\')
        {
            return Some(false);
        }
        if rooted {
            return None;
        }
        start += memchr::memchr(b'\\', &scope[start..])? + 1;
    }
}

fn split_name(fqn: &[u8]) -> (&[u8], &[u8]) {
    match memchr::memrchr(b'\\', fqn) {
        Some(index) => (&fqn[..index], &fqn[index + 1..]),
        None => (&[], fqn),
    }
}

fn class_site(meta: &ClassLikeMetadata) -> (SymbolKind, FileSpan) {
    (classlike_kind(meta.kind), meta.name_span.unwrap_or(meta.span))
}

fn classlike_kind(kind: MagoSymbolKind) -> SymbolKind {
    match kind {
        MagoSymbolKind::Class => SymbolKind::Class,
        MagoSymbolKind::Interface => SymbolKind::Interface,
        MagoSymbolKind::Trait => SymbolKind::Trait,
        MagoSymbolKind::Enum => SymbolKind::Enum,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(index: usize, name: &[u8]) -> Hit<'_> {
        Hit {
            rank: SymbolRank {
                weak: index.is_multiple_of(2),
                library: index.is_multiple_of(5),
                test: false,
                partial_qualifier: false,
                matched: matcher::ANY,
                kind: (index % 3) as u8,
            },
            short: name,
            container: None,
            name,
            kind: SymbolKind::Class,
            span: FileSpan::dummy(0, 0),
        }
    }

    #[test]
    fn hits_keep_the_best_in_a_bounded_buffer() {
        let names: Vec<String> = (0..3000usize).map(|index| format!("N{:04}", (index * 7919) % 3000)).collect();
        let mut hits = Hits::default();
        let mut all = Vec::new();
        for (index, name) in names.iter().enumerate() {
            hits.add(Some(hit(index, name.as_bytes())));
            assert!(hits.kept.len() < 2 * MAX_RESULTS);
            all.push(hit(index, name.as_bytes()));
        }
        assert!(hits.excludes(true) && !hits.excludes(false));
        all.sort_unstable_by(Hit::order);
        all.truncate(MAX_RESULTS);
        let expected: Vec<_> = all.into_iter().map(|hit| hit.into_item().name).collect();
        let kept: Vec<_> = hits.into_items().into_iter().map(|item| item.name).collect();
        assert_eq!(kept, expected);
    }
}
