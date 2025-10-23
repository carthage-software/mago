use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for AnonymousClass<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "anon_class".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        crate::modifier::fingerprint_modifiers(self.modifiers.iter(), hasher, resolved_names, options);
        self.argument_list.fingerprint_with_hasher(hasher, resolved_names, options);
        self.extends.fingerprint_with_hasher(hasher, resolved_names, options);
        self.implements.fingerprint_with_hasher(hasher, resolved_names, options);
        for member in self.members.iter() {
            member.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Extends<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "extends".hash(hasher);
        for type_name in self.types.iter() {
            type_name.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Implements<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "implements".hash(hasher);
        for type_name in self.types.iter() {
            type_name.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for ClassLikeMember<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            ClassLikeMember::TraitUse(trait_use) => {
                trait_use.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            ClassLikeMember::Constant(constant) => {
                constant.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            ClassLikeMember::Property(property) => {
                property.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            ClassLikeMember::EnumCase(enum_case) => {
                enum_case.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            ClassLikeMember::Method(method) => {
                method.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for TraitUse<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "trait_use".hash(hasher);
        for trait_name in self.trait_names.iter() {
            trait_name.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.specification.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for TraitUseSpecification<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            TraitUseSpecification::Abstract(spec) => {
                "trait_use_abstract".hash(hasher);
                spec.0.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            TraitUseSpecification::Concrete(spec) => {
                "trait_use_concrete".hash(hasher);
                for adaptation in spec.adaptations.iter() {
                    adaptation.fingerprint_with_hasher(hasher, resolved_names, options);
                }
            }
        }
    }
}

impl Fingerprintable for TraitUseAdaptation<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            TraitUseAdaptation::Precedence(adaptation) => {
                "precedence".hash(hasher);
                adaptation.method_reference.fingerprint_with_hasher(hasher, resolved_names, options);
                for trait_name in adaptation.trait_names.iter() {
                    trait_name.fingerprint_with_hasher(hasher, resolved_names, options);
                }
                adaptation.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            TraitUseAdaptation::Alias(adaptation) => {
                "alias".hash(hasher);
                adaptation.method_reference.fingerprint_with_hasher(hasher, resolved_names, options);
                adaptation.visibility.fingerprint_with_hasher(hasher, resolved_names, options);
                adaptation.alias.fingerprint_with_hasher(hasher, resolved_names, options);
                adaptation.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for TraitUseMethodReference<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            TraitUseMethodReference::Identifier(id) => {
                "method_ref_id".hash(hasher);
                id.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            TraitUseMethodReference::Absolute(abs) => {
                "method_ref_abs".hash(hasher);
                abs.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for TraitUseAbsoluteMethodReference<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        self.trait_name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.method_name.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ClassLikeConstant<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "class_const".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        crate::modifier::fingerprint_modifiers(self.modifiers.iter(), hasher, resolved_names, options);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for ClassLikeConstantItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "const_item".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for Property<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Property::Plain(plain) => {
                "property_plain".hash(hasher);
                plain.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            Property::Hooked(hooked) => {
                "property_hooked".hash(hasher);
                hooked.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for PlainProperty<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        crate::modifier::fingerprint_modifiers(self.modifiers.iter(), hasher, resolved_names, options);
        self.var.is_some().hash(hasher);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        for item in self.items.iter() {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for HookedProperty<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        crate::modifier::fingerprint_modifiers(self.modifiers.iter(), hasher, resolved_names, options);
        self.var.is_some().hash(hasher);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.item.fingerprint_with_hasher(hasher, resolved_names, options);
        self.hook_list.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for PropertyItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            PropertyItem::Abstract(item) => {
                "prop_item_abstract".hash(hasher);
                item.variable.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            PropertyItem::Concrete(item) => {
                "prop_item_concrete".hash(hasher);
                item.variable.fingerprint_with_hasher(hasher, resolved_names, options);
                item.value.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for PropertyHookList<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "hook_list".hash(hasher);
        for hook in self.hooks.iter() {
            hook.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for PropertyHook<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "hook".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        crate::modifier::fingerprint_modifiers(self.modifiers.iter(), hasher, resolved_names, options);
        if self.ampersand.is_some() {
            "by_ref".hash(hasher);
        }
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.parameter_list.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for PropertyHookBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            PropertyHookBody::Abstract(_) => {
                "hook_body_abstract".hash(hasher);
            }
            PropertyHookBody::Concrete(body) => {
                "hook_body_concrete".hash(hasher);
                body.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for PropertyHookConcreteBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            PropertyHookConcreteBody::Block(block) => {
                "hook_block".hash(hasher);
                block.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            PropertyHookConcreteBody::Expression(expr) => {
                "hook_expr".hash(hasher);
                expr.expression.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for EnumCase<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "enum_case".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.item.fingerprint_with_hasher(hasher, resolved_names, options);
        self.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for EnumCaseItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            EnumCaseItem::Unit(item) => {
                "enum_case_unit".hash(hasher);
                item.name.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            EnumCaseItem::Backed(item) => {
                "enum_case_backed".hash(hasher);
                item.name.fingerprint_with_hasher(hasher, resolved_names, options);
                item.value.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for Method<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "method".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        crate::modifier::fingerprint_modifiers(self.modifiers.iter(), hasher, resolved_names, options);
        if self.ampersand.is_some() {
            "by_ref".hash(hasher);
        }
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.parameter_list.fingerprint_with_hasher(hasher, resolved_names, options);
        self.return_type_hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.body.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

impl Fingerprintable for MethodBody<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            MethodBody::Abstract(_) => {
                "method_abstract".hash(hasher);
            }
            MethodBody::Concrete(block) => {
                "method_concrete".hash(hasher);
                block.fingerprint_with_hasher(hasher, resolved_names, options);
            }
        }
    }
}

impl Fingerprintable for Class<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "class".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        crate::modifier::fingerprint_modifiers(self.modifiers.iter(), hasher, resolved_names, options);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.extends.fingerprint_with_hasher(hasher, resolved_names, options);
        self.implements.fingerprint_with_hasher(hasher, resolved_names, options);
        for member in self.members.iter() {
            member.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Interface<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "interface".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.extends.fingerprint_with_hasher(hasher, resolved_names, options);
        for member in self.members.iter() {
            member.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Trait<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "trait".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        for member in self.members.iter() {
            member.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Enum<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "enum".hash(hasher);
        for attribute_list in self.attribute_lists.iter() {
            attribute_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.backing_type_hint.fingerprint_with_hasher(hasher, resolved_names, options);
        self.implements.fingerprint_with_hasher(hasher, resolved_names, options);
        for member in self.members.iter() {
            member.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for EnumBackingTypeHint<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "enum_backing_type".hash(hasher);
        self.hint.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
