//! Builtin variable names shared by bindings, completion, and edits.

/// Names omit `$`; spelling is case-sensitive.
pub(crate) const BUILTIN_VARIABLES: [&str; 10] =
    ["this", "GLOBALS", "_SERVER", "_GET", "_POST", "_FILES", "_COOKIE", "_SESSION", "_REQUEST", "_ENV"];

pub(crate) fn is_builtin_variable(name: &[u8]) -> bool {
    BUILTIN_VARIABLES.iter().any(|builtin| builtin.as_bytes() == name)
}
