#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::borrow::Cow;

use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::populator::populate_codebase;
use mago_codex::populator::populate_codebase_targeted;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::signature_builder;
use mago_codex::ttype::TType;
use mago_database::file::File;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;
use mago_word::WordSet;
use mago_word::ascii_lowercase_word;
use mago_word::empty_word;
use mago_word::word;

fn scan(name: &'static [u8], file_type: FileType, code: &'static str) -> CodebaseMetadata {
    let file = File::new(Cow::Borrowed(name), file_type, None, Cow::Borrowed(code.as_bytes()));
    let arena = LocalArena::new();
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "parse failed: {:?}", program.errors);
    let resolved_names = NameResolver::new(&arena).resolve(program);
    let mut codebase = scan_program(&arena, &file, program, &resolved_names, PHPVersion::LATEST);
    codebase.set_file_signature(file.id, signature_builder::build_file_signature(&file, program, &resolved_names));
    if file_type.is_patch() {
        codebase.convert_partial_to_patch();
    }
    codebase
}

fn populate(codebase: &mut CodebaseMetadata) {
    populate_codebase(codebase, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());
}

fn populate_cached(codebase: &mut CodebaseMetadata, targeted: bool) {
    let safe = codebase
        .class_likes
        .iter()
        .filter_map(|(name, metadata)| metadata.flags.is_populated().then_some(*name))
        .collect();
    if targeted {
        populate_codebase_targeted(
            codebase,
            &mut SymbolReferences::new(),
            safe,
            HashSet::default(),
            HashSet::default(),
        );
    } else {
        populate_codebase(codebase, &mut SymbolReferences::new(), safe, HashSet::default());
    }
}

#[test]
fn builtin_member_flag_patches_refresh_cached_descendants() {
    for targeted in [false, true] {
        for (source, patch, deprecated) in [
            (
                "<?php class P { public const X = 1; public string $value; }
                class C extends P {} class D extends C {}",
                "<?php class P {
                    /** @deprecated */ public const X = 1;
                    /** @deprecated */ public string $value;
                }",
                true,
            ),
            (
                "<?php class P {
                    /** @deprecated */ public const X = 1;
                    /** @deprecated */ public string $value;
                } class C extends P {} class D extends C {}",
                "<?php class P {
                    /** @not-deprecated */ public const X = 1;
                    /** @not-deprecated */ public string $value;
                }",
                false,
            ),
            (
                "<?php interface P { public const X = 1; }
                interface C extends P {} class D implements C {}",
                "<?php interface P { /** @deprecated */ public const X = 1; }",
                true,
            ),
            (
                "<?php trait P { public const X = 1; public string $value; }
                trait C { use P; } class D { use C; }",
                "<?php trait P {
                    /** @deprecated */ public const X = 1;
                    /** @deprecated */ public string $value;
                }",
                true,
            ),
        ] {
            let mut codebase = scan(b"builtin.php", FileType::Builtin, source);
            populate(&mut codebase);
            codebase.extend(scan(b"patch.php", FileType::Patch, patch));

            for _ in 0..2 {
                codebase.apply_patches_pass();
                populate_cached(&mut codebase, targeted);

                for name in ["p", "c", "d"] {
                    let class = &codebase.class_likes[&word(name)];
                    assert_eq!(class.constants[&word("X")].flags.is_deprecated(), deprecated, "{name}");
                    if codebase.get_property(b"P", b"$value").is_some() {
                        let property = codebase.get_declaring_property(name.as_bytes(), b"$value").unwrap();
                        assert_eq!(property.flags.is_deprecated(), deprecated, "{name}");
                    }
                }
            }
        }
    }
}

