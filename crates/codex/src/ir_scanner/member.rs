use mago_database::file::File;
use mago_hir::ir::member::ClassLikeConstant;
use mago_hir::ir::member::EnumCase;
use mago_hir::ir::member::Property;
use mago_hir::ir::member::PropertyFlags;
use mago_hir::ir::member::annotation::PropertyAnnotation;
use mago_hir::ir::member::annotation::PropertyAnnotationKind;
use mago_hir::ir::modifier::Modifier;
use mago_hir::ir::modifier::ModifierKind;
use mago_span::Span;
use mago_word::WordMap;
use mago_word::word;

use crate::consts::MAX_ENUM_CASES_FOR_ANALYSIS;
use crate::ir_scanner::attribute::scan_attributes;
use crate::ir_scanner::inference::infer;
use crate::ir_scanner::ttype::merge_type_preserving_nullability;
use crate::ir_scanner::ttype::type_metadata_from_annotation;
use crate::ir_scanner::ttype::type_metadata_from_type;
use crate::ir_scanner::version_constraint_from;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::get_string;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

pub(super) fn has(modifiers: &[Modifier], kind: ModifierKind) -> bool {
    modifiers.iter().any(|modifier| modifier.kind == kind)
}

pub(super) fn read_visibility(modifiers: &[Modifier]) -> Visibility {
    for modifier in modifiers {
        match modifier.kind {
            ModifierKind::Public => return Visibility::Public,
            ModifierKind::Protected => return Visibility::Protected,
            ModifierKind::Private => return Visibility::Private,
            _ => {}
        }
    }

    Visibility::Public
}

pub(super) fn write_visibility(modifiers: &[Modifier], read: Visibility, is_readonly: bool) -> Visibility {
    for modifier in modifiers {
        match modifier.kind {
            ModifierKind::PublicSet => return Visibility::Public,
            ModifierKind::ProtectedSet => return Visibility::Protected,
            ModifierKind::PrivateSet => return Visibility::Private,
            _ => {}
        }
    }

    if is_readonly { Visibility::Protected } else { read }
}

