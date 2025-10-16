use std::fmt;

use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::Deserializer;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::ser::SerializeStruct;
use serde::ser::Serializer;

use crate::path::NamespacePath;
use crate::path::Path;
use crate::path::is_valid_identifier_part;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct Settings {
    pub perimeter: PerimeterSettings,
    pub structural: StructuralSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PerimeterSettings {
    pub layers: HashMap<String, Vec<Path>>,
    pub layering: Vec<NamespacePath>,
    pub rules: Vec<PerimeterRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerimeterRule {
    pub namespace: NamespacePath,
    pub permit: Vec<PermittedDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermittedDependency {
    Dependency(Path),
    DependencyOfKind { path: Path, kinds: Vec<PermittedDependencyKind> },
}

/// Represents the specific types of symbols allowed from a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermittedDependencyKind {
    ClassLike,
    Function,
    Constant,
    Attribute,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StructuralSettings {
    /// A list of structural rules to enforce across the codebase.
    pub rules: Vec<StructuralRule>,
}

/// Represents a single structural enforcement rule from `[[guard.structural.rules]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StructuralRule {
    /// The namespace pattern this rule applies to.
    pub on: String,
    /// An optional exclusion pattern; if the namespace matches this, the rule is skipped.
    pub not_on: Option<String>,
    /// The kind of symbol this policy applies to (e.g., "class").
    pub target: Option<StructuralSymbolKind>,
    /// Restricts the namespace to only contain the specified symbol kinds.
    pub must_be: Option<Vec<StructuralSymbolKind>>,
    /// Optional naming pattern the symbol's name must match.
    pub must_be_named: Option<String>,
    /// If true, the symbol must be declared `final`.
    pub must_be_final: Option<bool>,
    /// If true, the symbol must be declared `abstract`.
    pub must_be_abstract: Option<bool>,
    /// If true, the symbol must be declared `readonly`.
    pub must_be_readonly: Option<bool>,
    /// Structural implementation constraints.
    pub must_implement: Option<StructuralInheritanceConstraint>,
    /// Structural extension constraints.
    pub must_extend: Option<StructuralInheritanceConstraint>,
    /// Structural trait usage constraints.
    pub must_use_trait: Option<StructuralInheritanceConstraint>,
    /// Structural attribute usage constraints.
    pub must_use_attribute: Option<StructuralInheritanceConstraint>,
    /// A human-readable reason for this rule.
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralSymbolKind {
    ClassLike,
    Class,
    Interface,
    Trait,
    Enum,
    Constant,
    Function,
}

/// Represents a logical constraint for `implement`, `extend`, or `use_traits`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum StructuralInheritanceConstraint {
    /// An OR of ANDs, e.g., `[["A", "B"], ["C"]]`
    AnyOfAllOf(Vec<Vec<String>>),
    /// An AND group, e.g., `["A", "B"]`
    AllOf(Vec<String>),
    /// A single required item, e.g., `"A"`
    Single(String),
    /// `None` indicates the rule requires no constraints.
    Nothing,
}

impl PermittedDependencyKind {
    /// Returns the string representation of the symbol type.
    pub const fn as_str(&self) -> &'static str {
        match self {
            PermittedDependencyKind::ClassLike => "class-like",
            PermittedDependencyKind::Function => "function",
            PermittedDependencyKind::Constant => "constant",
            PermittedDependencyKind::Attribute => "attribute",
        }
    }
}

impl StructuralSymbolKind {
    pub const fn is_constant(&self) -> bool {
        matches!(self, StructuralSymbolKind::Constant)
    }

    /// Returns the string representation of the symbol kind.
    pub const fn as_str(&self) -> &'static str {
        match self {
            StructuralSymbolKind::ClassLike => "class-like",
            StructuralSymbolKind::Class => "class",
            StructuralSymbolKind::Interface => "interface",
            StructuralSymbolKind::Trait => "trait",
            StructuralSymbolKind::Enum => "enum",
            StructuralSymbolKind::Constant => "constant",
            StructuralSymbolKind::Function => "function",
        }
    }
}

impl<'de> Deserialize<'de> for PermittedDependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AllowedPathVisitor;

        impl<'de> Visitor<'de> for AllowedPathVisitor {
            type Value = PermittedDependency;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a path string or a detailed object with path and types")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let path = Path::deserialize(de::value::StrDeserializer::new(value))?;
                Ok(PermittedDependency::Dependency(path))
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                #[derive(Deserialize)]
                struct DetailedHelper {
                    path: Path,
                    kinds: Vec<PermittedDependencyKind>,
                }

                let helper: DetailedHelper = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;

                Ok(PermittedDependency::DependencyOfKind { path: helper.path, kinds: helper.kinds })
            }
        }

        deserializer.deserialize_any(AllowedPathVisitor)
    }
}