#[test]
fn builtin_consistent_templates_patch_refreshes_cached_descendants() {
    for targeted in [false, true] {
        let mut codebase = scan(
            b"builtin.php",
            FileType::Builtin,
            "<?php class P {} class C extends P {} class D extends C {} class Unrelated {}",
        );
        populate(&mut codebase);
        codebase.extend(scan(b"patch.php", FileType::Patch, "<?php /** @consistent-templates */ class P {}"));
        codebase.extend(scan(b"source.php", FileType::Host, "<?php class UserChild extends D {}"));

        codebase.apply_patches_pass();
        populate_cached(&mut codebase, targeted);

        for name in ["p", "c", "d", "userchild"] {
            assert!(codebase.class_likes[&word(name)].flags.has_consistent_templates(), "{name}");
        }
        assert!(!codebase.class_likes[&word("unrelated")].flags.has_consistent_templates());
    }
}

#[test]
fn patches_preserve_scanner_control_and_structural_flags() {
    let plain = "<?php class C {} interface I {} function f(): void {}";
    let annotated = "<?php
        /**
         * @final
         * @seal-properties
         * @seal-methods
         */
        class C {}
        /** @enum-interface */
        interface I {}
        /**
         * @ignore-nullable-return
         * @ignore-falsable-return
         * @inheritDoc
         */
        function f(): void {}
    ";

    for (source, patch, enabled) in [(plain, annotated, false), (annotated, plain, true)] {
        let mut codebase = scan(b"vendor.php", FileType::Vendored, source);
        let original_class = codebase.class_likes[&word("c")].clone();
        let original_interface = codebase.class_likes[&word("i")].flags;
        let original_function = codebase.function_likes[&(empty_word(), word("f"))].flags;
        assert_eq!(original_class.flags.is_final(), enabled);
        assert_eq!(original_class.has_sealed_properties, enabled.then_some(true));
        assert_eq!(original_class.has_sealed_methods, enabled.then_some(true));
        assert_eq!(original_interface.contains(MetadataFlags::ENUM_INTERFACE), enabled);
        for flag in
            [MetadataFlags::IGNORE_NULLABLE_RETURN, MetadataFlags::IGNORE_FALSABLE_RETURN, MetadataFlags::INHERITS_DOCS]
        {
            assert_eq!(original_function.contains(flag), enabled);
        }

        codebase.extend(scan(b"patch.php", FileType::Patch, patch));
        codebase.apply_patches_pass();

        let class = &codebase.class_likes[&word("c")];
        assert_eq!(class.flags, original_class.flags);
        assert_eq!(class.has_sealed_properties, original_class.has_sealed_properties);
        assert_eq!(class.has_sealed_methods, original_class.has_sealed_methods);
        assert_eq!(codebase.class_likes[&word("i")].flags, original_interface);
        assert_eq!(codebase.function_likes[&(empty_word(), word("f"))].flags, original_function);
    }
}

#[test]
fn builtin_inherited_constant_patches_preserve_ancestor_flags() {
    for targeted in [false, true] {
        let mut codebase = scan(
            b"builtin.php",
            FileType::Builtin,
            "<?php
                class P { public const X = 1; }
                class C extends P {}
                class D extends C {}
            ",
        );
        populate(&mut codebase);
        codebase.extend(scan(
            b"patch.php",
            FileType::Patch,
            "<?php class C {
                /** @deprecated */ public const X = 1;
            }",
        ));

        codebase.apply_patches_pass();
        populate_cached(&mut codebase, targeted);

        for name in ["c", "d"] {
            assert!(codebase.class_likes[&word(name)].constants[&word("X")].flags.is_deprecated(), "{name}");
        }
        assert!(!codebase.class_likes[&word("p")].constants[&word("X")].flags.is_deprecated());
        assert!(codebase.take_issues(false).is_empty());
    }
}

