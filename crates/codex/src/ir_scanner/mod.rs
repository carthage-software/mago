use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::attribute::Attribute;
use mago_hir::ir::attribute::AttributeTarget;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::definition::AnonymousClass;
use mago_hir::ir::expression::definition::DefinitionExpressionKind;
use mago_hir::ir::flags::Flags;
use mago_hir::ir::identifier::Identifier;
use mago_hir::ir::member::Method;
use mago_hir::ir::member::TraitUse;
use mago_hir::ir::member::TraitUseAdaptation;
use mago_hir::ir::modifier::Modifier;
use mago_hir::ir::modifier::ModifierKind;
use mago_hir::ir::statement::definition::Class;
use mago_hir::ir::statement::definition::ClassFlags;
use mago_hir::ir::statement::definition::Constant;
use mago_hir::ir::statement::definition::ConstantFlags;
use mago_hir::ir::statement::definition::Enum;
use mago_hir::ir::statement::definition::EnumFlags;
use mago_hir::ir::statement::definition::Function;
use mago_hir::ir::statement::definition::Interface;
use mago_hir::ir::statement::definition::InterfaceFlags;
use mago_hir::ir::statement::definition::Trait;
use mago_hir::ir::statement::definition::TraitFlags;
use mago_hir::ir::r#type::TypeKind;
use mago_hir::walker::MutWalker;
use mago_php_version::PHPVersionRange;
use mago_word::Word;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::ascii_lowercase_word;
use mago_word::empty_word;
use mago_word::word;

use crate::flags::attribute::AttributeFlags;
use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeKind;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::version_constraint::VersionConstraint;
use crate::symbol::SymbolKind;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::scalar::TScalar;
use crate::visibility::Visibility;

mod attribute;
mod function_like;
mod generics;
mod hook;
mod inference;
mod member;
mod ttype;

struct Scanner<'ctx> {
    codebase: CodebaseMetadata,
    origin_flags: MetadataFlags,
    file: &'ctx File,
}

#[must_use]
pub fn scan(file: &File, ir: &IR<'_, (), (), ()>) -> CodebaseMetadata {
    let mut scanner = Scanner {
        codebase: CodebaseMetadata::default(),
        origin_flags: MetadataFlags::origin_flags(file.file_type),
        file,
    };
    scanner.walk_ir(ir, &mut ());
    scanner.codebase
}

fn has_modifier(modifiers: &[Modifier], kind: ModifierKind) -> bool {
    modifiers.iter().any(|modifier| modifier.kind == kind)
}

pub(super) fn version_constraint_from(ranges: &[PHPVersionRange]) -> VersionConstraint {
    VersionConstraint { ranges: ranges.to_vec() }
}

fn attribute_flags_from(target: Option<Flags<AttributeTarget>>) -> Option<AttributeFlags> {
    let target = target?;

    let mut flags = AttributeFlags::empty();
    if target.is_set(AttributeTarget::Class) {
        flags.insert(AttributeFlags::TARGET_CLASS);
    }
    if target.is_set(AttributeTarget::Function) {
        flags.insert(AttributeFlags::TARGET_FUNCTION);
    }
    if target.is_set(AttributeTarget::Method) {
        flags.insert(AttributeFlags::TARGET_METHOD);
    }
    if target.is_set(AttributeTarget::Property) {
        flags.insert(AttributeFlags::TARGET_PROPERTY);
    }
    if target.is_set(AttributeTarget::ClassConstant) {
        flags.insert(AttributeFlags::TARGET_CLASS_CONSTANT);
    }
    if target.is_set(AttributeTarget::Parameter) {
        flags.insert(AttributeFlags::TARGET_PARAMETER);
    }
    if target.is_set(AttributeTarget::Constant) {
        flags.insert(AttributeFlags::TARGET_CONSTANT);
    }
    if target.is_set(AttributeTarget::Repeatable) {
        flags.insert(AttributeFlags::IS_REPEATABLE);
    }

    Some(flags)
}

