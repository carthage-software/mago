use std::fmt;
use std::str::FromStr;

use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::Serializer;
#[cfg(feature = "serde")]
use serde::de;
#[cfg(feature = "serde")]
use serde::de::Deserializer;

use mago_syntax_core::part_of_identifier;
use mago_syntax_core::start_of_identifier;

use crate::matcher;

const INVALID_PATH_ERROR: &str = "Invalid path: must be '*', '@all', '@self', '@this', '@native', '@php', '@builtin', a layer (e.g., '@layer:name'), a valid namespace (ending with '\\'), a valid symbol name, or a pattern containing wildcards ('*').";
const INVALID_SELECTOR_ERROR: &str = "Invalid symbol selector: must be a valid namespace (ending with '\\'), a valid symbol name, or a pattern containing wildcards ('*').";
const INVALID_NAMESPACE_ERROR: &str = "Invalid namespace: must be '@global' or a valid namespace ending with '\\'.";

#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
#[schemars(with = "String")]
pub enum NamespacePath {
    Global,
    Specific(String),
}

/// Selects a specific symbol or a group of symbols.
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
#[schemars(with = "String")]
pub enum SymbolSelector {
    /// A specific namespace, e.g., `App\Domain\`
    Namespace(NamespacePath),
    /// A specific, fully qualified symbol name.
    Symbol(String),
    /// A glob-like pattern, e.g., `App\Domain\**`.
    Pattern(String),
}

/// Represents a path, which can be a standard namespace, a layer, or a special keyword.
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
#[schemars(with = "String")]
pub enum Path {
    /// Represents all namespaces, often denoted as `*` or `@all`.
    All,
    /// The current namespace, represented as `@self` or `@this`.
    Self_,
    /// Native PHP symbols, represented as `@native`, `@php`, or `@builtin`.
    Native,
    /// A reusable layer reference, e.g., `@layer:common`.
    Layer(String),
    /// A selector for symbols, namespaces, or patterns.
    Selector(SymbolSelector),
}

/// Checks if a string segment is a valid PHP identifier.
pub(crate) fn is_valid_identifier_part(part: &str) -> bool {
    if part.is_empty() {
        return false;
    }

    let bytes = part.as_bytes();

    matches!(bytes[0], start_of_identifier!()) && bytes[1..].iter().all(|byte| matches!(byte, part_of_identifier!()))
}

fn is_valid_pattern_part(part: &str) -> bool {
    if part == "*" || part == "**" {
        return true;
    }

    part.as_bytes()
        .iter()
        .all(|&byte| matches!(byte, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'\x80'..=b'\xff' | b'*'))
}

/// Validates a selector that contains a brace expression.
///
/// `matcher::matches` expands braces at match time, so the selector is stored verbatim and only
/// its expansions are validated here. Validating the expansions rather than widening the accepted
/// byte set keeps an unbalanced expression such as `App\\{Foo` a configuration error instead of a
/// pattern that silently never matches anything.
fn validate_brace_expansions(s: &str) -> Result<(), &'static str> {
    let expansions = matcher::expand_braces(s.as_bytes());
    if expansions.is_empty() {
        return Err(INVALID_SELECTOR_ERROR);
    }

    for expansion in expansions {
        let expansion = str::from_utf8(&expansion).map_err(|_| INVALID_SELECTOR_ERROR)?;

        // `expand_braces` returns the input untouched when the braces are unbalanced.
        if expansion.contains('{') || expansion.contains('}') {
            return Err(INVALID_SELECTOR_ERROR);
        }

        expansion.parse::<SymbolSelector>()?;
    }

    Ok(())
}

impl FromStr for NamespacePath {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("@global") {
            return Ok(NamespacePath::Global);
        }

        let path_to_validate = s.strip_suffix('\\').unwrap_or(s);
        if path_to_validate.split('\\').all(is_valid_identifier_part) {
            Ok(NamespacePath::Specific(s.to_string()))
        } else {
            Err(INVALID_NAMESPACE_ERROR)
        }
    }
}

impl FromStr for SymbolSelector {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('{') || s.contains('}') {
            return validate_brace_expansions(s).map(|()| SymbolSelector::Pattern(s.to_string()));
        }

        if s.contains('*') {
            if s.split('\\').all(is_valid_pattern_part) {
                Ok(SymbolSelector::Pattern(s.to_string()))
            } else {
                Err(INVALID_SELECTOR_ERROR)
            }
        } else if s.ends_with('\\') || s.eq_ignore_ascii_case("@global") {
            s.parse().map(SymbolSelector::Namespace)
        } else if s.split('\\').all(is_valid_identifier_part) {
            Ok(SymbolSelector::Symbol(s.to_string()))
        } else {
            Err(INVALID_SELECTOR_ERROR)
        }
    }
}

impl FromStr for Path {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" || s.eq_ignore_ascii_case("@all") {
            Ok(Path::All)
        } else if s.eq_ignore_ascii_case("@self") || s.eq_ignore_ascii_case("@this") {
            Ok(Path::Self_)
        } else if s.eq_ignore_ascii_case("@native")
            || s.eq_ignore_ascii_case("@php")
            || s.eq_ignore_ascii_case("@builtin")
        {
            Ok(Path::Native)
        } else if let Some(layer_name) = s.strip_prefix("@layer:").or_else(|| s.strip_prefix("@layers:")) {
            Ok(Path::Layer(layer_name.to_string()))
        } else {
            s.parse().ok().map(Path::Selector).ok_or(INVALID_PATH_ERROR)
        }
    }
}