#[test]
fn builtin_constant_flag_patches_respect_descendant_overrides() {
    let mut codebase = scan(
        b"builtin.php",
        FileType::Builtin,
        "<?php
            class P { /** @deprecated */ public const X = 1; }
            class C extends P {}
            class D extends C {}
            class Own extends P { /** @internal */ public const X = 2; }
        ",
    );
    populate(&mut codebase);
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php
            class C {
                /**
                 * @deprecated
                 * @experimental
                 */
                public const X = 1;
            }
            class P {
                /**
                 * @not-deprecated
                 * @api
                 */
                public const X = 1;
            }
        ",
    ));

    for _ in 0..2 {
        codebase.apply_patches_pass();
        populate_cached(&mut codebase, true);
        for name in ["p", "c", "d"] {
            let flags = codebase.class_likes[&word(name)].constants[&word("X")].flags;
            assert_eq!(flags.is_deprecated(), name != "p", "{name}");
            assert_eq!(flags.contains(MetadataFlags::EXPERIMENTAL), name != "p", "{name}");
            assert!(flags.contains(MetadataFlags::API), "{name}");
        }
        let flags = codebase.class_likes[&word("own")].constants[&word("X")].flags;
        assert!(flags.is_internal());
        assert!(!flags.is_deprecated());
        assert!(!flags.contains(MetadataFlags::API));
        assert!(!flags.contains(MetadataFlags::EXPERIMENTAL));
    }
}

#[test]
fn builtin_flag_patches_preserve_cached_descendant_docblocks() {
    for targeted in [false, true] {
        let mut codebase = scan(
            b"builtin.php",
            FileType::Builtin,
            "<?php class Failure {}
                /** @template T */
                class P { /** @throws T */ public function f(): void {} }
            ",
        );
        codebase.extend(scan(
            b"source.php",
            FileType::Host,
            "<?php /** @extends P<Failure> */ class C extends P { public function f(): void {} }",
        ));
        populate(&mut codebase);
        let throws = codebase.get_declaring_method(b"C", b"f").unwrap().thrown_types.clone();
        assert_eq!(throws.len(), 1);
        assert!(throws[0].inferred);
        codebase.extend(scan(b"patch.php", FileType::Patch, "<?php /** @api */ class P {}"));

        codebase.apply_patches_pass();
        populate_cached(&mut codebase, targeted);

        assert_eq!(codebase.get_declaring_method(b"C", b"f").unwrap().thrown_types, throws);
    }
}

fn assert_inherited_patch_types_populated(codebase: &CodebaseMetadata) {
    let method = &codebase.function_likes[&(word("c"), word("value"))];
    assert!(method.flags.is_pure(), "the patch must preserve its explicit purity flag");
    assert!(
        !method.return_type_metadata.as_ref().unwrap().type_union.needs_population(),
        "the patch return type must be resolved after population"
    );
    assert!(
        !method.parameters[0].type_metadata.as_ref().unwrap().type_union.needs_population(),
        "the patch parameter type must be resolved after population"
    );
}

#[test]
fn inherited_method_patch_populates_types_after_reapplication() {
    for file_type in [FileType::Vendored, FileType::Builtin] {
        let mut codebase = scan(
            b"vendor.php",
            file_type,
            "<?php class ResultType {} class P { public function value($value) {} } class C extends P {}",
        );
        codebase.extend(scan(
            b"patch.php",
            FileType::Patch,
            "<?php class C extends P {
                /**
                 * @pure
                 * @param ResultType $value
                 * @return ResultType
                 */
                public function value($value) {}
            }",
        ));
        codebase.apply_patches_pass();
        populate(&mut codebase);
        assert_inherited_patch_types_populated(&codebase);

        codebase.extend(scan(b"unrelated.php", FileType::Vendored, "<?php class Unrelated {}"));
        codebase.apply_patches_pass();
        populate(&mut codebase);
        assert_inherited_patch_types_populated(&codebase);
    }
}

#[test]
fn inherited_method_patch_populates_types_from_populated_builtin_parent() {
    let mut codebase = scan(
        b"builtin.php",
        FileType::Builtin,
        "<?php class ResultType {} class P { public function value($value) {} }",
    );
    populate(&mut codebase);
    codebase.extend(scan(b"vendor.php", FileType::Vendored, "<?php class C extends P {}"));
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php class C extends P {
            /**
             * @pure
             * @param ResultType $value
             * @return ResultType
             */
            public function value($value) {}
        }",
    ));
    codebase.apply_patches_pass();
    populate(&mut codebase);
    assert_inherited_patch_types_populated(&codebase);
}