pub fn scan_properties(
    metadata: &mut ClassLikeMetadata,
    properties: &[Property<'_, (), (), ()>],
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) {
    let classname = metadata.name;

    for property in properties {
        let is_readonly = has(property.modifiers, ModifierKind::Readonly);
        let read = read_visibility(property.modifiers);
        let write = write_visibility(property.modifiers, read, is_readonly);
        let type_declaration = property.r#type.map(|hint| type_metadata_from_type(hint, Some(classname)));
        let type_annotation = property.type_annotation.map(|annotation| {
            merge_type_preserving_nullability(
                type_metadata_from_annotation(annotation, Some(classname)),
                type_declaration.as_ref(),
            )
        });

        let mut flags = origin;
        if is_readonly || property.flags.is_set(PropertyFlags::Readonly) {
            flags |= MetadataFlags::READONLY;
        }
        if has(property.modifiers, ModifierKind::Static) {
            flags |= MetadataFlags::STATIC;
        }
        if has(property.modifiers, ModifierKind::Abstract) {
            flags |= MetadataFlags::ABSTRACT;
        }
        if has(property.modifiers, ModifierKind::Final) || property.flags.is_set(PropertyFlags::Final) {
            flags |= MetadataFlags::FINAL;
        }
        if property.flags.is_set(PropertyFlags::Deprecated) {
            flags |= MetadataFlags::DEPRECATED;
        }
        if property.flags.is_set(PropertyFlags::Internal) {
            flags |= MetadataFlags::INTERNAL;
        }
        if property.flags.is_set(PropertyFlags::Experimental) {
            flags |= MetadataFlags::EXPERIMENTAL;
        }
        if property.flags.is_set(PropertyFlags::API) {
            flags |= MetadataFlags::API;
        }

        for item in property.items {
            let name = word(item.variable.name);

            let mut item_flags = flags;
            if item.default_value.is_some() {
                item_flags |= MetadataFlags::HAS_DEFAULT;
            }

            let mut property_metadata = PropertyMetadata::new(VariableIdentifier(name), item_flags);
            property_metadata.version_constraint = version_constraint_from(property.version_constraint);
            property_metadata.set_visibility(read, write);
            property_metadata.set_name_span(Some(item.variable.span));
            if let Some(type_annotation) = &type_annotation {
                property_metadata.set_type_metadata(Some(type_annotation.clone()));
            }
            property_metadata.set_type_declaration_metadata(type_declaration.clone());

            if let Some(default_value) = item.default_value {
                property_metadata.set_default_type_metadata(
                    infer(default_value, Some(classname), file, constants).map(|union| {
                        let mut type_metadata = TypeMetadata::new(union, item.variable.span);
                        type_metadata.inferred = true;
                        type_metadata
                    }),
                );
            }

            if let Some(magic) = metadata.properties.get(&name) {
                property_metadata.read_visibility = magic.read_visibility;
                property_metadata.write_visibility = magic.read_visibility;
                property_metadata.flags.set(MetadataFlags::VIRTUAL_PROPERTY, false);
                property_metadata.flags.set(MetadataFlags::READONLY, magic.flags.is_readonly());
            }

            metadata.add_property(name, property_metadata);
        }
    }
}

pub fn scan_class_constants(
    metadata: &mut ClassLikeMetadata,
    class_constants: &[ClassLikeConstant<'_, (), (), ()>],
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) {
    let classname = metadata.name;

    for constant in class_constants {
        let visibility = read_visibility(constant.modifiers);
        let type_declaration = constant.r#type.map(|hint| type_metadata_from_type(hint, Some(classname)));
        let type_annotation = constant.type_annotation.map(|annotation| {
            merge_type_preserving_nullability(
                type_metadata_from_annotation(annotation, Some(classname)),
                type_declaration.as_ref(),
            )
        });

        let mut flags = origin;
        if has(constant.modifiers, ModifierKind::Final) {
            flags |= MetadataFlags::FINAL;
        }

        for item in constant.items {
            let name = word(item.name.value);

            let mut constant_metadata = ClassLikeConstantMetadata::new(name, item.name.span, visibility, flags);
            constant_metadata.attributes = scan_attributes(constant.attributes);
            constant_metadata.version_constraint = version_constraint_from(constant.version_constraint);
            if let Some(type_annotation) = &type_annotation {
                constant_metadata.type_metadata = Some(type_annotation.clone());
            }
            if let Some(type_declaration) = type_declaration.clone() {
                constant_metadata.set_type_declaration(type_declaration);
            }
            constant_metadata.inferred_type =
                infer(item.value, Some(classname), file, constants).map(TUnion::get_single_owned);

            if let Some(TAtomic::Reference(TReference::Member {
                class_like_name,
                member_selector: TReferenceMemberSelector::Identifier(member_name),
            })) = constant_metadata.inferred_type.as_ref()
                && class_like_name.as_bytes().eq_ignore_ascii_case(classname.as_bytes())
                && member_name.as_bytes() == item.name.value
            {
                constant_metadata.inferred_type = Some(TAtomic::Never);
            }

            metadata.constants.insert(name, constant_metadata);
        }
    }
}

pub fn scan_property_annotations(metadata: &mut ClassLikeMetadata, annotations: &[PropertyAnnotation<'_>]) {
    let classname = metadata.name;
    for annotation in annotations {
        let name = word(annotation.variable.name);

        let mut flags = MetadataFlags::MAGIC_PROPERTY;
        match annotation.kind {
            PropertyAnnotationKind::Read => flags |= MetadataFlags::READONLY,
            PropertyAnnotationKind::Write => flags |= MetadataFlags::WRITEONLY,
            PropertyAnnotationKind::ReadWrite => {}
        }

        let mut property = PropertyMetadata::new(VariableIdentifier(name), flags);
        property.type_metadata =
            annotation.r#type.map(|annotation| type_metadata_from_annotation(annotation, Some(classname)));
        metadata.add_property_metadata(property);
    }
}

pub fn scan_enum_properties(metadata: &mut ClassLikeMetadata, span: Span) {
    let backing_type = metadata.enum_type.clone();
    let mut name_types = Vec::new();
    let mut value_types = Vec::new();

    if metadata.enum_cases.len() <= MAX_ENUM_CASES_FOR_ANALYSIS {
        for (case_name, case) in &metadata.enum_cases {
            name_types.push(TAtomic::Scalar(TScalar::literal_string(*case_name)));

            if let Some(backing_type) = &backing_type {
                value_types.push(case.value_type.clone().unwrap_or_else(|| backing_type.clone()));
            }
        }
    }

    let name_union = if name_types.is_empty() { get_string() } else { TUnion::from_vec(name_types) };
    if value_types.is_empty() {
        if let Some(backing_type) = &backing_type {
            value_types.push(backing_type.clone());
        }
    }

    let flags = MetadataFlags::READONLY | MetadataFlags::HAS_DEFAULT;

    let mut name_property = PropertyMetadata::new(VariableIdentifier(word("$name")), flags);
    name_property.type_declaration_metadata = Some(TypeMetadata::new(get_string(), span));
    name_property.type_metadata = Some(TypeMetadata::new(name_union, span));
    metadata.add_property_metadata(name_property);

    if let Some(backing_type) = backing_type {
        let mut value_property = PropertyMetadata::new(VariableIdentifier(word("$value")), flags);
        value_property
            .set_type_declaration_metadata(Some(TypeMetadata::new(TUnion::from_vec(vec![backing_type]), span)));
        if !value_types.is_empty() {
            value_property.set_type_metadata(Some(TypeMetadata::new(TUnion::from_vec(value_types), span)));
        }
        metadata.add_property_metadata(value_property);
    }
}

pub fn scan_enum_cases(
    metadata: &mut ClassLikeMetadata,
    cases: &[EnumCase<'_, (), (), ()>],
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) {
    let classname = metadata.name;
    for case in cases {
        let name = word(case.name.value);
        let flags =
            origin | if case.value.is_some() { MetadataFlags::BACKED_ENUM_CASE } else { MetadataFlags::UNIT_ENUM_CASE };

        let mut case_metadata = EnumCaseMetadata::new(name, case.name.span, case.span, flags);
        case_metadata.attributes = scan_attributes(case.attributes);
        case_metadata.version_constraint = version_constraint_from(case.version_constraint);
        if let Some(value) = case.value {
            case_metadata.value_type = infer(value, Some(classname), file, constants).map(TUnion::get_single_owned);
        }

        metadata.enum_cases.insert(name, case_metadata);
    }
}
