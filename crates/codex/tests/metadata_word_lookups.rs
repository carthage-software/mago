use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_span::Span;
use mago_word::word;

fn codebase() -> CodebaseMetadata {
    let mut codebase = CodebaseMetadata::new();
    let parent = word(b"vendor\\\xffparent");
    let child = word(b"vendor\\\xffchild");
    let method = word(b"m\xffethod");
    let parent_metadata = ClassLikeMetadata::new(parent, parent, Span::zero(), None, MetadataFlags::empty());
    let mut child_metadata = ClassLikeMetadata::new(child, child, Span::zero(), None, MetadataFlags::empty());
    child_metadata.declaring_method_ids.insert(word("renamed"), MethodIdentifier::new(parent, method));
    codebase.class_likes.insert(parent, parent_metadata);
    codebase.class_likes.insert(child, child_metadata);
    codebase.class_like_aliases.insert(word("parentalias"), parent);
    codebase.class_like_aliases.insert(word("childalias"), child);
    codebase.function_likes.insert(
        (parent, method),
        FunctionLikeMetadata::new(FunctionLikeKind::Method, method, method, Span::zero(), MetadataFlags::empty()),
    );

    codebase
}

#[test]
fn word_class_lookup_keeps_case_alias_and_byte_rules() {
    let codebase = codebase();
    for name in
        [b"vendor\\\xffparent".as_slice(), b"Vendor\\\xffPARENT", b"ParentAlias", b"missing", b"vendor\\\xfeparent"]
    {
        assert_eq!(
            codebase.get_class_like_by_name(word(name)),
            codebase.get_class_like(name),
            "Word and byte lookups must agree for {name:?}",
        );
    }

    assert!(codebase.get_class_like_by_name(word("PARENTALIAS")).is_some(), "Aliases must resolve");
}

#[test]
fn word_method_lookup_keeps_alias_and_declaring_method_names() {
    let codebase = codebase();
    for (class, method) in [
        (b"Vendor\\\xffPARENT".as_slice(), b"M\xffETHOD".as_slice()),
        (b"ParentAlias", b"M\xffETHOD"),
        (b"ChildAlias", b"Renamed"),
        (b"Vendor\\\xffCHILD", b"RENAMED"),
        (b"missing", b"M\xffETHOD"),
        (b"ParentAlias", b"missing"),
    ] {
        let id = MethodIdentifier::new(word(class), word(method));
        assert_eq!(codebase.get_method_by_id(&id), codebase.get_method(class, method), "Direct lookups must agree");
        assert_eq!(
            codebase.get_declaring_method_by_id(&id),
            codebase.get_declaring_method(class, method),
            "Declaring lookups must agree",
        );
    }

    let inherited = MethodIdentifier::new(word("ChildAlias"), word("Renamed"));
    assert_eq!(
        codebase.get_declaring_method_by_id(&inherited).map(|metadata| metadata.name),
        Some(word(b"m\xffethod")),
        "The declared method name can differ from the inherited lookup name",
    );
}
