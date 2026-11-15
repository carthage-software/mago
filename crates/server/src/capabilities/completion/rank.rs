//! Completion ranks and the per-request signals they are built from.
//!
//! After the match itself, a candidate ranks by its group, whether it is a
//! test symbol offered outside tests, where it comes from (imported here, the
//! project, PHP core, or a library), whether it is discouraged, and its
//! namespace proximity. Library proximity prefers packages the current scope
//! already imports from.

use std::cmp::Ordering;

use mago_codex::metadata::flags::MetadataFlags;
use mago_database::file::File;
use mago_names::scope::NamespaceScope;

use crate::matcher::Match;
use crate::matcher::is_test_namespace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Origin {
    Scope,
    Host,
    Core,
    Library,
}

/// Field order is the sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Rank {
    pub(super) matched: Match,
    pub(super) group: u8,
    pub(super) test: bool,
    pub(super) origin: Origin,
    pub(super) discouraged: bool,
    pub(super) proximity: u16,
}

impl Rank {
    pub(super) const fn new(matched: Match) -> Self {
        Self { matched, group: 0, test: false, origin: Origin::Host, discouraged: false, proximity: 0 }
    }

    /// A name bound by an in-range `use`. Its alias settles the spelling, so
    /// case differences do not lower it, except that an exact match spelled
    /// differently still ranks after one spelled as typed.
    pub(super) const fn imported(matched: Match, flags: MetadataFlags) -> Self {
        let matched = Match { first_case_miss: false, case_miss: matched.case_miss && matched.is_exact(), ..matched };
        Self { origin: Origin::Scope, discouraged: flags.is_deprecated(), ..Self::new(matched) }
    }
}

pub(super) struct RankContext<'a> {
    namespace: &'a [u8],
    depth: u16,
    imports: Vec<&'a [u8]>,
    test_file: bool,
}

impl<'a> RankContext<'a> {
    pub(super) fn new(file: &File, scope: &'a NamespaceScope) -> Self {
        let namespace = scope.namespace_name().unwrap_or_default();
        let mut imports: Vec<&[u8]> = scope
            .default_aliases()
            .values()
            .chain(scope.function_aliases().values())
            .chain(scope.constant_aliases().values())
            .map(|target| namespace_of(target))
            .filter(|namespace| !namespace.is_empty())
            .collect();
        imports.sort_unstable_by(|left, right| segment_order(left, right));
        imports.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
        let test_file = is_test_namespace(namespace)
            || file
                .name
                .rsplit(|byte| matches!(byte, b'/' | b'\\'))
                .skip(1)
                .any(|dir| dir.eq_ignore_ascii_case(b"tests"));
        Self { namespace, depth: depth(namespace), imports, test_file }
    }

    /// A global name offered by its short name.
    pub(super) fn symbol(&self, matched: Match, flags: MetadataFlags, fqn: &[u8]) -> Rank {
        let namespace = namespace_of(fqn);
        let origin = origin(flags, namespace);
        let proximity = match origin {
            Origin::Host => {
                let shared = shared_segments(self.namespace, namespace);
                ((self.depth - shared).min(255) << 8) | (depth(namespace) - shared).min(255)
            }
            Origin::Library => ((255 - self.affinity(namespace).min(255)) << 8) | depth(namespace).min(255),
            Origin::Scope | Origin::Core => 0,
        };
        self.rank(matched, flags, namespace, origin, proximity)
    }

    /// A name offered relative to a typed namespace qualifier.
    pub(super) fn qualified(&self, matched: Match, flags: MetadataFlags, fqn: &[u8], relative: &[u8]) -> Rank {
        let namespace = namespace_of(fqn);
        self.rank(matched, flags, namespace, origin(flags, namespace), depth(namespace_of(relative)))
    }

    fn rank(&self, matched: Match, flags: MetadataFlags, namespace: &[u8], origin: Origin, proximity: u16) -> Rank {
        Rank {
            matched,
            group: 0,
            test: !self.test_file && is_test_namespace(namespace),
            origin,
            discouraged: flags.is_deprecated() || (origin == Origin::Library && flags.is_internal()),
            proximity,
        }
    }

    /// Leading segments `namespace` shares with the closest imported namespace.
    /// Sorting with `\` lowest keeps the closest one next to the search position.
    fn affinity(&self, namespace: &[u8]) -> u16 {
        if namespace.is_empty() {
            return 0;
        }
        let index = self.imports.partition_point(|import| segment_order(import, namespace) == Ordering::Less);
        let before = index.checked_sub(1).map(|before| shared_segments(self.imports[before], namespace));
        let after = self.imports.get(index).map(|after| shared_segments(after, namespace));
        before.max(after).unwrap_or(0)
    }
}

/// Orders two names case-insensitively, then by their exact bytes.
pub(super) fn compare_names(left: &[u8], right: &[u8]) -> Ordering {
    left.iter().map(u8::to_ascii_lowercase).cmp(right.iter().map(u8::to_ascii_lowercase)).then_with(|| left.cmp(right))
}

