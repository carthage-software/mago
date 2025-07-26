pub mod case_scope;
pub mod conditional_scope;
pub mod control_action;
pub mod finally_scope;
pub mod if_scope;
pub mod loop_scope;
pub mod switch_scope;

#[inline]
pub fn var_has_root(var_id: &str, root_var_id: &str) -> bool {
    if var_id == root_var_id {
        return true;
    }

    if !var_id.starts_with(root_var_id) {
        return false;
    }

    let after_root = &var_id[root_var_id.len()..];
    after_root.starts_with("->") || after_root.starts_with("::") || after_root.starts_with("[")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_has_root() {
        assert!(var_has_root("$foo", "$foo"));
        assert!(var_has_root("$foo[bar]", "$foo"));
        assert!(var_has_root("$foo->bar", "$foo"));
        assert!(var_has_root("$foo::bar", "$foo"));
        assert!(var_has_root("$foo->bar[0]", "$foo"));
        assert!(var_has_root("$foo->bar[0]->baz", "$foo"));
        assert!(!var_has_root("$foo[bar]", "$bar"));
        assert!(var_has_root("$foo[bar]", "$foo[bar]"));
        assert!(!var_has_root("$foo[bar]", "$foo[bar][baz]"));
        assert!(!var_has_root("$foo[bar]", "$foo[bar][baz]"));
    }
}
