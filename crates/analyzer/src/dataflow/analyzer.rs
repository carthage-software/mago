#![allow(dead_code)]

use mago_codex::data_flow::path::ArrayDataKind;
use mago_codex::data_flow::path::PathKind;

/// Determines if an array access path should be ignored during data flow analysis,
/// typically because it's not matched by a corresponding assignment at the correct nesting level.
#[inline]
pub fn should_ignore_array_access(
    path_type: &PathKind,
    match_kind: &ArrayDataKind,
    previous_path_types: &[PathKind],
) -> bool {
    let is_relevant_access = match path_type {
        PathKind::ArrayAccess(inner_expression_kind, _) => inner_expression_kind == match_kind,
        PathKind::UnknownArrayAccess(ArrayDataKind::ArrayKey) => match_kind == &ArrayDataKind::ArrayValue,
        _ => false,
    };

    if is_relevant_access {
        let mut fetch_nesting = 0;

        for previous_path_type in previous_path_types.iter().rev() {
            match previous_path_type {
                PathKind::UnknownArrayAssignment(inner) | PathKind::ArrayAssignment(inner, _)
                    if inner == match_kind =>
                {
                    if fetch_nesting == 0 {
                        if let PathKind::ArrayAssignment(_, previous_assignment_value) = previous_path_type {
                            if let PathKind::ArrayAccess(_, fetch_value) = path_type {
                                if fetch_value == previous_assignment_value {
                                    return false;
                                }

                                return true;
                            }

                            return true;
                        } else {
                            return false;
                        }
                    } else {
                        fetch_nesting -= 1;
                    }
                }
                PathKind::UnknownArrayAccess(inner) | PathKind::ArrayAccess(inner, _) if inner == match_kind => {
                    fetch_nesting += 1;
                }
                _ => {}
            }
        }

        return true;
    }

    if let PathKind::RemoveArrayKey(key_name) = path_type
        && match_kind == &ArrayDataKind::ArrayValue
        && let Some(PathKind::ArrayAssignment(ArrayDataKind::ArrayValue, assigned_name)) =
            previous_path_types.iter().rev().find(|t| !matches!(t, PathKind::Default))
        && assigned_name == key_name
    {
        return true;
    }

    false
}

/// Determines if a property access path should be ignored during data flow analysis,
/// similar to the array access logic but for object properties.
#[inline]
pub fn should_ignore_property_access(path_kind: &PathKind, previous_path_kinds: &[PathKind]) -> bool {
    if let PathKind::PropertyAccess(_, _) = path_kind {
        let mut fetch_nesting = 0;

        for previous_path_kind in previous_path_kinds.iter().rev() {
            match previous_path_kind {
                PathKind::UnknownPropertyAssignment | PathKind::PropertyAssignment(_, _) => {
                    if fetch_nesting == 0 {
                        if let PathKind::PropertyAssignment(_, previous_assignment_value) = previous_path_kind {
                            if let PathKind::PropertyAccess(_, fetch_value) = path_kind {
                                if fetch_value == previous_assignment_value {
                                    return false;
                                }

                                return true;
                            }

                            return true;
                        } else {
                            return false;
                        }
                    } else {
                        fetch_nesting -= 1;
                    }
                }
                PathKind::UnknownPropertyAccess | PathKind::PropertyAccess(_, _) => {
                    fetch_nesting += 1;
                }
                _ => {}
            }
        }

        return true;
    }

    false
}
