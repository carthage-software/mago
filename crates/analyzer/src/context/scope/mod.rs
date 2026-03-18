use mago_atom::Atom;

pub mod case_scope;
pub mod conditional_scope;
pub mod control_action;
pub mod finally_scope;
pub mod if_scope;
pub mod loop_scope;

#[inline]
pub fn var_has_root(var_id: Atom, root_var_id: Atom) -> bool {
    if var_id == root_var_id {
        return true;
    }

    if !var_id.starts_with(root_var_id.as_str()) {
        return false;
    }

    let after_root = &var_id[root_var_id.len()..];
    after_root.starts_with("->") || after_root.starts_with("::") || after_root.starts_with('[')
}

#[inline]
pub fn var_references_dynamic(var_id: Atom, referenced_var_id: Atom) -> bool {
    let var_str = var_id.as_str();
    let ref_str = referenced_var_id.as_str();

    let mut start = 1;
    while let Some(pos) = var_str.get(start..).and_then(|s| s.find(ref_str)) {
        let end = start + pos + ref_str.len();
        if end == var_str.len() || !var_str.as_bytes()[end].is_ascii_alphanumeric() && var_str.as_bytes()[end] != b'_' {
            return true;
        }

        start += pos + 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_atom::atom;

    #[test]
    fn test_var_has_root() {
        assert!(var_has_root(atom("$foo"), atom("$foo")));
        assert!(var_has_root(atom("$foo[bar]"), atom("$foo")));
        assert!(var_has_root(atom("$foo->bar"), atom("$foo")));
        assert!(var_has_root(atom("$foo::bar"), atom("$foo")));
        assert!(var_has_root(atom("$foo->bar[0]"), atom("$foo")));
        assert!(var_has_root(atom("$foo->bar[0]->baz"), atom("$foo")));
        assert!(!var_has_root(atom("$foo[bar]"), atom("$bar")));
        assert!(var_has_root(atom("$foo[bar]"), atom("$foo[bar]")));
        assert!(!var_has_root(atom("$foo[bar]"), atom("$foo[bar][baz]")));
        assert!(!var_has_root(atom("$foo[bar]"), atom("$foo[bar][baz]")));
    }

    #[test]
    fn test_var_references_dynamic() {
        assert!(var_references_dynamic(atom("$this->{$name}"), atom("$name")));
        assert!(var_references_dynamic(atom("$this->$name"), atom("$name")));
        assert!(var_references_dynamic(atom("$arr[$name]"), atom("$name")));
        assert!(var_references_dynamic(atom("$this->{$name}->foo"), atom("$name")));
        assert!(!var_references_dynamic(atom("$this->name"), atom("$name")));
        assert!(!var_references_dynamic(atom("$this->$names"), atom("$name")));
        assert!(!var_references_dynamic(atom("$name"), atom("$name")));
        assert!(!var_references_dynamic(atom("$name->foo"), atom("$name")));
        assert!(!var_references_dynamic(atom("$name[0]"), atom("$name")));
    }
}
