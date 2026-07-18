use mago_flags::U8Flags;
use mago_flags::U16Flags;
use mago_flags::U32Flags;
use mago_hir::ir::item::annotation::ItemAnnotation;
use mago_hir::ir::item::annotation::ItemAnnotationTag;

use crate::symbol::class_like::class::ClassFlag;
use crate::symbol::class_like::r#enum::EnumFlag;
use crate::symbol::class_like::interface::InterfaceFlag;
use crate::symbol::class_like::part::constant::ClassLikeConstantFlag;
use crate::symbol::class_like::part::method::MethodFlag;
use crate::symbol::class_like::part::property::PropertyFlag;
use crate::symbol::class_like::r#trait::TraitFlag;
use crate::symbol::function_like::function::FunctionFlag;

/// The docblock tags declared on a definition, or the empty set when it has no
/// annotation.
pub(crate) fn tags_of<I, St, Ex>(annotation: Option<&ItemAnnotation<'_, I, St, Ex>>) -> U32Flags<ItemAnnotationTag> {
    annotation.map_or_else(U32Flags::empty, |annotation| annotation.tags)
}

/// The class flags implied by docblock tags (`@deprecated`, `@final`, `@immutable`, ...).
pub(crate) fn class_flags(tags: U32Flags<ItemAnnotationTag>) -> U16Flags<ClassFlag> {
    let mut flags = U16Flags::<ClassFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(ClassFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(ClassFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(ClassFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(ClassFlag::Experimental);
    }

    if tags.contains(ItemAnnotationTag::Final) {
        flags = flags.with(ClassFlag::Final);
    }

    if tags.contains(ItemAnnotationTag::Abstract) {
        flags = flags.with(ClassFlag::Abstract);
    }

    if tags.contains(ItemAnnotationTag::Readonly) {
        flags = flags.with(ClassFlag::Readonly);
    }

    if tags.contains(ItemAnnotationTag::Immutable) {
        flags = flags.with(ClassFlag::Immutable);
    }

    if tags.contains(ItemAnnotationTag::ConsistentConstructor) {
        flags = flags.with(ClassFlag::ConsistentConstructor);
    }

    if tags.contains(ItemAnnotationTag::ConsistentTypeParameterAnnotations) {
        flags = flags.with(ClassFlag::ConsistentTemplates);
    }

    if tags.contains(ItemAnnotationTag::SealProperties) {
        flags = flags.with(ClassFlag::SealedProperties);
    }

    if tags.contains(ItemAnnotationTag::SealMethods) {
        flags = flags.with(ClassFlag::SealedMethods);
    }

    if tags.contains(ItemAnnotationTag::NoSealProperties) {
        flags = flags.with(ClassFlag::UnsealedProperties);
    }

    if tags.contains(ItemAnnotationTag::NoSealMethods) {
        flags = flags.with(ClassFlag::UnsealedMethods);
    }

    flags
}

/// The interface flags implied by docblock tags.
pub(crate) fn interface_flags(tags: U32Flags<ItemAnnotationTag>) -> U16Flags<InterfaceFlag> {
    let mut flags = U16Flags::<InterfaceFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(InterfaceFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(InterfaceFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(InterfaceFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(InterfaceFlag::Experimental);
    }

    if tags.contains(ItemAnnotationTag::Immutable) {
        flags = flags.with(InterfaceFlag::Immutable);
    }

    if tags.contains(ItemAnnotationTag::ConsistentConstructor) {
        flags = flags.with(InterfaceFlag::ConsistentConstructor);
    }

    if tags.contains(ItemAnnotationTag::ConsistentTypeParameterAnnotations) {
        flags = flags.with(InterfaceFlag::ConsistentTemplates);
    }

    if tags.contains(ItemAnnotationTag::SealProperties) {
        flags = flags.with(InterfaceFlag::SealedProperties);
    }

    if tags.contains(ItemAnnotationTag::SealMethods) {
        flags = flags.with(InterfaceFlag::SealedMethods);
    }

    if tags.contains(ItemAnnotationTag::NoSealProperties) {
        flags = flags.with(InterfaceFlag::UnsealedProperties);
    }

    if tags.contains(ItemAnnotationTag::NoSealMethods) {
        flags = flags.with(InterfaceFlag::UnsealedMethods);
    }

    if tags.contains(ItemAnnotationTag::EnumInterface) {
        flags = flags.with(InterfaceFlag::EnumInterface);
    }

    flags
}

/// The trait flags implied by docblock tags.
pub(crate) fn trait_flags(tags: U32Flags<ItemAnnotationTag>) -> U16Flags<TraitFlag> {
    let mut flags = U16Flags::<TraitFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(TraitFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(TraitFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(TraitFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(TraitFlag::Experimental);
    }

    if tags.contains(ItemAnnotationTag::Immutable) {
        flags = flags.with(TraitFlag::Immutable);
    }

    if tags.contains(ItemAnnotationTag::ConsistentConstructor) {
        flags = flags.with(TraitFlag::ConsistentConstructor);
    }

    if tags.contains(ItemAnnotationTag::ConsistentTypeParameterAnnotations) {
        flags = flags.with(TraitFlag::ConsistentTemplates);
    }

    if tags.contains(ItemAnnotationTag::SealProperties) {
        flags = flags.with(TraitFlag::SealedProperties);
    }

    if tags.contains(ItemAnnotationTag::SealMethods) {
        flags = flags.with(TraitFlag::SealedMethods);
    }

    if tags.contains(ItemAnnotationTag::NoSealProperties) {
        flags = flags.with(TraitFlag::UnsealedProperties);
    }

    if tags.contains(ItemAnnotationTag::NoSealMethods) {
        flags = flags.with(TraitFlag::UnsealedMethods);
    }

    flags
}

/// The enum flags implied by docblock tags.
pub(crate) fn enum_flags(tags: U32Flags<ItemAnnotationTag>) -> U8Flags<EnumFlag> {
    let mut flags = U8Flags::<EnumFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(EnumFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(EnumFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(EnumFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(EnumFlag::Experimental);
    }

    flags
}

/// The method flags implied by docblock tags.
pub(crate) fn method_flags(tags: U32Flags<ItemAnnotationTag>) -> U32Flags<MethodFlag> {
    let mut flags = U32Flags::<MethodFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(MethodFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(MethodFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(MethodFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(MethodFlag::Experimental);
    }

    if tags.contains(ItemAnnotationTag::MustUse) {
        flags = flags.with(MethodFlag::MustUse);
    }

    if tags.contains(ItemAnnotationTag::Pure) {
        flags = flags.with(MethodFlag::Pure);
    }

    if tags.contains(ItemAnnotationTag::IgnoreNullableReturnType) {
        flags = flags.with(MethodFlag::IgnoreNullableReturn);
    }

    if tags.contains(ItemAnnotationTag::IgnoreFalsableReturnType) {
        flags = flags.with(MethodFlag::IgnoreFalsableReturn);
    }

    if tags.contains(ItemAnnotationTag::NoNamedArguments) {
        flags = flags.with(MethodFlag::NoNamedArguments);
    }

    if tags.contains(ItemAnnotationTag::SuspendsFiber) {
        flags = flags.with(MethodFlag::SuspendsFiber);
    }

    flags
}

/// The property flags implied by docblock tags.
pub(crate) fn property_flags(tags: U32Flags<ItemAnnotationTag>) -> U16Flags<PropertyFlag> {
    let mut flags = U16Flags::<PropertyFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(PropertyFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(PropertyFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(PropertyFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(PropertyFlag::Experimental);
    }

    if tags.contains(ItemAnnotationTag::Readonly) {
        flags = flags.with(PropertyFlag::Readonly);
    }

    flags
}

/// The class-constant flags implied by docblock tags.
pub(crate) fn constant_flags(tags: U32Flags<ItemAnnotationTag>) -> U8Flags<ClassLikeConstantFlag> {
    let mut flags = U8Flags::<ClassLikeConstantFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(ClassLikeConstantFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(ClassLikeConstantFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(ClassLikeConstantFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(ClassLikeConstantFlag::Experimental);
    }

    if tags.contains(ItemAnnotationTag::Final) {
        flags = flags.with(ClassLikeConstantFlag::Final);
    }

    flags
}

/// The function flags implied by docblock tags.
pub(crate) fn function_flags(tags: U32Flags<ItemAnnotationTag>) -> U16Flags<FunctionFlag> {
    let mut flags = U16Flags::<FunctionFlag>::empty();
    if tags.contains(ItemAnnotationTag::Deprecated) {
        flags = flags.with(FunctionFlag::Deprecated);
    }

    if tags.contains(ItemAnnotationTag::Internal) {
        flags = flags.with(FunctionFlag::Internal);
    }

    if tags.contains(ItemAnnotationTag::Api) {
        flags = flags.with(FunctionFlag::API);
    }

    if tags.contains(ItemAnnotationTag::Experimental) {
        flags = flags.with(FunctionFlag::Experimental);
    }

    if tags.contains(ItemAnnotationTag::MustUse) {
        flags = flags.with(FunctionFlag::MustUse);
    }

    if tags.contains(ItemAnnotationTag::Pure) {
        flags = flags.with(FunctionFlag::Pure);
    }

    if tags.contains(ItemAnnotationTag::IgnoreNullableReturnType) {
        flags = flags.with(FunctionFlag::IgnoreNullableReturn);
    }

    if tags.contains(ItemAnnotationTag::IgnoreFalsableReturnType) {
        flags = flags.with(FunctionFlag::IgnoreFalsableReturn);
    }

    if tags.contains(ItemAnnotationTag::NoNamedArguments) {
        flags = flags.with(FunctionFlag::NoNamedArguments);
    }

    if tags.contains(ItemAnnotationTag::SuspendsFiber) {
        flags = flags.with(FunctionFlag::SuspendsFiber);
    }

    flags
}