impl<'de> Deserialize<'de> for StructuralInheritanceConstraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Helper enum to let serde handle the shape detection (string vs array vs array of arrays).
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Untagged {
            AnyOfAllOf(Vec<Vec<String>>),
            AllOf(Vec<String>),
            Single(String),
        }

        match Untagged::deserialize(deserializer)? {
            Untagged::Single(s) => {
                if s.eq_ignore_ascii_case("@nothing") {
                    Ok(Self::Nothing)
                } else if s.split('\\').all(is_valid_identifier_part) {
                    Ok(Self::Single(s))
                } else {
                    Err(de::Error::custom(format!(
                        "Expected a valid fully qualified name or '@nothing', found '{}'",
                        s
                    )))
                }
            }
            Untagged::AllOf(items) => {
                for item in &items {
                    if !item.split('\\').all(is_valid_identifier_part) {
                        return Err(de::Error::custom(format!("'{}' is not a valid fully qualified name", item)));
                    }
                }

                Ok(Self::AllOf(items))
            }
            Untagged::AnyOfAllOf(groups) => {
                for group in &groups {
                    for item in group {
                        if !item.split('\\').all(is_valid_identifier_part) {
                            return Err(de::Error::custom(format!("'{}' is not a valid fully qualified name", item)));
                        }
                    }
                }

                Ok(Self::AnyOfAllOf(groups))
            }
        }
    }
}

impl Serialize for PermittedDependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PermittedDependency::Dependency(path) => path.serialize(serializer),
            PermittedDependency::DependencyOfKind { path, kinds } => {
                let mut state = serializer.serialize_struct("DependencyOfKind", 2)?;
                state.serialize_field("path", path)?;
                state.serialize_field("kinds", kinds)?;
                state.end()
            }
        }
    }
}

impl fmt::Display for PermittedDependencyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for StructuralInheritanceConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // "nothing"
            Self::Nothing => write!(f, "<nothing>"),
            // "`SomeInterface`"
            Self::Single(item) => write!(f, "`{}`", item),
            // "`InterfaceA` and `InterfaceB`"
            Self::AllOf(items) => {
                let formatted = items.iter().map(|item| format!("`{}`", item)).collect::<Vec<_>>().join(" and ");
                write!(f, "{}", formatted)
            }
            // "(`InterfaceA` and `InterfaceB`) or `InterfaceC`"
            Self::AnyOfAllOf(groups) => {
                let formatted = groups
                    .iter()
                    .map(|group| {
                        let inner = group.iter().map(|item| format!("`{}`", item)).collect::<Vec<_>>().join(" and ");
                        if group.len() > 1 { format!("({})", inner) } else { inner }
                    })
                    .collect::<Vec<_>>()
                    .join(" or ");
                write!(f, "{}", formatted)
            }
        }
    }
}

impl fmt::Display for StructuralSymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structural_inheritance_constraint_display() {
        let single = StructuralInheritanceConstraint::Single("SomeInterface".to_string());
        assert_eq!(single.to_string(), "`SomeInterface`");

        let all_of = StructuralInheritanceConstraint::AllOf(vec!["InterfaceA".to_string(), "InterfaceB".to_string()]);
        assert_eq!(all_of.to_string(), "`InterfaceA` and `InterfaceB`");

        let any_of_all_of = StructuralInheritanceConstraint::AnyOfAllOf(vec![
            vec!["InterfaceA".to_string(), "InterfaceB".to_string()],
            vec!["InterfaceC".to_string()],
        ]);
        assert_eq!(any_of_all_of.to_string(), "(`InterfaceA` and `InterfaceB`) or `InterfaceC`");

        let none = StructuralInheritanceConstraint::Nothing;
        assert_eq!(none.to_string(), "<nothing>");
    }

    #[derive(Deserialize)]
    struct Wrapper {
        constraint: StructuralInheritanceConstraint,
    }

    #[test]
    fn it_deserializes_none_keyword() {
        let toml = r#"constraint = "@nothing""#;
        let wrapped: Wrapper = toml::from_str(toml).unwrap();
        assert_eq!(wrapped.constraint, StructuralInheritanceConstraint::Nothing);
    }

    #[test]
    fn it_deserializes_valid_single_string() {
        let toml = r#"constraint = "App\\Domain\\MyInterface""#;
        let wrapped: Wrapper = toml::from_str(toml).unwrap();
        assert_eq!(wrapped.constraint, StructuralInheritanceConstraint::Single("App\\Domain\\MyInterface".to_string()));
    }

    #[test]
    fn it_deserializes_valid_array_of_strings() {
        let toml = r#"constraint = ["App\\InterfaceA", "App\\InterfaceB"]"#;
        let wrapped: Wrapper = toml::from_str(toml).unwrap();
        assert_eq!(
            wrapped.constraint,
            StructuralInheritanceConstraint::AllOf(vec!["App\\InterfaceA".to_string(), "App\\InterfaceB".to_string()])
        );
    }

    #[test]
    fn it_deserializes_valid_array_of_arrays() {
        let toml = r#"constraint = [["App\\A", "App\\B"], ["App\\C"]]"#;
        let wrapped: Wrapper = toml::from_str(toml).unwrap();
        assert_eq!(
            wrapped.constraint,
            StructuralInheritanceConstraint::AnyOfAllOf(vec![
                vec!["App\\A".to_string(), "App\\B".to_string()],
                vec!["App\\C".to_string()]
            ])
        );
    }

    #[test]
    fn it_fails_on_invalid_identifier_in_single_string() {
        let toml = r#"constraint = "Invalid-Interface""#;
        assert!(toml::from_str::<Wrapper>(toml).is_err());
    }

    #[test]
    fn it_fails_on_invalid_identifier_in_array() {
        let toml = r#"constraint = ["App\\InterfaceA", "Invalid-Interface"]"#;
        assert!(toml::from_str::<Wrapper>(toml).is_err());
    }

    #[test]
    fn it_fails_on_invalid_identifier_in_nested_array() {
        let toml = r#"constraint = [["App\\A", "Invalid-B"], ["App\\C"]]"#;
        assert!(toml::from_str::<Wrapper>(toml).is_err());
    }
}
