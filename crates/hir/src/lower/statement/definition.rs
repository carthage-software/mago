use bumpalo::collections::Vec;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::tag::TagValue;
use mago_phpdoc_syntax::cst::tag::TagVendor;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::effect::annotation::AssertAnnotation;
use crate::ir::effect::annotation::SelfOutAnnotation;
use crate::ir::effect::annotation::ThrowsAnnotation;
use crate::ir::flags::Flags;
use crate::ir::generics::TypeParameterDefiningEntity;
use crate::ir::generics::Variance;
use crate::ir::generics::annotation::TypeParameterAnnotation;
use crate::ir::generics::annotation::WhereConstraintAnnotation;
use crate::ir::identifier::Identifier;
use crate::ir::inheritance::annotation::ExtendsAnnotation;
use crate::ir::inheritance::annotation::ImplementsAnnotation;
use crate::ir::inheritance::annotation::MixinAnnotation;
use crate::ir::inheritance::annotation::RequireExtendsAnnotation;
use crate::ir::inheritance::annotation::RequireImplementsAnnotation;
use crate::ir::inheritance::annotation::SealedAnnotation;
use crate::ir::inheritance::annotation::UseAnnotation;
use crate::ir::member::MethodFlags;
use crate::ir::member::PropertyFlags;
use crate::ir::member::annotation::ImportedTypeAliasAnnotation;
use crate::ir::member::annotation::MethodAnnotation;
use crate::ir::member::annotation::PropertyAnnotation;
use crate::ir::member::annotation::PropertyAnnotationKind;
use crate::ir::member::annotation::TypeAliasAnnotation;
use crate::ir::parameter::Parameter;
use crate::ir::statement::definition::Class;
use crate::ir::statement::definition::ClassFlags;
use crate::ir::statement::definition::Constant;
use crate::ir::statement::definition::ConstantFlags;
use crate::ir::statement::definition::ConstantItem;
use crate::ir::statement::definition::Enum;
use crate::ir::statement::definition::EnumBackingType;
use crate::ir::statement::definition::EnumFlags;
use crate::ir::statement::definition::Function;
use crate::ir::statement::definition::FunctionFlags;
use crate::ir::statement::definition::Interface;
use crate::ir::statement::definition::InterfaceFlags;
use crate::ir::statement::definition::Trait;
use crate::ir::statement::definition::TraitFlags;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::lower::Lowering;

pub(crate) struct ClassLikeAnnotations<'arena> {
    pub(crate) type_parameters: &'arena [TypeParameterAnnotation<'arena>],
    pub(crate) type_aliases: &'arena [TypeAliasAnnotation<'arena>],
    pub(crate) imported_type_aliases: &'arena [ImportedTypeAliasAnnotation<'arena>],
    pub(crate) extends: &'arena [ExtendsAnnotation<'arena>],
    pub(crate) implements: &'arena [ImplementsAnnotation<'arena>],
    pub(crate) require_extends: &'arena [RequireExtendsAnnotation<'arena>],
    pub(crate) require_implements: &'arena [RequireImplementsAnnotation<'arena>],
    pub(crate) sealed: Option<&'arena SealedAnnotation<'arena>>,
    pub(crate) mixins: &'arena [MixinAnnotation<'arena>],
    pub(crate) methods: &'arena [MethodAnnotation<'arena, (), (), ()>],
    pub(crate) properties: &'arena [PropertyAnnotation<'arena>],
}

pub(crate) struct FunctionLikeAnnotations<'arena> {
    pub(crate) type_parameters: &'arena [TypeParameterAnnotation<'arena>],
    pub(crate) where_constraints: &'arena [WhereConstraintAnnotation<'arena>],
    pub(crate) return_type: Option<&'arena TypeAnnotation<'arena>>,
    pub(crate) throws: &'arena [ThrowsAnnotation<'arena>],
    pub(crate) asserts: &'arena [AssertAnnotation<'arena>],
    pub(crate) asserts_if_true: &'arena [AssertAnnotation<'arena>],
    pub(crate) asserts_if_false: &'arena [AssertAnnotation<'arena>],
    pub(crate) self_out: Option<&'arena SelfOutAnnotation<'arena>>,
    pub(crate) parameters: &'arena [(&'arena [u8], &'arena TypeAnnotation<'arena>)],
    pub(crate) parameter_outs: &'arena [(&'arena [u8], &'arena TypeAnnotation<'arena>)],
}

