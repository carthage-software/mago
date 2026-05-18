use mago_word::Word;

pub mod case_scope;
pub mod conditional_scope;
pub mod control_action;
pub mod finally_scope;
pub mod if_scope;
pub mod loop_scope;

#[inline]
pub fn var_has_root(var_id: Word, root_var_id: Word) -> bool {
    if var_id == root_var_id {
        return true;
    }

    let var_bytes = var_id.as_bytes();
    let root_bytes = root_var_id.as_bytes();
    if !var_bytes.starts_with(root_bytes) {
        return false;
    }

    let after_root = &var_bytes[root_bytes.len()..];
    after_root.starts_with(b"->") || after_root.starts_with(b"::") || after_root.starts_with(b"[")
}

#[inline]
pub fn var_references_dynamic(var_id: Word, referenced_var_id: Word) -> bool {
    let var_bytes = var_id.as_bytes();
    let ref_bytes = referenced_var_id.as_bytes();

    let mut start = 1;
    while start <= var_bytes.len() {
        let Some(pos) = memchr::memmem::find(&var_bytes[start..], ref_bytes) else {
            return false;
        };
        let end = start + pos + ref_bytes.len();
        if end == var_bytes.len() || (!var_bytes[end].is_ascii_alphanumeric() && var_bytes[end] != b'_') {
            return true;
        }

        start += pos + 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_word::word;

    #[test]
    fn test_var_has_root() {
        assert!(var_has_root(word(b"$foo"), word(b"$foo")));
        assert!(var_has_root(word(b"$foo[bar]"), word(b"$foo")));
        assert!(var_has_root(word(b"$foo->bar"), word(b"$foo")));
        assert!(var_has_root(word(b"$foo::bar"), word(b"$foo")));
        assert!(var_has_root(word(b"$foo->bar[0]"), word(b"$foo")));
        assert!(var_has_root(word(b"$foo->bar[0]->baz"), word(b"$foo")));
        assert!(!var_has_root(word(b"$foo[bar]"), word(b"$bar")));
        assert!(var_has_root(word(b"$foo[bar]"), word(b"$foo[bar]")));
        assert!(!var_has_root(word(b"$foo[bar]"), word(b"$foo[bar][baz]")));
        assert!(!var_has_root(word(b"$foo[bar]"), word(b"$foo[bar][baz]")));
    }

    #[test]
    fn test_var_references_dynamic() {
        assert!(var_references_dynamic(word(b"$this->{$name}"), word(b"$name")));
        assert!(var_references_dynamic(word(b"$this->$name"), word(b"$name")));
        assert!(var_references_dynamic(word(b"$arr[$name]"), word(b"$name")));
        assert!(var_references_dynamic(word(b"$this->{$name}->foo"), word(b"$name")));
        assert!(!var_references_dynamic(word(b"$this->name"), word(b"$name")));
        assert!(!var_references_dynamic(word(b"$this->$names"), word(b"$name")));
        assert!(!var_references_dynamic(word(b"$name"), word(b"$name")));
        assert!(!var_references_dynamic(word(b"$name->foo"), word(b"$name")));
        assert!(!var_references_dynamic(word(b"$name[0]"), word(b"$name")));
    }
}
