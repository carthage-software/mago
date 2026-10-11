//! Type spelling for editor displays, with shared class-name collision checks.

use std::collections::hash_map::Entry;

use foldhash::{HashMap, HashSet};

use mago_bytes::BytesDisplay;
use mago_word::{Word, ascii_lowercase_word};

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::union::TUnion;

fn short_name(name: &[u8]) -> String {
    BytesDisplay(name.rsplit(|byte| *byte == b'\\').next().unwrap_or(name)).to_string()
}

/// Names displayed together share one collision check.
pub struct NameDisplay<'code> {
    codebase: &'code CodebaseMetadata,
    ambiguous: HashSet<Vec<u8>>,
    fully_qualified: bool,
}

impl<'code> NameDisplay<'code> {
    /// Use absolute class names when part of the display has already been rendered.
    #[must_use]
    pub fn fully_qualified(codebase: &'code CodebaseMetadata) -> Self {
        Self { codebase, ambiguous: HashSet::default(), fully_qualified: true }
    }

    /// Choose short class names unless the displayed types share a name.
    #[must_use]
    pub fn for_types<'ty>(codebase: &'code CodebaseMetadata, types: impl Iterator<Item = &'ty TUnion>) -> Self {
        let names = types.flat_map(|ty| ty.get_all_child_nodes()).filter_map(type_name);
        Self::for_names(codebase, names)
    }

    /// Build one spelling context for a group of class names.
    #[must_use]
    pub fn for_names(codebase: &'code CodebaseMetadata, names: impl Iterator<Item = Word>) -> Self {
        let mut seen = HashMap::default();
        let mut ambiguous = HashSet::default();
        for name in names {
            let name = codebase.get_class_like_by_name(name).map_or(name, |class| class.name);
            let full = ascii_lowercase_word(name.as_bytes());
            let short =
                name.as_bytes().rsplit(|byte| *byte == b'\\').next().unwrap_or(name.as_bytes()).to_ascii_lowercase();
            match seen.entry(short) {
                Entry::Vacant(entry) => {
                    entry.insert(full);
                }
                Entry::Occupied(entry) if *entry.get() != full => {
                    ambiguous.insert(entry.key().clone());
                }
                Entry::Occupied(_) => {}
            }
        }
        Self { codebase, ambiguous, fully_qualified: false }
    }

    /// Use the declaration casing and qualify colliding short names.
    #[must_use]
    pub fn class_name(&self, name: Word) -> String {
        let name = self.codebase.get_class_like_by_name(name).map_or(name, |class| class.original_name);
        if self.fully_qualified {
            return format!("\\{}", BytesDisplay(name.as_bytes()));
        }

        let short = short_name(name.as_bytes());
        if !self.ambiguous.is_empty() && self.ambiguous.contains(short.as_bytes().to_ascii_lowercase().as_slice()) {
            format!("\\{}", BytesDisplay(name.as_bytes()))
        } else {
            short
        }
    }

    /// Render owned text without interning display spellings or copying shared types.
    #[must_use]
    pub fn union(&self, ty: &TUnion) -> String {
        BytesDisplay(&render::union(ty, render::Names::Short(self))).to_string()
    }
}

/// Render the full type-id spelling without interning it or escaping PHP bytes.
#[must_use]
pub fn canonical_bytes(ty: &TUnion) -> Vec<u8> {
    render::union(ty, render::Names::Canonical)
}

/// Keep the full type-id spelling, restoring class names from their declarations.
#[must_use]
pub fn qualified_bytes(codebase: &CodebaseMetadata, ty: &TUnion) -> Vec<u8> {
    render::union(ty, render::Names::Qualified(codebase))
}

mod render;

fn type_name(node: TypeRef<'_>) -> Option<Word> {
    match node {
        TypeRef::Atomic(TAtomic::Object(TObject::Named(object))) => Some(object.name),
        TypeRef::Atomic(TAtomic::Object(TObject::Enum(object))) => Some(object.name),
        TypeRef::Atomic(TAtomic::Reference(TReference::Symbol { name, .. })) => Some(*name),
        TypeRef::Atomic(TAtomic::Reference(TReference::Member { class_like_name, .. })) => Some(*class_like_name),
        TypeRef::Atomic(TAtomic::Alias(alias)) => Some(alias.get_class_name()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
