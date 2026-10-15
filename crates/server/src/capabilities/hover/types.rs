//! Type spelling for hover and the class bound to `$this`.

use mago_bytes::BytesDisplay;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::{TAtomic, object::TObject};
use mago_codex::ttype::union::TUnion;

pub(super) use mago_codex::ttype::display::NameDisplay;

pub(super) fn short_name(name: &[u8]) -> String {
    BytesDisplay(name.rsplit(|byte| *byte == b'\\').next().unwrap_or(name)).to_string()
}

pub(super) fn display_union(codebase: &CodebaseMetadata, ty: &TUnion) -> String {
    NameDisplay::for_types(codebase, std::iter::once(ty)).union(ty)
}

pub(super) fn display_variable(codebase: &CodebaseMetadata, ty: &TUnion, name: &[u8]) -> String {
    if name != b"$this"
        && !ty.types.iter().any(|atomic| matches!(atomic, TAtomic::Object(TObject::Named(object)) if object.is_this))
    {
        return display_union(codebase, ty);
    }
    let mut ty = ty.clone();
    for atomic in ty.types.to_mut() {
        if let TAtomic::Object(TObject::Named(object)) = atomic
            && (name == b"$this" || object.is_this)
        {
            object.is_this = false;
            object.is_static = false;
        }
    }
    display_union(codebase, &ty)
}

#[cfg(test)]
mod tests {
    use mago_codex::ttype::atomic::object::named::TNamedObject;
    use mago_word::word;

    use super::*;

    #[test]
    fn variable_alias_display_keeps_other_static_alternatives_and_source_types() {
        let ty = TUnion::from_vec(vec![
            TAtomic::Object(TObject::Named(TNamedObject::new_this(word("App\\Widget")))),
            TAtomic::Object(TObject::Named(TNamedObject::new_static(word("App\\Other")))),
            TAtomic::Null,
        ]);
        let original = ty.clone();
        assert_eq!(display_variable(&CodebaseMetadata::new(), &ty, b"$alias"), "Other&static|Widget|null");
        assert_eq!(ty, original);
    }
}
