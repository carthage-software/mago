//! Owned resolved names for editor queries, without retaining the parse arena.

use std::ops::Range;

use foldhash::HashMap;
use mago_span::HasPosition;

#[derive(Debug)]
struct Entry {
    start: u32,
    end: u32,
    name: Range<usize>,
    imported: bool,
}

/// Source spans sorted by start offset, with each distinct name stored once.
#[derive(Debug, Default)]
pub struct ResolvedNames {
    entries: Box<[Entry]>,
    names: Box<[u8]>,
}

impl From<&mago_names::ResolvedNames<'_>> for ResolvedNames {
    fn from(resolved: &mago_names::ResolvedNames<'_>) -> Self {
        let mut names = Vec::new();
        let mut ranges = HashMap::default();
        let mut entries = Vec::with_capacity(resolved.len());

        for (start, end, name, imported) in resolved.iter() {
            let range = ranges.entry(name).or_insert_with(|| {
                let start = names.len();
                names.extend_from_slice(name);
                start..names.len()
            });
            entries.push(Entry { start, end, name: range.clone(), imported });
        }

        entries.sort_unstable_by_key(|entry| entry.start);
        Self { entries: entries.into_boxed_slice(), names: names.into_boxed_slice() }
    }
}

impl ResolvedNames {
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Resolve a name at its exact start offset.
    pub fn resolve<T>(&self, position: &T) -> Option<&[u8]>
    where
        T: HasPosition,
    {
        let index = self.entries.binary_search_by_key(&position.offset(), |entry| entry.start).ok()?;
        Some(&self.names[self.entries[index].name.clone()])
    }

    /// Find the name whose half-open source span contains `offset`.
    #[must_use]
    pub fn at_offset(&self, offset: u32) -> Option<(u32, u32, &[u8], bool)> {
        let index = self.entries.partition_point(|entry| entry.start <= offset).checked_sub(1)?;
        let entry = &self.entries[index];
        (offset < entry.end).then(|| (entry.start, entry.end, &self.names[entry.name.clone()], entry.imported))
    }

    /// Visit names in source order.
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, &[u8], bool)> + '_ {
        self.entries.iter().map(|entry| (entry.start, entry.end, &self.names[entry.name.clone()], entry.imported))
    }

    /// Find matching names, using the name resolver's case-insensitive comparison.
    #[must_use]
    pub fn references_to(&self, fqcn: &[u8], exclude_offset: Option<u32>) -> Vec<(u32, u32)> {
        self.iter()
            .filter(|(start, _, _, _)| exclude_offset != Some(*start))
            .filter_map(|(start, end, name, _)| name.eq_ignore_ascii_case(fqcn).then_some((start, end)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_span::Position;
    use mago_syntax::parser::parse_file_borrowed_with_settings;
    use mago_syntax::settings::ParserSettings;

    use super::ResolvedNames;

    #[test]
    fn preserves_resolver_queries() {
        let source = br"<?php
namespace App;
use Library\{Thing as Alias, Other};
use function Library\work as run;
use const Library\VALUE;
class Local extends Alias {}
function process(Other $value): Local { run(); return new Local(); }
$a = new Alias(); $b = new Alias(); echo VALUE;
";
        let file = File::ephemeral(Cow::Borrowed(b"names.php"), Cow::Borrowed(source));
        let arena = LocalArena::new();
        let program = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
        assert!(!program.has_errors());
        let original = NameResolver::new(&arena).resolve(program);
        let owned = ResolvedNames::from(&original);

        let mut expected = original.iter().collect::<Vec<_>>();
        expected.sort_unstable_by_key(|entry| entry.0);
        assert_eq!(owned.iter().collect::<Vec<_>>(), expected);
        assert_eq!(owned.len(), original.len());
        assert_eq!(owned.is_empty(), original.is_empty());

        for offset in 0..=file.size {
            assert_eq!(owned.at_offset(offset), original.at_offset(offset), "offset {offset}");
            assert_eq!(owned.resolve(&Position { offset }), original.resolve(&Position { offset }));
        }

        for (start, _, name, _) in original.iter() {
            for excluded in [None, Some(start)] {
                let mut expected = original.references_to(&name.to_ascii_uppercase(), excluded);
                expected.sort_unstable();
                assert_eq!(owned.references_to(&name.to_ascii_uppercase(), excluded), expected);
            }
        }
    }

    #[test]
    fn names_outlive_source_and_arena() {
        let names = {
            let file = File::ephemeral(
                Cow::Borrowed(b"bytes.php"),
                Cow::Owned(b"<?php use Library\\\xffName as Alias; new Alias(); new Alias();".to_vec()),
            );
            let arena = LocalArena::new();
            let program = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
            assert!(!program.has_errors());
            ResolvedNames::from(&NameResolver::new(&arena).resolve(program))
        };

        assert_eq!(names.references_to(b"Library\\\xffName", None).len(), 3);
        assert!(names.iter().all(|(_, _, name, imported)| name == b"Library\\\xffName" && imported));
        if let Err(payload) = std::thread::spawn(move || assert_eq!(names.len(), 3)).join() {
            std::panic::resume_unwind(payload);
        }
    }

    #[test]
    fn empty_names_have_no_matches() {
        let names = ResolvedNames::default();
        assert!(names.is_empty());
        assert_eq!(names.at_offset(0), None);
        assert_eq!(names.at_offset(u32::MAX), None);
        assert_eq!(names.resolve(&Position { offset: 0 }), None);
        assert!(names.references_to(b"Missing", None).is_empty());
    }
}