#[derive(Default, Clone, Copy)]
pub(crate) struct MarkerFlags {
    deprecated: bool,
    not_deprecated: bool,
    internal: bool,
    api: bool,
    experimental: bool,
    pure: bool,
    r#final: bool,
    mutation_free: bool,
    external_mutation_free: bool,
    no_named_arguments: bool,
    must_use: bool,
    ignore_nullable_return: bool,
    ignore_falsable_return: bool,
    consistent_constructor: bool,
    consistent_templates: bool,
    seal_properties: bool,
    no_seal_properties: bool,
    seal_methods: bool,
    no_seal_methods: bool,
    enum_interface: bool,
    inherit_doc: bool,
}

impl MarkerFlags {
    pub(crate) fn class_flags(self) -> Flags<ClassFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(ClassFlags::Deprecated);
        }
        if self.internal {
            flags.set(ClassFlags::Internal);
        }
        if self.api {
            flags.set(ClassFlags::API);
        }
        if self.experimental {
            flags.set(ClassFlags::Experimental);
        }
        if self.consistent_constructor {
            flags.set(ClassFlags::ConsistentConstructor);
        }
        if self.consistent_templates {
            flags.set(ClassFlags::ConsistentTypeParameterAnnotations);
        }
        if self.seal_properties {
            flags.set(ClassFlags::SealedProperties);
        }
        if self.no_seal_properties {
            flags.set(ClassFlags::UnsealedProperties);
        }
        if self.seal_methods {
            flags.set(ClassFlags::SealedMethods);
        }
        if self.no_seal_methods {
            flags.set(ClassFlags::UnsealedMethods);
        }

        flags
    }

    pub(crate) fn interface_flags(self) -> Flags<InterfaceFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(InterfaceFlags::Deprecated);
        }
        if self.internal {
            flags.set(InterfaceFlags::Internal);
        }
        if self.api {
            flags.set(InterfaceFlags::API);
        }
        if self.experimental {
            flags.set(InterfaceFlags::Experimental);
        }
        if self.enum_interface {
            flags.set(InterfaceFlags::EnumInterface);
        }
        if self.consistent_constructor {
            flags.set(InterfaceFlags::ConsistentConstructor);
        }
        if self.consistent_templates {
            flags.set(InterfaceFlags::ConsistentTypeParameterAnnotations);
        }
        if self.seal_properties {
            flags.set(InterfaceFlags::SealedProperties);
        }
        if self.no_seal_properties {
            flags.set(InterfaceFlags::UnsealedProperties);
        }
        if self.seal_methods {
            flags.set(InterfaceFlags::SealedMethods);
        }
        if self.no_seal_methods {
            flags.set(InterfaceFlags::UnsealedMethods);
        }

        flags
    }

    pub(crate) fn trait_flags(self) -> Flags<TraitFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(TraitFlags::Deprecated);
        }
        if self.internal {
            flags.set(TraitFlags::Internal);
        }
        if self.api {
            flags.set(TraitFlags::API);
        }
        if self.experimental {
            flags.set(TraitFlags::Experimental);
        }
        if self.seal_properties {
            flags.set(TraitFlags::SealedProperties);
        }
        if self.no_seal_properties {
            flags.set(TraitFlags::UnsealedProperties);
        }
        if self.seal_methods {
            flags.set(TraitFlags::SealedMethods);
        }
        if self.no_seal_methods {
            flags.set(TraitFlags::UnsealedMethods);
        }

        flags
    }

    pub(crate) fn enum_flags(self) -> Flags<EnumFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(EnumFlags::Deprecated);
        }
        if self.internal {
            flags.set(EnumFlags::Internal);
        }
        if self.api {
            flags.set(EnumFlags::API);
        }
        if self.experimental {
            flags.set(EnumFlags::Experimental);
        }
        if self.seal_methods {
            flags.set(EnumFlags::SealedMethods);
        }
        if self.no_seal_methods {
            flags.set(EnumFlags::UnsealedMethods);
        }

        flags
    }

    pub(crate) fn function_flags(self) -> Flags<FunctionFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(FunctionFlags::Deprecated);
        }
        if self.internal {
            flags.set(FunctionFlags::Internal);
        }
        if self.api {
            flags.set(FunctionFlags::API);
        }
        if self.experimental {
            flags.set(FunctionFlags::Experimental);
        }
        if self.pure {
            flags.set(FunctionFlags::Pure);
        }
        if self.no_named_arguments {
            flags.set(FunctionFlags::NoNamedArguments);
        }
        if self.must_use {
            flags.set(FunctionFlags::MustUse);
        }
        if self.ignore_nullable_return {
            flags.set(FunctionFlags::IgnoreNullableReturnType);
        }
        if self.ignore_falsable_return {
            flags.set(FunctionFlags::IgnoreFalsableReturnType);
        }

        flags
    }

    pub(crate) fn method_flags(self) -> Flags<MethodFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(MethodFlags::Deprecated);
        }
        if self.internal {
            flags.set(MethodFlags::Internal);
        }
        if self.api {
            flags.set(MethodFlags::API);
        }
        if self.experimental {
            flags.set(MethodFlags::Experimental);
        }
        if self.r#final {
            flags.set(MethodFlags::Final);
        }
        if self.pure {
            flags.set(MethodFlags::Pure);
        }
        if self.mutation_free {
            flags.set(MethodFlags::MutationFree);
        }
        if self.external_mutation_free {
            flags.set(MethodFlags::ExternalMutationFree);
        }
        if self.no_named_arguments {
            flags.set(MethodFlags::NoNamedArguments);
        }
        if self.must_use {
            flags.set(MethodFlags::MustUse);
        }
        if self.ignore_nullable_return {
            flags.set(MethodFlags::IgnoreNullableReturnType);
        }
        if self.ignore_falsable_return {
            flags.set(MethodFlags::IgnoreFalsableReturnType);
        }
        if self.inherit_doc {
            flags.set(MethodFlags::InheritDoc);
        }

        flags
    }

    pub(crate) fn property_flags(self) -> Flags<PropertyFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(PropertyFlags::Deprecated);
        }
        if self.internal {
            flags.set(PropertyFlags::Internal);
        }
        if self.api {
            flags.set(PropertyFlags::API);
        }
        if self.experimental {
            flags.set(PropertyFlags::Experimental);
        }
        if self.r#final {
            flags.set(PropertyFlags::Final);
        }

        flags
    }

    pub(crate) fn constant_flags(self) -> Flags<ConstantFlags> {
        let mut flags = Flags::new();
        if self.deprecated {
            flags.set(ConstantFlags::Deprecated);
        }
        if self.internal {
            flags.set(ConstantFlags::Internal);
        }
        if self.api {
            flags.set(ConstantFlags::API);
        }
        if self.experimental {
            flags.set(ConstantFlags::Experimental);
        }

        flags
    }
}

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_class_like_annotations(
        &mut self,
        document: Option<&Document<'arena>>,
        owner: Identifier<'arena>,
    ) -> ClassLikeAnnotations<'arena> {
        let arena = self.arena;
        let mut type_parameters = Vec::new_in(arena);
        let mut type_aliases = Vec::new_in(arena);
        let mut imported_type_aliases = Vec::new_in(arena);
        let mut extends = Vec::new_in(arena);
        let mut implements = Vec::new_in(arena);
        let mut require_extends = Vec::new_in(arena);
        let mut require_implements = Vec::new_in(arena);
        let mut sealed = None;
        let mut sealed_vendor: Option<TagVendor> = None;
        let mut mixins = Vec::new_in(arena);
        let mut methods = Vec::new_in(arena);
        let mut properties = Vec::new_in(arena);

        if let Some(document) = document {
            for element in document.elements.iter() {
                let Element::Tag(tag) = element else { continue };
                let tag = *tag;
                if let TagValue::Template(template) = &tag.value {
                    let mut buffer = [0u8; 32];
                    let variance = match self.normalize_tag_name(tag.name.value, &mut buffer) {
                        b"templatecovariant" => Variance::Covariant,
                        b"templatecontravariant" => Variance::Contravariant,
                        _ => Variance::Invariant,
                    };
                    let annotation = self.lower_type_parameter_annotation(template, variance);
                    self.type_resolution.add_template(template.name.value, annotation.bound);
                    type_parameters.push(annotation);
                }
            }

            for element in document.elements.iter() {
                let Element::Tag(tag) = element else { continue };
                let tag = *tag;
                match &tag.value {
                    TagValue::TypeAlias(alias) => {
                        let annotation = self.lower_type_alias_annotation(alias);
                        self.type_resolution.add_alias(alias.alias.value, owner, annotation.name);
                        type_aliases.push(annotation);
                    }
                    TagValue::TypeAliasImport(import) => {
                        let annotation = self.lower_imported_type_alias_annotation(import);
                        let local = import
                            .imported_as
                            .as_ref()
                            .map_or(import.imported_alias.value, |imported_as| imported_as.local.value);
                        self.type_resolution.add_alias(local, annotation.from, annotation.name);
                        imported_type_aliases.push(annotation);
                    }
                    _ => {}
                }
            }

            for element in document.elements.iter() {
                let Element::Tag(tag) = element else { continue };
                let tag = *tag;

                match &tag.value {
                    TagValue::Extends(value) => extends.extend(self.lower_extends_annotation(value)),
                    TagValue::Implements(value) => implements.extend(self.lower_implements_annotation(value)),
                    TagValue::RequireExtends(value) => {
                        require_extends.extend(self.lower_require_extends_annotation(value));
                    }
                    TagValue::RequireImplements(value) => {
                        require_implements.extend(self.lower_require_implements_annotation(value));
                    }
                    TagValue::Mixin(value) => mixins.extend(self.lower_mixin_annotation(value)),
                    TagValue::Sealed(value) => {
                        if sealed.is_none() || tag.vendor > sealed_vendor {
                            sealed_vendor = tag.vendor;
                            sealed = Some(&*arena.alloc(self.lower_sealed_annotation(value)));
                        }
                    }
                    TagValue::Inheritors(value) => {
                        if sealed.is_none() || tag.vendor > sealed_vendor {
                            sealed_vendor = tag.vendor;
                            sealed = Some(&*arena.alloc(self.lower_inheritors_annotation(value)));
                        }
                    }
                    TagValue::Method(value) => methods.push(self.lower_method_annotation(value, owner)),
                    TagValue::Property(value) => {
                        let mut buffer = [0u8; 32];
                        let kind = match self.normalize_tag_name(tag.name.value, &mut buffer) {
                            b"propertyread" => PropertyAnnotationKind::Read,
                            b"propertywrite" => PropertyAnnotationKind::Write,
                            _ => PropertyAnnotationKind::ReadWrite,
                        };
                        properties.push(self.lower_property_annotation(value, kind));
                    }
                    _ => {}
                }
            }
        }

        ClassLikeAnnotations {
            type_parameters: type_parameters.into_bump_slice(),
            type_aliases: type_aliases.into_bump_slice(),
            imported_type_aliases: imported_type_aliases.into_bump_slice(),
            extends: extends.into_bump_slice(),
            implements: implements.into_bump_slice(),
            require_extends: require_extends.into_bump_slice(),
            require_implements: require_implements.into_bump_slice(),
            sealed,
            mixins: mixins.into_bump_slice(),
            methods: methods.into_bump_slice(),
            properties: properties.into_bump_slice(),
        }
    }

    pub(crate) fn lower_function_like_annotations(
        &mut self,
        document: Option<&Document<'arena>>,
    ) -> FunctionLikeAnnotations<'arena> {
        let arena = self.arena;
        let mut type_parameters = Vec::new_in(arena);
        let mut where_constraints = Vec::new_in(arena);
        let mut return_type = None;
        let mut return_vendor: Option<TagVendor> = None;
        let mut throws = Vec::new_in(arena);
        let mut asserts = Vec::new_in(arena);
        let mut asserts_if_true = Vec::new_in(arena);
        let mut asserts_if_false = Vec::new_in(arena);
        let mut self_out = None;
        let mut self_out_vendor: Option<TagVendor> = None;
        let mut parameters: Vec<'arena, (&'arena [u8], &'arena TypeAnnotation<'arena>, Option<TagVendor>)> =
            Vec::new_in(arena);
        let mut parameter_outs: Vec<'arena, (&'arena [u8], &'arena TypeAnnotation<'arena>, Option<TagVendor>)> =
            Vec::new_in(arena);

        if let Some(document) = document {
            for element in document.elements.iter() {
                let Element::Tag(tag) = element else { continue };
                let tag = *tag;
                if let TagValue::Template(template) = &tag.value {
                    let mut buffer = [0u8; 32];
                    let variance = match self.normalize_tag_name(tag.name.value, &mut buffer) {
                        b"templatecovariant" => Variance::Covariant,
                        b"templatecontravariant" => Variance::Contravariant,
                        _ => Variance::Invariant,
                    };

                    let annotation = self.lower_type_parameter_annotation(template, variance);
                    self.type_resolution.add_template(template.name.value, annotation.bound);
                    type_parameters.push(annotation);
                }
            }

            for element in document.elements.iter() {
                let Element::Tag(tag) = element else { continue };
                let tag = *tag;
                match &tag.value {
                    TagValue::Where(value) => where_constraints.push(self.lower_where_constraint_annotation(value)),
                    TagValue::Return(value) => {
                        if return_type.is_none() || tag.vendor > return_vendor {
                            return_vendor = tag.vendor;
                            return_type = Some(self.lower_type_annotation(value.r#type));
                        }
                    }
                    TagValue::Throws(value) => throws.push(self.lower_throws_annotation(value)),
                    TagValue::Assert(value) => {
                        let annotation = self.lower_assert_annotation(value);
                        let mut buffer = [0u8; 32];
                        match self.normalize_tag_name(tag.name.value, &mut buffer) {
                            b"assertiftrue" => asserts_if_true.push(annotation),
                            b"assertiffalse" => asserts_if_false.push(annotation),
                            _ => asserts.push(annotation),
                        }
                    }
                    TagValue::AssertMethod(value) => {
                        let annotation = self.lower_assert_method_annotation(value);
                        let mut buffer = [0u8; 32];
                        match self.normalize_tag_name(tag.name.value, &mut buffer) {
                            b"assertiftrue" => asserts_if_true.push(annotation),
                            b"assertiffalse" => asserts_if_false.push(annotation),
                            _ => asserts.push(annotation),
                        }
                    }
                    TagValue::AssertProperty(value) => {
                        let annotation = self.lower_assert_property_annotation(value);
                        let mut buffer = [0u8; 32];
                        match self.normalize_tag_name(tag.name.value, &mut buffer) {
                            b"assertiftrue" => asserts_if_true.push(annotation),
                            b"assertiffalse" => asserts_if_false.push(annotation),
                            _ => asserts.push(annotation),
                        }
                    }
                    TagValue::SelfOut(value) => {
                        if self_out.is_none() || tag.vendor > self_out_vendor {
                            self_out_vendor = tag.vendor;
                            self_out = Some(&*arena.alloc(self.lower_self_out_annotation(value)));
                        }
                    }
                    TagValue::Param(value) => {
                        let name = value.parameter.value;
                        if let Some(entry) = parameters.iter_mut().find(|entry| entry.0 == name) {
                            if tag.vendor > entry.2 {
                                entry.1 = self.lower_type_annotation(value.r#type);
                                entry.2 = tag.vendor;
                            }
                        } else {
                            parameters.push((name, self.lower_type_annotation(value.r#type), tag.vendor));
                        }
                    }
                    TagValue::ParamOut(value) => {
                        let name = value.parameter.value;
                        if let Some(entry) = parameter_outs.iter_mut().find(|entry| entry.0 == name) {
                            if tag.vendor > entry.2 {
                                entry.1 = self.lower_type_annotation(value.r#type);
                                entry.2 = tag.vendor;
                            }
                        } else {
                            parameter_outs.push((name, self.lower_type_annotation(value.r#type), tag.vendor));
                        }
                    }
                    _ => {}
                }
            }
        }

        FunctionLikeAnnotations {
            type_parameters: type_parameters.into_bump_slice(),
            where_constraints: where_constraints.into_bump_slice(),
            return_type,
            throws: throws.into_bump_slice(),
            asserts: asserts.into_bump_slice(),
            asserts_if_true: asserts_if_true.into_bump_slice(),
            asserts_if_false: asserts_if_false.into_bump_slice(),
            self_out,
            parameters: arena.alloc_slice_fill_iter(parameters.iter().map(|entry| (entry.0, entry.1))),
            parameter_outs: arena.alloc_slice_fill_iter(parameter_outs.iter().map(|entry| (entry.0, entry.1))),
        }
    }

    pub(crate) fn merge_parameter_annotations(
        &self,
        parameters: &'arena [Parameter<'arena, (), (), ()>],
        parameter_types: &[(&'arena [u8], &'arena TypeAnnotation<'arena>)],
        parameter_out_types: &[(&'arena [u8], &'arena TypeAnnotation<'arena>)],
    ) -> &'arena [Parameter<'arena, (), (), ()>] {
        self.arena.alloc_slice_fill_iter(parameters.iter().map(|parameter| {
            let mut parameter = *parameter;
            parameter.type_annotation =
                parameter_types.iter().find(|entry| entry.0 == parameter.variable.name).map(|entry| entry.1);
            parameter.out_annotation =
                parameter_out_types.iter().find(|entry| entry.0 == parameter.variable.name).map(|entry| entry.1);

            parameter
        }))
    }

    pub(crate) fn lower_var_annotation(
        &self,
        document: Option<&Document<'arena>>,
    ) -> Option<&'arena TypeAnnotation<'arena>> {
        let document = document?;
        let mut annotation = None;
        let mut vendor: Option<TagVendor> = None;
        for element in document.elements.iter() {
            let Element::Tag(tag) = element else { continue };
            let tag = *tag;
            if let TagValue::Var(value) = &tag.value {
                if annotation.is_none() || tag.vendor > vendor {
                    vendor = tag.vendor;
                    annotation = Some(self.lower_type_annotation(value.r#type));
                }
            }
        }

        annotation
    }

    pub(crate) fn lower_use_annotations(&self, document: Option<&Document<'arena>>) -> &'arena [UseAnnotation<'arena>] {
        let arena = self.arena;
        let mut uses = Vec::new_in(arena);
        if let Some(document) = document {
            for element in document.elements.iter() {
                let Element::Tag(tag) = element else { continue };
                let tag = *tag;
                if let TagValue::Uses(value) = &tag.value {
                    uses.extend(self.lower_use_annotation(value));
                }
            }
        }

        uses.into_bump_slice()
    }

    fn normalize_tag_name<'buffer>(&self, name: &[u8], buffer: &'buffer mut [u8; 32]) -> &'buffer [u8] {
        let mut length = 0;
        for &byte in name {
            if byte == b'-' {
                continue;
            }

            if length == buffer.len() {
                return &buffer[..length];
            }

            buffer[length] = byte.to_ascii_lowercase();
            length += 1;
        }

        &buffer[..length]
    }

    pub(crate) fn detect_marker_flags(&self, document: Option<&Document<'arena>>) -> MarkerFlags {
        let mut markers = MarkerFlags::default();
        let Some(document) = document else { return markers };

        markers.inherit_doc = document.has_inherit_doc();
        for element in document.elements.iter() {
            let Element::Tag(tag) = element else { continue };
            let tag = *tag;
            let mut buffer = [0u8; 32];
            match self.normalize_tag_name(tag.name.value, &mut buffer) {
                b"deprecated" => markers.deprecated = true,
                b"notdeprecated" => markers.not_deprecated = true,
                b"internal" => markers.internal = true,
                b"api" => markers.api = true,
                b"experimental" => markers.experimental = true,
                b"pure" => markers.pure = true,
                b"final" => markers.r#final = true,
                b"mutationfree" => markers.mutation_free = true,
                b"externalmutationfree" => markers.external_mutation_free = true,
                b"nonamedarguments" => markers.no_named_arguments = true,
                b"mustuse" => markers.must_use = true,
                b"ignorenullablereturn" => markers.ignore_nullable_return = true,
                b"ignorefalsablereturn" => markers.ignore_falsable_return = true,
                b"consistentconstructor" => markers.consistent_constructor = true,
                b"consistenttemplates" => markers.consistent_templates = true,
                b"sealproperties" => markers.seal_properties = true,
                b"nosealproperties" => markers.no_seal_properties = true,
                b"sealmethods" => markers.seal_methods = true,
                b"nosealmethods" => markers.no_seal_methods = true,
                b"enuminterface" => markers.enum_interface = true,
                _ => {}
            }
        }

        if markers.not_deprecated {
            markers.deprecated = false;
        }

        markers
    }

    pub(crate) fn lower_class(&mut self, class: &'arena cst::Class<'arena>) -> &'arena Class<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&class.attribute_lists);
        let modifiers = self.lower_modifiers(&class.modifiers);
        let name = self.lower_declaration_name(&class.name);
        let extends = class.extends.as_ref().and_then(|extends| self.lower_extends_one(extends));
        let implements = class.implements.as_ref().map(|implements| self.lower_implements(implements));

        let document = self.phpdoc_resolution.get(class.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotations = self.lower_class_like_annotations(document.as_ref(), name);

        let mut trait_uses = Vec::new_in(self.arena);
        let mut constants = Vec::new_in(self.arena);
        let mut properties = Vec::new_in(self.arena);
        let mut hooked_properties = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);

        for member in class.members.iter() {
            match member {
                cst::ClassLikeMember::TraitUse(trait_use) => trait_uses.push(self.lower_trait_use(trait_use)),
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::Property(cst::Property::Plain(property)) => {
                    properties.push(self.lower_plain_property(property));
                }
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    hooked_properties.push(self.lower_hooked_property(property));
                }
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method, name)),
                cst::ClassLikeMember::EnumCase(_) => {}
            }
        }

        self.type_resolution.leave_scope();

        self.arena.alloc(Class {
            flags: self.detect_marker_flags(document.as_ref()).class_flags(),
            attributes,
            name,
            type_parameter_annotations: annotations.type_parameters,
            modifiers,
            type_alias_annotations: annotations.type_aliases,
            imported_type_alias_annotations: annotations.imported_type_aliases,
            extends,
            extends_annotations: annotations.extends,
            implements,
            implements_annotations: annotations.implements,
            sealed_annotation: annotations.sealed,
            mixin_annotations: annotations.mixins,
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            properties: properties.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            property_annotations: annotations.properties,
            methods: methods.into_bump_slice(),
            method_annotations: annotations.methods,
        })
    }

    pub(crate) fn lower_interface(
        &mut self,
        interface: &'arena cst::Interface<'arena>,
    ) -> &'arena Interface<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&interface.attribute_lists);
        let name = self.lower_declaration_name(&interface.name);
        let extends = interface.extends.as_ref().map(|extends| self.lower_extends_one_or_more(extends));

        let document = self.phpdoc_resolution.get(interface.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotations = self.lower_class_like_annotations(document.as_ref(), name);

        let mut constants = Vec::new_in(self.arena);
        let mut hooked_properties = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);

        for member in interface.members.iter() {
            match member {
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    hooked_properties.push(self.lower_hooked_property(property));
                }
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method, name)),
                _ => {}
            }
        }

        self.type_resolution.leave_scope();

        self.arena.alloc(Interface {
            flags: self.detect_marker_flags(document.as_ref()).interface_flags(),
            attributes,
            name,
            type_parameter_annotations: annotations.type_parameters,
            type_alias_annotations: annotations.type_aliases,
            imported_type_alias_annotations: annotations.imported_type_aliases,
            extends,
            extends_annotations: annotations.extends,
            sealed_annotation: annotations.sealed,
            mixin_annotations: annotations.mixins,
            constants: constants.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            methods: methods.into_bump_slice(),
            method_annotations: annotations.methods,
        })
    }

    pub(crate) fn lower_trait(&mut self, r#trait: &'arena cst::Trait<'arena>) -> &'arena Trait<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&r#trait.attribute_lists);
        let name = self.lower_declaration_name(&r#trait.name);

        let document = self.phpdoc_resolution.get(r#trait.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotations = self.lower_class_like_annotations(document.as_ref(), name);

        let mut trait_uses = Vec::new_in(self.arena);
        let mut constants = Vec::new_in(self.arena);
        let mut properties = Vec::new_in(self.arena);
        let mut hooked_properties = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);

        for member in r#trait.members.iter() {
            match member {
                cst::ClassLikeMember::TraitUse(trait_use) => trait_uses.push(self.lower_trait_use(trait_use)),
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::Property(cst::Property::Plain(property)) => {
                    properties.push(self.lower_plain_property(property));
                }
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    hooked_properties.push(self.lower_hooked_property(property));
                }
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method, name)),
                cst::ClassLikeMember::EnumCase(_) => {}
            }
        }

        self.type_resolution.leave_scope();

        self.arena.alloc(Trait {
            flags: self.detect_marker_flags(document.as_ref()).trait_flags(),
            attributes,
            name,
            type_parameter_annotations: annotations.type_parameters,
            type_alias_annotations: annotations.type_aliases,
            imported_type_alias_annotations: annotations.imported_type_aliases,
            require_extends_annotations: annotations.require_extends,
            require_implements_annotations: annotations.require_implements,
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            properties: properties.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            property_annotations: annotations.properties,
            methods: methods.into_bump_slice(),
            method_annotations: annotations.methods,
        })
    }

    pub(crate) fn lower_enum(&mut self, r#enum: &'arena cst::Enum<'arena>) -> &'arena Enum<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&r#enum.attribute_lists);
        let name = self.lower_declaration_name(&r#enum.name);
        let backing_type = r#enum
            .backing_type_hint
            .as_ref()
            .map(|hint| EnumBackingType { span: hint.span(), r#type: self.lower_type(&hint.hint) });
        let implements = r#enum.implements.as_ref().map(|implements| self.lower_implements(implements));

        let document = self.phpdoc_resolution.get(r#enum.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(name));
        let annotations = self.lower_class_like_annotations(document.as_ref(), name);

        let mut trait_uses = Vec::new_in(self.arena);
        let mut constants = Vec::new_in(self.arena);
        let mut enum_cases = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);
        for member in r#enum.members.iter() {
            match member {
                cst::ClassLikeMember::TraitUse(trait_use) => trait_uses.push(self.lower_trait_use(trait_use)),
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::EnumCase(enum_case) => enum_cases.push(self.lower_enum_case(enum_case)),
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method, name)),
                _ => {}
            }
        }

        self.type_resolution.leave_scope();

        self.arena.alloc(Enum {
            flags: self.detect_marker_flags(document.as_ref()).enum_flags(),
            attributes,
            name,
            backing_type,
            type_alias_annotations: annotations.type_aliases,
            imported_type_alias_annotations: annotations.imported_type_aliases,
            implements,
            implements_annotations: annotations.implements,
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            enum_cases: enum_cases.into_bump_slice(),
            methods: methods.into_bump_slice(),
            method_annotations: annotations.methods,
        })
    }

    pub(crate) fn lower_function(
        &mut self,
        function: &'arena cst::Function<'arena>,
    ) -> &'arena Function<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&function.attribute_lists);
        let name = self.lower_declaration_name(&function.name);
        let return_type = function.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));

        let document = self.phpdoc_resolution.get(function.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Function(name));
        let annotations = self.lower_function_like_annotations(document.as_ref());

        let lowered_parameters = self.lower_parameter_list(&function.parameter_list);
        let parameters =
            self.merge_parameter_annotations(lowered_parameters, annotations.parameters, annotations.parameter_outs);
        let body = self.statements_to_statement(function.body.statements.as_slice(), function.body.span());

        self.type_resolution.leave_scope();

        self.arena.alloc(Function {
            attributes,
            flags: self.detect_marker_flags(document.as_ref()).function_flags(),
            name,
            type_parameter_annotations: annotations.type_parameters,
            parameters,
            where_constraint_annotations: annotations.where_constraints,
            return_by_reference: function.ampersand.is_some(),
            return_type,
            return_type_annotation: annotations.return_type,
            throws_annotations: annotations.throws,
            assert_annotations: annotations.asserts,
            assert_if_true_annotations: annotations.asserts_if_true,
            assert_if_false_annotations: annotations.asserts_if_false,
            body,
        })
    }

    pub(crate) fn lower_constant(
        &mut self,
        constant: &'arena cst::Constant<'arena>,
    ) -> &'arena Constant<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&constant.attribute_lists);
        let document = self.phpdoc_resolution.get(constant.span());
        let type_annotation = self.lower_var_annotation(document.as_ref());
        let items = self.arena.alloc_slice_fill_iter(constant.items.iter().map(|item| ConstantItem {
            name: self.lower_declaration_name(&item.name),
            value: self.arena.alloc(self.lower_expression(item.value)),
        }));

        self.arena.alloc(Constant {
            flags: self.detect_marker_flags(document.as_ref()).constant_flags(),
            attributes,
            type_annotation,
            items,
        })
    }
}
