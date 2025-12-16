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
}