/// Preselect a case-sensitive exact match no other entry matches the same way.
/// Such matches sort first, so only the second entry needs checking.
pub(super) fn preselects(first: Option<&Rank>, second: Option<&Rank>) -> bool {
    first.is_some_and(|first| first.matched.is_strict_exact())
        && second.is_none_or(|second| !second.matched.is_strict_exact())
}

fn origin(flags: MetadataFlags, namespace: &[u8]) -> Origin {
    if flags.is_user_defined() {
        Origin::Host
    } else if flags.is_built_in() && namespace.is_empty() {
        Origin::Core
    } else {
        Origin::Library
    }
}

fn namespace_of(fqn: &[u8]) -> &[u8] {
    memchr::memrchr(b'\\', fqn).map_or(&[], |index| &fqn[..index])
}

fn segments(namespace: &[u8]) -> impl Iterator<Item = &[u8]> {
    namespace.split(|byte| *byte == b'\\').filter(|segment| !segment.is_empty())
}

fn depth(namespace: &[u8]) -> u16 {
    segments(namespace).count().min(u16::MAX.into()) as u16
}

fn shared_segments(left: &[u8], right: &[u8]) -> u16 {
    segments(left).zip(segments(right)).take_while(|(left, right)| left.eq_ignore_ascii_case(right)).count() as u16
}

fn segment_order(left: &[u8], right: &[u8]) -> Ordering {
    let fold = |byte: &u8| if *byte == b'\\' { 0 } else { byte.to_ascii_lowercase() };
    left.iter().map(fold).cmp(right.iter().map(fold))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn scored(query: &[u8], candidate: &[u8]) -> Match {
        crate::matcher::score(query, candidate).unwrap()
    }

    #[test]
    fn exact_spelling_outranks_an_imported_case_variant() {
        let imported = |query: &[u8], alias: &[u8]| Rank::imported(scored(query, alias), MetadataFlags::empty());
        assert!(Rank::new(scored(b"route", b"route")) < imported(b"route", b"Route"));
        assert!(imported(b"Route", b"Route") < Rank::new(scored(b"Route", b"route")));
        assert!(imported(b"req", b"Request") < Rank::new(scored(b"req", b"request_parse_body")));
        assert!(imported(b"request", b"Request") < Rank::new(scored(b"request", b"request_parse_body")));
    }

    #[test]
    fn only_case_sensitive_exact_matches_block_preselect() {
        let exact = Rank::new(scored(b"route", b"route"));
        let variant = Rank::imported(scored(b"route", b"Route"), MetadataFlags::empty());
        assert!(preselects(Some(&exact), Some(&variant)));
        assert!(preselects(Some(&exact), Some(&Rank::new(scored(b"route", b"ROUTE")))));
        assert!(preselects(Some(&exact), None));
        assert!(!preselects(Some(&exact), Some(&exact)));
        assert!(!preselects(Some(&variant), None));
    }

    #[test]
    fn affinity_finds_the_closest_import() {
        let scope = NamespaceScope::new(None);
        let mut context = RankContext::new(&File::ephemeral(b"a.php".as_slice().into(), b"".to_vec().into()), &scope);
        context.imports = vec![b"Acme".as_slice(), b"Acme2\\Http", b"Acme\\Http\\Client", b"Zeta"];
        context.imports.sort_unstable_by(|left, right| segment_order(left, right));
        assert_eq!(context.affinity(b"Acme\\Http\\Exception"), 2);
        assert_eq!(context.affinity(b"acme\\http\\client\\middleware"), 3);
        assert_eq!(context.affinity(b"Acme2"), 1);
        assert_eq!(context.affinity(b"Acme1"), 0);
        assert_eq!(context.affinity(b"Other"), 0);
    }

    #[test]
    fn host_proximity_walks_the_namespace_tree() {
        let scope = NamespaceScope::for_namespace(b"App\\Http".to_vec());
        let context = RankContext::new(&File::ephemeral(b"a.php".as_slice().into(), b"".to_vec().into()), &scope);
        let rank = |fqn: &[u8]| context.symbol(crate::matcher::ANY, MetadataFlags::USER_DEFINED, fqn).proximity;
        assert!(rank(b"App\\Http\\Kernel") < rank(b"App\\Kernel"));
        assert!(rank(b"App\\Kernel") < rank(b"App\\Mail\\Mailer"));
        assert!(rank(b"App\\Mail\\Mailer") < rank(b"Kernel"));
    }

    #[test]
    fn test_files_keep_test_symbols() {
        let scope = NamespaceScope::for_namespace(b"App".to_vec());
        let source = File::ephemeral(b"src/a.php".as_slice().into(), b"".to_vec().into());
        let test = File::ephemeral(b"tests/Unit/a.php".as_slice().into(), b"".to_vec().into());
        let fqn = b"Acme\\Tests\\CurrencyTest";
        assert!(RankContext::new(&source, &scope).symbol(crate::matcher::ANY, MetadataFlags::empty(), fqn).test);
        assert!(!RankContext::new(&test, &scope).symbol(crate::matcher::ANY, MetadataFlags::empty(), fqn).test);
    }
}