#[test]
fn patches_merge_explicit_semantic_docblock_flags() {
    let mut codebase = scan(
        b"vendor.php",
        FileType::Vendored,
        "<?php
            /**
             * @deprecated
             */
            class Uid
            {
                /**
                 * @pure
                 * @deprecated
                 */
                public function mutate(): void {}

                public function equals(): bool { return true; }

                public function externalOnly(): void {}

                /** @deprecated */
                public string $id;

                /** @deprecated */
                public const VERSION = 1;
            }

            class ParentUid
            {
                /** @deprecated */
                public function inherited(): void {}
            }

            class ChildUid extends ParentUid {}

            /**
             * @deprecated
             */
            function vendor_function(): void {}

            /** @deprecated */
            const GLOBAL_VERSION = 1;
        ",
    );
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php
            /**
             * @not-deprecated
             * @api
             * @consistent-constructor
             * @consistent-templates
             * @final
             */
            class Uid
            {
                /**
                 * @impure
                 * @not-deprecated
                 */
                public function mutate(): void {}

                /**
                 * @phpstan-pure
                 * @must-use
                 * @psalm-mutation-free
                 * @no-named-arguments
                 * @suspends-fiber
                 * @ignore-nullable-return
                 */
                public function equals(): bool { return true; }

                /** @external-mutation-free */
                public function externalOnly(): void {}

                /**
                 * @internal
                 * @readonly
                 */
                public string $id;

                /** @experimental */
                public const VERSION = 1;
            }

            class ChildUid extends ParentUid
            {
                /** @pure */
                public function inherited(): void {}
            }

            /**
             * @not-deprecated
             * @pure
             */
            function vendor_function(): void {}

            /**
             * @psalm-not-deprecated
             * @api
             */
            const GLOBAL_VERSION = 1;
        ",
    ));
    codebase.apply_patches_pass();

    let uid = &codebase.class_likes[&ascii_lowercase_word(b"Uid")];
    assert!(!uid.flags.is_deprecated());
    assert!(!uid.flags.is_final());
    assert!(uid.flags.contains(MetadataFlags::API));
    assert!(uid.flags.has_consistent_constructor());
    assert!(uid.flags.has_consistent_templates());

    let mutate = &codebase.function_likes[&(ascii_lowercase_word(b"Uid"), ascii_lowercase_word(b"mutate"))];
    assert!(!mutate.flags.is_pure());
    assert!(!mutate.flags.is_deprecated());

    let equals = &codebase.function_likes[&(ascii_lowercase_word(b"Uid"), ascii_lowercase_word(b"equals"))];
    assert!(equals.flags.is_pure());
    assert!(equals.flags.must_use());
    assert!(equals.flags.contains(MetadataFlags::MUTATION_FREE));
    assert!(equals.flags.contains(MetadataFlags::EXTERNAL_MUTATION_FREE));
    assert!(equals.flags.contains(MetadataFlags::NO_NAMED_ARGUMENTS));
    assert!(equals.flags.contains(MetadataFlags::SUSPENDS_FIBER));
    assert!(!equals.flags.ignore_nullable_return());

    let external_only =
        &codebase.function_likes[&(ascii_lowercase_word(b"Uid"), ascii_lowercase_word(b"externalonly"))];
    assert!(!external_only.flags.contains(MetadataFlags::MUTATION_FREE));
    assert!(external_only.flags.contains(MetadataFlags::EXTERNAL_MUTATION_FREE));

    let property = &uid.properties[&word("$id")];
    assert!(property.flags.is_deprecated());
    assert!(property.flags.is_internal());
    assert!(!property.flags.is_readonly());

    let class_constant = &uid.constants[&word("VERSION")];
    assert!(class_constant.flags.is_deprecated());
    assert!(class_constant.flags.contains(MetadataFlags::EXPERIMENTAL));

    let inherited = &codebase.function_likes[&(ascii_lowercase_word(b"ChildUid"), ascii_lowercase_word(b"inherited"))];
    assert!(inherited.flags.is_pure());
    assert!(inherited.flags.is_deprecated());

    let global_function = &codebase.function_likes[&(empty_word(), ascii_lowercase_word(b"vendor_function"))];
    assert!(!global_function.flags.is_deprecated());
    assert!(global_function.flags.is_pure());

    let global_constant = &codebase.constants[&word("GLOBAL_VERSION")];
    assert!(!global_constant.flags.is_deprecated());
    assert!(global_constant.flags.contains(MetadataFlags::API));
}

