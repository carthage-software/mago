use serde::Deserialize;
use serde::Serialize;

use mago_word::Word;
use mago_word::concat_word;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

/// Describes how the class hosting a type alias is referenced.
///
/// An alias such as `!self::Member` or `!Acme\Repository::Row` records both the
/// resolved class name and the keyword (if any) used to reach it, mirroring the
/// `ReferenceKind` carried by the HIR so the source spelling survives population.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub enum ReferenceKind {
    Identifier(Word),
    Self_(Word),
    Static(Word),
    Parent(Word),
}

impl ReferenceKind {
    #[inline]
    #[must_use]
    pub const fn class_name(&self) -> Word {
        match self {
            ReferenceKind::Identifier(class_name)
            | ReferenceKind::Self_(class_name)
            | ReferenceKind::Static(class_name)
            | ReferenceKind::Parent(class_name) => *class_name,
        }
    }
}

/// Represents a reference to a type alias that needs to be expanded during analysis.
///
/// Unlike regular type expansion during building, `TAlias` preserves the alias
/// reference through population and expands it during analysis. This enables:
/// - Proper reference tracking for go-to-definition
/// - Type ID preservation showing the alias name
/// - Analysis-time expansion with full codebase context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TAlias {
    /// How the class hosting the alias is referenced
    reference: ReferenceKind,
    /// The name of the type alias
    alias_name: Word,
}

impl TAlias {
    #[must_use]
    pub const fn new(reference: ReferenceKind, alias_name: Word) -> Self {
        Self { reference, alias_name }
    }

    #[inline]
    #[must_use]
    pub const fn get_reference(&self) -> ReferenceKind {
        self.reference
    }

    #[inline]
    #[must_use]
    pub const fn get_class_name(&self) -> Word {
        self.reference.class_name()
    }

    #[inline]
    #[must_use]
    pub const fn get_alias_name(&self) -> Word {
        self.alias_name
    }

    /// Expands this type alias to its actual type.
    ///
    /// Returns None if the alias cannot be resolved.
    #[must_use]
    pub fn resolve<'codebase>(&self, codebase: &'codebase CodebaseMetadata) -> Option<&'codebase TUnion> {
        let class_name = self.reference.class_name();
        let class_like = codebase.get_class_like(class_name.as_bytes())?;
        if let Some(type_alias) = class_like.type_aliases.get(&self.alias_name) {
            return Some(&type_alias.type_union);
        }

        let (source_class, type_alias, _) = class_like.imported_type_aliases.get(&self.alias_name)?;

        let type_metadata = if source_class == &class_name {
            class_like.type_aliases.get(type_alias)
        } else {
            codebase
                .get_class_like(source_class.as_bytes())
                .and_then(|source_class_like| source_class_like.type_aliases.get(type_alias))
        };

        Some(&type_metadata?.type_union)
    }
}

impl TType for TAlias {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        vec![]
    }

    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Word {
        concat_word!(b"!", self.reference.class_name().as_bytes(), b"::", self.alias_name.as_bytes())
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Word {
        self.get_id()
    }
}