impl fmt::Display for NamespacePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NamespacePath::Global => write!(f, "@global"),
            NamespacePath::Specific(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for SymbolSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolSelector::Namespace(ns) => write!(f, "{ns}"),
            SymbolSelector::Symbol(s) => write!(f, "{s}"),
            SymbolSelector::Pattern(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Path::All => write!(f, "@all"),
            Path::Self_ => write!(f, "@self"),
            Path::Native => write!(f, "@native"),
            Path::Layer(name) => write!(f, "@layer:{name}"),
            Path::Selector(selector) => write!(f, "{selector}"),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NamespacePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SymbolSelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for NamespacePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SymbolSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_path_parsing_and_formatting() {
        assert_eq!("@all".parse::<Path>().unwrap(), Path::All);
        assert_eq!("@self".parse::<Path>().unwrap(), Path::Self_);
        assert_eq!("@native".parse::<Path>().unwrap(), Path::Native);
        assert_eq!("@layer:core".parse::<Path>().unwrap(), Path::Layer("core".to_string()));
        assert_eq!(
            "App\\Domain\\".parse::<Path>().unwrap(),
            Path::Selector(SymbolSelector::Namespace(NamespacePath::Specific("App\\Domain\\".to_string())))
        );
        assert_eq!(
            "App\\Domain\\Model".parse::<Path>().unwrap(),
            Path::Selector(SymbolSelector::Symbol("App\\Domain\\Model".to_string()))
        );
        assert_eq!("App\\**".parse::<Path>().unwrap(), Path::Selector(SymbolSelector::Pattern("App\\**".to_string())));

        assert_eq!(Path::All.to_string(), "@all");
        assert_eq!(Path::Self_.to_string(), "@self");
        assert_eq!(Path::Native.to_string(), "@native");
        assert_eq!(Path::Layer("core".to_string()).to_string(), "@layer:core");
        assert_eq!(Path::Selector(SymbolSelector::Namespace(NamespacePath::Global)).to_string(), "@global");
        assert_eq!(
            Path::Selector(SymbolSelector::Namespace(NamespacePath::Specific("App\\Domain\\".to_string()))).to_string(),
            "App\\Domain\\"
        );
        assert_eq!(Path::Selector(SymbolSelector::Symbol("My\\Class".to_string())).to_string(), "My\\Class");
        assert_eq!(Path::Selector(SymbolSelector::Pattern("My\\**".to_string())).to_string(), "My\\**");
    }

    #[test]
    fn test_valid_patterns_parse_correctly() {
        "App\\*".parse::<Path>().unwrap();
        "App\\**".parse::<Path>().unwrap();
        "App\\*Something".parse::<Path>().unwrap();
        "App\\*Something*".parse::<Path>().unwrap();
        "App\\*Some*thing".parse::<Path>().unwrap();
    }

    #[test]
    fn test_brace_patterns_parse_correctly() {
        // The matcher expands braces, so the selector is kept verbatim as a pattern.
        assert_eq!(
            "App\\{Foo,Bar}\\Thing".parse::<Path>().unwrap(),
            Path::Selector(SymbolSelector::Pattern("App\\{Foo,Bar}\\Thing".to_string()))
        );
        assert_eq!(
            "App\\{Foo,Bar}\\**".parse::<Path>().unwrap(),
            Path::Selector(SymbolSelector::Pattern("App\\{Foo,Bar}\\**".to_string()))
        );

        // A trailing separator keeps namespace-prefix semantics, which `matcher::matches` honours.
        assert_eq!(
            "App\\{Foo,Bar}\\".parse::<Path>().unwrap(),
            Path::Selector(SymbolSelector::Pattern("App\\{Foo,Bar}\\".to_string()))
        );

        "{Foo,Bar}".parse::<Path>().unwrap();
        "App\\{Foo,Bar}\\{Baz,Qux}".parse::<Path>().unwrap();
        "App\\{Foo,Bar{Baz,Qux}}\\**".parse::<Path>().unwrap();
        "*{Repository,Builder}".parse::<Path>().unwrap();
    }

    #[test]
    fn test_brace_patterns_match_the_same_names_as_their_expansions() {
        let Path::Selector(SymbolSelector::Pattern(pattern)) = "App\\{Foo,Bar}\\Thing".parse::<Path>().unwrap() else {
            panic!("expected a pattern selector");
        };

        assert!(matcher::matches(b"App\\Foo\\Thing", pattern.as_bytes(), false, false));
        assert!(matcher::matches(b"App\\Bar\\Thing", pattern.as_bytes(), false, false));
        assert!(!matcher::matches(b"App\\Baz\\Thing", pattern.as_bytes(), false, false));
    }

    #[test]
    fn test_malformed_brace_patterns_fail_to_parse() {
        // Unbalanced braces would expand to themselves and never match anything.
        "App\\{Foo".parse::<Path>().unwrap_err();
        "App\\Foo}".parse::<Path>().unwrap_err();
        "App\\{Foo,Bar".parse::<Path>().unwrap_err();

        // Every alternative still has to be a valid selector on its own.
        "App\\{Foo,Invalid-Bar}\\Thing".parse::<Path>().unwrap_err();
        "App\\{Foo,1Leading}\\Thing".parse::<Path>().unwrap_err();
    }

    #[test]
    fn test_invalid_paths_fail_to_parse() {
        "Invalid-Class".parse::<Path>().unwrap_err();
        "My\\Invalid-Namespace\\".parse::<Path>().unwrap_err();
        "1LeadingNumber".parse::<Path>().unwrap_err();
        "My\\1LeadingNumber".parse::<Path>().unwrap_err();
        "@My\\Namespace\\".parse::<Path>().unwrap_err();
        "My\\Invalid-Namespace\\".parse::<Path>().unwrap_err();
        "App\\Invalid-*.php".parse::<Path>().unwrap_err();
    }
}