impl Scanner<'_> {
    fn base_metadata(
        &self,
        name: &Identifier<'_>,
        kind: SymbolKind,
        flags: MetadataFlags,
        attributes: &[Attribute<'_, (), (), ()>],
        version_constraint: &[PHPVersionRange],
        attribute_target: Option<Flags<AttributeTarget>>,
    ) -> ClassLikeMetadata {
        self.base_metadata_named(
            ascii_lowercase_word(name.value),
            word(name.value),
            name.span,
            Some(name.span),
            kind,
            flags,
            attributes,
            version_constraint,
            attribute_target,
        )
    }

    fn base_metadata_named(
        &self,
        name: Word,
        original_name: Word,
        span: mago_span::Span,
        name_span: Option<mago_span::Span>,
        kind: SymbolKind,
        flags: MetadataFlags,
        attributes: &[Attribute<'_, (), (), ()>],
        version_constraint: &[PHPVersionRange],
        attribute_target: Option<Flags<AttributeTarget>>,
    ) -> ClassLikeMetadata {
        let mut metadata = ClassLikeMetadata::new(name, original_name, span, name_span, flags);
        metadata.kind = kind;
        metadata.attributes = attribute::scan_attributes(attributes);
        metadata.version_constraint = version_constraint_from(version_constraint);
        metadata.attribute_flags = attribute_flags_from(attribute_target);
        metadata
    }

    fn add_require_constraints(
        metadata: &mut ClassLikeMetadata,
        require_extends: &[mago_hir::ir::inheritance::annotation::RequireExtendsAnnotation<'_>],
        require_implements: &[mago_hir::ir::inheritance::annotation::RequireImplementsAnnotation<'_>],
    ) {
        for annotation in require_extends {
            metadata.require_extends.insert(ascii_lowercase_word(annotation.r#type.kind.identifier().value));
        }
        for annotation in require_implements {
            metadata.require_implements.insert(ascii_lowercase_word(annotation.r#type.kind.identifier().value));
        }
    }

    fn add_used_traits(metadata: &mut ClassLikeMetadata, trait_uses: &[TraitUse<'_>]) {
        for trait_use in trait_uses {
            for trait_name in trait_use.traits {
                metadata.add_used_trait(ascii_lowercase_word(trait_name.value));
            }

            for adaptation in trait_use.adaptations {
                let TraitUseAdaptation::Alias(adaptation) = adaptation else {
                    continue;
                };

                let method_name = ascii_lowercase_word(adaptation.method.value);
                let method_alias = ascii_lowercase_word(adaptation.alias.value);
                if method_name != method_alias {
                    metadata.add_trait_alias(method_name, method_alias);
                }

                if let Some(modifier) = adaptation.visibility {
                    match modifier.kind {
                        ModifierKind::Public => {
                            metadata.add_trait_visibility(method_alias, Visibility::Public);
                        }
                        ModifierKind::Protected => {
                            metadata.add_trait_visibility(method_alias, Visibility::Protected);
                        }
                        ModifierKind::Private => {
                            metadata.add_trait_visibility(method_alias, Visibility::Private);
                        }
                        ModifierKind::Final => {
                            metadata.trait_final_map.insert(method_alias);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn finish(&mut self, metadata: ClassLikeMetadata) {
        let name = metadata.name;
        match metadata.kind {
            SymbolKind::Class => self.codebase.symbols.add_class_name(name),
            SymbolKind::Interface => self.codebase.symbols.add_interface_name(name),
            SymbolKind::Trait => self.codebase.symbols.add_trait_name(name),
            SymbolKind::Enum => self.codebase.symbols.add_enum_name(name),
        }

        self.codebase.class_likes.insert(name, metadata);
    }

    fn is_already_scanned(&self, name: &Identifier<'_>) -> bool {
        self.codebase.class_likes.contains_key(&ascii_lowercase_word(name.value))
    }

    fn scan_methods(&mut self, metadata: &mut ClassLikeMetadata, methods: &[Method<'_, (), (), ()>]) {
        let class_name = metadata.name;
        let is_trait = metadata.kind.is_trait();
        let class_context = generics::class_template_context(metadata);
        let mut has_constructor = false;
        for method in methods {
            let lookup_name = ascii_lowercase_word(method.name.value);

            if metadata.methods.contains(&lookup_name) {
                if metadata.pseudo_methods.contains(&lookup_name) {
                    metadata.pseudo_methods.remove(&lookup_name);
                    if let Some(existing) = self.codebase.function_likes.get_mut(&(class_name, lookup_name)) {
                        existing.flags.remove(MetadataFlags::MAGIC_METHOD);
                    }
                }

                continue;
            }

            let function_like = function_like::scan_method(
                method,
                class_name,
                self.origin_flags,
                &class_context,
                self.file,
                &self.codebase.constants,
            );
            let (visibility, is_final, is_constructor) =
                function_like.method_metadata.as_ref().map_or((Visibility::Public, false, false), |method_metadata| {
                    (method_metadata.visibility, method_metadata.is_final, method_metadata.is_constructor)
                });
            let is_clone = method.name.value.eq_ignore_ascii_case(b"__clone");
            has_constructor |= is_constructor;

            metadata.methods.insert(lookup_name);
            let identifier = MethodIdentifier::new(class_name, lookup_name);
            metadata.add_declaring_method_id(lookup_name, identifier);
            if !visibility.is_private() || is_constructor || is_clone || is_trait {
                metadata.inheritable_method_ids.insert(lookup_name, identifier);
            }
            if is_final && is_constructor {
                metadata.flags |= MetadataFlags::CONSISTENT_CONSTRUCTOR;
            }

            if is_constructor {
                for (parameter, parameter_metadata) in method.parameters.iter().zip(function_like.parameters.iter()) {
                    if function_like::is_promoted(parameter) {
                        let property = function_like::promoted_property(
                            parameter,
                            parameter_metadata,
                            self.origin_flags,
                            class_name,
                        );
                        metadata.add_property_metadata(property);
                    }
                }
            }

            self.codebase.function_likes.insert((class_name, lookup_name), function_like);
        }

        if !has_constructor && metadata.flags.has_consistent_constructor() {
            let constructor_name = word("__construct");
            metadata.methods.insert(constructor_name);
            let identifier = MethodIdentifier::new(class_name, constructor_name);
            metadata.add_declaring_method_id(constructor_name, identifier);
            metadata.inheritable_method_ids.insert(constructor_name, identifier);

            self.codebase.function_likes.insert(
                (class_name, constructor_name),
                FunctionLikeMetadata::new(
                    FunctionLikeKind::Method,
                    constructor_name,
                    constructor_name,
                    metadata.span,
                    MetadataFlags::PURE | self.origin_flags,
                ),
            );
        }
    }

    fn scan_method_annotations(
        &mut self,
        metadata: &mut ClassLikeMetadata,
        annotations: &[mago_hir::ir::member::annotation::MethodAnnotation<'_, (), (), ()>],
    ) {
        let class_name = metadata.name;
        for annotation in annotations {
            let lookup_name = ascii_lowercase_word(annotation.name.value);
            metadata.methods.insert(lookup_name);
            metadata.pseudo_methods.insert(lookup_name);
            metadata.inheritable_method_ids.insert(lookup_name, MethodIdentifier::new(class_name, lookup_name));

            let function_like = function_like::magic_method(annotation, class_name);
            self.codebase.function_likes.insert((class_name, lookup_name), function_like);
        }
    }

    fn add_enum_method(&mut self, metadata: &mut ClassLikeMetadata, name: &[u8], function_like: FunctionLikeMetadata) {
        let lookup_name = ascii_lowercase_word(name);
        let identifier = MethodIdentifier::new(metadata.name, lookup_name);
        metadata.methods.insert(lookup_name);
        metadata.add_declaring_method_id(lookup_name, identifier);
        metadata.inheritable_method_ids.insert(lookup_name, identifier);
        self.codebase.function_likes.insert((metadata.name, lookup_name), function_like);
    }

    fn add_enum_synthetic_methods(&mut self, metadata: &mut ClassLikeMetadata, span: mago_span::Span) {
        let enum_name = metadata.name;
        if let Some(backing) = metadata.enum_type.clone() {
            self.add_enum_method(metadata, b"from", function_like::enum_from_method(enum_name, span, backing.clone()));
            self.add_enum_method(metadata, b"tryFrom", function_like::enum_try_from_method(enum_name, span, backing));
        }

        let has_cases = !metadata.enum_cases.is_empty();
        self.add_enum_method(metadata, b"cases", function_like::enum_cases_method(enum_name, span, has_cases));
    }

    fn scan_class(&mut self, class: &Class<'_, (), (), ()>) {
        if self.is_already_scanned(&class.name) {
            return;
        }

        let mut flags = self.origin_flags;
        if has_modifier(class.modifiers, ModifierKind::Final) {
            flags |= MetadataFlags::FINAL;
        }
        if has_modifier(class.modifiers, ModifierKind::Abstract) {
            flags |= MetadataFlags::ABSTRACT;
        }
        if has_modifier(class.modifiers, ModifierKind::Readonly) {
            flags |= MetadataFlags::READONLY;
        }
        if class.flags.is_set(ClassFlags::Internal) {
            flags |= MetadataFlags::INTERNAL;
        }
        if class.flags.is_set(ClassFlags::API) {
            flags |= MetadataFlags::API;
        }
        if class.flags.is_set(ClassFlags::Experimental) {
            flags |= MetadataFlags::EXPERIMENTAL;
        }
        if class.flags.is_set(ClassFlags::Deprecated) {
            flags |= MetadataFlags::DEPRECATED;
        }
        if class.flags.is_set(ClassFlags::ConsistentConstructor) {
            flags |= MetadataFlags::CONSISTENT_CONSTRUCTOR;
        }
        if class.flags.is_set(ClassFlags::ConsistentTypeParameterAnnotations) {
            flags |= MetadataFlags::CONSISTENT_TEMPLATES;
        }

        let mut metadata = self.base_metadata(
            &class.name,
            SymbolKind::Class,
            flags,
            class.attributes,
            class.version_constraint,
            class.attribute_target,
        );

        if class.flags.is_set(ClassFlags::SealedMethods) {
            metadata.has_sealed_methods = Some(true);
        }
        if class.flags.is_set(ClassFlags::UnsealedMethods) {
            metadata.has_sealed_methods = Some(false);
        }
        if class.flags.is_set(ClassFlags::SealedProperties) {
            metadata.has_sealed_properties = Some(true);
        }
        if class.flags.is_set(ClassFlags::UnsealedProperties) {
            metadata.has_sealed_properties = Some(false);
        }

        if let Some(sealed) = class.sealed_annotation {
            metadata.permitted_inheritors = Some(
                sealed.types.iter().map(|inheritor| ascii_lowercase_word(inheritor.kind.identifier().value)).collect(),
            );
        }

        if let Some(extends) = class.extends {
            let parent = ascii_lowercase_word(extends.r#type.value);
            metadata.direct_parent_class = Some(parent);
            metadata.all_parent_classes.insert(parent);
        }

        if let Some(implements) = class.implements {
            for interface in implements.types {
                metadata.add_direct_parent_interface(ascii_lowercase_word(interface.value));
            }
        }

        Self::add_used_traits(&mut metadata, class.trait_uses);
        Self::add_require_constraints(
            &mut metadata,
            class.require_extends_annotations,
            class.require_implements_annotations,
        );

        generics::scan_type_aliases(&mut metadata, class.type_alias_annotations, class.imported_type_alias_annotations);
        generics::scan_mixins(&mut metadata, class.mixin_annotations);
        generics::scan_template_types(&mut metadata, class.type_parameter_annotations);
        generics::scan_extends_offsets(&mut metadata, class.extends_annotations);
        generics::scan_implements_offsets(&mut metadata, class.implements_annotations);
        generics::scan_use_offsets(&mut metadata, class.trait_uses);

        member::scan_class_constants(
            &mut metadata,
            class.constants,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        member::scan_property_annotations(&mut metadata, class.property_annotations);
        member::scan_properties(
            &mut metadata,
            class.properties,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        hook::scan_hooked_properties(
            &mut metadata,
            class.hooked_properties,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        self.scan_method_annotations(&mut metadata, class.method_annotations);
        self.scan_methods(&mut metadata, class.methods);

        self.finish(metadata);
    }

    fn scan_interface(&mut self, interface: &Interface<'_, (), (), ()>) {
        if self.is_already_scanned(&interface.name) {
            return;
        }

        let mut flags = self.origin_flags | MetadataFlags::ABSTRACT;
        if interface.flags.is_set(InterfaceFlags::Internal) {
            flags |= MetadataFlags::INTERNAL;
        }
        if interface.flags.is_set(InterfaceFlags::API) {
            flags |= MetadataFlags::API;
        }
        if interface.flags.is_set(InterfaceFlags::Experimental) {
            flags |= MetadataFlags::EXPERIMENTAL;
        }
        if interface.flags.is_set(InterfaceFlags::Deprecated) {
            flags |= MetadataFlags::DEPRECATED;
        }
        if interface.flags.is_set(InterfaceFlags::ConsistentConstructor) {
            flags |= MetadataFlags::CONSISTENT_CONSTRUCTOR;
        }
        if interface.flags.is_set(InterfaceFlags::ConsistentTypeParameterAnnotations) {
            flags |= MetadataFlags::CONSISTENT_TEMPLATES;
        }
        if interface.flags.is_set(InterfaceFlags::EnumInterface) {
            flags |= MetadataFlags::ENUM_INTERFACE;
        }

        let mut metadata = self.base_metadata(
            &interface.name,
            SymbolKind::Interface,
            flags,
            interface.attributes,
            interface.version_constraint,
            interface.attribute_target,
        );

        if interface.flags.is_set(InterfaceFlags::SealedMethods) {
            metadata.has_sealed_methods = Some(true);
        }
        if interface.flags.is_set(InterfaceFlags::UnsealedMethods) {
            metadata.has_sealed_methods = Some(false);
        }
        if interface.flags.is_set(InterfaceFlags::SealedProperties) {
            metadata.has_sealed_properties = Some(true);
        }
        if interface.flags.is_set(InterfaceFlags::UnsealedProperties) {
            metadata.has_sealed_properties = Some(false);
        }

        if let Some(sealed) = interface.sealed_annotation {
            metadata.permitted_inheritors = Some(
                sealed.types.iter().map(|inheritor| ascii_lowercase_word(inheritor.kind.identifier().value)).collect(),
            );
        }

        if let Some(extends) = interface.extends {
            for parent in extends.types {
                metadata.add_direct_parent_interface(ascii_lowercase_word(parent.value));
            }
        }

        Self::add_require_constraints(
            &mut metadata,
            interface.require_extends_annotations,
            interface.require_implements_annotations,
        );

        generics::scan_type_aliases(
            &mut metadata,
            interface.type_alias_annotations,
            interface.imported_type_alias_annotations,
        );
        generics::scan_mixins(&mut metadata, interface.mixin_annotations);
        generics::scan_template_types(&mut metadata, interface.type_parameter_annotations);
        generics::scan_extends_offsets(&mut metadata, interface.extends_annotations);

        member::scan_class_constants(
            &mut metadata,
            interface.constants,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        hook::scan_hooked_properties(
            &mut metadata,
            interface.hooked_properties,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        self.scan_method_annotations(&mut metadata, interface.method_annotations);
        self.scan_methods(&mut metadata, interface.methods);

        self.finish(metadata);
    }

    fn scan_trait(&mut self, r#trait: &Trait<'_, (), (), ()>) {
        if self.is_already_scanned(&r#trait.name) {
            return;
        }

        let mut flags = self.origin_flags;
        if r#trait.flags.is_set(TraitFlags::Internal) {
            flags |= MetadataFlags::INTERNAL;
        }
        if r#trait.flags.is_set(TraitFlags::API) {
            flags |= MetadataFlags::API;
        }
        if r#trait.flags.is_set(TraitFlags::Experimental) {
            flags |= MetadataFlags::EXPERIMENTAL;
        }
        if r#trait.flags.is_set(TraitFlags::Deprecated) {
            flags |= MetadataFlags::DEPRECATED;
        }

        let mut metadata = self.base_metadata(
            &r#trait.name,
            SymbolKind::Trait,
            flags,
            r#trait.attributes,
            r#trait.version_constraint,
            r#trait.attribute_target,
        );

        if r#trait.flags.is_set(TraitFlags::SealedMethods) {
            metadata.has_sealed_methods = Some(true);
        }
        if r#trait.flags.is_set(TraitFlags::UnsealedMethods) {
            metadata.has_sealed_methods = Some(false);
        }
        if r#trait.flags.is_set(TraitFlags::SealedProperties) {
            metadata.has_sealed_properties = Some(true);
        }
        if r#trait.flags.is_set(TraitFlags::UnsealedProperties) {
            metadata.has_sealed_properties = Some(false);
        }

        if let Some(sealed) = r#trait.sealed_annotation {
            metadata.permitted_inheritors = Some(
                sealed.types.iter().map(|inheritor| ascii_lowercase_word(inheritor.kind.identifier().value)).collect(),
            );
        }

        Self::add_used_traits(&mut metadata, r#trait.trait_uses);
        Self::add_require_constraints(
            &mut metadata,
            r#trait.require_extends_annotations,
            r#trait.require_implements_annotations,
        );

        generics::scan_type_aliases(
            &mut metadata,
            r#trait.type_alias_annotations,
            r#trait.imported_type_alias_annotations,
        );
        generics::scan_mixins(&mut metadata, r#trait.mixin_annotations);
        generics::scan_template_types(&mut metadata, r#trait.type_parameter_annotations);
        generics::scan_use_offsets(&mut metadata, r#trait.trait_uses);

        member::scan_class_constants(
            &mut metadata,
            r#trait.constants,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        member::scan_property_annotations(&mut metadata, r#trait.property_annotations);
        member::scan_properties(
            &mut metadata,
            r#trait.properties,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        hook::scan_hooked_properties(
            &mut metadata,
            r#trait.hooked_properties,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        self.scan_method_annotations(&mut metadata, r#trait.method_annotations);
        self.scan_methods(&mut metadata, r#trait.methods);

        self.finish(metadata);
    }

    fn scan_enum(&mut self, r#enum: &Enum<'_, (), (), ()>) {
        if self.is_already_scanned(&r#enum.name) {
            return;
        }

        let mut flags = self.origin_flags | MetadataFlags::FINAL;
        if r#enum.flags.is_set(EnumFlags::Internal) {
            flags |= MetadataFlags::INTERNAL;
        }
        if r#enum.flags.is_set(EnumFlags::API) {
            flags |= MetadataFlags::API;
        }
        if r#enum.flags.is_set(EnumFlags::Experimental) {
            flags |= MetadataFlags::EXPERIMENTAL;
        }
        if r#enum.flags.is_set(EnumFlags::Deprecated) {
            flags |= MetadataFlags::DEPRECATED;
        }

        let mut metadata = self.base_metadata(
            &r#enum.name,
            SymbolKind::Enum,
            flags,
            r#enum.attributes,
            r#enum.version_constraint,
            r#enum.attribute_target,
        );

        if r#enum.flags.is_set(EnumFlags::SealedMethods) {
            metadata.has_sealed_methods = Some(true);
        }
        if r#enum.flags.is_set(EnumFlags::UnsealedMethods) {
            metadata.has_sealed_methods = Some(false);
        }

        match r#enum.backing_type {
            Some(backing_type) => match backing_type.r#type.kind {
                TypeKind::String => {
                    metadata.enum_type = Some(TAtomic::Scalar(TScalar::string()));
                    metadata.add_direct_parent_interface(word("__internal_do_not_use__stringbackedenum"));
                }
                TypeKind::Integer => {
                    metadata.enum_type = Some(TAtomic::Scalar(TScalar::int()));
                    metadata.add_direct_parent_interface(word("__internal_do_not_use__intbackedenum"));
                }
                _ => {
                    metadata.add_direct_parent_interface(word("backedenum"));
                }
            },
            None => {
                metadata.add_direct_parent_interface(word("unitenum"));
            }
        }

        if let Some(implements) = r#enum.implements {
            for interface in implements.types {
                metadata.add_direct_parent_interface(ascii_lowercase_word(interface.value));
            }
        }

        Self::add_used_traits(&mut metadata, r#enum.trait_uses);
        Self::add_require_constraints(
            &mut metadata,
            r#enum.require_extends_annotations,
            r#enum.require_implements_annotations,
        );

        generics::scan_type_aliases(
            &mut metadata,
            r#enum.type_alias_annotations,
            r#enum.imported_type_alias_annotations,
        );
        generics::scan_mixins(&mut metadata, r#enum.mixin_annotations);
        generics::scan_implements_offsets(&mut metadata, r#enum.implements_annotations);
        generics::scan_use_offsets(&mut metadata, r#enum.trait_uses);

        self.add_enum_synthetic_methods(&mut metadata, r#enum.name.span);
        member::scan_class_constants(
            &mut metadata,
            r#enum.constants,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        member::scan_enum_cases(
            &mut metadata,
            r#enum.enum_cases,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        member::scan_enum_properties(&mut metadata, r#enum.name.span);
        self.scan_method_annotations(&mut metadata, r#enum.method_annotations);
        self.scan_methods(&mut metadata, r#enum.methods);

        self.finish(metadata);
    }

    fn scan_anonymous_class(&mut self, anonymous_class: &AnonymousClass<'_, (), (), ()>, span: mago_span::Span) {
        let original_name = crate::build_synthetic_name("anonymous-class", self.file, span);
        let name = ascii_lowercase_word(original_name.as_bytes());
        if self.codebase.class_likes.contains_key(&name) {
            return;
        }

        let mut metadata = self.base_metadata_named(
            name,
            original_name,
            span,
            None,
            SymbolKind::Class,
            self.origin_flags,
            anonymous_class.attributes,
            anonymous_class.version_constraint,
            anonymous_class.attribute_target,
        );

        if let Some(extends) = anonymous_class.extends {
            let parent = ascii_lowercase_word(extends.r#type.value);
            metadata.direct_parent_class = Some(parent);
            metadata.all_parent_classes.insert(parent);
        }

        if let Some(implements) = anonymous_class.implements {
            for interface in implements.types {
                metadata.add_direct_parent_interface(ascii_lowercase_word(interface.value));
            }
        }

        Self::add_used_traits(&mut metadata, anonymous_class.trait_uses);
        generics::scan_mixins(&mut metadata, anonymous_class.mixin_annotations);

        generics::scan_extends_offsets(&mut metadata, anonymous_class.extends_annotations);
        generics::scan_implements_offsets(&mut metadata, anonymous_class.implements_annotations);
        generics::scan_use_offsets(&mut metadata, anonymous_class.trait_uses);

        member::scan_class_constants(
            &mut metadata,
            anonymous_class.constants,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        member::scan_properties(
            &mut metadata,
            anonymous_class.properties,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        hook::scan_hooked_properties(
            &mut metadata,
            anonymous_class.hooked_properties,
            self.origin_flags,
            self.file,
            &self.codebase.constants,
        );
        self.scan_methods(&mut metadata, anonymous_class.methods);

        self.finish(metadata);
    }

    fn scan_function(&mut self, function: &Function<'_, (), (), ()>) {
        let key = (empty_word(), ascii_lowercase_word(function.name.value));
        if self.codebase.function_likes.contains_key(&key) {
            return;
        }

        let function_like =
            function_like::scan_function(function, self.origin_flags, self.file, &self.codebase.constants);
        self.codebase.function_likes.insert(key, function_like);
    }

    fn scan_constant_declaration(&mut self, constant: &Constant<'_, (), (), ()>) {
        let mut flags = self.origin_flags;
        if constant.flags.is_set(ConstantFlags::Deprecated) {
            flags |= MetadataFlags::DEPRECATED;
        }
        if constant.flags.is_set(ConstantFlags::Internal) {
            flags |= MetadataFlags::INTERNAL;
        }
        if constant.flags.is_set(ConstantFlags::Experimental) {
            flags |= MetadataFlags::EXPERIMENTAL;
        }
        if constant.flags.is_set(ConstantFlags::API) {
            flags |= MetadataFlags::API;
        }

        let attributes = attribute::scan_attributes(constant.attributes);
        let type_metadata =
            constant.type_annotation.map(|annotation| ttype::type_metadata_from_annotation(annotation, None));

        for item in constant.items {
            let name = ascii_lowercase_constant_name_word(item.name.value);
            if self.codebase.constants.contains_key(&name) {
                continue;
            }

            let mut metadata = ConstantMetadata::new(name, item.name.span, flags);
            metadata.attributes.clone_from(&attributes);
            metadata.version_constraint = version_constraint_from(constant.version_constraint);
            metadata.type_metadata.clone_from(&type_metadata);
            metadata.inferred_type = inference::infer(item.value, None, self.file, &self.codebase.constants);
            self.codebase.constants.insert(name, metadata);
        }
    }
}

impl<'arena> MutWalker<'arena, (), (), (), ()> for Scanner<'_> {
    fn walk_in_class(&mut self, class: &Class<'arena, (), (), ()>, _context: &mut ()) {
        self.scan_class(class);
    }

    fn walk_in_expression(&mut self, expression: &Expression<'arena, (), (), ()>, _context: &mut ()) {
        let ExpressionKind::Definition(definition) = &expression.kind else {
            return;
        };

        match &definition.kind {
            DefinitionExpressionKind::AnonymousClass(anonymous_class) => {
                self.scan_anonymous_class(anonymous_class, expression.span);
            }
            DefinitionExpressionKind::Closure(closure) => {
                let name = crate::build_synthetic_name("closure", self.file, expression.span);
                let key = (empty_word(), name);
                if !self.codebase.function_likes.contains_key(&key) {
                    let function_like = function_like::scan_closure(
                        closure,
                        name,
                        expression.span,
                        self.origin_flags,
                        self.file,
                        &self.codebase.constants,
                    );
                    self.codebase.function_likes.insert(key, function_like);
                }
            }
            DefinitionExpressionKind::ArrowFunction(arrow) => {
                let name = crate::build_synthetic_name("closure", self.file, expression.span);
                let key = (empty_word(), name);
                if !self.codebase.function_likes.contains_key(&key) {
                    let function_like = function_like::scan_arrow_function(
                        arrow,
                        name,
                        expression.span,
                        self.origin_flags,
                        self.file,
                        &self.codebase.constants,
                    );
                    self.codebase.function_likes.insert(key, function_like);
                }
            }
        }
    }

    fn walk_in_interface(&mut self, interface: &Interface<'arena, (), (), ()>, _context: &mut ()) {
        self.scan_interface(interface);
    }

    fn walk_in_trait_definition(&mut self, r#trait: &Trait<'arena, (), (), ()>, _context: &mut ()) {
        self.scan_trait(r#trait);
    }

    fn walk_in_enum_definition(&mut self, r#enum: &Enum<'arena, (), (), ()>, _context: &mut ()) {
        self.scan_enum(r#enum);
    }

    fn walk_in_function(&mut self, function: &Function<'arena, (), (), ()>, _context: &mut ()) {
        self.scan_function(function);
    }

    fn walk_in_constant(&mut self, constant: &Constant<'arena, (), (), ()>, _context: &mut ()) {
        self.scan_constant_declaration(constant);
    }
}