#[test]
fn inherited_method_patch_preserves_flags_from_trait_before_parent_class() {
    let mut codebase = scan(
        b"vendor.php",
        FileType::Vendored,
        "<?php
            trait TraitUid
            {
                /** @internal */
                public function inherited(): void {}
            }

            class ParentUid
            {
                /** @deprecated */
                public function inherited(): void {}
            }

            class ChildUid extends ParentUid
            {
                use TraitUid;
            }
        ",
    );
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php
            class ChildUid extends ParentUid
            {
                /** @pure */
                public function inherited(): void {}
            }
        ",
    ));
    codebase.apply_patches_pass();

    let inherited = &codebase.function_likes[&(ascii_lowercase_word(b"ChildUid"), ascii_lowercase_word(b"inherited"))];
    assert!(inherited.flags.is_pure());
    assert!(inherited.flags.is_internal());
    assert!(!inherited.flags.is_deprecated());
}

#[test]
fn inherited_method_patch_ignores_flags_from_excluded_trait() {
    let mut codebase = scan(
        b"vendor.php",
        FileType::Vendored,
        "<?php
            trait ATrait
            {
                /** @internal */
                public function inherited(): void {}
            }

            trait BTrait
            {
                /** @deprecated */
                public function inherited(): void {}
            }

            class ChildUid
            {
                use ATrait, BTrait {
                    BTrait::inherited insteadof ATrait;
                }
            }
        ",
    );
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php
            class ChildUid
            {
                /** @pure */
                public function inherited(): void {}
            }
        ",
    ));
    codebase.apply_patches_pass();

    let inherited = &codebase.function_likes[&(ascii_lowercase_word(b"ChildUid"), ascii_lowercase_word(b"inherited"))];
    assert!(inherited.flags.is_pure());
    assert!(inherited.flags.is_deprecated());
    assert!(!inherited.flags.is_internal());
}

#[test]
fn inherited_method_patches_use_updated_ancestor_flags() {
    for file_type in [FileType::Vendored, FileType::Builtin] {
        let mut codebase = scan(
            b"vendor.php",
            file_type,
            r#"<?php
                class RootUid
                {
                    /**
                     * @pure
                     * @deprecated
                     * @ignore-nullable-return
                     */
                    public function &value(): \Generator { yield 1; }
                }

                class ParentUid extends RootUid {}
                class ChildUid extends ParentUid {}
                class SiblingUid extends ParentUid {}
            "#,
        );
        codebase.extend(scan(
            b"patch.php",
            FileType::Patch,
            r#"<?php
                class ChildUid extends ParentUid
                {
                    /** @internal */
                    public function value(): \Generator { throw new \Exception(); }
                }

                class SiblingUid extends ParentUid
                {
                    /** @api */
                    public function value(): \Generator {}
                }

                class ParentUid extends RootUid
                {
                    /** @not-deprecated */
                    public function value(): \Generator {}
                }

                class RootUid
                {
                    /** @impure */
                    public function &value(): \Generator {}
                }
            "#,
        ));
        codebase.apply_patches_pass();

        for name in [b"ParentUid".as_slice(), b"ChildUid".as_slice(), b"SiblingUid".as_slice()] {
            let inherited = &codebase.function_likes[&(ascii_lowercase_word(name), word("value"))];
            assert!(!inherited.flags.is_pure());
            assert!(!inherited.flags.is_deprecated());
            assert!(inherited.flags.has_yield());
            assert!(!inherited.flags.has_throw());
            assert!(inherited.flags.contains(MetadataFlags::BY_REFERENCE));
            assert!(inherited.flags.ignore_nullable_return());
        }

        let child = &codebase.function_likes[&(ascii_lowercase_word(b"ChildUid"), word("value"))];
        let sibling = &codebase.function_likes[&(ascii_lowercase_word(b"SiblingUid"), word("value"))];
        assert!(child.flags.is_internal());
        assert!(!sibling.flags.is_internal());
        assert!(sibling.flags.contains(MetadataFlags::API));
    }
}

