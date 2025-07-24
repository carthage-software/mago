use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum ArrayDataKind {
    ArrayKey,
    ArrayValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathKind {
    Default,
    UnknownArrayAccess(ArrayDataKind),
    UnknownArrayAssignment(ArrayDataKind),
    ArrayAccess(ArrayDataKind, String),
    ArrayAssignment(ArrayDataKind, String),
    PropertyAccess(StringIdentifier, StringIdentifier),
    PropertyAssignment(StringIdentifier, StringIdentifier),
    UnknownPropertyAccess,
    UnknownPropertyAssignment,
    Serialize,
    RemoveArrayKey(String),
    RefineSymbol(StringIdentifier),
    ScalarTypeGuard,
    Aggregate,
}

impl PathKind {
    pub fn to_unique_string(&self) -> String {
        match &self {
            PathKind::Default => "".to_string(),
            PathKind::UnknownArrayAccess(a) => {
                format!(
                    "array-{}-access",
                    match a {
                        ArrayDataKind::ArrayKey => "key",
                        ArrayDataKind::ArrayValue => "value",
                    }
                )
            }
            PathKind::ArrayAccess(a, b) => {
                format!(
                    "array-{}-access({})",
                    match a {
                        ArrayDataKind::ArrayKey => "key",
                        ArrayDataKind::ArrayValue => "value",
                    },
                    b
                )
            }
            PathKind::UnknownArrayAssignment(a) => {
                format!(
                    "array-{}-assignment",
                    match a {
                        ArrayDataKind::ArrayKey => "key",
                        ArrayDataKind::ArrayValue => "value",
                    }
                )
            }
            PathKind::ArrayAssignment(a, b) => {
                format!(
                    "array-{}-assignment({})",
                    match a {
                        ArrayDataKind::ArrayKey => "key",
                        ArrayDataKind::ArrayValue => "value",
                    },
                    b
                )
            }
            PathKind::UnknownPropertyAccess => "property-access".to_string(),
            PathKind::PropertyAccess(a, b) => {
                format!("property-access({a},{b})")
            }
            PathKind::UnknownPropertyAssignment => "property-assignment".to_string(),
            PathKind::PropertyAssignment(a, b) => {
                format!("property-assignment({a},{b})")
            }
            PathKind::RemoveArrayKey(_) => "remove-array-key".to_string(),
            PathKind::RefineSymbol(_) => "refine-symbol".to_string(),
            PathKind::ScalarTypeGuard => "scalar-type-guard".to_string(),
            PathKind::Serialize => "serialize".to_string(),
            PathKind::Aggregate => "aggregate".to_string(),
        }
    }
}

impl std::fmt::Display for PathKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            PathKind::Default => std::fmt::Result::Ok(()),
            PathKind::UnknownArrayAccess(_) | PathKind::ArrayAccess(_, _) => {
                write!(f, "array-access")
            }
            PathKind::UnknownArrayAssignment(_) | PathKind::ArrayAssignment(_, _) => {
                write!(f, "array-assignment")
            }
            PathKind::PropertyAccess(_, _) | PathKind::UnknownPropertyAccess => {
                write!(f, "property-access")
            }
            PathKind::PropertyAssignment(_, _) | PathKind::UnknownPropertyAssignment => {
                write!(f, "property-assignment")
            }
            PathKind::RemoveArrayKey(_) => write!(f, "remove-array-key"),
            PathKind::RefineSymbol(_) => write!(f, "refine-symbol"),
            PathKind::ScalarTypeGuard => write!(f, "scalar-type-guard"),
            PathKind::Serialize => write!(f, "serialize"),
            PathKind::Aggregate => write!(f, "aggregate"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataFlowPath {
    pub kind: PathKind,
}
