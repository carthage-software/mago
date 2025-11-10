use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

/// Represents a reference to a type alias that needs to be expanded during analysis.
///
/// Unlike regular type expansion during building, `TAlias` preserves the alias
/// reference through population and expands it during analysis. This enables:
/// - Proper reference tracking for go-to-definition
/// - Type ID preservation showing the alias name
/// - Analysis-time expansion with full codebase context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TAlias {
    /// The FQCN of the class where the alias is defined or imported
    class_name: Atom,
    /// The name of the type alias
    alias_name: Atom,
}

impl TAlias {
    pub fn new(class_name: Atom, alias_name: Atom) -> Self {
        Self { class_name, alias_name }
    }

    #[inline]
    pub const fn get_class_name(&self) -> Atom {
        self.class_name
    }

    #[inline]
    pub const fn get_alias_name(&self) -> Atom {
        self.alias_name
    }

    /// Expands this type alias to its actual type.
    ///
    /// Returns None if the alias cannot be resolved.
    pub fn resolve<'a>(&self, codebase: &'a CodebaseMetadata) -> Option<&'a TUnion> {
        let class_like = codebase.get_class_like(&self.class_name)?;

        class_like.type_aliases.get(&self.alias_name).map(|type_metadata| &type_metadata.type_union).or_else(|| {
            class_like
                .imported_type_aliases
                .get(&self.alias_name)
                .and_then(|(source_class, type_alias, _)| {
                    codebase
                        .get_class_like(source_class)
                        .and_then(|source_class_like| source_class_like.type_aliases.get(type_alias))
                })
                .map(|type_metadata| &type_metadata.type_union)
        })
    }
}

impl TType for TAlias {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
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

    fn get_id(&self) -> Atom {
        concat_atom!("!", self.class_name.as_str(), "::", self.alias_name.as_str())
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Atom {
        self.get_id()
    }
}