#[test]
fn inherited_method_patch_preserves_flags_from_qualified_trait_alias() {
    let mut codebase = scan(
        b"vendor.php",
        FileType::Vendored,
        "<?php
            trait ATrait
            {
                /**
                 * @deprecated
                 * @internal
                 */
                public function original(): void {}
            }

            trait BTrait
            {
                /** @api */
                public function original(): void {}
            }

            class ChildUid
            {
                use ATrait, BTrait {
                    BTrait::original insteadof ATrait;
                    ATrait::original as aliased;
                }
            }
        ",
    );
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php
            class ChildUid
            {
                /**
                 * @pure
                 * @not-deprecated
                 */
                public function aliased(): void {}
            }
        ",
    ));
    codebase.apply_patches_pass();

    let aliased = &codebase.function_likes[&(ascii_lowercase_word(b"ChildUid"), word("aliased"))];
    assert!(aliased.flags.is_pure());
    assert!(aliased.flags.is_internal());
    assert!(!aliased.flags.is_deprecated());
    assert!(!aliased.flags.contains(MetadataFlags::API));
    assert!(codebase.take_issues(false).is_empty());
}

#[test]
fn inherited_method_patch_preserves_flags_through_nested_trait_aliases() {
    let mut codebase = scan(
        b"vendor.php",
        FileType::Vendored,
        "<?php
            trait SourceTrait
            {
                /** @deprecated */
                public function original(): void {}
            }

            trait WrapperTrait
            {
                use SourceTrait { original as nested; }
            }

            class ChildUid
            {
                use WrapperTrait { nested as visible; }
            }
        ",
    );
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php
            class ChildUid
            {
                /** @pure */
                public function visible(): void {}
            }
        ",
    ));
    codebase.apply_patches_pass();

    let visible = &codebase.function_likes[&(ascii_lowercase_word(b"ChildUid"), word("visible"))];
    assert!(visible.flags.is_pure());
    assert!(visible.flags.is_deprecated());
    assert!(codebase.take_issues(false).is_empty());
}

#[test]
fn inherited_method_patches_use_overrides_through_unpatched_classes() {
    let mut codebase = scan(
        b"vendor.php",
        FileType::Vendored,
        "<?php
            trait SourceTrait
            {
                /**
                 * @pure
                 * @deprecated
                 */
                public function original(): void {}
            }

            class ParentUid
            {
                use SourceTrait { original as visible; }
            }

            class BridgeUid extends ParentUid {}
            class ChildUid extends BridgeUid {}
            class SiblingUid extends BridgeUid {}
        ",
    );
    codebase.extend(scan(
        b"patch.php",
        FileType::Patch,
        "<?php
            class ChildUid extends BridgeUid
            {
                /** @internal */
                public function visible(): void {}
            }

            class SiblingUid extends BridgeUid
            {
                /** @api */
                public function visible(): void {}
            }

            class ParentUid
            {
                /**
                 * @impure
                 * @not-deprecated
                 */
                public function visible(): void {}
            }
        ",
    ));
    codebase.apply_patches_pass();

    for name in [b"ParentUid".as_slice(), b"ChildUid".as_slice(), b"SiblingUid".as_slice()] {
        let inherited = &codebase.function_likes[&(ascii_lowercase_word(name), word("visible"))];
        assert!(!inherited.flags.is_pure());
        assert!(!inherited.flags.is_deprecated());
    }

    let original = &codebase.function_likes[&(ascii_lowercase_word(b"SourceTrait"), word("original"))];
    assert!(original.flags.is_pure());
    assert!(original.flags.is_deprecated());

    let child = &codebase.function_likes[&(ascii_lowercase_word(b"ChildUid"), word("visible"))];
    let sibling = &codebase.function_likes[&(ascii_lowercase_word(b"SiblingUid"), word("visible"))];
    assert!(child.flags.is_internal());
    assert!(!sibling.flags.is_internal());
    assert!(sibling.flags.contains(MetadataFlags::API));
    assert!(codebase.take_issues(false).is_empty());
}

