/// The maximum number of cases an enum can have before certain expensive
/// analysis.
///
/// This constant acts as a safeguard against combinatorial explosion when analyzing
/// enums with an exceptionally large number of cases.
pub const MAX_ENUM_CASES_FOR_ANALYSIS: usize = 200;

/// Names of classes whose instances can cast to `false`.
///
/// Objects are normally always truthy, but an instance of one of these classes
/// is neither always truthy nor always falsy. For example, an empty
/// `SimpleXMLElement` casts to `false`.
///
/// Names are matched case-insensitively.
pub const POSSIBLY_FALSY_CLASS_NAMES: &[&[u8]] = &[b"SimpleXMLElement", b"SimpleXMLIterator"];