#[test]
fn inherited_method_patch_prefers_concrete_parent_over_trait_requirements() {
    for file_type in [FileType::Vendored, FileType::Builtin] {
        for source in [
            "<?php
                trait T { /** @internal */ abstract public function value(): void; }
                class P { /** @deprecated */ public function value(): void {} }
                class C extends P { use T; }
            ",
            "<?php
                /** @method void value() */
                trait T {}
                class P { /** @deprecated */ public function value(): void {} }
                class C extends P { use T; }
            ",
        ] {
            let mut codebase = scan(b"vendor.php", file_type, source);
            codebase.extend(scan(
                b"patch.php",
                FileType::Patch,
                "<?php class C extends P { /** @pure */ public function value(): void {} }",
            ));
            codebase.apply_patches_pass();
            populate(&mut codebase);

            let method = codebase.get_declaring_method(b"C", b"value").unwrap();
            assert!(method.flags.is_pure());
            assert!(method.flags.is_deprecated());
            assert!(!method.flags.is_internal());
            assert!(!method.flags.is_magic_method());
        }
    }
}

#[test]
fn consistent_constructor_patch_preserves_existing_constructors() {
    for file_type in [FileType::Vendored, FileType::Builtin] {
        for source in [
            "<?php
                class P {}
                class C extends P { public function __construct(int $value) {} }
            ",
            "<?php
                class P { public function __construct(int $value) {} }
                class C extends P {}
            ",
            "<?php
                trait T { public function __construct(int $value) {} }
                class P {}
                class C extends P { use T; }
            ",
            "<?php
                class Root { public function __construct(int $value) {} }
                class P extends Root {}
                class C extends P {}
            ",
        ] {
            let mut codebase = scan(b"vendor.php", file_type, source);
            codebase.extend(scan(
                b"patch.php",
                FileType::Patch,
                "<?php /** @consistent-constructor */ class C extends P {}",
            ));
            codebase.apply_patches_pass();
            populate(&mut codebase);

            let constructor = codebase.get_declaring_method(b"C", b"__construct").unwrap();
            assert_eq!(constructor.parameters.len(), 1);
            assert!(!constructor.flags.is_pure());
            assert!(codebase.take_issues(false).is_empty());
        }
    }
}

#[test]
fn consistent_constructor_patch_records_implicit_constructor() {
    for file_type in [FileType::Vendored, FileType::Builtin] {
        let mut codebase = scan(b"vendor.php", file_type, "<?php class P {} class C extends P {}");
        codebase.extend(scan(
            b"patch.php",
            FileType::Patch,
            "<?php /** @consistent-constructor */ class C extends P {}",
        ));
        codebase.apply_patches_pass();
        populate(&mut codebase);

        let constructor = codebase.get_declaring_method(b"C", b"__construct").unwrap();
        assert!(constructor.parameters.is_empty());
        assert!(constructor.flags.is_pure());
        assert_eq!(constructor.flags.is_built_in(), file_type.is_builtin());
        assert!(!constructor.flags.is_patch());
    }
}

#[test]
fn inherited_method_patch_refreshes_flags_after_vendor_rescan() {
    for (original, updated) in [
        (
            "<?php class P { /** @deprecated */ public function value(): void {} }",
            "<?php class P { public function value(): void {} }",
        ),
        (
            "<?php
                trait T { /** @internal */ public function value(): void {} }
                class P { use T; /** @deprecated */ public function value(): void {} }
            ",
            "<?php
                trait T { /** @internal */ public function value(): void {} }
                class P { use T; public function value(): void {} }
            ",
        ),
    ] {
        let parent = scan(b"parent.php", FileType::Vendored, original);
        let child = scan(b"child.php", FileType::Vendored, "<?php class Bridge extends P {} class C extends Bridge {}");
        let patch_source = "<?php class C extends Bridge { /** @pure */ public function value(): void {} }";
        let patch = scan(b"patch.php", FileType::Patch, patch_source);
        let parent_spans = parent.entry_spans();
        let patch_spans = patch.entry_spans();
        let mut codebase = CodebaseMetadata::new();
        codebase.extend(parent);
        codebase.extend(child);
        codebase.extend(patch);
        codebase.apply_patches_pass();
        populate(&mut codebase);
        assert!(codebase.get_declaring_method(b"C", b"value").unwrap().flags.is_deprecated());

        let parent_keys = parent_spans.into_owned_keys(&codebase);
        let patch_keys = patch_spans.into_owned_keys(&codebase);
        codebase.remove_entries_by_keys(&parent_keys);
        codebase.remove_entries_by_keys(&patch_keys);
        codebase.extend(scan(b"parent.php", FileType::Vendored, updated));
        codebase.extend(scan(b"patch.php", FileType::Patch, patch_source));
        codebase.apply_patches_pass();
        populate(&mut codebase);

        let method = codebase.get_declaring_method(b"C", b"value").unwrap();
        assert!(method.flags.is_pure());
        assert!(!method.flags.is_deprecated());
        assert!(!method.flags.is_internal());

        codebase.remove_entries_by_keys(&parent_keys);
        codebase.remove_entries_by_keys(&patch_keys);
        codebase.extend(scan(b"parent.php", FileType::Vendored, "<?php class P {}"));
        codebase.extend(scan(b"patch.php", FileType::Patch, patch_source));
        codebase.apply_patches_pass();
        populate(&mut codebase);

        assert!(codebase.get_declaring_method(b"C", b"value").is_none());
    }
}

#[test]
fn consistent_constructor_patch_uses_constructor_added_after_vendor_rescan() {
    let parent = scan(b"parent.php", FileType::Vendored, "<?php class P {}");
    let child = scan(b"child.php", FileType::Vendored, "<?php class C extends P {}");
    let patch_source = "<?php /** @consistent-constructor */ class C extends P {}";
    let patch = scan(b"patch.php", FileType::Patch, patch_source);
    let parent_spans = parent.entry_spans();
    let patch_spans = patch.entry_spans();
    let mut codebase = CodebaseMetadata::new();
    codebase.extend(parent);
    codebase.extend(child);
    codebase.extend(patch);
    codebase.apply_patches_pass();
    populate(&mut codebase);
    assert!(codebase.get_declaring_method(b"C", b"__construct").unwrap().parameters.is_empty());

    let parent_keys = parent_spans.into_owned_keys(&codebase);
    let patch_keys = patch_spans.into_owned_keys(&codebase);
    codebase.remove_entries_by_keys(&parent_keys);
    codebase.remove_entries_by_keys(&patch_keys);
    codebase.extend(scan(
        b"parent.php",
        FileType::Vendored,
        "<?php class P { public function __construct(int $value) {} }",
    ));
    codebase.extend(scan(b"patch.php", FileType::Patch, patch_source));
    codebase.apply_patches_pass();
    populate(&mut codebase);

    let constructor = codebase.get_declaring_method(b"C", b"__construct").unwrap();
    assert_eq!(constructor.parameters.len(), 1);
    assert!(!constructor.flags.is_pure());
}
